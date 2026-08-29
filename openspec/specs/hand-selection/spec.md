# hand-selection Specification

## Purpose
TBD - created by archiving change hand-selection-filter. Update Purpose after archive.
## Requirements
### Requirement: Hand Selection State

The player SHALL hold a selected-hand value with exactly three options — **left**,
**right**, and **both** — in immutable Freezed state, default to **both**, and be
changeable at runtime through a notifier method. The selection SHALL be
session-only: it is not persisted and resets to **both** on app launch and is not
required to follow the loaded piece. For a keyboard score the staff ↔ hand
mapping SHALL follow the engine's MusicXML convention: **staff 1 is the right
hand** and **staff 2 (and above) is the left hand**.

For a **percussion** score the same three-valued state SHALL read as
**hands / feet / both**: a drum part is written on a single staff, so the staff
mapping cannot apply, and the split SHALL instead follow the hands/feet
classification stated normatively in `hand-color-coding` — voice 1 (stems up) is
the hands and voice 2 (stems down) the feet, with the single-voice fallback by
General MIDI number (kick 35/36 and pedal hi-hat 44 are feet, everything else
hands). **Right** selects the hands, **left** the feet, so one state machine
serves both instruments.

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

#### Scenario: The percussion selection splits by voice, not staff
- **WHEN** a percussion score is loaded and **right** (hands) is selected
- **THEN** the selection covers exactly the events the voice convention
  classifies as hands, single staff notwithstanding

### Requirement: Hand Selector Control

The player screen SHALL expose a hand selector reachable from a settings control
in its top bar that is available in all three render modes (Synthesia, Staff,
Partition). Opening the settings SHALL present the hand choices — Left, Right,
Both — for a piece that has a second staff; the control SHALL reflect the current
selection and SHALL change it through the notifier method. The hand selector MAY
be hidden for single-staff **keyboard** pieces, where isolating a hand is
meaningless and the default Both applies.

For a **percussion** score the selector SHALL be offered despite the single
staff, whenever the score contains both hand and foot events, and its choices
SHALL be labelled **hands / feet / both** rather than right/left — the split a
drummer actually practises is hands against feet, and hiding the selector on the
single-staff test would deny percussion the filter entirely.

#### Scenario: Selector reachable in every mode
- **WHEN** the player screen is shown in Synthesia, Staff, or Partition mode
- **THEN** the settings control is present and the hand selector can be reached
  from it, showing the current selection

#### Scenario: Choosing a hand updates state
- **WHEN** the user picks Left (or Right, or Both) from the selector
- **THEN** the selected-hand state becomes that value and the selector reflects it

#### Scenario: Hidden for single-staff keyboard pieces
- **WHEN** the loaded keyboard piece has only one staff
- **THEN** the hand selector is not offered and the selection stays at Both

#### Scenario: Offered for a single-staff drum part
- **WHEN** a percussion score containing hand and foot events is loaded
- **THEN** the selector is offered with hands / feet / both labels, although the
  part is written on one staff

### Requirement: Hand Visibility Filter

In every render mode, the notes belonging to an unselected hand SHALL be excluded
from what is drawn. For a keyboard score the split is by staff: when **right** is
selected only staff-1 notes are shown, when **left** is selected only staff-2+
notes are shown, and when **both** is selected all notes are shown. In Synthesia
mode only the selected hand's falling-note columns SHALL be drawn; in Staff and
Partition modes the selected hand's notes SHALL be drawn and the unselected
hand's notes SHALL NOT appear.

For a **percussion** score the filter SHALL apply the hands/feet classification
in place of the staff test: selecting **hands** draws only the hand events and
hides every foot event — **including the kick's full-width bar** — and selecting
**feet** draws the kick bar and the other foot events while hiding the hand
lanes' notes. The bar is a note in a different shape; a filter that hid foot
notes but kept the bar would hide nothing that matters.

#### Scenario: Right hand only in Synthesia
- **WHEN** the selection is **right** in Synthesia mode
- **THEN** only staff-1 note columns fall and staff-2 notes are not drawn

#### Scenario: Left hand only in Staff mode
- **WHEN** the selection is **left** in Staff mode
- **THEN** only staff-2 notes are drawn and staff-1 notes are not drawn

#### Scenario: Both hands show everything
- **WHEN** the selection is **both**
- **THEN** notes from every staff are drawn in the active mode

#### Scenario: Hiding the feet actually hides the bar
- **WHEN** a percussion score is shown and the selection is **hands**
- **THEN** no kick bar is drawn and no other foot event appears, while the hand
  notes remain

#### Scenario: Isolating the feet keeps the bar
- **WHEN** a percussion score is shown and the selection is **feet**
- **THEN** the kick bar and the other foot events are drawn, and the hand lanes
  fall empty

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

