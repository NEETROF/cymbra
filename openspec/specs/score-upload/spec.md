# score-upload Specification

## Purpose
TBD - created by archiving change add-user-score-upload. Update Purpose after archive.
## Requirements
### Requirement: Authenticated-only contribution flow

The app SHALL expose a score contribution flow only to a signed-in user. When no
user is authenticated, the contribution entry point MUST be unavailable (hidden
or disabled) and the flow MUST NOT be reachable.

#### Scenario: Entry point hidden when signed out

- **WHEN** no user is authenticated
- **THEN** the contribution entry point is not available and the contribution
  screen cannot be opened

#### Scenario: Entry point available when signed in

- **WHEN** a user is authenticated
- **THEN** the contribution entry point is available and opens the contribution
  screen

### Requirement: Three-step contribution wizard

The contribution screen SHALL guide the user through three ordered steps —
**Upload**, **Verification**, **Confirmation** — and SHALL NOT allow advancing to
a later step until the current step's requirements are satisfied. The flow SHALL
be exposed through injectable state so it can be driven in tests without the
native library or a live backend.

#### Scenario: Steps are ordered and gated

- **WHEN** the user is on the Upload step without a validated file
- **THEN** the Verification and Confirmation steps are not reachable

#### Scenario: Advancing after each step's requirements are met

- **WHEN** the current step's requirements are satisfied
- **THEN** the user can advance to the next step, and can return to a previous
  step to change their input

### Requirement: Accept plain and zipped MusicXML

The Upload step SHALL let the user pick a single file that is either an
uncompressed MusicXML file (`.musicxml` / `.xml`) or a zipped MusicXML container
(`.mxl`). A `.mxl` file MUST be decoded to its underlying MusicXML before
validation. Files of other types MUST be rejected.

#### Scenario: Plain MusicXML accepted

- **WHEN** the user picks a `.musicxml` or `.xml` file
- **THEN** the file is accepted for validation

#### Scenario: Zipped MusicXML decoded

- **WHEN** the user picks a `.mxl` file
- **THEN** the container is decoded to its MusicXML payload before validation

#### Scenario: Unsupported file rejected

- **WHEN** the user picks a file that is neither MusicXML nor a MusicXML zip
- **THEN** the file is rejected with a message and the flow does not advance

### Requirement: Client-side validation

The Upload step SHALL validate the selected file locally, using the shared
MusicXML decoding/parsing seam, before any upload. Validation MUST reject a file
that cannot be decoded, cannot be parsed as MusicXML, or contains no playable
notes, and MUST surface a reason. A **percussion** score contains playable notes —
its notes are unpitched rather than pitched — and MUST therefore pass this check
rather than be rejected as empty. Only a locally-validated file MAY proceed
to Verification.

#### Scenario: Valid file proceeds

- **WHEN** the selected file decodes, parses, and contains playable notes
- **THEN** validation passes and the Verification step becomes available

#### Scenario: Corrupt or empty file rejected

- **WHEN** the selected file cannot be decoded, cannot be parsed, or contains no
  playable notes
- **THEN** validation fails with a reason and the flow does not advance

#### Scenario: Percussion score is no longer rejected as empty

- **WHEN** the selected file is a drum score whose notes are all unpitched
- **THEN** the playable-notes check passes, instead of rejecting it for containing
  none

### Requirement: Mandatory rights attestation before upload

