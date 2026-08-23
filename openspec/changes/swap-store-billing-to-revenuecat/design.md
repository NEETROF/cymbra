## Context

`add-premium-subscription` (implemented, dark, manual sandbox validation pending) built the
premium plan on a provider-agnostic ledger — `plans.plan_entitlements` upserted by
`(source, provider_ref)`, `billing_events(provider, event_id, payload_ref)` for idempotency —
fed by three in-house channel adapters in `backend/plans/src/billing/`:

- `apple.rs` (30 K): x5c chain + ES256 verification of StoreKit 2 JWS against bundled Apple root
  CAs, App Store Server Notifications v2 mapper, App Store Server API client (ASC key JWT);
- `google.rs` (21 K): Play Developer API client (service-account JWT), `subscriptionsv2.get` +
  `acknowledge`, RTDN Pub/Sub push with OIDC verification;
- `web.rs` (15 K): Paddle Billing (HMAC webhook, checkout, portal, cancel);
- `reconcile.rs` / `env.rs`: the `plans_reconcile` job re-reading each provider; channel wiring
  from `CYMBRA_APPLE_*` / `CYMBRA_GOOGLE_*` / `CYMBRA_PADDLE_*`.

The app drives StoreKit 2 / Play Billing through `in_app_purchase` behind the `StoreClient`
seam ([store_client.dart](../../../apps/music/lib/services/store_client.dart)) and posts each
receipt (Apple JWS / Google token) to `ReportStorePurchase`; `PurchaseFlow`
([plan_notifier.dart](../../../apps/music/lib/state/plan_notifier.dart)) maps the gRPC
`permissionDenied` of a receipt bound to another account to the `otherAccount` outcome.

Design D7/D8 of that change explicitly refused RevenueCat ("~1 % and one more third party in the
payment truth path"). The owner has since stated the operating need — **live active-subscription
and monthly-revenue visibility across all stores, current month included** — and the ledger has
no amounts at all. Reproducing an aggregator's dashboard in-house is a week of work with a
permanent maintenance tail; RevenueCat is free below 2 500 $ MTR, 1 % above, and ingests Paddle
natively. Decision taken 2026-08-18: swap the store channels to RevenueCat; keep the ledger as
the truth for **access**; RevenueCat is the truth for **store facts** and **analytics**.

Constraints inherited and kept: identifiers only in Cymbra's storage; `plans.enabled` and
`billing.<channel>.enabled` kill-switches; withdrawal on lapse (D13); trials / betas / codes /
admin grants untouched; store builds show no external purchase link, no code entry; sign-in is
required before any purchase.

## Goals / Non-Goals

**Goals:**

- One aggregator (RevenueCat) fronts the App Store (iOS + macOS) and Google Play; one webhook
  and one REST client replace two provider stacks. Paddle is routed into the same aggregator in
  a final, deferred section so Windows / Linux land on the same dashboard.
- The Cymbra ledger keeps `source ∈ {apple, google, web}` and stays the sole input of
  `GetMyPlan`, `can_purchase_here`, "managed on", withdrawal on lapse — those surfaces do not
  change.
- Owner visibility = RevenueCat Overview / Charts (active subscriptions, MRR, revenue with
  month-to-date, trials, churn, by store / product / country) — no in-house revenue storage or
  chart.
- Privacy posture preserved: RevenueCat sees an opaque Cymbra account id and store transaction
  facts, nothing else; the customer is deleted on account erasure; DPA + policies updated.
- Everything removed is removed (no dead adapters behind a flag): less code to keep under 80 %
  and no second truth path.

**Non-Goals:**

- Storing prices / currency / proceeds in `plans` (RevenueCat is the analytics store; the
  identifiers-only rule stays).
- Adopting RevenueCat Offerings / Paywalls / Experiments / Customer Center — the paywall stays
  Cymbra's, products stay the runtime config `plans.premium.products`.
- Changing the site checkout (`site-checkout`, `music-plan-web-api`) or the Paddle portal flow.
- Any change to plan semantics, trials, betas, codes, admin console (beyond one deep link),
  withdrawal, or the flags model.

## Decisions

### D1 — RevenueCat in front of the stores; the ledger stays the truth for access

The app never talks to Apple / Google billing directly any more: `purchases_flutter` performs the
purchase / restore, RevenueCat verifies with the store and tracks the subscription lifecycle.
Cymbra learns store facts two ways, both landing in the same `PlanService` upsert path:

1. **Push** — `POST /billing/revenuecat/webhook` (RevenueCat webhook, D4);
2. **Pull** — `SyncStorePlan` (app after purchase / restore) and the `plans_reconcile` job read
   the customer's subscriptions from the RevenueCat REST API (`GET /v1/subscribers/{app_user_id}`,
   secret API key) and upsert (D3).

