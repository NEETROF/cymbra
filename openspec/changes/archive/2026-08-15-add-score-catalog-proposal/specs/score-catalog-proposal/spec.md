## ADDED Requirements

### Requirement: Uploading a score never enters the public catalog or review

A signed-in user's uploaded score SHALL remain in their **private** library
(`user_scores`) and MUST NOT be added to the public catalog (`catalog_scores`) nor enter
the moderation review queue as a side effect of the upload. A private score enters the
public catalog and its review queue **only** through an explicit proposal (see
"Opt-in proposal of a private score to the public catalog"). Absent a proposal, the
score is reachable only by its owner and is never visible to other users, to public
search/listing, or to a moderator's review queue.

#### Scenario: Upload stays private

- **WHEN** a signed-in user uploads a score
- **THEN** the score is stored in that user's private library and is not present in the
  public catalog nor in the moderation review queue

#### Scenario: Un-proposed score is absent from review

- **WHEN** a moderator opens the review queue and the user has not proposed their score
- **THEN** the user's private score does not appear in the queue

### Requirement: Opt-in proposal of a private score to the public catalog

The backend SHALL expose an authenticated, owner-scoped operation (`ProposeScore`) that
proposes one of the caller's private scores to the public catalog. The operation SHALL
require, captured at proposal time, a **licence declaration** and an explicit
**right-to-distribute attestation**; a proposal missing either MUST be refused and MUST
NOT create a catalog entry. A valid proposal SHALL create a new public-catalog entry
carrying the private score's server-derived metadata and a catalog-owned copy of its
bytes, entering the moderation lifecycle (see `score-moderation`). The caller MUST own
the referenced private score; proposing a score the caller does not own, or a
non-existent score, MUST be refused. The created catalog entry SHALL be independent of
the private score thereafter, so later deleting the private score does not remove the
catalog entry.

#### Scenario: Proposal requires licence and attestation

- **WHEN** a user proposes a private score without a licence declaration and
  right-to-distribute attestation
- **THEN** the proposal is refused and no catalog entry is created

#### Scenario: Valid proposal enters the catalog

- **WHEN** a user proposes a private score with a licence declaration and attestation
- **THEN** a public-catalog entry is created from that score, attributed to the proposer,
  and enters the moderation lifecycle

#### Scenario: Cannot propose a score you do not own

- **WHEN** a caller proposes a score id that is not in their private library (or does not
  exist)
- **THEN** the proposal is refused and no catalog entry is created

#### Scenario: Catalog entry survives deletion of the private score

- **WHEN** a user deletes a private score that they previously proposed
- **THEN** the catalog entry (and its bytes) remain as the moderation record

### Requirement: Proposed status branches on the proposer's role

The server SHALL derive the proposed entry's initial moderation status from the
proposer's role rather than from any client-supplied value: a music-scope `admin`
proposal SHALL be recorded `accepted` (immediately publicly visible); any other
authenticated identity's proposal SHALL be recorded `pending`. The proposer SHALL be
recorded on the catalog entry (`proposed_by`) and the entry SHALL be tagged with a
**user-proposal origin** distinct from crawler-ingested entries, so the review queue and
audit trail identify the contributor and the entry's provenance. The client MUST NOT be
able to set the status directly.

#### Scenario: Plain user proposal is pending

- **WHEN** a plain user proposes a private score
- **THEN** the catalog entry is recorded `pending`, attributed to that user, and is not
  publicly visible until reviewed

#### Scenario: Admin proposal is auto-accepted

- **WHEN** a music-scope `admin` proposes a score
- **THEN** the catalog entry is recorded `accepted` and immediately publicly visible

#### Scenario: Client cannot self-assign a status

- **WHEN** a proposal request attempts to set its own moderation status
- **THEN** the server ignores it and derives the status from the proposer's role

### Requirement: Privileged moderation read distinguishes and attributes a proposed score

