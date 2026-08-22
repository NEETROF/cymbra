## MODIFIED Requirements

### Requirement: Per-user private library quota

The system SHALL bound each user's private library by a quota — a maximum font count and
a maximum total size — in addition to the existing per-file size cap. The quota SHALL be
**resolved per request from runtime configuration keyed by the caller's effective plan**
(`plans.soundfont_library.max_fonts.free`, `.premium`; a plan whose unlock set includes
`soundfont_library.extended` uses the premium value), with the free value defaulting to
today's constant. Music-scope moderators and admins remain exempt. An import that would
exceed the quota MUST be refused with a typed error, and no bytes stored; the refusal SHALL
tell the app that a higher plan raises the limit so the surface can upsell.

#### Scenario: Import within quota succeeds

- **WHEN** a user imports a font and their library is within the count and size quota
- **THEN** the import succeeds

#### Scenario: Import over quota is refused

- **WHEN** an import would push the user's library past its font-count or total-size quota
- **THEN** the import is refused with a typed error and nothing is stored

#### Scenario: Premium quota applies to a plan holder

- **WHEN** a user whose plan grants `soundfont_library.extended` imports beyond the free count but within the premium count
- **THEN** the import succeeds

#### Scenario: Quota change needs no release

- **WHEN** an operator edits `plans.soundfont_library.max_fonts.premium`
- **THEN** the next import request is evaluated against the new value