Ledger rows keep **`source` = the store**, mapped from RevenueCat's `store`:
`APP_STORE | MAC_APP_STORE → apple`, `PLAY_STORE → google`, `PADDLE → web`; other stores
(`STRIPE`, `AMAZON`, `RC_BILLING`, `PROMOTIONAL`, `TEST_STORE`, …) are **not mapped** and their
events are acknowledged as no-ops (counted). Keeping the store as `source` is what leaves
`GetMyPlan.managed_on`, `can_purchase_here`, the cross-channel refusal and the paywall copy
untouched. `billing_events.provider` gains `revenuecat` (that is where the aggregator shows up),
`provider_ref` = the store's original transaction id **as RevenueCat reports it**
(`original_transaction_id` in webhook events); the REST subscriber payload's per-product
subscription is keyed on the same identity — verified as an explicit sandbox task, with the
fallback of keying on `(store, product_id, app_user_id)` if the two ever disagree.
**Resolution (sandbox runs 7.2–7.4, 2026-08-22/23): they disagree — the fallback is the
implemented key.** A Play `CANCELLATION` carries the renewal-suffixed order id where
`INITIAL_PURCHASE`/`RENEWAL` carry the base one, and an Apple resubscription after a lapse
re-keys the events onto fresh transaction ids; each variant split one subscription across
rows. `provider_ref` is therefore the synthesized `{app_user_id}:{base product id}` (Play's
`:base_plan_id` suffix stripped), present in every webhook event and subscriber entry:
one row per (user, store, product) for the subscription's whole life, resubscriptions land
on the same row, and no store transaction id is persisted at all (a strictly smaller
identifier footprint than the original design).

*Alternatives*: a `source = revenuecat` row family — rejected: it would push the store identity
into a second column and force every consumer (`managed_on`, paywall, BO) to learn a new source
for zero benefit. Adapty (5 000 $ free tier) — second choice; no Paddle integration and a
smaller macOS story. Building the dashboard in-house — rejected in the proposal.

### D2 — App: `RevenueCatStoreClient` behind the unchanged `StoreClient` seam

`in_app_purchase` (+ storekit / android sub-packages) is replaced by `purchases_flutter`, wrapped
in `RevenueCatStoreClient implements StoreClient`:

- `isAvailable()` — configured on iOS / macOS / Android with a non-empty key; `NoopStoreClient`
  stays for Linux / Windows / tests. Public SDK keys per platform come from
  `--dart-define=CYMBRA_RC_APPLE_KEY` / `CYMBRA_RC_GOOGLE_KEY` (public keys, not secrets;
  empty ⇒ no-op client, so CI / dev builds keep working).
- **Identity**: `Purchases.configure(... appUserID: <cymbra user id>)` / `logIn(userId)` on
  sign-in and `logOut()` on sign-out, driven by the same session listener that already scopes
  the plan provider — RevenueCat never sees an anonymous purchase (the paywall requires sign-in
  today) and never sees a second identity for one account. **No attributes** (`setEmail`,
  `setDisplayName`, `$…`) are ever set; ad-network / IDFA collection stays off (SDK default).
- `products(ids)` — `Purchases.getProducts(ids)` (store-localized prices) driven by
  `plans.premium.products` exactly as before; RevenueCat Offerings are not used.
- `buy(productId)` — purchase through the SDK; the SDK finishes the store transaction itself, so
  `complete()` becomes a no-op kept on the interface for symmetry; the outcome is a
  `StoreEvent.receipt` whose `payload` is now **empty** — its meaning changes from "verify this"
  to "the store side is done; sync the plan". `StoreEvent.cancelled / pending / error` map from
  the SDK's `PurchasesErrorCode` (`purchaseCancelledError`, `paymentPendingError`, others), and
  `receiptAlreadyInUseError` becomes a new `StoreEvent.otherAccount` so `PurchaseFlow` keeps
  the `otherAccount` outcome (it used to come from a server `permissionDenied`).