Before a validated file can be submitted, the user MUST declare the **basis** on
which they may contribute it — that they are its **author**, that it is in the
**public domain** (or under a free licence permitting its use), or that they
import it **for their strictly personal use** (`private_use`) — via a distinct
choice, AND MUST tick a **single confirmation checkbox** that attests the
declaration is accurate (e.g. "Je certifie que cette déclaration est exacte et que
je dispose des droits nécessaires pour mettre cette partition à disposition"),
localised per `app-localization`. The `private_use` option MUST make clear, in
its localised label or helper text, that such a score remains private forever
and can never be shared or proposed to the public catalog. The confirmation
MUST NOT restate the basis options (the basis choice already captures which
applies). The submit action MUST remain disabled until a basis is selected and
the checkbox is ticked, and both the declared basis and the confirmation MUST be
sent with the upload.

#### Scenario: Submit blocked without the attestation

- **WHEN** no rights basis is selected or the confirmation checkbox is not ticked
- **THEN** the upload cannot be submitted

#### Scenario: Attestation enables and accompanies submit

- **WHEN** the user selects a rights basis and ticks the confirmation checkbox
- **THEN** submit becomes available and both the declared basis and the
  confirmation are included in the upload request

#### Scenario: Personal-use basis is offered and explained

- **WHEN** the attestation step is shown
- **THEN** a third "strictly personal use" basis is available alongside author
  and public domain, with localised copy stating the score can never be shared
  or proposed to the public catalog

### Requirement: Verification preview is horizontal and tempo-locked

The Verification step SHALL render the validated score as an engraved partition
in a **horizontal layout only**, and SHALL allow playback **only at the score's
own tempo**. It MUST NOT expose tempo control, practice/wait modes, hand
isolation, or other player controls beyond plain play/pause of the score at its
notated tempo, so the user can confirm the decoded score is correct.

#### Scenario: Horizontal-only engraved preview

- **WHEN** the Verification step is shown for a validated score
- **THEN** the score is displayed as a horizontal engraved partition

#### Scenario: Playback locked to score tempo

- **WHEN** the user plays the score in the Verification step
- **THEN** it plays at the score's notated tempo and no tempo or practice
  controls are offered

### Requirement: Derived metadata shown read-only before upload

Before the upload is finalized, the flow SHALL display the score's descriptive
metadata parsed from the file — at least its title, composer, key, time signature,
and measure count — so the user can review exactly what will be stored. These
fields MUST be presented **read-only** when the file provides them: the user MUST
NOT be able to edit a value the file already carries. As the single exception, when
the file carries **no** title (resp. composer), the flow MAY offer an editable
**fallback** field for it, sent as a fallback the server uses only because the file
lacks that value (a parsed value always wins). Musical fields (key, time, measures)
are always read-only. The values
shown come from the same shared parsing seam used for validation, so they match
what the server derives and stores.

#### Scenario: Parsed metadata is displayed for review

- **WHEN** a validated score reaches the Verification or Confirmation step
- **THEN** its parsed title, composer, and musical metadata are shown to the user
  for review

#### Scenario: Metadata fields are not editable

- **WHEN** the parsed metadata is shown
- **THEN** the user cannot edit those fields and no user-entered metadata is sent
  with the upload

### Requirement: Mandatory difficulty selection before confirm

The Confirmation step SHALL require the user to choose exactly one difficulty
level from the fixed set — Beginner, Intermediate, Advanced — before the upload
can be finalized. Finalizing MUST be blocked until a level is chosen, and the
chosen level MUST be sent with the upload.

#### Scenario: Confirm blocked without a difficulty

- **WHEN** no difficulty level has been chosen
- **THEN** the upload cannot be finalized

#### Scenario: Chosen difficulty accompanies the upload

- **WHEN** the user selects a difficulty level and confirms
- **THEN** the upload is submitted with that level

### Requirement: Submit contribution to the backend

Finalizing the flow SHALL submit the (decoded) score bytes together with the
chosen difficulty and the rights attestation (declared basis + confirmation) to the
backend through an
injectable service, and SHALL reflect success or a typed failure to the user
without losing the user's inputs on a recoverable error. The backend client MUST
be overridable with a fake in tests.

#### Scenario: Successful submission

- **WHEN** the backend accepts the contribution
- **THEN** the flow reports success and the score is recorded as the user's
  contribution

#### Scenario: Server rejection surfaced

- **WHEN** the backend rejects the contribution (e.g. server-side validation
  failure)
- **THEN** the flow surfaces the reason and the user can correct and retry
  without re-entering unrelated inputs

### Requirement: Owner lists and deletes their contributions

A signed-in user SHALL be able to see the scores they have contributed and delete
any of them. Deleting a contribution MUST request its removal from the backend
and reflect the result; the user MUST NOT be offered deletion of scores they do
not own or of bundled-catalog scores. For each contributed score, the list SHALL
also surface its **public-catalog proposal state** — not proposed, `pending`,
`accepted`, or `rejected` — and, for a not-yet-proposed score, offer the opt-in
"propose to the public catalog" action (see `score-catalog-proposal`). A
contribution that has not been proposed remains private; the list is the entry point
to the proposal flow, and once a score is proposed the propose action is hidden for
it.

#### Scenario: Owner sees their contributions

- **WHEN** a signed-in user opens their contributed scores
- **THEN** only that user's contributions are listed

#### Scenario: Owner deletes a contribution

- **WHEN** the owner deletes one of their contributions
- **THEN** the app requests deletion from the backend and, on success, removes it
  from the list

#### Scenario: No delete affordance for non-owned scores

- **WHEN** a bundled-catalog score or a score the user does not own is shown
- **THEN** no delete affordance is offered for it

#### Scenario: Contribution shows its proposal state

- **WHEN** a contribution has been proposed to the public catalog
- **THEN** the list shows its proposal state (`pending`/`accepted`/`rejected`) and does
  not offer the propose action for it

#### Scenario: Not-yet-proposed contribution offers propose

- **WHEN** a contribution has not been proposed
- **THEN** the list marks it as private/not-proposed and offers the opt-in propose action

### Requirement: The detected instrument is shown, never declared

The Upload flow SHALL display the **instrument detected by the parse** in the
read-only summary the contributor reviews before submitting, and SHALL NOT offer
any control to declare or override it. The instrument is a derived facet like the
key or the time signature: a contributor can no more assert it than they can assert
the score's note count, and accepting a declared value would open the same spoofing
surface the upload flow already closes for title and composer — with more
consequence, since the instrument decides which renderer and which synthesizer
channel the score gets.

#### Scenario: Detected instrument is displayed

- **WHEN** a contributor validates a score locally
- **THEN** the read-only summary shows the instrument the parse detected

#### Scenario: Instrument cannot be declared

- **WHEN** a contributor reviews the summary
- **THEN** no control lets them set or change the instrument

### Requirement: A percussion upload requires the drum audience

The Upload flow SHALL refuse a percussion score from a contributor to whom the drum
feature is not visible, surfacing a localised reason like any other validation
refusal, and the **backend SHALL refuse the same upload** independently of the
client (see `music-drums-visibility`). Without the client-side refusal a
contributor would be told to proceed only to be rejected on submit; without the
backend refusal the restriction would not hold at all.

#### Scenario: Eligible contributor may upload a drum score

- **WHEN** a contributor for whom the drum feature is visible submits a percussion
  score
- **THEN** the flow advances and the backend accepts it

#### Scenario: Ineligible contributor is refused in the app

- **WHEN** a contributor for whom the drum feature is not visible validates a
  percussion score
- **THEN** the flow does not advance and a localised reason is surfaced

#### Scenario: Ineligible upload is refused by the backend too

- **WHEN** an ineligible caller submits a percussion score directly, bypassing the
  app
- **THEN** the backend refuses it

#### Scenario: Keyboard uploads are unaffected

- **WHEN** any contributor uploads a keyboard score
- **THEN** the drum gate does not apply and the flow behaves exactly as before

### Requirement: No public-catalog affordance for personal-use scores

The app SHALL omit, in any owner-facing listing of the user's own scores, every
affordance that initiates sharing or a proposal to the public catalog for a
score whose rights basis is `private_use`. Scores with the author or
public-domain basis SHALL keep the affordances they have today.

#### Scenario: Personal-use score shows no propose affordance

- **WHEN** the owner views a score imported with the `private_use` basis
- **THEN** no share/propose-to-catalog affordance is shown for that score

#### Scenario: Other bases unaffected

- **WHEN** the owner views a score with the author or public-domain basis
- **THEN** its existing affordances are unchanged

