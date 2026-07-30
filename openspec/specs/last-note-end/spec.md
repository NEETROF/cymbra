# last-note-end Specification

## Purpose
TBD - created by archiving change stop-piano-at-last-note. Update Purpose after archive.
## Requirements
### Requirement: Effective End Trims Trailing Silence

The effective end of a piece SHALL be derived from the last sounding note of the
**currently selected hand(s)** — the largest note end (onset plus duration) among the
notes actually shown and played for that selection — so that trailing rests and empty
trailing measures after the last note are skipped. Trailing rests SHALL NOT count as
sounding notes and SHALL NOT extend the effective end.

A piece that contains no sounding notes for the current selection SHALL fall back to an
effective end equal to the raw song end (`songEndMs`), preserving today's behaviour.
Rests or empty measures that occur **before** the last note SHALL NOT move the effective
end earlier. The effective end SHALL never exceed the raw song end and SHALL be greater
than the effective start whenever at least one sounding note exists.

The set of notes that are played or judged SHALL NOT change; only where the run ends does.

#### Scenario: Trailing empty measures are skipped
- **WHEN** a piece's last sounding note resolves in the 5th measure but the piece has three
  more empty measures of rests afterward, and playback reaches the end
- **THEN** the run ends when the last note resolves, so the trailing empty measures are not
  played through

#### Scenario: Effective end is the last note's resolution
- **WHEN** the effective end is computed for a piece with trailing rests
- **THEN** the end position is the maximum of (note onset + note duration) across the
  selection's sounding notes, ignoring any trailing rests

#### Scenario: A selection with no notes falls back to the raw song end
- **WHEN** the current hand selection has no sounding notes
- **THEN** the effective end equals the raw song end and behaviour is unchanged

#### Scenario: Earlier rests do not move the end
- **WHEN** a piece ends on a note but contains rests or an empty measure partway through
- **THEN** the effective end stays at the last note's resolution (only trailing silence
  after the last note is trimmed)

### Requirement: End-Of-Song Transport Honours The Effective End

Every end-of-song transport decision SHALL key off the effective end rather than the raw
`songEndMs`: a scored run SHALL finish (produce its summary and pause) when the playhead
reaches the effective end, and an unscored run SHALL loop back to the effective start when
the playhead reaches the effective end. Because the end is derived from the selected
hand(s), a hand-selection change SHALL recompute the effective end for the new selection.

#### Scenario: Scored run finishes at the last note
- **WHEN** a scored run is active and the playhead reaches the effective end of a piece with
  trailing rests
- **THEN** the run finishes and its summary is produced right after the last note resolves,
  not after the trailing silence

#### Scenario: Unscored run loops at the last note
- **WHEN** an unscored run reaches the effective end of a piece with trailing rests
- **THEN** the playhead wraps back to the effective start, skipping both the trailing and the
  leading silence

#### Scenario: Changing hands recomputes the effective end
- **WHEN** the selected hand(s) change
- **THEN** the effective end is recomputed from the new selection's last sounding note

### Requirement: Effective End Applies In Every Render Mode And Wait-Mode State

Stopping at the effective end SHALL apply in all three render modes — waterfall (Synthesia),
the horizontal scrolling staff, and the engraved vertical Partition — and with Wait Mode
either on or off. In Wait Mode the run SHALL complete once the last note is resolved rather
than gating through the trailing rests.

#### Scenario: Trimmed end in every render mode
- **WHEN** playback reaches the end of a piece with trailing rests, in any render mode
  (Synthesia, scrolling staff, or Partition)
- **THEN** the run ends at the same effective end (the last note's resolution) in each mode

#### Scenario: Wait Mode completes at the last note
- **WHEN** Wait Mode is on, the piece has trailing rests, and the last note has been resolved
- **THEN** the run completes at the last note without gating through the trailing rests

