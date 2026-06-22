## ADDED Requirements

### Requirement: MIDI Port Enumeration

The system SHALL list the names of available MIDI input ports, with virtual or
loopback ports (e.g. "Midi Through", rtpMIDI, network sessions) ordered after
real hardware devices using a stable sort.

#### Scenario: Real devices listed before virtual ports
- **WHEN** the host exposes both a hardware device and a virtual "Midi Through" port
- **THEN** the returned list places the hardware device before the virtual port

#### Scenario: Virtual ports detected case-insensitively
- **WHEN** a port name contains "through", "rtpmidi", or "network" in any casing
- **THEN** that port is classified as virtual and ordered last

### Requirement: MIDI Port Selection

The system SHALL allow selecting the input device by name, and SHALL support an
automatic mode (no name) that connects to the first non-virtual port. Changing
the selection SHALL force a reconnection to the newly chosen port.

#### Scenario: Manual selection by name
- **WHEN** the user selects a device by name
- **THEN** the engine releases any current connection and reconnects to the named port

#### Scenario: Automatic mode
- **WHEN** no device name is selected (auto mode)
- **THEN** the engine connects to the first non-virtual port if one is present

### Requirement: Connection Status Reporting

The system SHALL report the name of the currently connected MIDI port, or report
that no device is connected, so the UI can render a connection indicator.

#### Scenario: A device is connected
- **WHEN** a port is connected
- **THEN** the reported status is the connected port's name

#### Scenario: No device connected
- **WHEN** no port is connected
- **THEN** the reported status is empty/none

### Requirement: Real-time MIDI Event Streaming

The system SHALL stream normalized note events to the UI in real time. A NoteOn
message with velocity greater than zero SHALL produce a NoteOn event; a NoteOff
message, or a NoteOn message with zero velocity, SHALL produce a NoteOff event.
Each event SHALL carry the pitch, velocity, and a timestamp in milliseconds
relative to the stream's start. Non-note messages SHALL be ignored.

#### Scenario: Note on
- **WHEN** a `0x90` message arrives with velocity > 0
- **THEN** a NoteOn event is emitted with the message's pitch and velocity

#### Scenario: Note off via zero-velocity note on
- **WHEN** a `0x90` message arrives with velocity 0
- **THEN** a NoteOff event is emitted with velocity normalized to 0

#### Scenario: Non-note messages ignored
- **WHEN** a Control Change or malformed (too short) message arrives
- **THEN** no event is emitted

### Requirement: Hot-plug and Unplug Handling

The system SHALL automatically connect to a device that appears after startup,
and SHALL release the connection when the connected device disappears, without
requiring an application restart.

#### Scenario: Device plugged in after startup
- **WHEN** a MIDI device appears while the app is running and none is connected
- **THEN** the engine connects to it on the next watch cycle

#### Scenario: Connected device unplugged
- **WHEN** the currently connected device is removed
- **THEN** the engine clears the connection and reports no device connected

### Requirement: Cross-platform Native Integration

The system SHALL operate across macOS, iOS, Android, Linux and Windows via a
single MIDI backend, with the platform-specific wiring required for hot-plug and
device access on each platform.

#### Scenario: macOS/iOS hot-plug delivery
- **WHEN** the app runs on macOS or iOS
- **THEN** a CoreMIDI client is created on the main run loop so configuration-change (hot-plug) notifications keep the process's MIDI view current

#### Scenario: Android native context
- **WHEN** the app runs on Android
- **THEN** the native library is loaded via `System.loadLibrary` so `JNI_OnLoad` initializes the Android context required by the AMidi backend

#### Scenario: Desktop backends
- **WHEN** the app runs on Linux or Windows
- **THEN** MIDI ports are enumerated and opened via the platform's native backend (ALSA / WinMM) without additional wiring
