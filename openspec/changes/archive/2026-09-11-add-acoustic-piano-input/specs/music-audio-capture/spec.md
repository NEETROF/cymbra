## ADDED Requirements

### Requirement: Microphone Capture Lifecycle

The system SHALL capture microphone audio only while a feature that consumes it
is active (a calibration run, or a play/practice session whose input source is
the microphone), SHALL stop capture when that feature ends or the app leaves
the foreground, and SHALL never capture while the microphone input source is
not in use.

#### Scenario: Capture starts with the consuming feature

- **WHEN** the user starts a session whose input source is the microphone
- **THEN** capture starts, and an OS-level recording indicator may appear

#### Scenario: Capture stops with the consuming feature

- **WHEN** the session ends or the app is backgrounded
- **THEN** capture stops and no audio is read from the microphone

#### Scenario: No capture outside the microphone source

- **WHEN** the input source is MIDI or the on-screen keyboard
- **THEN** the microphone is not opened at any point

### Requirement: Microphone Permission Flow

The system SHALL request the platform microphone permission only at the moment
a microphone-consuming feature is first used, with a localized rationale, and
SHALL degrade to actionable guidance — never a crash or a silent failure —
when the permission is denied or restricted.

#### Scenario: Permission asked in context

- **WHEN** the user first starts calibration or selects the microphone source
- **THEN** the platform permission prompt is preceded or accompanied by a
  localized explanation of why the microphone is needed

#### Scenario: Denial degrades to guidance

- **WHEN** the permission is denied
- **THEN** the microphone source is shown as unavailable with a localized
  explanation and a pointer to the system settings, and the MIDI and
  on-screen paths remain fully usable

### Requirement: Unprocessed Capture Configuration

The system SHALL configure capture to bypass the platform's voice-processing
chain (automatic gain control, noise suppression, voice echo cancellation),
using the platform's measurement/unprocessed capture mode where available, and
SHALL fall back to the least-processed available source — never a
voice-communication source — while recording which configuration was obtained.

#### Scenario: Unprocessed mode used when supported

- **WHEN** the platform reports support for unprocessed capture
- **THEN** capture uses it and no automatic gain control, noise suppression or
  voice echo cancellation is applied to the signal

#### Scenario: Fallback stays away from voice processing

- **WHEN** the platform does not support unprocessed capture
- **THEN** capture uses the least-processed available source, never a
  voice-communication source, and the obtained configuration is recorded and
  available to diagnostics

### Requirement: Input Route Classification

The system SHALL classify the active capture route by connection kind —
built-in, wired, USB, Bluetooth, or other — from the platform's route
description and never from the route's display name, so that an unrecognized
connection degrades to *other* rather than breaking the classification.

#### Scenario: Built-in microphone classified

- **WHEN** capture runs on the device's own microphone
- **THEN** the route is classified as built-in

#### Scenario: USB receiver classified as USB

- **WHEN** a wireless microphone's USB-C receiver is connected and presents as
  a class-compliant audio device
- **THEN** the route is classified as USB and treated as wired-equivalent

#### Scenario: Unknown route degrades safely

- **WHEN** the platform reports a connection kind this build does not know
- **THEN** the route is classified as *other* and capture remains usable

### Requirement: Bluetooth Input Refusal

The system SHALL refuse to acquire from a Bluetooth microphone route and SHALL
explain the refusal in localized copy, because Bluetooth capture runs over the
voice profile whose bandwidth and jitter are incompatible with note timing —
the refusal is a hard rule, not a compensable warning like the wireless
*output* warning.

#### Scenario: Bluetooth microphone refused

- **WHEN** the active or selected capture route is a Bluetooth microphone
- **THEN** acquisition does not start, and a localized explanation names the
  route and suggests the built-in microphone or a USB alternative

#### Scenario: Refusal lifts when the route changes

- **WHEN** the Bluetooth route disappears or the user switches to an accepted
  route
- **THEN** acquisition becomes available again without restarting the app

### Requirement: Measured Input-Offset Calibration

The system SHALL provide a calibration flow that emits a reference sound,
detects it through the active capture route, and measures the device's real
input round-trip latency; the measured value SHALL be stored per capture
route, be re-runnable at any time, and expose both the latency and a
usability verdict to consumers. A calibration that fails to detect the
reference sound SHALL end with localized guidance, never an indefinite wait.

#### Scenario: Round-trip measured closed-loop

- **WHEN** the user runs calibration in a quiet-enough environment
- **THEN** the app emits the reference sound, detects it at the microphone,
  and stores the measured round-trip latency for the active route

#### Scenario: Calibration failure guides the user

- **WHEN** the reference sound is not detected within a bounded time
- **THEN** calibration ends with localized guidance (volume, distance, noise)
  and no value is stored

#### Scenario: Route change invalidates the stored value

- **WHEN** the active capture route changes to one with no stored measurement
- **THEN** consumers see no measured value for the new route and calibration
  is offered again

### Requirement: Desktop Capture Device Selection

The system SHALL let the user choose the capture input device on desktop —
where the engine owns the device — from the enumerated list or follow the
system default; the selection SHALL persist across restarts, SHALL apply to a
capture already running, and SHALL fall back to the system default — never
fail — when the chosen device is absent. On mobile platforms the OS owns the
input route and no device list is offered.

#### Scenario: A chosen input device captures

- **WHEN** the user selects an enumerated input device on desktop
- **THEN** capture (current and future) acquires from that device, and the
  choice is restored on the next launch

#### Scenario: An absent device falls back to the default

- **WHEN** the persisted input device is not present at capture time
- **THEN** capture opens the system default input instead of failing

#### Scenario: Mobile offers no device list

- **WHEN** the platform is iOS or Android
- **THEN** no input-device picker is offered; the active route is reported as
  the OS resolves it

### Requirement: Injectable Capture Seam

The system SHALL expose capture — device lifecycle, route classification,
permission state, calibration — behind an injectable seam so that state and
widget tests can drive every capture condition deterministically with no
microphone, no permission prompt, and no native library.

#### Scenario: Tests drive capture without hardware

- **WHEN** a test overrides the capture seam with a scripted double
- **THEN** permission outcomes, route kinds, calibration results and capture
  lifecycle transitions can all be simulated with no native audio dependency
