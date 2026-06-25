## ADDED Requirements

### Requirement: Hand Selection State

The player SHALL hold a selected-hand value with exactly three options — **left**,
**right**, and **both** — in immutable Freezed state, default to **both**, and be
changeable at runtime through a notifier method. The selection SHALL be
session-only: it is not persisted and resets to **both** on app launch and is not
required to follow the loaded piece. The staff ↔ hand mapping SHALL follow the
engine's MusicXML convention: **staff 1 is the right hand** and **staff 2 (and
above) is the left hand**.

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

### Requirement: Hand Selector Control

The player screen SHALL present a hand selector control in its top bar, available
in all three render modes (Synthesia, Staff, Partition), that lets the user pick
Left, Right, or Both. The control SHALL reflect the current selection and SHALL
change it through the notifier method, following the existing top-bar chooser
pattern.

#### Scenario: Selector available in every mode
- **WHEN** the player screen is shown in Synthesia, Staff, or Partition mode
- **THEN** the hand selector is present and shows the current selection

#### Scenario: Choosing a hand updates state
- **WHEN** the user picks Left (or Right, or Both) from the selector
- **THEN** the selected-hand state becomes that value and the selector reflects it

### Requirement: Hand Visibility Filter

In every render mode, the notes belonging to an unselected hand SHALL be excluded
from what is drawn: when **right** is selected only staff-1 notes are shown, when
**left** is selected only staff-2+ notes are shown, and when **both** is selected
all notes are shown. In Synthesia mode only the selected hand's falling-note
columns SHALL be drawn; in Staff and Partition modes the selected hand's notes
SHALL be drawn and the unselected hand's notes SHALL NOT appear.

#### Scenario: Right hand only in Synthesia
- **WHEN** the selection is **right** in Synthesia mode
- **THEN** only staff-1 note columns fall and staff-2 notes are not drawn

#### Scenario: Left hand only in Staff mode
- **WHEN** the selection is **left** in Staff mode
- **THEN** only staff-2 notes are drawn and staff-1 notes are not drawn

#### Scenario: Both hands show everything
- **WHEN** the selection is **both**
- **THEN** notes from every staff are drawn in the active mode

### Requirement: Hidden Hand Excluded From Gate

The required-notes gate SHALL include only notes of the selected hand(s) — the
gate drives Wait Mode and the keyboard's expected/correct feedback. A note
belonging to an unselected hand SHALL NOT be awaited, SHALL NOT mark its key as
expected, and SHALL NOT block Wait Mode from advancing.

#### Scenario: Hidden hand's notes are not awaited
- **WHEN** the selection is **right** and the playhead reaches a position where
  only a staff-2 (left-hand) note is required
- **THEN** the required-notes set at that position is empty and Wait Mode advances
  without waiting for that note

#### Scenario: Hidden hand's key is never expected
- **WHEN** the selection is **right** and a staff-2 note is at the playhead
- **THEN** its key is not shown in the expected/press-this state

#### Scenario: Selected hand still gates
- **WHEN** the selection is **right** and a staff-1 note is required at the
  playhead
- **THEN** that note is in the required set and Wait Mode waits for it as usual
