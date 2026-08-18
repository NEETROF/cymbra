# offline-score-cache Specification

## Purpose
TBD - created by archiving change add-premium-subscription. Update Purpose after archive.
## Requirements
### Requirement: Offline caching of catalog scores is a premium unlock

The app SHALL cache **catalog** scores for offline play only when the user's effective plan
grants the `offline.cache` unlock; on the free plan, catalog scores are played online only and
the offline availability feedback SHALL say so and lead to the paywall. Bundled demo scores stay
playable offline for everyone. The user's **own favorited uploads** MAY be cached offline on any
plan (they own the file). Cached catalog entries SHALL be evicted, and their key material rotated
by the server, when the plan lapses (see `music-plan-entitlements` withdrawal).

#### Scenario: Free plan does not cache catalog scores

- **WHEN** a free user opens a favorited catalog score online
- **THEN** no encrypted local copy is written and the offline indicator explains offline play is a premium feature

#### Scenario: Premium caches catalog scores

- **WHEN** a premium (or trial) user opens a favorited catalog score online
- **THEN** the encrypted local copy is written as specified for the offline cache

#### Scenario: Own uploads cache on any plan

- **WHEN** a free user opens one of their own favorited uploads
- **THEN** it is cached offline as before

#### Scenario: Lapse evicts catalog cache

- **WHEN** the plan lapses and the app reconnects
- **THEN** cached catalog entries are deleted, own-upload entries are kept, and the rotated secret makes any residual catalog file unreadable

