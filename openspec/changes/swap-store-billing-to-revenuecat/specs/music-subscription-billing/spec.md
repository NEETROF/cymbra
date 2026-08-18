## MODIFIED Requirements

### Requirement: Every purchase is verified server-side before it grants anything

A store purchase (App Store on iOS / macOS, Google Play on Android) SHALL be verified against
the store by the subscription aggregator (RevenueCat) that the app purchases through, and the
entitlement ledger SHALL be written **only** from the aggregator's state — its webhook events or
its customer record read by the backend with a server secret. The app's own purchase state, the
SDK's local customer info, and any payload carried by the app MUST NOT be trusted as a source of
entitlement. After a purchase or a restore the app SHALL ask the backend to synchronise its plan
(`SyncStorePlan`); the backend SHALL read only the calling account's aggregator customer, so no
account can claim another account's subscription. Verification failures at the aggregator SHALL
surface to the app as a localized failure and never as a client-trusted grant.

#### Scenario: Valid App Store purchase grants premium

- **WHEN** a signed-in user completes an App Store purchase of a premium product through the aggregator SDK and the app requests a plan sync
- **THEN** the backend reads the account's aggregator customer, upserts a `premium` row `source = apple` keyed by the store's original transaction id, and returns the premium plan view

#### Scenario: Google purchase grants premium the same way

- **WHEN** a signed-in user completes a Google Play purchase through the aggregator SDK and the app requests a plan sync
- **THEN** a `premium` row `source = google` keyed by the store's original transaction id is upserted from the aggregator customer state; the app never handles or forwards a purchase token

#### Scenario: Sync cannot claim someone else's subscription

- **WHEN** account B requests a plan sync while the store receipt belongs to account A's aggregator customer
- **THEN** account B's customer holds no subscription, no row is written for B, and B's plan is unchanged

#### Scenario: Sandbox purchase in production is ignored

- **WHEN** an aggregator event or customer subscription carries `environment = SANDBOX` and sandbox acceptance is off
- **THEN** no row is written and the event is acknowledged and counted

### Requirement: Provider notifications are ingested idempotently and drive the row's lifecycle

The backend SHALL expose one aggregator notification endpoint (`POST /billing/revenuecat/webhook`)
for the store channels, authenticated by a shared secret carried in the `Authorization` header
and compared in constant time before the body is read. Each event SHALL be stored in
`billing_events` keyed by the aggregator's event id and applied at most once. The event's
`store` SHALL select the ledger `source` (`APP_STORE` / `MAC_APP_STORE` → `apple`,
`PLAY_STORE` → `google`, `PADDLE` → `web`); events from any other store, for products outside
the premium product set, or with a malformed account id SHALL be acknowledged as no-ops and
counted. Initial purchase, renewal, un-cancellation, extension, product change, billing issue
(grace), cancellation (keeps the row until its end), refund (ends now), expiration, pause and
transfer SHALL each map to a defined transition of the entitlement row; informational event
types SHALL be no-ops. When the store's channel flag (`billing.<channel>.enabled`) is disabled
the endpoint SHALL acknowledge and ignore (logged) rather than error, so the aggregator does not
retry indefinitely. The web channel's own provider webhook (`/billing/web/webhook`, HMAC) is
unchanged by this requirement until the web channel is routed through the aggregator.

#### Scenario: Replayed notification is a no-op

- **WHEN** the same aggregator event id is delivered twice
- **THEN** the second delivery changes nothing and is acknowledged

#### Scenario: Renewal moves the row forward

- **WHEN** a `RENEWAL` event arrives for an active `apple` row with a later expiration
- **THEN** the row's `ends_at` moves to the event's expiration and its status is active

#### Scenario: Cancellation keeps access until the end, refund revokes now

- **WHEN** a `CANCELLATION` event with reason `UNSUBSCRIBE` arrives for a row ending in the future
- **THEN** the row is marked cancelled and stays active until its `ends_at`
- **WHEN** a `CANCELLATION` event with reason `CUSTOMER_SUPPORT` (refund) arrives
- **THEN** the row is ended immediately and the effective plan recomputed

#### Scenario: Billing issue opens the grace period

- **WHEN** a `BILLING_ISSUE` event arrives with a grace-period expiration
- **THEN** the row is marked in grace with `ends_at` = that expiration, and access continues until then

