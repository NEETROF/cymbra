## MODIFIED Requirements

### Requirement: Hidden Hand Excluded From Gate

The required-notes gate SHALL include only notes of the selected hand(s) — the
gate drives Wait Mode and the keyboard's expected/correct feedback. A note
belonging to an unselected hand SHALL NOT be awaited, SHALL NOT mark its key as
expected, and SHALL NOT block Wait Mode from advancing.

For a **percussion** score the same rule applies over the hands/feet
classification (`hand-color-coding`, as re-keyed by this capability's selection
requirements): selecting **hands** removes every foot event from the required
set — the kick bar and the pedal hi-hat are neither awaited nor expected, so
Wait Mode never blocks on a foot the player has hidden — and selecting **feet**
removes every hand event likewise. Scoring judges only the selected side and
records the selection (`performance-scoring`), so the gate and the judgment
always split the score the same way.

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

#### Scenario: Hiding the feet un-awaits the kick

- **WHEN** a percussion score is loaded, the selection is **hands**, and the
  playhead reaches an onset where only a kick is required
- **THEN** the required set at that onset is empty and Wait Mode advances without
  waiting for the foot

#### Scenario: A feet-only run gates and judges only the feet

- **WHEN** a percussion score is loaded with **feet** selected and a scored run
  is active
- **THEN** only foot events gate Wait Mode and are judged, and the run's result
  records the feet selection
