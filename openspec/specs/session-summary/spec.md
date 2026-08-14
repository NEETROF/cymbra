# session-summary Specification

## Purpose
TBD - created by archiving change gamify-piano-practice. Update Purpose after archive.
## Requirements
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

### Requirement: Mistake Replay On The Horizontal Score

From the summary modal the player SHALL be able to replay the just-finished run rendered
on the **actual** horizontal scrolling-staff view (the same engraving used during play,
with the real notes, measures, clefs, and key/time signatures), with the notes they
mis-played ringed **in place on the staff** by verdict — missed onsets, mistimed
(early/late) onsets, poor-sustain notes, and wrong extra notes SHALL each be visually
distinguished from correctly-played notes. The replay SHALL be driven by the
session-result record's per-note judgments and SHALL NOT require re-playing the piece live.

The replay SHALL provide a transport (play/pause and seek) that scrubs a playhead across
the staff with synchronized audio. The audio SHALL be the **player's own performance** —
each note sounded at the pitch and time the player actually played it (with missed notes
silent) — not the score, so the player hears how they played rather than the reference. The
replay SHALL present the mistakes as a list the player can tap to jump the playhead straight
to that note. When the run had no mistakes the replay SHALL say so rather than showing an
empty list.

#### Scenario: Replay highlights mistakes
- **WHEN** the player starts the replay from the summary modal
- **THEN** the horizontal score is shown and each mis-played note is highlighted according
  to its verdict, distinct from correctly-played notes

#### Scenario: Correct notes are not flagged
- **WHEN** the replay is shown
- **THEN** notes judged `perfect`/`good` with adequate sustain are rendered without a
  mistake highlight

#### Scenario: Replay uses recorded judgments
- **WHEN** the replay runs
- **THEN** it is driven from the stored per-note judgments and does not depend on live
  input

#### Scenario: Transport plays back the player's performance
- **WHEN** the player presses play in the replay
- **THEN** a playhead advances across the real staff and the **player's own** played notes
  sound in time (missed notes stay silent, not the score), and pausing or seeking stops the
  audio

#### Scenario: Tapping a mistake jumps to it
- **WHEN** the player taps a mistake in the replay's mistake list
- **THEN** the playhead seeks to that note's position on the staff

#### Scenario: A clean run reports no mistakes
- **WHEN** the replay opens for a run with no mis-played notes
- **THEN** it shows a no-mistakes message instead of an empty mistake list

#### Scenario: A mistimed note shows early/late and its offset
- **WHEN** a mistimed note is shown in the replay mistake list
- **THEN** it states whether it was played early or late and by how many
  milliseconds (or, in Wait Mode, the reaction time in milliseconds)

#### Scenario: The summary shows the average timing tendency
- **WHEN** the summary modal shows the tempo and/or reaction sub-scores
- **THEN** each sub-score also shows the average timing tendency (mean early/late
  offset in free run, or mean reaction time in Wait Mode)

### Requirement: Local Persistence Of The Last Summary

The system SHALL persist the most recent session-result record to device-local storage
(via the injectable preferences seam) so the summary survives the modal being closed and
can be re-opened, and so the record is available for a later change to upload to the
server. Persistence SHALL go through the existing preferences seam so state and widgets
remain testable without native storage. This change SHALL NOT transmit the record to any
server.

#### Scenario: Last summary persists across modal close
- **WHEN** a session-result record has been produced and the modal is dismissed
- **THEN** the record remains available in device-local storage and the last summary can be
  re-opened

#### Scenario: Persistence uses the injectable seam
- **WHEN** a test provides a fake preferences service
- **THEN** the session summary is stored and read without touching native storage

#### Scenario: No server transmission in this change
- **WHEN** a session-result record is persisted
- **THEN** it is written only to device-local storage and is not sent to any server

