## MODIFIED Requirements

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

## ADDED Requirements

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
