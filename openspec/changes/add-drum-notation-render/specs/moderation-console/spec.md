## MODIFIED Requirements

### Requirement: The console badges percussion work and refuses a piano audition

The review queue SHALL badge each row with the score's instrument, and the
console's Play control SHALL refuse to audition a percussion score for as long
as the console's audio path is keyboard-shaped. Of the two interim guards
`add-drums-access` installed, this change (`add-drum-notation-render`) lifts
the **preview** one — a percussion score now renders in the console per
`web-notation-render` — while the **Play** guard remains: Play still runs
through `audio-wasm`'s hardcoded piano channel, which would render a drum part
as silence or piano nonsense, and `add-drum-audio-channel` is the change that
lifts it. The Play guard SHALL keep presenting as an explicit "not
auditionable yet" affordance (disabled or refusing with a localised reason),
distinct from an error. The badge stays: it is instrument information a
moderator uses before opening a row, and it now explains why **Play** — no
longer the preview — is unavailable on a percussion row.

#### Scenario: A percussion row is badged

- **WHEN** a moderator views the review queue or the catalog table
- **THEN** each percussion score's row shows a percussion/drums badge before the
  score is opened

#### Scenario: Play refuses a percussion score

- **WHEN** a moderator triggers Play on a percussion score
- **THEN** no audio is synthesized through the piano channel; the control shows a
  localised "not auditionable yet" state instead

#### Scenario: The preview and the Play guard are now independent

- **WHEN** a moderator opens a percussion score in the console
- **THEN** the notation preview renders while Play still refuses — the lifted
  guard and the remaining one do not travel together

#### Scenario: Keyboard rows are unaffected

- **WHEN** a moderator reviews a keyboard score
- **THEN** the row, the preview and Play behave exactly as before
