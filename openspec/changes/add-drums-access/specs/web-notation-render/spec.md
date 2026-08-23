## MODIFIED Requirements

### Requirement: Read-only notation rendered in the browser

The system SHALL render the notation geometry produced by the wasm module into a
**read-only** on-screen view in the browser, drawing staves, clefs, note heads,
stems, beams, accidentals, dots, and rests with SMuFL glyphs, faithfully to how the
app draws the same geometry. The rendered view MUST NOT offer any affordance to edit
the score. The SMuFL font SHALL be served same-origin (no external font fetch), and
the module SHALL be instantiable under the application's content-security policy.

**Percussion scores are carved out** of this obligation until the renderer supports
percussion notation: a percussion score's geometry is available (it parses and lays
out), but drawing it with the renderer's pitched treble/bass assumptions would be a
confident wrong rendering, so the unpreviewable state defined below applies instead.
Once percussion drawing lands (`add-drum-notation-render`) the carve-out disappears
and this requirement applies to percussion scores like any other.

#### Scenario: Notation is drawn from the geometry

- **WHEN** a non-percussion score's geometry is available
- **THEN** its notation is drawn on screen (staves, clefs, notes, stems, beams,
  accidentals, rests) rather than shown as a bytes/placeholder message

#### Scenario: Rendering is faithful to the app

- **WHEN** a score is rendered in the browser and in the app
- **THEN** the browser view reflects the same notation content and layout the app
  shows for that score, so a reviewer judges it as users will see it

#### Scenario: The rendered view is read-only

- **WHEN** a user views a rendered score
- **THEN** no control edits, reorders, or alters the notation; the view only displays

#### Scenario: A percussion score is not drawn

- **WHEN** a percussion score's geometry is available but the renderer lacks
  percussion support
- **THEN** the unpreviewable state is shown instead of a drawing

## ADDED Requirements

### Requirement: A percussion score is declared unpreviewable until it can be drawn

The browser notation renderer SHALL detect that a score is percussion and present an
explicit "not previewable yet" state instead of drawing it, for as long as the
renderer lacks percussion support (percussion clef, alternative noteheads,
two-voice layout on a single staff).

This matters because opening the admission gate makes drum scores reachable in the
console immediately — moderators are staff, so they are inside the drum audience by
construction — while the renderer still assumes pitched notation on a treble or
bass staff. Drawing a drum score with those assumptions produces a confident,
wrong-looking rendering that a moderator would reasonably read as a corrupt file
and reject. An explicit refusal is honest; a plausible wrong drawing is not.

The state SHALL be distinguishable from the existing failure states (undecodable,
unparseable, oversized), because the cause and the remedy differ: the file is fine,
the renderer is not ready.

#### Scenario: Percussion score shows the unpreviewable state

- **WHEN** a moderator opens a percussion score in the console
- **THEN** the preview shows an explicit "not previewable yet" state rather than a
  rendering

#### Scenario: The state is not a parse failure

- **WHEN** that state is shown
- **THEN** it is distinguishable from an undecodable or unparseable file, so the
  score is not mistaken for corrupt

#### Scenario: Keyboard scores render unchanged

- **WHEN** a moderator opens a keyboard score
- **THEN** it renders exactly as before
