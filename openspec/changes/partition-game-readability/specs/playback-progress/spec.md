# playback-progress delta — partition-game-readability

## ADDED Requirements

### Requirement: Playback Progress Bar

The player SHALL display a thin, full-width progress bar directly above the
on-screen keyboard representing the loaded piece's total duration, with a fill
proportional to the current playback position. The bar SHALL be shown in every
render mode (Synthesia, scrolling staff, Partition); when the on-screen
keyboard is hidden it SHALL sit at the bottom of the render area. Its fill
SHALL advance as the playhead advances, freeze when playback is paused or Wait
Mode freezes the playhead (it reads the same position), and reset when playback
restarts from the top. The bar SHALL be hidden when no timed score is loaded
(no known duration), and SHALL be purely informative (not a seek control), so
it never intercepts keyboard or gesture input.

#### Scenario: Bar reflects the playback position
- **WHEN** a timed score is loaded and playback has advanced partway
- **THEN** the bar above the keyboard is filled by the elapsed fraction of the
  piece's duration

#### Scenario: Bar advances with playback and freezes on pause
- **WHEN** the playhead advances, then playback is paused
- **THEN** the fill grows with the playhead and stops moving while paused

#### Scenario: Hidden without a timed score
- **WHEN** no score with a known duration is loaded
- **THEN** no progress bar is shown

#### Scenario: Shown in every render mode
- **WHEN** the user switches between Synthesia, the scrolling staff and the
  Partition
- **THEN** the progress bar remains visible above the keyboard (or at the
  bottom of the render area when the keyboard is hidden)

#### Scenario: Never intercepts input
- **WHEN** the user plays via the on-screen keyboard or gestures over the play
  surface
- **THEN** the progress bar does not capture or block any input
