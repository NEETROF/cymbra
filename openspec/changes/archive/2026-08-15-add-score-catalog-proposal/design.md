## Context

The music backend already separates two stores in the `music` schema:

- `music.user_scores` — a signed-in user's **private** contributions (owner-scoped,
  rights attestation captured at upload, server-derived metadata). Populated by the
  `UploadScore` RPC. **Not** in the public catalog, **not** moderated.
- `music.catalog_scores` — the **public** corpus, currently populated only by the
  crawler, carrying the moderation lifecycle (`moderation_status` `pending`/`accepted`/
  `rejected` + `reviewed_by`/`reviewed_at`, migration `0008`). Only `accepted` rows are
  publicly visible; the back-office review queue surfaces `pending` rows.

So a user upload never reaches `catalog_scores` and therefore never reaches review —
there is no bridge between the two. This change adds exactly one bridge, and makes it
**explicit and opt-in**, mirroring the SoundFont feature merged in #168:

- `user-soundfont-library` — private per-user `.sf2` library, owner-only, unmoderated,
  with an **opt-in proposal** (`POST /me/soundfonts/:id/propose`) requiring a licence
  declaration + right-to-distribute attestation, creating a `pending` catalog entry.
- `soundfont-moderation` — role-branched initial status (admin → `accepted`, else
  `pending`), client cannot self-assign, SHA-256 content dedup against non-`rejected`
  catalog entries.

Scores use gRPC (`ScoreService`) rather than the HTTP delivery routes SoundFonts use,
so the proposal is a new `ScoreService.ProposeScore` RPC — the idiomatic mirror on the
score stack.

## Goals / Non-Goals

**Goals:**
- A user's private score enters the public catalog **only** through an explicit
  `ProposeScore` call — never as a side effect of uploading.
- The propose op mirrors the SoundFont contract: licence + right-to-distribute
  attestation at propose time, role-based initial status, content dedup, proposer
  attribution, idempotent/guarded re-propose.
- The app shows each contribution's proposal state and offers the propose action,
  mirroring the SoundFont management screen.
- A proposed `pending` score flows through the **existing** moderation review queue
  and back office with no changes there.

**Non-Goals:**
- No change to the private-upload wizard's own validation or its existing
  `rights_basis`/`rights_ack` capture (that path stays private and unchanged).
- No change to the crawler ingest path, to public read-side visibility gating, or to
  the swipe rating-deck sourcing.
- No withdraw/un-publish of a proposed catalog entry (a rejected/retained catalog row
  is the audit trail, exactly as a rejected soundfont keeps its row). Out of scope.
- No auto-accept of the existing private corpus.
- **SoundFont** uploader attribution (console pseudo + public credit) is out of scope here
  — it is the separate `add-soundfont-uploader-attribution` change, which reuses the same
  `UserPort` seam this change wires into the music service.

## Decisions

### One propose RPC, reachable from two entry points

`ProposeScore { score_id, license, rights_ack, attribution? }` takes an existing
`user_scores` id owned by the caller, mirroring soundfont `propose(id, …)`. Rationale:
keeps the private upload path frictionless and unchanged, and makes "review only on
explicit proposal" structurally true — proposal is always a distinct RPC from upload,
never a side effect of it.

The app offers this single RPC from **both** entry points (decision: _les deux_):
1. the owner's **contributions list** (the primary, soundfont-parallel surface), and
2. an **opt-in step at the end of the upload wizard**, shown after a successful upload —
   a deliberate, separate action (a "propose to the public catalog?" step with the same
   licence + attestation gate), never a pre-ticked default folded into the upload
   submission.

Both call `ProposeScore` with identical semantics, so the gating contract holds
regardless of where the user starts.

_Alternative considered_: an "also publish to catalog" checkbox baked into the
`UploadScore` request itself. Rejected — it entangles the private and public paths and
makes the gating implicit rather than an explicit, separate gesture. The wizard's
post-upload step is explicit and calls the separate RPC, so it keeps the contract intact
while still being reachable inline.

