## MODIFIED Requirements

### Requirement: End-Of-Song Summary Modal

When a scored run reaches the end of the piece, the system SHALL present a summary modal
built from that run's session-result record. The modal SHALL show the overall
synchronization percentage, a per-dimension breakdown (timing, correct notes, sustain),
the best combo/streak, and a count of onsets by verdict (e.g. perfect / good / missed /
wrong). The modal SHALL require the player to make an **explicit choice** — see the
mistakes (replay), retry the piece, or quit via a close cross — and SHALL NOT dismiss on a
tap outside it or a back gesture. The modal SHALL keep its action controls reachable on
small/short viewports (the stats scroll while the replay/retry buttons and the close cross
stay visible), so the player can always close or retry.

When the piece just played is eligible for rating, the modal SHALL additionally carry the
rating affordance for that score. That affordance is **not** one of the explicit choices:
using it MUST NOT dismiss the modal, ignoring it MUST NOT block any of the three actions,
and it MUST NOT displace the action controls on a short viewport.

#### Scenario: Modal appears at song end
- **WHEN** a scored run reaches the end of the piece
- **THEN** the summary modal is shown with the overall percentage, per-dimension
  breakdown, best combo, and per-verdict counts

#### Scenario: Modal not shown for an unscored run
- **WHEN** playback reaches the end of the piece while no scored run was active (e.g. the
  run was cancelled, or playback resumed past a finished run without restarting)
- **THEN** no summary modal is shown

#### Scenario: Mixed run shows both sub-scores
- **WHEN** the finished run is classified `mixed` (some onsets Wait-Mode-on, some off)
- **THEN** the modal shows both the tempo (free-run) and reaction (Wait-Mode) sub-scores,
  each labelled by its mode, in addition to the overall percentage

#### Scenario: Pure run shows its single sub-score
- **WHEN** the finished run is classified `free` or `wait`
- **THEN** the modal shows the one relevant sub-score and does not show an empty other-mode score

#### Scenario: The modal does not auto-dismiss
- **WHEN** the player taps outside the summary modal or triggers a back gesture
- **THEN** the modal stays open and awaits an explicit see-mistakes / retry / quit choice

#### Scenario: The close cross leaves play mode
- **WHEN** the player taps the close cross on the summary modal
- **THEN** the modal closes and the app leaves the player, returning to the previous screen

#### Scenario: Actions stay reachable on a short viewport
- **WHEN** the summary modal is shown on a short (e.g. phone-landscape) viewport
- **THEN** the statistics scroll within the modal and the replay/retry buttons and close
  cross remain visible and tappable

#### Scenario: The rating affordance rides along when the score is eligible
- **WHEN** the summary modal is shown for a run on a score eligible for rating
- **THEN** the modal also offers to rate that score, and the replay/retry buttons and
  close cross remain visible and tappable on a short viewport

#### Scenario: An ineligible score leaves the modal unchanged
- **WHEN** the summary modal is shown for a run on a score that is not eligible for rating
- **THEN** the modal shows exactly the statistics and the three explicit actions, with no
  rating affordance
