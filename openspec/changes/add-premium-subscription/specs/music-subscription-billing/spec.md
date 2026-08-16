## ADDED Requirements

### Requirement: Every purchase is verified server-side before it grants anything

A purchase reported by the app (Apple signed transaction, Google purchase token) SHALL be
verified by the backend against the provider — Apple by validating the transaction's signature
chain, bundle identifier and environment; Google by reading the subscription state from the Play
Developer API — before any entitlement row is written. The app's own purchase state MUST NOT be
trusted as a source of entitlement. Verification failures SHALL be rejected and counted, never
downgraded to a client-trusted grant.

#### Scenario: Valid Apple transaction grants premium

- **WHEN** the app reports a signed transaction whose chain, bundle id and environment verify and whose product maps to premium
- **THEN** a `premium` row `source = apple` keyed by the original transaction id is upserted

#### Scenario: Tampered transaction is rejected

- **WHEN** the app reports a transaction whose signature does not verify
- **THEN** no row is written and the attempt is counted

#### Scenario: Google purchase is validated and acknowledged

- **WHEN** the app reports a Google purchase token
- **THEN** the backend reads the subscription from the Play Developer API, upserts the row from that state, and acknowledges the purchase server-side

### Requirement: Provider notifications are ingested idempotently and drive the row's lifecycle

The backend SHALL expose one notification endpoint per channel (Apple App Store Server
Notifications v2, Google Real-Time Developer Notifications, web-provider webhooks). Each request
SHALL be authenticated by the provider's mechanism (Apple JWS chain, Google push OIDC token,
web-provider HMAC) before any side effect, stored in `billing_events` keyed by the provider's
event id, and applied at most once. Renewal, grace, billing retry, cancellation, refund, revocation
and plan change SHALL each map to a defined transition of the entitlement row. When the channel's
flag is disabled the endpoint SHALL acknowledge and ignore (logged) rather than error, so the
provider does not retry indefinitely.

#### Scenario: Replayed notification is a no-op

- **WHEN** the same provider event id is delivered twice
- **THEN** the second delivery changes nothing and is acknowledged

#### Scenario: Refund revokes access

- **WHEN** a provider reports a refund or revocation for a subscription
- **THEN** the corresponding row is ended immediately and the effective plan recomputed

#### Scenario: Unauthenticated notification is rejected without side effect

- **WHEN** a notification arrives with a missing or invalid signature/token
- **THEN** it is rejected and no row or event is written

#### Scenario: Disabled channel acknowledges silently

- **WHEN** `billing.<channel>.enabled` is off and a notification arrives
- **THEN** the endpoint acknowledges, ignores the payload and logs the skip

### Requirement: Web checkout through a Merchant-of-Record, identifiers only

For platforms without a store (Linux, Windows, the web site) the backend SHALL create a hosted
checkout session with the web provider carrying the Cymbra user id as custom data, and return its
URL. The provider SHALL be the merchant of record: Cymbra MUST NOT collect or store card data,
addresses, tax identifiers or invoices; the backend SHALL keep only the provider customer and
subscription identifiers. Managing the subscription SHALL open the provider's hosted portal.

#### Scenario: Desktop purchase lands in the ledger

- **WHEN** a signed-in desktop user completes the hosted checkout
- **THEN** the provider webhook creates a `premium` row `source = web` for that user id

#### Scenario: Manage opens the provider portal

- **WHEN** a web subscriber activates "manage subscription"
- **THEN** a provider-hosted portal URL fetched at request time is opened

#### Scenario: No billing personal data on Cymbra's side

- **WHEN** the web adapter's storage is inspected
- **THEN** it holds provider identifiers and event payload references only

### Requirement: A subscription bought on any channel is active on every platform

The effective plan SHALL be computed from the ledger regardless of the channel that produced the
row, so a subscription purchased on one platform is active on all others for the same account.
Purchase entry points SHALL be hidden on a platform when an active paid row from another channel
exists, and the plan surface SHALL say where it is managed instead.

#### Scenario: Web subscription works on iPhone

- **WHEN** a user subscribes on the web from Linux and signs in on the iOS build
- **THEN** the iOS build reports `premium` and offers no purchase

#### Scenario: No double subscription

- **WHEN** a user with an active `google` row opens the paywall on the App Store build
- **THEN** no purchase is offered and the surface says the subscription is managed on Google Play

### Requirement: Restore and reconciliation keep the ledger truthful

The app SHALL be able to re-assert its store transactions ("restore purchases") at any time,
which re-runs server-side verification. A scheduled reconciliation SHALL re-read the provider
state for rows nearing their end without a recent event, so a missed notification cannot leave a
paying user degraded or a refunded user active.

#### Scenario: Restore after reinstall

- **WHEN** a subscriber reinstalls the app and restores purchases
- **THEN** the transactions are re-verified and the effective plan is `premium`

#### Scenario: Missed renewal is repaired

- **WHEN** a row nears its end and no notification was received
- **THEN** the reconciliation job reads the provider state and moves the row forward or lets it lapse accordingly
