## ADDED Requirements

### Requirement: Moderation status on catalog scores

Every `catalog_scores` entry SHALL carry a moderation status drawn from the fixed
set `pending` / `accepted` / `rejected`, defaulting to `pending`. `accepted` means
the score is validated and publicly visible; `pending` means it has not yet been
reviewed; `rejected` means a reviewer has refused it. The store SHALL also persist,
for each entry, the reviewer who last set the status and when — nullable and unset
while the status is `pending`, so a later `accepted`/`rejected` decision is traceable
to a specific reviewer and time. The status SHALL be constrained to the allowed set at
the storage layer so no other value can be persisted.

#### Scenario: Status defaults to pending

- **WHEN** a catalog score is created without an explicit moderation status
- **THEN** its moderation status is `pending` and its reviewer/timestamp are unset

#### Scenario: Only the allowed status values are accepted

- **WHEN** a write attempts to set a moderation status outside `pending` / `accepted` / `rejected`
- **THEN** the store rejects it rather than persisting an unknown value

#### Scenario: Review attribution is recorded when set

- **WHEN** a score's status is later set to `accepted` or `rejected`
- **THEN** the entry records which reviewer set it and at what time (traceable),
  whereas a `pending` entry has no reviewer or timestamp

### Requirement: Only validated scores are publicly visible

The system SHALL treat `accepted` as the only publicly visible moderation status:
a score that is `pending` or `rejected` MUST NOT be exposed to a normal (non-admin,
non-moderator) caller through any public read path — neither catalog listing/search
nor score-bytes retrieval. Visibility SHALL be enforced server-side, so a client that
is unaware of the status still cannot reach an unvalidated score. A caller authorized
as a moderator or admin, by contrast, SHALL be able to list/search and open (fetch the
bytes of) scores in any moderation status, so a reviewer can inspect unvalidated
material. Until a dedicated moderator role exists, "authorized" means an `admin`
identity.

#### Scenario: Pending score hidden from a normal caller

- **WHEN** a normal caller lists, searches, or requests the bytes of a `pending` score
- **THEN** the score is not returned (it is absent from results, or its bytes request is
  refused as if not found)

#### Scenario: Rejected score hidden from a normal caller

- **WHEN** a normal caller would otherwise encounter a `rejected` score
- **THEN** the score is not returned through any public read path

#### Scenario: Accepted score is visible

- **WHEN** a normal caller searches or lists the catalog
- **THEN** `accepted` scores are returned as before

#### Scenario: Authorized reviewer sees unvalidated scores

- **WHEN** a moderator/admin caller lists/searches or opens a `pending` or `rejected` score
- **THEN** the score is returned and its bytes can be fetched, so the reviewer can evaluate it

### Requirement: Existing catalog scores start unvalidated

Introducing moderation SHALL reset the existing corpus to unvalidated: on migration,
every pre-existing `catalog_scores` row SHALL become `pending`, so no previously
auto-published score remains publicly visible until it is explicitly validated. The
migration MUST be additive and reversible at the schema level.

#### Scenario: Existing corpus becomes pending on migration

- **WHEN** the moderation migration is applied to a populated catalog
- **THEN** every existing row has moderation status `pending` and none is publicly
  visible until validated

#### Scenario: Hub reflects the reset

- **WHEN** the app hub loads immediately after the migration, before any validation
- **THEN** it shows no catalog scores, because none is yet `accepted`
