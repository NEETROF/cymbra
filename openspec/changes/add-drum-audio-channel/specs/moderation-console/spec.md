## RENAMED Requirements

- FROM: `### Requirement: The console badges percussion work and refuses a piano audition`
  TO: `### Requirement: The console badges percussion work and auditions it with a drum kit`

## MODIFIED Requirements

### Requirement: The console badges percussion work and auditions it with a drum kit

The review queue SHALL badge each row with the score's instrument, and the
console's Play control SHALL audition a percussion score through the wasm
renderer on the **drum channel** with a **percussion-family font** — the
interim **Play** guard `add-drums-access` installed (and
`add-drum-notation-render` kept in place while lifting the preview one) is
lifted by exactly this rule, since the console's audio path is no longer
keyboard-shaped (`music-drum-audio`). The font offered for a percussion row
SHALL come from the console's SoundFont picker filtered to the **score's**
family, defaulting to the seeded kit; a keyboard row's picker keeps offering
keyboard-family fonts, so neither family is ever auditioned through the
other's font. When the catalog holds no accepted percussion-family font, Play
SHALL present a localised "no drum kit available" state — distinct from an
error, since the score is fine — rather than synthesizing through a keyboard
font.

The badge stays for the same reason it was introduced: it is instrument
information a moderator uses before opening a row. The console's **notation
preview** is not this change's concern — it is governed by
`web-notation-render`, and the Play lift neither depends on it nor alters it.

#### Scenario: A percussion row is badged

- **WHEN** a moderator views the review queue or the catalog table
- **THEN** each percussion score's row shows a percussion/drums badge before the
  score is opened

#### Scenario: Play auditions a percussion score with a kit

- **WHEN** a moderator triggers Play on a percussion score with an accepted
  percussion-family font available
- **THEN** the score is synthesized on the drum channel with that kit font and
  audibly plays as drums, never through a keyboard font

#### Scenario: The picker follows the score's family

- **WHEN** a moderator opens the SoundFont picker on a percussion score, then on
  a keyboard score
- **THEN** the first lists only percussion-family fonts and the second only
  keyboard-family fonts

#### Scenario: No kit font degrades to a localised state

- **WHEN** a moderator triggers Play on a percussion score while the catalog
  holds no accepted percussion-family font
- **THEN** no audio is synthesized and a localised "no drum kit available" state
  is shown instead of an error

#### Scenario: The Play lift does not touch the notation preview

- **WHEN** a moderator opens a percussion score in the console
- **THEN** the notation preview behaves exactly as `web-notation-render`
  specifies, unaffected by the Play lift — the two guards never travelled
  together

#### Scenario: Keyboard rows are unaffected

- **WHEN** a moderator reviews a keyboard score
- **THEN** the row, the preview and Play behave exactly as before
