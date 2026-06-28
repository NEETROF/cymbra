## ADDED Requirements

### Requirement: Metronome Toggle From The Tempo Chip

The player header's Tempo chip SHALL be a tappable control that toggles the
metronome on and off. The chip SHALL visibly reflect the current on/off state.
The toggle SHALL be available in all three view modes — waterfall (synthesia),
staff, and partition — because the chip lives in the shared player header. The
enabled/disabled choice is a user preference and SHALL persist while playback is
paused or stopped (it is not cleared by pausing).

#### Scenario: Tapping the chip enables the metronome
- **WHEN** the metronome is off and the user taps the Tempo chip
- **THEN** the metronome becomes enabled and the chip shows its active state

#### Scenario: Tapping again disables it
- **WHEN** the metronome is on and the user taps the Tempo chip
- **THEN** the metronome becomes disabled and the chip shows its inactive state

#### Scenario: Available in every view mode
- **WHEN** the player is in waterfall, staff, or partition mode
- **THEN** the Tempo chip toggles the metronome the same way in each

#### Scenario: Enabled state survives pause
- **WHEN** the metronome is enabled and the user pauses playback
- **THEN** the metronome stays enabled (it resumes ticking when playback resumes)

### Requirement: Beat Generation Synchronised With The Score

While the metronome is enabled and playback is running, the app SHALL produce one
beat event per beat of the measure, derived from the score's tempo and time
signature so that beats align with the measures. Beat timing SHALL follow the
playhead — the same timing that drives note playback, honouring the speed
multiplier — so the metronome does not drift from the notes. The first beat of
each measure (the downbeat) SHALL be marked as an accent; the remaining beats are
normal beats.

#### Scenario: One beat per beat of the measure
- **WHEN** playback advances across a beat boundary with the metronome enabled
- **THEN** exactly one beat event fires for that beat

#### Scenario: Downbeat is accented
- **WHEN** the playhead crosses the start of a measure
- **THEN** that beat fires as an accent and the other beats of the measure fire as
  normal beats

#### Scenario: Stays in sync with the score
- **WHEN** the speed multiplier changes during playback
- **THEN** the beat spacing changes with it so beats remain aligned to the notes

#### Scenario: Seeking or looping re-aligns the beat
- **WHEN** playback seeks, restarts, or loops to a new position
- **THEN** the next beat fires at the correct beat boundary for that position
  without a spurious extra tick at the seam

### Requirement: Audible And Visual Beat

Each beat event SHALL be expressed both audibly and visually. Audibly, the beat
SHALL sound a short metronome click, with the accented downbeat distinct from
normal beats. Visually, the Tempo chip SHALL pulse on each beat (an accent pulse
on the downbeat) so the beat is visible in every view mode without depending on a
mode-specific cursor.

#### Scenario: Beat clicks and pulses together
- **WHEN** a beat event fires with the metronome enabled
- **THEN** a click sounds and the Tempo chip pulses for that beat

#### Scenario: Accent is distinguishable
- **WHEN** a downbeat fires
- **THEN** its click and chip pulse are distinct from those of a normal beat

#### Scenario: Visible in every view mode
- **WHEN** the metronome is enabled in waterfall, staff, or partition mode
- **THEN** the chip pulse is visible the same way in each mode

### Requirement: Metronome Silent While Paused

The metronome SHALL emit neither clicks nor visual pulses while playback is not
running (paused or stopped), even when it is enabled. It SHALL resume on the next
beat boundary once playback runs again.

#### Scenario: No ticks while paused
- **WHEN** the metronome is enabled and playback is paused or stopped
- **THEN** no click sounds and the chip does not pulse

#### Scenario: Resumes with playback
- **WHEN** playback resumes with the metronome still enabled
- **THEN** beats resume on the score's beat boundaries

### Requirement: Metronome Testable Without Native Audio

Beat-boundary detection and the enabled/accent logic SHALL be expressed in
host-testable code that drives the injectable audio seam (`audioServiceProvider`),
so tests can assert which beats fire (and which are accents) using a fake audio
service via a provider override, without the native audio library.

#### Scenario: Tests assert beats via a fake
- **WHEN** a test overrides the audio service with a fake and advances the playhead
  across beat boundaries with the metronome enabled
- **THEN** the fake records the expected click calls (with accent flags) and the
  state exposes the beat pulses, with no native library loaded
