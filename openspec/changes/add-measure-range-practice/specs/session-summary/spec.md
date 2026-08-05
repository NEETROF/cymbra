## MODIFIED Requirements

### Requirement: End-Of-Song Summary Modal

When a scored run reaches the end of the piece, the system SHALL present a summary modal
built from that run's session-result record. The modal SHALL show the overall
synchronization percentage, a per-dimension breakdown (timing, correct notes, sustain),
the best combo/streak, and a count of onsets by verdict (e.g. perfect / good / missed /
wrong). The modal SHALL require the player to make an **explicit choice** — see the
mistakes (replay), **practice a section**, retry the piece, or quit via a close cross — and
SHALL NOT dismiss on a tap outside it or a back gesture. The **practice a section** action SHALL
open the measure-range picker and start a selective (unscored) practice run of the chosen range.
The modal SHALL keep its action controls reachable on small/short viewports (the stats scroll
while the action buttons and the close cross stay visible), so the player can always close,
retry, or start practicing a section.

#### Scenario: Modal appears at song end
- **WHEN** a scored run reaches the end of the piece
- **THEN** the summary modal is shown with the overall percentage, per-dimension
  breakdown, best combo, and per-verdict counts

#### Scenario: Modal not shown for an unscored run
- **WHEN** playback reaches the end of the piece while no scored run was active (e.g. the
  run was cancelled, a selective/practice run, or playback resumed past a finished run without
  restarting)
- **THEN** no summary modal is shown

#### Scenario: Practice-a-section opens the range picker
- **WHEN** the player taps "practice a section" on the summary modal
- **THEN** the measure-range picker opens and choosing a range starts a selective (unscored)
  practice run of that range

#### Scenario: Mixed run shows both sub-scores
- **WHEN** the finished run is classified `mixed` (some onsets Wait-Mode-on, some off)
- **THEN** the modal shows both the tempo (free-run) and reaction (Wait-Mode) sub-scores,
  each labelled by its mode, in addition to the overall percentage

#### Scenario: Pure run shows its single sub-score
- **WHEN** the finished run is classified `free` or `wait`
- **THEN** the modal shows the one relevant sub-score and does not show an empty other-mode score

#### Scenario: The modal does not auto-dismiss
- **WHEN** the player taps outside the summary modal or triggers a back gesture
- **THEN** the modal stays open and awaits an explicit see-mistakes / practice / retry / quit choice

#### Scenario: The close cross leaves play mode
- **WHEN** the player taps the close cross on the summary modal
- **THEN** the modal closes and the app leaves the player, returning to the previous screen

#### Scenario: Actions stay reachable on a short viewport
- **WHEN** the summary modal is shown on a short (e.g. phone-landscape) viewport
- **THEN** the statistics scroll within the modal and the action buttons and close
  cross remain visible and tappable
