## RENAMED Requirements

- FROM: `### Requirement: A percussion score is declared unpreviewable until it can be drawn`
  TO: `### Requirement: A percussion score is drawn like any other`

## MODIFIED Requirements

### Requirement: Read-only notation rendered in the browser

The system SHALL render the notation geometry produced by the wasm module into a
**read-only** on-screen view in the browser, drawing staves, clefs, note heads,
stems, beams, accidentals, dots, and rests with SMuFL glyphs, faithfully to how the
app draws the same geometry. The rendered view MUST NOT offer any affordance to edit
the score. The SMuFL font SHALL be served same-origin (no external font fetch), and
the module SHALL be instantiable under the application's content-security policy.

The percussion carve-out introduced by `add-drums-access` is **lifted**: the
renderer now supports percussion notation (percussion clef, written-position
placement, the shared head classification, two-voice layout — see
`music-percussion-engraving`), so this requirement applies to percussion scores
like any other, exactly as that carve-out announced it would. Its "A percussion
score is not drawn" scenario is deliberately replaced by the percussion
scenario below — the inversion is this change's purpose, not an omission.

#### Scenario: Notation is drawn from the geometry

- **WHEN** a score's geometry is available
- **THEN** its notation is drawn on screen (staves, clefs, notes, stems, beams,
  accidentals, rests) rather than shown as a bytes/placeholder message

#### Scenario: Rendering is faithful to the app

- **WHEN** a score is rendered in the browser and in the app
- **THEN** the browser view reflects the same notation content and layout the app
  shows for that score, so a reviewer judges it as users will see it

#### Scenario: The rendered view is read-only

- **WHEN** a user views a rendered score
- **THEN** no control edits, reorders, or alters the notation; the view only displays

#### Scenario: A percussion score is drawn

- **WHEN** a percussion score's geometry is available
- **THEN** its notation is drawn per `music-percussion-engraving` — percussion
  clef, x-form cymbal heads, two-voice layout — with no unpreviewable state

### Requirement: A percussion score is drawn like any other

The browser notation renderer SHALL detect that a score is percussion and draw
it per `music-percussion-engraving` — percussion clef and no armature,
written-position placement, the shared head classification, two-voice
layout — instead of presenting the interim "not previewable yet" state, which
this change retires. The percussion-detection seam (`isPercussionScore` — a
percussion clef on any staff, or any note on the unpitched channel) SHALL be
kept and repurposed to **route** the score to the percussion drawing rules,
because a drum part exported without its percussion clef still needs them.

The genuinely-broken states are untouched: a percussion file that is
undecodable, unparseable or oversized SHALL show the corresponding existing
failure state, exactly as a keyboard file would — never a drum drawing of
garbage, and never the retired "not previewable yet" message.

#### Scenario: A percussion score renders in the console

- **WHEN** a moderator opens a percussion score in the console
- **THEN** its engraved notation is shown — percussion clef, written
  positions, x-form cymbal heads — and the "not previewable yet" state does
  not appear

#### Scenario: A clef-less drum export still routes to percussion drawing

- **WHEN** a score carries unpitched notes but no percussion clef declaration
- **THEN** it is detected as percussion and drawn with the percussion rules,
  not with treble-staff assumptions

#### Scenario: Broken percussion files fail like broken keyboard files

- **WHEN** a percussion score's bytes are undecodable or unparseable
- **THEN** the existing failure state for that cause is shown, not a drawing
  and not the retired unpreviewable state

#### Scenario: Keyboard scores render unchanged

- **WHEN** a moderator opens a keyboard score
- **THEN** it renders exactly as before