- `restore()` — `Purchases.restorePurchases()` returns `CustomerInfo` synchronously: the client
  emits one `receipt(restored: true)` when an active entitlement is present, or completes with
  nothing; `PurchaseFlow` drops the 3-second "give the store a moment" wait.
- After any `receipt` event `PurchaseFlow` calls **`SyncStorePlan`** (D3) instead of
  `ReportStorePurchase`, adopts the returned plan view, and finishes with `purchased` /
  `restored` / `nothingToRestore` / `otherAccount` / `failed`. Fire-and-observe unchanged.

*Alternative*: read `CustomerInfo.entitlements` in the app and grant locally — rejected: the
ledger (with trials, admin grants, betas, withdrawal) is the truth; the app only ever adopts
server plan views.

### D3 — `SyncStorePlan` (pull) replaces payload verification; the same path reconciles

`ReportStorePurchase(channel, payload, product_id)` and the payload-carrying restore go away.
New RPC `SyncStorePlan()` (caller = the signed-in account, no arguments): the server fetches
`GET /v1/subscribers/{user_id}` from RevenueCat, runs the **pure** `subscriptions → Vec<EntitlementWrite>`
mapper (same rules as the webhook mapper: active while `expires_date` in the future, grace via
`grace_period_expires_date`, ended on `refunded_at`, cancelled-not-renewing on
`unsubscribe_detected_at`, environment rule below), upserts, and returns the `GetMyPlanResponse`.
Because it reads only the **caller's** customer, a user can never claim another user's
subscription through it. Rate-limited per user (`ratelimit::check`, burst small) since it hits a
third party.

`plans_reconcile` keeps its schedule and its selection (rows ending within N days without an
event in the last M days) and calls the same customer fetch — one code path, one mapper.

**Environment rule** kept from D7: `environment = SANDBOX` events / subscriptions are applied
only when `CYMBRA_REVENUECAT_ALLOW_SANDBOX` is on (staging); production acknowledges and
ignores them (counted).

**Restore behaviour** in the RevenueCat project is set to **Keep with original App User ID**:
a receipt already owned by another Cymbra account makes the SDK fail with
`receiptAlreadyInUseError` (→ `otherAccount`) instead of migrating the subscription. `TRANSFER`
events are still mapped defensively (end the rows of `transferred_from`, upsert for
`transferred_to`) so a later policy change cannot desynchronise the ledger.

*Alternative*: keep a payload-carrying RPC and have the server post the receipt to RevenueCat
(`POST /v1/receipts`) — rejected: the SDK already does that with the right store context; the
server would only add a hop and a second place where receipts travel.

### D4 — One webhook, same guarantees as before

`POST /billing/revenuecat/webhook`, mounted only when `CYMBRA_REVENUECAT_WEBHOOK_SECRET` is set:

- **Auth**: constant-time compare of the `Authorization` header with the configured value before
  reading the body; mismatch ⇒ 401, no side effect, counted.
- **Idempotency**: `billing_events (provider = 'revenuecat', event_id = event.id)` — RevenueCat
  reuses `id` on retries; a duplicate is acknowledged and changes nothing.
- **Per-store kill-switch**: the event's `store` selects `billing.apple.enabled` /
  `billing.google.enabled` / `billing.web.enabled`; a disabled store ⇒ 200 + logged skip
  (RevenueCat retries on non-2xx, so we never error on purpose).
