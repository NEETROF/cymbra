## MODIFIED Requirements

### Requirement: Hand Selection State

The player SHALL hold a selected-hand value with exactly three options — **left**,
**right**, and **both** — in immutable Freezed state, default to **both**, and be
changeable at runtime through a notifier method. The selection SHALL be
session-only: it is not persisted and resets to **both** on app launch and is not
required to follow the loaded piece. The staff ↔ hand mapping SHALL follow the
engine's MusicXML convention: **staff 1 is the right hand** and **staff 2 (and
above) is the left hand**.

The value SHALL apply to **keyboard scores only**. A percussion score SHALL leave
it at **both** and SHALL isolate part of a groove through per-piece focus
(`music-kit-piece-focus`) instead: a drum part is written on one staff, so the
staff mapping never applied to it, and the hands/feet reading that stood in for
it split a groove at the wrong grain — it classified the kick as a foot event, so
"hands only" removed the one piece a drummer never stops playing.

#### Scenario: Default selection is both hands
- **WHEN** the app starts
- **THEN** the selected hand is **both** and every hand's notes are shown

#### Scenario: Selection changed at runtime
- **WHEN** the user selects Left, Right, or Both
- **THEN** the state updates to that value via `copyWith` and the views re-render
  against the new selection

#### Scenario: Selection is session-only
- **WHEN** the app is relaunched after the user selected a single hand
- **THEN** the selected hand is **both** again (no persisted value is restored)

#### Scenario: A percussion score is unaffected by the value
- **WHEN** a percussion score is loaded
- **THEN** the selected-hand value stays **both** and does not filter anything

### Requirement: Hand Selector Control

The player screen SHALL expose a hand selector reachable from a settings control
in its top bar that is available in all three render modes (Synthesia, Staff,
Partition). Opening the settings SHALL present the hand choices — Left, Right,
Both — for a keyboard piece that has a second staff; the control SHALL reflect
the current selection and SHALL change it through the notifier method. The hand
selector MAY be hidden for single-staff pieces, where isolating a hand is
meaningless and the default Both applies.

For a **percussion** score the hand selector SHALL NOT be offered; the focus
control of `music-kit-piece-focus` takes its place.

#### Scenario: Selector reachable in every mode
- **WHEN** a keyboard score is shown in Synthesia, Staff, or Partition mode
- **THEN** the settings control is present and the hand selector can be reached
  from it, showing the current selection

#### Scenario: Choosing a hand updates state
- **WHEN** the user picks Left (or Right, or Both) from the selector
- **THEN** the selected-hand state becomes that value and the selector reflects it

#### Scenario: Hidden for single-staff pieces
- **WHEN** the loaded piece has only one staff
- **THEN** the hand selector is not offered and the selection stays at Both

#### Scenario: Not offered for a drum part
- **WHEN** a percussion score is loaded
- **THEN** the hand selector is not offered and the focus control is

### Requirement: Hand Visibility Filter

In every render mode, the notes belonging to an unselected hand SHALL be excluded
from what is drawn: when **right** is selected only staff-1 notes are shown, when
**left** is selected only staff-2+ notes are shown, and when **both** is selected
all notes are shown. In Synthesia mode only the selected hand's falling-note
columns SHALL be drawn; in Staff and Partition modes the selected hand's notes
SHALL be drawn and the unselected hand's notes SHALL NOT appear.

The filter SHALL apply to keyboard scores only. A percussion score's visibility
is governed by `music-kit-piece-focus`.

#### Scenario: Right hand only in Synthesia
- **WHEN** the selection is **right** in Synthesia mode
- **THEN** only staff-1 note columns fall and staff-2 notes are not drawn

#### Scenario: Left hand only in Staff mode
- **WHEN** the selection is **left** in Staff mode
- **THEN** only staff-2 notes are drawn and staff-1 notes are not drawn

#### Scenario: Both hands show everything
- **WHEN** the selection is **both**
- **THEN** notes from every staff are drawn in the active mode

#### Scenario: A drum part ignores the staff filter
- **WHEN** a percussion score is drawn
- **THEN** every note in focus is drawn regardless of the selected-hand value
