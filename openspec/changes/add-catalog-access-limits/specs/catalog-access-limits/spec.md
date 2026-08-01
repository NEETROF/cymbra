## ADDED Requirements

### Requirement: Per-user download burst cap on raw score bytes

The system SHALL enforce a per-user short-window burst cap on every RPC that returns
raw score bytes — `GetCatalogScoreBytes` and `GetRatingPreviewBytes` — keyed on the
authenticated caller's `AuthIdentity.user_id` (the token subject). The burst cap
limits the request *rate* only and SHALL apply regardless of the user's play
activity. When the burst cap is exceeded, the system SHALL reject the request with
gRPC status `RESOURCE_EXHAUSTED` and MUST NOT return the score bytes.

#### Scenario: Burst cap rejects a fast download flood
- **WHEN** a signed-in user issues more score-bytes requests within the burst window
  than the configured burst maximum
- **THEN** requests beyond the maximum are rejected with `RESOURCE_EXHAUSTED`
- **AND** no score bytes are returned for the rejected requests

#### Scenario: Rating-preview bytes share the burst cap
- **WHEN** a user requests raw bytes through `GetRatingPreviewBytes`
- **THEN** the request is counted against the same per-user burst cap as
  `GetCatalogScoreBytes`

#### Scenario: Burst cap is isolated per user
- **WHEN** user A has hit the burst cap
- **THEN** user B's downloads are unaffected because counters are keyed by `user_id`

### Requirement: Play-aware download volume allowance

The system SHALL bound each user's total download volume over a rolling window with
a **play-aware allowance** rather than a flat cap, so a user whose downloads track
their real usage is not penalised. The effective allowance SHALL be a configured
base floor plus additional headroom derived from the user's play activity
(`PlayService` sessions) over the window, and SHALL never exceed a configured high
hard ceiling. When a user's download volume in the window exceeds their current
effective allowance, the system SHALL reject further score-bytes requests with
`RESOURCE_EXHAUSTED`. The allowance SHALL be keyed on `AuthIdentity.user_id`.

#### Scenario: Downloads proportional to play are not blocked
- **WHEN** a user's download volume over the window stays in proportion to their
  play activity (they play the scores they download)
- **THEN** their downloads continue to succeed even well above the base floor
- **AND** they are not rejected with `RESOURCE_EXHAUSTED`

#### Scenario: Download-heavy, play-light profile falls back to the floor
- **WHEN** a user downloads far more scores than their play activity justifies
  (the scraping signature)
- **THEN** once their download volume exceeds the base floor plus play-earned
  headroom, further requests are rejected with `RESOURCE_EXHAUSTED`

#### Scenario: Hard ceiling backstops even very active players
- **WHEN** a user's play-earned headroom would exceed the configured hard ceiling
- **THEN** the effective allowance is capped at the hard ceiling
- **AND** download volume beyond the hard ceiling is rejected with
  `RESOURCE_EXHAUSTED`

#### Scenario: New user can download up to the base floor
- **WHEN** a user with no play activity yet downloads scores
- **THEN** they may download up to the base floor before any rejection

#### Scenario: Allowance is isolated per user
- **WHEN** user A has exhausted their play-aware allowance
- **THEN** user B's allowance is unaffected because it is computed per `user_id`

### Requirement: Per-user enumeration rate limiting on catalog browse

The system SHALL enforce a per-user request-rate limit on the catalog-enumeration
RPCs — `SearchCatalog`, `GetCatalogScore`, and `ListRatingDeck` — keyed on
`AuthIdentity.user_id`, to slow programmatic walk-through of the catalog. This
guardrail is in addition to the existing server-side page-size clamp on
`SearchCatalog`, which SHALL remain in force. When the request-rate limit is
exceeded, the system SHALL respond with `RESOURCE_EXHAUSTED`.

#### Scenario: Rapid catalog enumeration is throttled
- **WHEN** a user issues more `SearchCatalog` requests within the enumeration
  window than the configured maximum
- **THEN** requests beyond the maximum are rejected with `RESOURCE_EXHAUSTED`

#### Scenario: Page-size clamp still applies
- **WHEN** a user requests a `SearchCatalog` page with a `limit` above the server
  maximum
- **THEN** the returned page size is clamped to the server maximum regardless of
  the rate limit