### Initial status branches on the proposer's role; the client cannot set it

The server derives `moderation_status` from the caller's role: a music-scope `admin`
proposal is recorded `accepted` (immediately visible); any other identity's proposal is
`pending`. The request carries no status field. Mirrors
`soundfont-moderation`'s "Upload status branches on uploader role".

### Materialise a catalog row from the private row (copy bytes)

`ProposeScore` reads the `user_scores` row + its stored (decoded/canonical) bytes and
writes a **new** `catalog_scores` row: server-derived metadata copied across, a catalog
object-store copy of the bytes (its own `object_key`), `moderation_status` per role,
`proposed_by = caller`, `license` from the declaration, `source = 'user-proposal'`,
`source_url = ''`, `source_item_id = <user_scores id>`, `confidence = 'unverified'`,
`conversion_status = 'converted'`, `origin_format` from the stored file. The catalog row
is **independent** of the private row thereafter (its own object copy), so deleting the
private score does not unpublish the catalog entry — matching how a rejected soundfont
retains its row/object.

### Content dedup + re-propose guard (reopen-on-rejected, because sha256 is UNIQUE)

`catalog_scores.sha256` is `UNIQUE` — there is **at most one** catalog row per content,
so (unlike soundfonts, which keep separate rows) a re-proposal cannot insert a second
row. The propose path therefore branches on the existing row's status:

- **No existing row for this sha** → insert a fresh row (`pending`, or `accepted` for an
  admin), attributed to the proposer. First proposals need no justification.
- **Existing `pending`/`accepted` row** → refuse as a **duplicate** (report its id). The
  `user_scores.proposed_catalog_id` link also short-circuits an owner re-proposing their
  own still-open score with a typed already-proposed error.
- **Existing `rejected` row (same content)** → **reopen it**: transition it back to
  `pending`, re-attribute `proposed_by` to the current proposer, clear the prior
  `review_reason`, and require a non-empty **`resubmission_note`** (justification). This
  is the only schema-compatible way to honour "a rejection does not permanently block".

`user_scores.proposed_catalog_id` links a private score to its catalog row so the app can
show status and the guard can fire without a content scan.

### Rejection reason + re-proposal justification

Two human-readable notes close the moderator↔proposer loop:
- **`catalog_scores.review_reason`** — set when a moderator **rejects** (via the extended
  `SetModerationStatus`, which now carries an optional reason; a non-`rejected` decision
  clears it). Surfaced to the proposer through `ScoreRecord.rejection_reason` on their
  contributions read, so they know *why*.
- **`catalog_scores.resubmission_note`** — the proposer's mandatory justification when
  reopening a `rejected` row (`ProposeScore.resubmission_note`). Surfaced to the moderator
  on the privileged review-queue read, so they see *why it is back*.

### Distinguish user-proposed rows and attribute them by pseudo

A proposed catalog row is tagged `source = 'user-proposal'` (vs the crawler's dataset
sentinel) so the back office can visually distinguish user contributions from ingested
ones. Its `proposed_by` holds the proposer's `user_id` (a plain UUID, no cross-schema
FK — matching every other `music` table).

To show the proposer's **pseudo** in the review queue, the score/catalog gRPC service
resolves `proposed_by` → display name through the existing **`cymbra_user_port::UserPort`**
seam — the same seam `play_module` already uses to resolve profiles for leaderboards,
doubled with `MockUserPort` in tests. Resolution happens **only on the privileged
(moderator/admin) read**, and the resolved pseudo + `proposed_by` ride on `CatalogHit` as
**privileged-only fields** — populated exactly like the existing `moderation_status` /
`needs_review` moderation facets, i.e. defaulted/empty for a normal app caller so a
contributor's identity is never leaked through the public read paths.

_Alternative considered_: snapshotting the pseudo onto the catalog row at propose time.
Rejected — the token/`AuthIdentity` carries no display name (only `user_id`/roles), a
snapshot would go stale on rename, and `UserPort` is the established resolution seam.

