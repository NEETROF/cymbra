## ADDED Requirements

### Requirement: Per-Device Input Mapping Is Persisted Locally

The calibrated input mapping SHALL be persisted in the local preferences store,
keyed by the MIDI device it was learned from, and SHALL survive an app restart.
It SHALL NOT be written to secure storage (it holds no credential) and SHALL NOT
be sent to a server. Storage that is unavailable or holds an unreadable value
SHALL be treated as "no mapping for this device" — the uncalibrated behaviour —
rather than failing the app or corrupting an unrelated device's mapping.

#### Scenario: A calibrated mapping survives a restart
- **WHEN** a device is calibrated and the app is relaunched
- **THEN** that device's mapping is restored from local storage

#### Scenario: Two devices are remembered independently
- **WHEN** two different devices are each calibrated
- **THEN** each keeps its own mapping and connecting one does not disturb the
  other

#### Scenario: Unreadable storage degrades to uncalibrated
- **WHEN** the stored value is missing or cannot be parsed
- **THEN** the app behaves as though the device had never been calibrated,
  without error
