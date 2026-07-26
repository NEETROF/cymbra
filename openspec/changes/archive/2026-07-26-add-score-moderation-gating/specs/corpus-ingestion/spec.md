## ADDED Requirements

### Requirement: Ingested scores are unvalidated by default

Ingestion SHALL persist every newly ingested `catalog_scores` row with moderation
status `pending` (unvalidated). The crawler MUST NOT auto-validate any score, and the
licensing `confidence` value (`verified` / `unverified`) MUST NOT be used to grant
validation — a high-confidence licence still yields a `pending` score. Consequently a
freshly crawled score SHALL NOT appear in the public hub until a reviewer validates
it. This is independent of and additional to the existing confidence separation.

#### Scenario: Newly ingested score is pending

- **WHEN** the crawler ingests a score that passes the licence gate and conversion
- **THEN** its `catalog_scores` row has moderation status `pending` and it is not
  publicly visible

#### Scenario: High-confidence licence does not auto-validate

- **WHEN** an ingested score has `confidence = 'verified'`
- **THEN** its moderation status is still `pending`, not `accepted`

#### Scenario: Freshly crawled score is absent from the hub

- **WHEN** a normal caller browses the hub right after a crawl run
- **THEN** the newly ingested scores do not appear, because they are `pending`