- **Pure mapper** (`billing/revenuecat.rs`, table-tested against RevenueCat sample payloads):

  | `type` | ledger transition (row = `(source(store), original_transaction_id)`, product = `product_id`) |
  |---|---|
  | `INITIAL_PURCHASE`, `RENEWAL`, `UNCANCELLATION`, `SUBSCRIPTION_EXTENDED`, `REFUND_REVERSED`, `TEMPORARY_ENTITLEMENT_GRANT` | upsert **active**, `ends_at = expiration_at_ms` (forward-only) |
  | `PRODUCT_CHANGE` | upsert with `product = new_product_id`, `ends_at = expiration_at_ms` |
  | `BILLING_ISSUE` | status **grace**, `ends_at = grace_period_expiration_at_ms` (or unchanged if null) |
  | `CANCELLATION` with `cancel_reason ∈ {UNSUBSCRIBE, PRICE_INCREASE, BILLING_ERROR, DEVELOPER_INITIATED}` | status **cancelled**, row keeps its `ends_at` (active until then) |
  | `CANCELLATION` with `cancel_reason = CUSTOMER_SUPPORT` (refund) or `expiration_at_ms ≤ now` | **ended now** (refund path) |
  | `EXPIRATION` | ended at `expiration_at_ms` (`expiration_reason` logged; `SUBSCRIPTION_PAUSED` ⇒ same) |
  | `SUBSCRIPTION_PAUSED` | ended at `expiration_at_ms` (resume arrives as `RENEWAL`) |
  | `TRANSFER` | end rows of `transferred_from`, upsert for `transferred_to` (D3) |
  | `TEST`, `NON_RENEWING_PURCHASE`, `INVOICE_ISSUANCE`, `VIRTUAL_CURRENCY_TRANSACTION`, `PAYWALL_*`, `EXPERIMENT_ENROLLMENT`, `PURCHASE_REDEEMED`, `PRICE_INCREASE_CONSENT_*`, `SUBSCRIBER_ALIAS`, unknown | **no-op**, acknowledged, counted by type |

  Only `product_id ∈ plans.premium.products` (or `new_product_id`) grants `premium`; other
  products are no-ops. `app_user_id` must be a Cymbra user id (uuid) — malformed ⇒ no-op,
  counted. Unmapped `store` ⇒ no-op.

- **Grace**: `plans.grace_days` still applies on top after `ends_at` (D13 unchanged).

### D5 — Visibility lives in RevenueCat; the ledger stays identifiers-only

The owner's dashboard is RevenueCat Overview + Charts (Active Subscriptions, MRR, Revenue with
month-to-date and net-of-store-fees estimate, Trials, Churn; filters by store / product /
country; Customer Lists). Nothing about amounts is stored on Cymbra's side even though webhook
events carry `price` / `currency` / `commission_percentage`: keeping the identifiers-only rule
avoids a second, drifting revenue store and a new privacy surface. The back-office plan console
gets one **"open in RevenueCat"** link on the account lookup
(`https://app.revenuecat.com/customers/<project>/<app_user_id>`) — support goes to the place
that has the store facts.

*Alternative*: store `price`/`currency` per event for a future BO chart — rejected for now
(RevenueCat exports / Charts API cover it if ever needed).

### D6 — Privacy, RGPD, erasure

- RevenueCat receives: the opaque Cymbra account id, platform / SDK metadata, store transaction
  facts (product, price, currency, country, dates). No name, email, handle, or attribute is ever
  sent (enforced by the seam: no attribute API is exposed).
- RevenueCat is listed as a **sub-processor** in the privacy policies (docs/legal fr/en + site
  copy) with its purpose ("subscription verification and analytics"), its DPA is signed and its
  transfer mechanism (US company, SCCs under the DPA) recorded in the runbook.
- **Erasure**: the account-erasure job gains a `RevenueCatCustomerEraser` port
  (`DELETE /v1/subscribers/{app_user_id}`), called before purging the `plans` rows; failure is
  retried by the job, never silently dropped. The existing web-row cancel (Paddle) stays.
- Sandbox / test customers are deleted the same way.

### D7 — Web / Paddle: direct now, into RevenueCat later, without touching the site

This change keeps `CreateWebCheckout`, `/billing/web/webhook` (HMAC), portal and cancel exactly
as implemented. The **deferred** last section (only when Windows / Linux purchases open) routes
Paddle into RevenueCat by **manual purchase tracking**: our existing HMAC-verified webhook, on
`transaction.completed`, forwards `fetch_token = <paddle subscription id>` +
`app_user_id = <cymbra user id from custom_data>` to `POST /v1/receipts`; from then on the
`PADDLE` store events arrive on the RevenueCat webhook (`source = web`) and the direct Paddle
mapper is retired. Chosen because the site checkout (`apps/site` + Paddle.js overlay,
`site-checkout`) exists and needs no change; RevenueCat Web Purchase Links or the Web SDK would
replace it for no gain. Portal / cancel keep calling Paddle directly (RevenueCat has no portal
for Paddle).

### D8 — Rollout, rollback, what is deleted

