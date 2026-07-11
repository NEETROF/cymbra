## ADDED Requirements

### Requirement: End-Of-Song Summary Modal

When a scored run reaches the end of the piece, the system SHALL present a summary modal
built from that run's session-result record. The modal SHALL show the overall
synchronization percentage, a per-dimension breakdown (timing, correct notes, sustain),
the best combo/streak, and a count of onsets by verdict (e.g. perfect / good / missed /
wrong). The modal SHALL offer actions to replay the run, retry the piece, and dismiss.

#### Scenario: Modal appears at song end
- **WHEN** a scored run reaches the end of the piece
- **THEN** the summary modal is shown with the overall percentage, per-dimension
  breakdown, best combo, and per-verdict counts

#### Scenario: Modal not shown for an unscored run
- **WHEN** playback reaches the end of the piece while no scored run was active (e.g. the
  engraved Partition view was showing)
- **THEN** no summary modal is shown

#### Scenario: Mixed run shows both sub-scores
- **WHEN** the finished run is classified `mixed` (some onsets Wait-Mode-on, some off)
- **THEN** the modal shows both the tempo (free-run) and reaction (Wait-Mode) sub-scores,
  each labelled by its mode, in addition to the overall percentage

#### Scenario: Pure run shows its single sub-score
- **WHEN** the finished run is classified `free` or `wait`
- **THEN** the modal shows the one relevant sub-score and does not show an empty other-mode score

#### Scenario: Dismiss returns to the player
- **WHEN** the player dismisses the summary modal
- **THEN** the player view is restored with the piece reset for another attempt

### Requirement: Mistake Replay On The Horizontal Score

From the summary modal the player SHALL be able to replay the just-finished run rendered
on the horizontal scrolling-staff view, with the notes they mis-played highlighted by
verdict — missed onsets, mistimed (early/late) onsets, poor-sustain notes, and wrong
extra notes SHALL each be visually distinguished from correctly-played notes. The replay
SHALL be driven by the session-result record's per-note judgments and SHALL NOT require
re-playing the piece live.

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
