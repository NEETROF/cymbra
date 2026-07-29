## MODIFIED Requirements

### Requirement: Only validated scores are publicly visible

The system SHALL treat `accepted` as the only publicly visible moderation status through
the general public read paths: a score that is `pending` or `rejected` MUST NOT be
exposed to a normal (non-admin, non-moderator) caller through **catalog listing/search**,
the **player-open score-bytes retrieval**, or **library save**. Visibility SHALL be
enforced server-side, so a client that is unaware of the status still cannot reach an
unvalidated score through those paths. The sole exception is the **deck-scoped
rating-preview byte path** (see "Rating-preview byte access for deck scores"), which may
serve a `pending` score's bytes to a signed-in rater for previewing-before-rating only;
`rejected` scores remain unreachable through every path. A caller authorized as a
moderator or admin, by contrast, SHALL be able to list/search and open (fetch the bytes
of) scores in any moderation status, so a reviewer can inspect unvalidated material.
Until a dedicated moderator role exists, "authorized" means an `admin` identity.

#### Scenario: Pending score hidden from a normal caller through general paths

- **WHEN** a normal caller lists, searches, requests the player-open bytes of, or tries to
  save a `pending` score
- **THEN** the score is not returned (it is absent from results, or its bytes/save request is
  refused as if not found)

#### Scenario: Rejected score hidden from every path

- **WHEN** a normal caller would otherwise encounter a `rejected` score, including the
  rating-preview path
- **THEN** the score is not returned through any read path

#### Scenario: Accepted score is visible

- **WHEN** a normal caller searches or lists the catalog
- **THEN** `accepted` scores are returned as before

#### Scenario: Authorized reviewer sees unvalidated scores

- **WHEN** a moderator/admin caller lists/searches or opens a `pending` or `rejected` score
- **THEN** the score is returned and its bytes can be fetched, so the reviewer can evaluate it

## ADDED Requirements

### Requirement: Rating-preview byte access for deck scores

The backend SHALL expose a **dedicated rating-preview byte read**, distinct from the
player-open score-bytes retrieval, that returns the bytes of a `pending` or `accepted`
catalog score to a **signed-in** caller for previewing it in the rating deck. This path
SHALL serve `pending` bytes (its purpose is community evaluation of candidate scores) but
MUST refuse a `rejected` or non-existent score. It SHALL NOT be a substitute for the
player-open path: it exists so a rater can hear a candidate before rating, and the
general player-open bytes retrieval and library save remain `accepted`-only. Unauthenticated
requests MUST be rejected.

#### Scenario: Signed-in rater previews a pending score

- **WHEN** a signed-in user requests rating-preview bytes for a `pending` score
- **THEN** the score's bytes are returned so the deck can play a read-only preview

#### Scenario: Rejected score is refused on the preview path

- **WHEN** any caller requests rating-preview bytes for a `rejected` or non-existent score
- **THEN** the request is refused as not-found

#### Scenario: Preview path does not open the full player or save

- **WHEN** a `pending` score's bytes are served through the rating-preview path
- **THEN** the player-open bytes retrieval and library save for that score remain refused for
  a normal caller (the pending score cannot be opened in the full player or saved)

#### Scenario: Unauthenticated preview rejected

- **WHEN** a rating-preview byte request arrives without a valid authenticated identity
- **THEN** the request is rejected
