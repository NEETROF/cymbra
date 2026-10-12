## ADDED Requirements

### Requirement: The extension reads its effective flags
The extension SHALL fetch its effective flags from the server for the signed-in account, keep them as one snapshot, and let every surface read a key locally, without a network call per key.
The background SHALL fetch at sign-in, at start-up, on wake and when a surface opens, at most once
per few minutes outside sign-in, sending its known version so an unchanged set costs nothing. A
failed fetch SHALL keep the last-good snapshot. The persisted snapshot SHALL be keyed by account
and discarded on sign-out; a signed-out reader, or a key never fetched, SHALL read as off. Flags
SHALL only decide what the extension shows: the server stays the authority for any action they
gate.

#### Scenario: A reader is enrolled while signed in
- **WHEN** an operator enrols a signed-in reader in a Lingua campaign and the reader then opens a surface
- **THEN** the flags scoped to that campaign read as on in that surface, without signing in again

#### Scenario: Sign-out
- **WHEN** the reader signs out
- **THEN** the snapshot is discarded and every flag reads as off

#### Scenario: Offline
- **WHEN** a fetch fails
- **THEN** surfaces keep reading the last-good snapshot

#### Scenario: Another account on the same browser
- **WHEN** a different account signs in
- **THEN** the previous account's snapshot is not used, and the new account's flags are fetched
