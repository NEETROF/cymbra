## ADDED Requirements

### Requirement: Local Preferences Store

The app SHALL provide a local, on-device key-value store for small user
preferences that survives app restarts. It SHALL support reading and writing
simple typed values (at minimum a string keyed by name) and SHALL return a
well-defined absent result when a key has never been written. This store SHALL be
separate from secure/credential storage, which remains reserved for auth tokens.

#### Scenario: Written value is persisted
- **WHEN** a preference is written under a key
- **THEN** reading that key afterward returns the written value

#### Scenario: Value survives a restart
- **WHEN** a preference is written and the app is relaunched
- **THEN** reading that key returns the previously written value

#### Scenario: Missing key returns absent
- **WHEN** a key that was never written is read
- **THEN** the store reports the value as absent (rather than returning a spurious
  value)

### Requirement: Injectable Preferences Provider

The local preferences store SHALL be exposed as an injectable Riverpod provider
behind an abstract seam, with a production implementation backed by on-device
storage and a fake implementation for tests, so that state and widgets depending
on preferences are testable without touching real device storage.

#### Scenario: Production wiring uses on-device storage
- **WHEN** the app runs normally
- **THEN** the preferences provider resolves to the implementation backed by
  on-device persistence

#### Scenario: Tests override with a fake
- **WHEN** a test needs deterministic preferences
- **THEN** it overrides the preferences provider in a `ProviderScope`/
  `ProviderContainer` with an in-memory fake, without initializing native
  storage