### Public contributor credit, gated on public-profile opt-in

Beyond the privileged console attribution, an **`accepted`** user-proposed score MAY
carry a **public contributor credit** ("proposé par @pseudo") visible to any app user on
the score's cover/detail. Because this is a **public** read path, it is exposed **only
when the proposer has opted into a public profile** — reusing `UserPort::get_player_profile`,
whose `visibility` is **fail-closed `Private`** by default (and whose public-sharing
opt-in already carries the minimum-age safeguard from `public-player-profile`). When the
profile is `Private` (or has no handle/display name), the credit is simply omitted — the
score is still shown, just without attribution.

Two distinct fields on `CatalogHit`, resolved by the same port but under different gates,
so the two surfaces never bleed into each other:
- `proposer_display_name` — **privileged** (moderator/admin read only); always the pseudo,
  for audit. Never populated for a normal caller.
- `contributor_credit` — **public**; populated for any caller **only** when the score is
  `accepted`, user-proposed, and the proposer's profile is `Public`. The raw `proposed_by`
  **id** is never exposed publicly — only the opt-in display handle, as a credit.

_Rationale_: the console attribution is internal accountability (unconditional, id-based);
the public credit is player-facing recognition (conditional on the existing public-profile
opt-in). Keeping them as separate fields with separate gates prevents a private-profile
contributor's identity from leaking through the public path while still letting a moderator
do their job.

### App mirrors the SoundFont management screen

`ScoreRecord` gains `optional string proposal_status` (server-joined from the linked
catalog row: unset = not proposed, else `pending`/`accepted`/`rejected`), exactly like
`RemoteSoundFont.proposalStatus`. The contributed-scores state gains a
`proposeToPublicCatalog(id, {license, attestation})` method that calls the RPC and
optimistically tags the row `pending`; the contributions UI shows the status tag and
hides the propose action once submitted — mirroring `imported_soundfonts` /
`soundfonts_screen`.

## Risks / Trade-offs

- **Duplicate object storage** (private copy + catalog copy of the same bytes) → Accepted;
  it matches the soundfont propose model and keeps the catalog entry independent for
  audit. A future dedup by `sha256` at the object layer is out of scope.
- **A proposer deletes their private score after proposing** → the catalog entry (and its
  own object copy) survives as the moderation record; the app simply loses the local
  link/status tag. Acceptable and intentional (audit trail parity with soundfonts).
- **Licence/attestation quality** → the propose-time attestation is a declaration, not a
  verification; validation remains the moderator's job through the existing review queue.
  This is the same posture as the soundfont propose.
- **Same work proposed by two users** → the content-dedup guard refuses the second one,
  pointing at the existing catalog id; the second user is not blocked from keeping it in
  their private library.

## Migration Plan

- Add `backend/music/migrations/0014_score_catalog_proposal.sql` (additive, idempotent,
  fully-qualified): `catalog_scores.proposed_by UUID` (nullable; NULL for crawler rows)
  and `user_scores.proposed_catalog_id UUID` (nullable) + a supporting index. No backfill
  needed — no existing private score is proposed.
- Regenerate the flutter_rust_bridge / gRPC stubs after the proto change
  (`ProposeScore` + `ScoreRecord.proposal_status`).
- Rollback: the migration is additive and reversible at the schema level; the new RPC is
  additive (old clients simply never call it).

## Open Questions

_Both prior open questions are now resolved:_
- **Wizard vs list entry point** → **both**: propose is reachable from the contributions
  list and as an opt-in post-upload step in the wizard, both calling the one
  `ProposeScore` RPC (see "One propose RPC, reachable from two entry points").
- **Distinguish user-proposed rows** → **yes, and attribute by pseudo**: `source =
  'user-proposal'` + `proposed_by`, with the pseudo resolved via `UserPort` on the
  privileged read (see "Distinguish user-proposed rows and attribute them by pseudo").

- Remaining: should the back office let a reviewer click the pseudo through to the
  proposer's public profile? Deferred — display-only pseudo in v1.