The backend SHALL, on the **privileged** (moderator/admin) catalog read used by the
review queue, expose for each catalog entry its **origin** (user-proposed vs
crawler-ingested) and, for a user-proposed entry, the **proposer's identity resolved to a
display name/pseudo** (the privileged attribution field). The proposer's user **id** and
this **privileged pseudo field** MUST NOT be exposed to a normal
(non-moderator/non-admin) caller — they are populated only for a privileged read, exactly
as the moderation-status facets are. The display name SHALL be resolved through the
user-directory seam (not stored denormalised on the catalog row), so a later rename is
reflected. This privileged attribution is **unconditional** (it does not depend on the
proposer's profile visibility); a separate, opt-in **public contributor credit** is
governed by "Public contributor credit gated on public-profile opt-in".

#### Scenario: Reviewer sees origin and proposer pseudo

- **WHEN** a moderator/admin reads the review queue
- **THEN** each user-proposed score shows a user-proposal origin and the proposer's pseudo,
  distinct from crawler-ingested scores, regardless of the proposer's profile visibility

#### Scenario: Privileged attribution is hidden from normal callers

- **WHEN** a normal caller reads the catalog through a public path
- **THEN** no `proposed_by` id and no privileged proposer-pseudo field are returned (any
  public credit is governed separately, only under the opt-in condition)

### Requirement: Public contributor credit gated on public-profile opt-in

The system SHALL surface a **public contributor credit** (the proposer's public
handle/display name) to any caller on an `accepted` user-proposed score's public read
path, but **only** when the proposer has opted into a **public** profile; when the
proposer's profile is private (the default) or carries no handle/display name, the credit
MUST be omitted and the score is still served without attribution. Visibility SHALL be resolved through the user-directory
seam and MUST be **fail-closed** — an unknown or unresolvable visibility yields no credit.
The credit SHALL be shown only for `accepted`, user-proposed scores; a crawler-ingested
score carries its dataset attribution, not a contributor credit. The raw `proposed_by`
id MUST NOT be exposed on this public path — only the opt-in display credit.

#### Scenario: Public profile yields a credit

- **WHEN** a normal caller reads an `accepted` user-proposed score whose proposer has a
  public profile
- **THEN** the score carries a public contributor credit with the proposer's handle/display
  name, and no `proposed_by` id

#### Scenario: Private profile yields no credit

- **WHEN** a normal caller reads an `accepted` user-proposed score whose proposer's profile
  is private (or has no handle)
- **THEN** no contributor credit is included and the score is still served

#### Scenario: Credit resolution is fail-closed

- **WHEN** the proposer's profile visibility cannot be resolved
- **THEN** no contributor credit is included

#### Scenario: App shows the credit when present

- **WHEN** the app displays an `accepted` score that carries a public contributor credit
- **THEN** it shows a "proposé par @pseudo" style attribution, and shows none when the
  credit is absent

### Requirement: Identical score content is detected across proposals

The system SHALL use the exact-byte content digest (SHA-256) of the proposed score to
prevent duplicate catalog entries. Because a catalog entry's content digest is **unique**
(at most one catalog row per content), the system SHALL treat content identity — not only
the client-facing id — as the uniqueness guard: a proposal whose bytes match a
**non-`rejected`** catalog entry MUST be refused as a duplicate, reporting the existing
entry's id, rather than creating a second entry, so two different users proposing the same
work do not create duplicate catalog rows. A proposal whose bytes match a **`rejected`**
catalog entry is handled by "Re-proposing a rejected score reopens its row" (it is not a
duplicate refusal).

#### Scenario: Byte-identical proposal is refused as duplicate

- **WHEN** a user proposes a score whose bytes match an existing non-`rejected` catalog
  entry
- **THEN** the proposal is refused, identifies the existing entry, and no second entry is
  created

#### Scenario: Distinct content is admitted

- **WHEN** a user proposes a score whose bytes differ from every catalog entry
- **THEN** it is admitted (subject to the role-based status branching)

### Requirement: Re-proposing a score is guarded

The system SHALL link a private score to the catalog entry created from it. If a private
score has already been proposed and its catalog entry is `pending` or `accepted`,
re-proposing the same private score MUST be refused with a typed already-proposed error
rather than creating a second entry. If the linked catalog entry is `rejected`, the owner
MAY re-propose it, subject to "Re-proposing a rejected score reopens its row".

#### Scenario: Re-proposing a pending/accepted score is refused

- **WHEN** an owner proposes a private score whose existing catalog entry is `pending` or
  `accepted`
- **THEN** the proposal is refused with an already-proposed error and no second entry is
  created

### Requirement: Rejected proposal records and surfaces its reason

When a moderator rejects a user-proposed score, the system SHALL record a **rejection
reason** (a moderator-supplied motive) on the catalog entry, and SHALL surface that reason
back to the proposer so they know why it was refused. The reason SHALL travel with the
proposal state on the owner's contributions read (alongside the `rejected` status). A
`pending` or `accepted` entry carries no rejection reason.

#### Scenario: Rejection records a reason

- **WHEN** a moderator rejects a user-proposed score
- **THEN** the catalog entry stores the moderator's rejection reason

#### Scenario: Proposer sees why it was rejected

- **WHEN** the proposer views a contribution whose proposal was rejected
- **THEN** the app shows the `rejected` state together with the moderator's rejection reason

### Requirement: Re-proposing a rejected score reopens its row and requires a justification

The system SHALL, when a private score whose linked catalog entry is `rejected` is
re-proposed, **reopen that existing entry** — transition it back to `pending`,
re-attribute it to the current proposer (`proposed_by`), and clear its prior rejection
reason — rather than create a second entry (content identity is unique). The re-proposal
MUST carry a
non-empty **justification** (an explanation motivating reconsideration); a re-proposal of a
`rejected` entry without a justification MUST be refused and nothing changes. The
justification SHALL be recorded on the entry and surfaced to the moderator on re-review, so
the reviewer sees why it is being resubmitted. A **first** proposal (no prior rejected
entry) SHALL NOT require a justification.

#### Scenario: Re-proposing a rejected score reopens the row

- **WHEN** an owner re-proposes a private score whose linked catalog entry is `rejected`,
  with a justification
- **THEN** that same catalog entry returns to `pending`, is re-attributed to the proposer,
  its prior rejection reason is cleared, and no second entry is created

#### Scenario: Re-proposal without a justification is refused

- **WHEN** an owner re-proposes a `rejected` score without a justification
- **THEN** the proposal is refused and the entry stays `rejected`

#### Scenario: Justification reaches the moderator on re-review

- **WHEN** a reopened score appears in the review queue
- **THEN** the moderator sees the proposer's resubmission justification

#### Scenario: First proposal needs no justification

- **WHEN** an owner proposes a score that has never been in the catalog
- **THEN** the proposal succeeds without a justification (subject to licence + attestation)

### Requirement: App surfaces and drives proposal from two entry points

The app SHALL let the owner propose a private score to the public catalog from **both**
(a) the owner's **contributions list** and (b) an **opt-in step at the end of the upload
wizard**, shown after a successful upload. For each of the owner's contributed scores the
app SHALL surface its proposal state — not proposed, `pending`, `accepted`, or `rejected`
— and SHALL offer the "propose to the public catalog" action only for a not-yet-proposed
score. The wizard's propose step SHALL be **opt-in and separate** from the upload
submission — never a pre-ticked default folded into the upload — so declining it leaves
the score private. The action SHALL require the user to supply a licence declaration and
tick a right-to-distribute attestation before it can be submitted, and MUST be routed
through an injectable service seam overridable in tests. For a score whose prior proposal
was **rejected**, the app SHALL show the moderator's rejection reason and SHALL offer a
**re-propose** action that additionally requires a non-empty **justification** before it
can be submitted. Once a score has been proposed (or re-proposed), the app SHALL reflect
the pending state and hide the propose action for it. A server refusal (duplicate, already
proposed, missing attestation, missing justification) MUST be surfaced as a localized
message, never a raw technical error.

#### Scenario: Propose reachable from the contributions list

- **WHEN** the owner opens their contributions list for a not-yet-proposed score
- **THEN** a propose action is offered for it

#### Scenario: Propose offered at the end of the upload wizard

- **WHEN** the user completes a successful upload
- **THEN** the wizard offers an opt-in step to propose the just-uploaded score to the
  public catalog, and declining it leaves the score private

#### Scenario: Propose action gated on licence and attestation

- **WHEN** the user opens the propose action (from either entry point) without a licence
  declaration or without ticking the attestation
- **THEN** submission is blocked until both are provided

#### Scenario: Proposal reflected and action hidden after submit

- **WHEN** the user submits a valid proposal for a contribution
- **THEN** the contribution shows a `pending` proposal state and no longer offers the
  propose action

#### Scenario: Rejected contribution shows reason and gates re-propose on a justification

- **WHEN** the user opens the re-propose action for a contribution whose prior proposal was
  rejected
- **THEN** the app shows the rejection reason and blocks submission until a non-empty
  justification is provided

#### Scenario: Server refusal shown as a localized message

- **WHEN** the server refuses a proposal (duplicate, already proposed, or missing
  attestation)
- **THEN** the app surfaces a localized message and leaves the contribution un-proposed
