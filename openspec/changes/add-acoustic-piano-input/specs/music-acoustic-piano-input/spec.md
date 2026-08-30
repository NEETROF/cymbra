## ADDED Requirements

### Requirement: Detected Notes Enter The Standard Input Stream

Acoustic piano detection SHALL emit normalized note events of the same shape
and through the same injectable input seam as MIDI input, so that the player,
Wait Mode, scoring, courses and every other downstream consumer runs
unchanged and unaware of the source. Attack events SHALL always be emitted;
release events are best-effort only, and no downstream behavior reachable
from an audio-sourced run may depend on releases (the damper pedal makes them
undetectable).

#### Scenario: Downstream consumers are source-blind

- **WHEN** a detected note event reaches the player during an audio-sourced
  session
- **THEN** it satisfies Wait Mode gates, key feedback and scoring exactly as
  the same event arriving from a MIDI port would

#### Scenario: Missing releases do not wedge the player

- **WHEN** a detected note's release is never observed (e.g. sustained by the
  damper pedal)
- **THEN** the session continues normally and no downstream state waits on
  that release

### Requirement: Score-Informed Presence Detection

Detection SHALL be score-informed: it evaluates the presence and onset time of
the pitches the score expects rather than performing blind polyphonic
transcription, and it SHALL NOT be required to report pitches outside the
expected set — downstream judgment operates only on the events actually
reported.

#### Scenario: Expected note detected on presence

- **WHEN** the player sounds an expected pitch on the acoustic piano at its
  onset
- **THEN** a note event for that pitch is emitted with the onset's timestamp

#### Scenario: Unreported extras carry no penalty

- **WHEN** the player sounds a pitch the detector does not report
- **THEN** no event is emitted for it and downstream judgment simply never
  sees it

### Requirement: Onset Timing Decoupled From Pitch Confirmation

Detection SHALL timestamp a note event at the acoustic onset (the attack
transient), not at the completion of pitch analysis, so that the latency of
identifying *which* note was played never delays *when* it was played.

#### Scenario: Timestamp reflects the attack

- **WHEN** a note's pitch confirmation completes some time after its attack
  transient was detected
- **THEN** the emitted event carries the attack's timestamp, not the
  confirmation's

### Requirement: Detected Notes Are Never Synthesized

The system SHALL never synthesize a note event whose source is acoustic
detection — the piano sounds itself, and re-sounding a detected note is both
redundant and a feedback path into the microphone. This is inherent to the
source, not a setting; the existing "Instrument Sounds Itself" setting keeps
its MIDI-only scope. All other app audio — score playback, the metronome,
preview clips — SHALL be unaffected, and the on-screen keyboard SHALL still
sound when used alongside an audio-sourced session.

#### Scenario: Detected note is not re-sounded

- **WHEN** a detected note event enters the player
- **THEN** the app synthesizes nothing for it

#### Scenario: Other audio is unaffected

- **WHEN** an audio-sourced session plays back the score or sounds the
  metronome
- **THEN** each is heard exactly as in a MIDI-sourced session

### Requirement: Input Source Selection

The system SHALL let the user choose the player's input source — MIDI or
microphone — with MIDI (and its existing auto-connect behavior) remaining the
default; selecting the microphone SHALL NOT alter the remembered MIDI port
selection, and the choice SHALL be visible wherever input status is shown.

#### Scenario: Microphone selected without losing MIDI state

- **WHEN** the user switches the input source to the microphone and later back
  to MIDI
- **THEN** the previously remembered MIDI port selection still applies

#### Scenario: Source visible in input status

- **WHEN** the input source is the microphone
- **THEN** the surfaces that today show the connected MIDI port show the
  microphone source and its route instead

### Requirement: Wait-Mode-First Availability

An audio-sourced session SHALL always offer Wait Mode, whose gate-relative
reaction judgment absorbs detection latency; Wait Mode availability SHALL NOT
depend on a calibration measurement.

#### Scenario: Wait Mode works uncalibrated

- **WHEN** the user starts an audio-sourced session on a route with no stored
  calibration
- **THEN** Wait Mode play is available and judged normally

### Requirement: Free-Run Gated On Measured Latency

Scored free-run (Wait Mode off) play with the microphone source SHALL be
available only when the active route's measured input round-trip latency fits
the free-run timing windows; otherwise the session SHALL steer the user to
Wait Mode (or to calibration if none was run) with localized copy explaining
why — never a silent degradation of scores.

#### Scenario: Fit device opens free-run

- **WHEN** the active route's stored calibration shows a round-trip compatible
  with the free-run windows
- **THEN** scored free-run play is available with the microphone source

#### Scenario: Unfit or unmeasured device steers to Wait Mode

- **WHEN** the route has no stored calibration, or its measured latency
  exceeds what the free-run windows tolerate
- **THEN** scored free-run play with the microphone source is unavailable,
  and the user is pointed to Wait Mode and/or calibration with a localized
  explanation

### Requirement: Feature Flag Audience

The microphone input source SHALL be visible only to the audience of a
server-evaluated feature flag that defaults to off, following the established
runtime-flag pattern; while the flag is off for a caller, no microphone
surface appears anywhere and the microphone permission is never requested.
Hiding is defence in depth: absence of the flag leaves every existing input
path untouched.

#### Scenario: Flag off means no trace

- **WHEN** the flag is off for the caller (including signed-out and
  cold-start-unresolved states)
- **THEN** no microphone source, calibration entry or related copy is shown,
  and no permission prompt can be triggered

#### Scenario: Flag on reveals the source

- **WHEN** the flag resolves on for the caller
- **THEN** the microphone input source and its calibration entry become
  available

### Requirement: Input Source Stamped On The Run Record

Every scored run SHALL record which input source produced it (MIDI or
microphone), so that later policy — leaderboard eligibility, analytics,
support — can distinguish audio-sourced runs without re-deriving anything.

#### Scenario: Audio-sourced run is stamped

- **WHEN** a scored run played with the microphone source is finalized
- **THEN** its immutable session record carries the microphone input source