#### Scenario: Unauthenticated notification is rejected without side effect

- **WHEN** a notification arrives with a missing or wrong `Authorization` value
- **THEN** it is rejected with 401 and no row or event is written

#### Scenario: Disabled channel acknowledges silently

- **WHEN** `billing.google.enabled` is off and a `PLAY_STORE` event arrives
- **THEN** the endpoint acknowledges, ignores the payload and logs the skip

#### Scenario: Unmapped store or product is a counted no-op

- **WHEN** an event arrives with `store = STRIPE`, or with a `product_id` outside `plans.premium.products`
- **THEN** it is acknowledged, no row is written, and the skip is counted by type

### Requirement: Restore and reconciliation keep the ledger truthful

The app SHALL be able to restore its store purchases at any time through the aggregator SDK and
then request a plan sync, which re-reads the account's aggregator customer and upserts the ledger.
The aggregator's restore policy SHALL be "keep with the original account": a receipt already
bound to another account is refused by the aggregator, surfaced to the user as a localized
"belongs to another account" outcome, and MUST NOT move the subscription. A scheduled
reconciliation SHALL re-read the aggregator customer for rows nearing their end without a recent
event, through the same code path as the plan sync, so a missed notification cannot leave a
paying user degraded or a refunded user active. `TRANSFER` events SHALL nevertheless be applied
(rows of the source account ended, rows for the destination upserted) so a policy change at the
aggregator cannot desynchronise the ledger.

#### Scenario: Restore after reinstall

- **WHEN** a subscriber reinstalls the app, restores purchases and the app requests a plan sync
- **THEN** the aggregator customer shows the active subscription, the row is upserted and the effective plan is `premium`

#### Scenario: Restore from another account is refused

- **WHEN** a user signed in as account B restores a receipt owned by account A
- **THEN** the aggregator refuses the restore, the app shows the localized "another account" outcome, and A's subscription is unchanged

#### Scenario: Missed renewal is repaired

- **WHEN** a row nears its end and no notification was received
- **THEN** the reconciliation job reads the aggregator customer and moves the row forward or lets it lapse accordingly

#### Scenario: Refund caught by reconciliation

- **WHEN** the aggregator customer shows a subscription refunded and no refund event was received
- **THEN** the reconciliation job ends the row

## ADDED Requirements

### Requirement: The aggregator holds an opaque account id only, and forgets it on erasure

The aggregator's app user id SHALL be the Cymbra account id and nothing else: the app SHALL
identify the aggregator SDK with that id at sign-in and reset it at sign-out; no name, email,
handle, or subscriber attribute SHALL ever be sent; ad-network / device-advertising identifiers
SHALL NOT be collected. Account erasure SHALL delete the account's aggregator customer before
purging the account's plan rows; a failed deletion SHALL be retried by the erasure job, never
dropped. The aggregator SHALL be listed as a sub-processor, with its purpose, in the published
privacy policies.

#### Scenario: Only the account id reaches the aggregator

- **WHEN** the aggregator SDK is configured and a purchase is made
- **THEN** the aggregator customer is keyed by the Cymbra account id and carries no attribute set by the app

#### Scenario: Sign-out resets the aggregator identity

- **WHEN** the user signs out
- **THEN** the aggregator SDK is reset so the next sign-in configures the new account's id, and no purchase can be attributed to the previous account

#### Scenario: Erasure deletes the aggregator customer

- **WHEN** an account is erased
- **THEN** the aggregator customer for that account id is deleted and the account's plan rows are purged

### Requirement: Store subscription facts and revenue are observed at the aggregator, not stored

Amounts, currencies and proceeds SHALL NOT be stored in the plan ledger; live active-subscription
counts, monthly revenue (current month included) and lifecycle analytics per store and product
SHALL be read on the aggregator's dashboards. The back-office plan console SHALL link an account
lookup to the aggregator's customer page for support. The ledger keeps identifiers, dates,
status and provider references only.

#### Scenario: Ledger stays amount-free

- **WHEN** an aggregator event carrying `price` and `currency` is ingested
- **THEN** the ledger row and the billing event record hold no amount or currency

#### Scenario: Support jumps to the aggregator

- **WHEN** a music admin looks up an account in the plan console
- **THEN** a link opens that account's customer page at the aggregator