#### Scenario: Rating-deck listing is throttled
- **WHEN** a user pages through `ListRatingDeck` faster than the configured
  enumeration rate
- **THEN** excess requests are rejected with `RESOURCE_EXHAUSTED`

### Requirement: Music-scope admins are exempt; moderators and other-scope admins are not

The catalog is a music-domain resource, so the system SHALL exempt from all catalog
access limits (burst cap, play-aware volume allowance, and enumeration cap) exactly
the callers who hold the `admin` role **in the music scope** — i.e. a `music/admin`
or the `global/admin` break-glass — since they perform legitimate bulk operations.
The exemption SHALL be evaluated with the scope-matched role primitive
(`has_role_in_scope("music", "admin")`), not a scope-agnostic admin check. Every
other caller SHALL be subject to the limits exactly like a regular user; in
particular the `moderator` role SHALL NOT confer any exemption, and an `admin` held
only in an unrelated scope (e.g. `live`) SHALL NOT confer an exemption on the music
catalog.

#### Scenario: Music-scope admin bypasses the guardrail
- **WHEN** a caller holding `admin` in the music scope (a `music/admin`, or a
  `global/admin`) issues catalog browse or download requests
- **THEN** no catalog access limit is applied and the requests are not rejected
  with `RESOURCE_EXHAUSTED` on account of these limits

#### Scenario: Admin scoped only to another domain is not exempt
- **WHEN** a caller who holds `admin` only in an unrelated scope (e.g. `live`) — and
  not in `music` or `global` — exceeds a catalog access limit
- **THEN** the request is rejected with `RESOURCE_EXHAUSTED` like a regular user

#### Scenario: Moderator is subject to the limits
- **WHEN** a caller holding the `moderator` role (but not music-scope admin) exceeds
  a catalog access limit
- **THEN** the request is rejected with `RESOURCE_EXHAUSTED` just as for a regular
  user

#### Scenario: Regular user is subject to the limits
- **WHEN** a caller with no elevated role exceeds a catalog access limit
- **THEN** the request is rejected with `RESOURCE_EXHAUSTED`

### Requirement: Operator-tunable thresholds with a runtime kill-switch

The system SHALL expose each rate-limit threshold (the window duration and maximum
for the download burst, download daily, and enumeration tiers) as a configuration
value. The system SHALL allow these values to be adjusted at runtime through the
feature-flags / config platform without a redeploy. The system SHALL support
disabling the guardrail entirely via a runtime kill-switch so it can be turned off
if it misfires.

#### Scenario: Operator tightens a limit at runtime
- **WHEN** an operator lowers the configured download daily maximum through the
  config platform
- **THEN** subsequent requests are evaluated against the new maximum without a
  service restart

#### Scenario: Kill-switch disables enforcement
- **WHEN** the guardrail is disabled via its runtime flag
- **THEN** catalog egress requests are served without rate-limit rejection

#### Scenario: Safe defaults when config is absent
- **WHEN** no threshold overrides are configured
- **THEN** the system applies documented default thresholds rather than leaving
  egress unlimited

### Requirement: Rate-limit rejections are observable

The system SHALL emit telemetry for rate-limit rejections on catalog egress so that
operators can detect scraping attempts. A user driving a scrape SHALL be visible as
sustained `RESOURCE_EXHAUSTED` outcomes on the protected methods.

#### Scenario: Rejections are counted in metrics
- **WHEN** a request is rejected by a catalog access limit
- **THEN** the rejection is recorded in the RPC metrics with a
  `RESOURCE_EXHAUSTED` outcome for the corresponding method

### Requirement: Client degrades gracefully on rate-limit rejection

The Flutter client (`apps/music`) SHALL translate a `RESOURCE_EXHAUSTED` response
from the score service into a localized, non-technical message indicating the user
should slow down or that a limit was reached, and SHALL NOT surface the raw gRPC
error or enter a retry storm.

#### Scenario: User sees a friendly limit message
- **WHEN** the score gRPC client receives `RESOURCE_EXHAUSTED`
- **THEN** the app shows a localized "slow down / limit reached" message
- **AND** the raw gRPC status string is not shown to the user

#### Scenario: No automatic retry storm
- **WHEN** a request is rejected with `RESOURCE_EXHAUSTED`
- **THEN** the client does not immediately and repeatedly re-issue the same request
