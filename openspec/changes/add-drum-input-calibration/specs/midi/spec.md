## MODIFIED Requirements

### Requirement: Real-time MIDI Event Streaming

The system SHALL stream normalized note events to the UI in real time. A NoteOn
message with velocity greater than zero SHALL produce a NoteOn event; a NoteOff
message, or a NoteOn message with zero velocity, SHALL produce a NoteOff event.
Each event SHALL carry the pitch, velocity, and a timestamp in milliseconds
relative to the stream's start. Non-note messages SHALL be ignored.

An event SHALL additionally carry the **transmitting channel**, so a diagnostic
surface can report what the instrument is actually sending. Interpretation
remains channel-agnostic: the channel is reported, never used to accept or
reject an event.

The stream SHALL support more than one reader, so a diagnostic surface can
observe events without competing with the player for them, and closing such a
reader SHALL leave the player's own consumption unchanged.

#### Scenario: Note on
- **WHEN** a `0x90` message arrives with velocity > 0
- **THEN** a NoteOn event is emitted with the message's pitch and velocity

#### Scenario: Note off via zero-velocity note on
- **WHEN** a `0x90` message arrives with velocity 0
- **THEN** a NoteOff event is emitted with velocity normalized to 0

#### Scenario: Non-note messages ignored
- **WHEN** a Control Change or malformed (too short) message arrives
- **THEN** no event is emitted

#### Scenario: Channel is reported, not enforced
- **WHEN** a note message arrives on any channel
- **THEN** the event carries that channel and is emitted exactly as it would be
  on any other

#### Scenario: A second reader does not starve the first
- **WHEN** a diagnostic surface observes the stream while a score is playing
- **THEN** the player continues to receive every event, and stops observing
  cleanly when the surface closes

## ADDED Requirements

### Requirement: Live Events Are Translated Before They Are Interpreted

A live MIDI note event SHALL pass through the device's input mapping before any
part of the app interprets it — including the engine's own low-latency echo,
which sounds a stroke from the MIDI callback before the event reaches the app.
With no mapping in force the translation SHALL be the identity, so behaviour is
unchanged from an uncalibrated app.

#### Scenario: The echo sounds the translated number
- **WHEN** the engine echo is armed and a mapping covers an incoming number
- **THEN** the sound produced is the mapped piece's, not the raw number's

#### Scenario: No mapping means no change
- **WHEN** no mapping is in force
- **THEN** every live event is interpreted exactly as it arrives