Nothing has shipped: channels are dark, no store product is live, no client is released. So the
Apple / Google adapters, their env variables, the Apple root-CA bundle and the `x509-parser` /
`p256` / `p384` dependencies are **deleted**, not flag-gated; rollback is `git revert`. The
`plans.enabled` and `billing.<channel>.enabled` flags keep their meaning. Order:

1. Land the code dark (webhook not mounted until the secret is set).
2. RevenueCat project: apps (iOS+macOS bundle `com.cymbra.music`, Android package), entitlement
   `premium` attached to `premium_monthly` / `premium_yearly` on each store, ASC In-App Purchase
   key + App Store Server Notifications URL → RevenueCat's URL, Play service account JSON + RTDN
   Pub/Sub → RevenueCat's topic, restore behaviour "keep with original", webhook URL + auth
   header, secret API key (v1) for the server, DPA signed.
3. Staging with `CYMBRA_REVENUECAT_ALLOW_SANDBOX=true`: sandbox purchase on iOS / Android →
   RevenueCat customer + webhook → ledger row → `GetMyPlan` premium; renewal, cancel, refund,
   restore on a second device, restore from another account (`otherAccount`), reconciliation.
4. Production: `billing.apple.enabled` then `billing.google.enabled` per D12 of the parent.

## Risks / Trade-offs

- [RevenueCat outage or slow webhook right after a purchase] → `SyncStorePlan` pulls on the spot;
  if that fails too the app shows the localized "pending" outcome and re-syncs on resume / next
  refresh; the ledger is never written from the app.
- [Third party in the ingest path, US data transfer] → identifiers only, DPA + SCCs, sub-processor
  listed, erasure wired; the ledger — not RevenueCat — decides access.
- [1 % above 2 500 $ MTR; RevenueCat prices on gross] → accepted; it starts with revenue and
  buys the dashboards; Adapty stays a documented fallback (same `StoreClient` seam, same webhook
  shape family) if pricing changes.
- [`original_transaction_id` (webhook) vs the REST subscriber keys disagree] → explicit sandbox
  check; fallback key `(store, product_id, app_user_id)` documented in D1.
- [`SUBSCRIPTION_PAUSED` / `auto_resume_at_ms` on Play] → mapped as ended, resumed by
  `RENEWAL`; withdrawal on lapse applies as for any lapse (documented in the store checklist).
- [Sandbox events reaching production] → environment rule (D3), counted.
- [`purchases_flutter` on macOS] → supported by the SDK (App Store build); verified in step 3;
  Linux / Windows stay `NoopStoreClient` + web checkout.
- [Products fetched by id without Offerings] → `getProducts` reads the stores directly; if
  RevenueCat ever requires imported products for entitlement attachment, they are imported in
  step 2 anyway.
- [Lock-in] → moderate: RevenueCat exports customers / transactions; the seam and the mapper
  are the only RevenueCat-specific code.

## Migration Plan

1. Archive (or `/opsx:sync`) `add-premium-subscription` so `music-subscription-billing` and
   `music-premium-paywall` exist in `openspec/specs/` before this change is archived.
2. Backend: `billing/revenuecat.rs` (auth, mapper, REST client, eraser) + route + `SyncStorePlan`
   + reconcile + erasure; delete `apple.rs` / `google.rs` and their wiring; migration widening
   `billing_events.provider` CHECK; deps and env; README / checklist / `.env.example`.
3. App: `RevenueCatStoreClient`, identity hooks, `PurchaseFlow` on `SyncStorePlan`, drop
   `in_app_purchase`; regenerate stubs; tests.
4. Back office: deep link; regenerate client.
5. Legal: policies (fr/en) + site copy; DPA.
6. Sandbox validation per D8; then flags per channel.
Rollback: revert the merge (nothing shipped); the ledger schema change is additive.

## Open Questions

- RevenueCat REST **v1** (`/v1/subscribers`) vs **v2** (`/v2/projects/{id}/customers/{id}/subscriptions`)
  for the customer fetch — v1 is simpler and stable; v2 has richer subscription objects. Start on
  v1; revisit if the provider-ref check in D1 favours v2.
- Whether to also point RevenueCat's **Charts "net revenue"** setting at the Small Business
  Program rate (15 %) once enrolled — an owner dashboard setting, not code.
- When the Paddle section (D7) opens, whether Windows / Linux should move to RevenueCat Web
  Purchase Links instead of the manual forwarder (would touch `site-checkout`) — decide then.
