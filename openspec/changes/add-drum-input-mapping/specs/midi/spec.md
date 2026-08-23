## MODIFIED Requirements

### Requirement: Real-time MIDI Event Streaming

The system SHALL stream normalized note events to the UI in real time. A NoteOn
message with velocity greater than zero SHALL produce a NoteOn event; a NoteOff
message, or a NoteOn message with zero velocity, SHALL produce a NoteOff event.
Each event SHALL carry the pitch, velocity, and a timestamp in milliseconds
relative to the stream's start. Non-note messages SHALL be ignored.

Note events SHALL be produced from the message's status high nibble alone,
whatever channel the status low nibble carries, and events SHALL deliberately
carry no channel: the stream is channel-agnostic by design, not by accident.
Percussion devices commonly transmit on channel 10 — General MIDI's
percussion channel — but are configurable, so filtering by channel would
silently answer a misconfigured kit with dropped strokes; and no consumer
needs the channel, because the loaded score's instrument, not the wire,
decides how a note number is interpreted (see `music-drum-input`).

#### Scenario: Note on
- **WHEN** a `0x90` message arrives with velocity > 0
- **THEN** a NoteOn event is emitted with the message's pitch and velocity

#### Scenario: Note off via zero-velocity note on
- **WHEN** a `0x90` message arrives with velocity 0
- **THEN** a NoteOff event is emitted with velocity normalized to 0

#### Scenario: Non-note messages ignored
- **WHEN** a Control Change or malformed (too short) message arrives
- **THEN** no event is emitted

#### Scenario: Channel does not gate the event
- **WHEN** a NoteOn arrives on channel 10 (status `0x99`) with velocity > 0,
  as an e-kit sends by default
- **THEN** a NoteOn event is emitted exactly as from any other channel
