## ADDED Requirements

### Requirement: The console badges percussion work and refuses a piano audition

The review queue SHALL badge each row with the score's instrument, and the
console's Play control SHALL refuse to audition a percussion score for as long as
the console's audio path is keyboard-shaped. The two guards share one purpose:
moderators are staff, so opening the admission gate makes percussion proposals
reachable in the console immediately, while both of its evaluation surfaces still
assume a keyboard — the preview is covered by `web-notation-render`'s
unpreviewable state, and Play runs through `audio-wasm`'s hardcoded piano channel,
which would render a drum part as silence or piano nonsense. The badge makes a
percussion proposal identifiable **before** it is opened, so a moderator knows why
the preview is unavailable rather than reading a fine file as corrupt; the Play
guard SHALL present as an explicit "not auditionable yet" affordance (disabled or
refusing with a localised reason), distinct from an error. Both are interim:
`add-drum-notation-render` lifts the preview state and `add-drum-audio-channel`
lifts the Play guard — they are named here so the later changes know these guards
exist to be lifted.

#### Scenario: A percussion row is badged

- **WHEN** a moderator views the review queue or the catalog table
- **THEN** each percussion score's row shows a percussion/drums badge before the
  score is opened

#### Scenario: Play refuses a percussion score

- **WHEN** a moderator triggers Play on a percussion score
- **THEN** no audio is synthesized through the piano channel; the control shows a
  localised "not auditionable yet" state instead

#### Scenario: Keyboard rows are unaffected

- **WHEN** a moderator reviews a keyboard score
- **THEN** the row, the preview and Play behave exactly as before
