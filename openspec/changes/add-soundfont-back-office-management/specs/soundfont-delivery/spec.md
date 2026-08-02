## ADDED Requirements

### Requirement: Authenticated SoundFont upload route

The private SoundFont store SHALL have an authenticated **upload** counterpart to its
download route, so a font's bytes can be placed into the store through the backend
(never a direct browser-to-bucket write). The upload SHALL require a music-scope
moderator/admin identity; any lesser or unauthenticated caller MUST be refused before
any store write. The route SHALL accept the font body (with its metadata) up to a
**configured maximum size**, rejecting an over-large body, and SHALL validate that the
body is a real SoundFont before storing it, rejecting an invalid body without writing.
The bytes SHALL be written through the backend/API origin into the same private store
the delivery route reads.

#### Scenario: Admin uploads a font

- **WHEN** a music-scope admin uploads a valid `.sf2` through the upload route
- **THEN** the backend streams it into the private SoundFont store and it becomes
  fetchable by the delivery route

#### Scenario: Unauthorized upload is refused

- **WHEN** an unauthenticated caller, or one lacking music-scope moderator/admin,
  attempts to upload a font
- **THEN** the request is refused before any store write and nothing is stored

#### Scenario: Invalid upload body is rejected

- **WHEN** an authorized admin uploads a body that is not a valid SoundFont
- **THEN** the route rejects it and stores nothing
