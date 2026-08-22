## Why

`add-premium-subscription` shipped three in-house purchase channels (Apple JWS verification +
App Store Server Notifications, Google Play Developer API + RTDN, Paddle webhook) that write into
one provider-agnostic ledger. Before the first sandbox validation, the owner restated the actual
business need: **live visibility of active subscriptions and of monthly revenue (current month
included) across iOS / macOS / Android — and later Windows / Linux through the web channel**. The
ledger stores identifiers only (no amount, no currency: `billing_events` = provider, event id,
payload digest), so that dashboard does not exist and building it in-house means a week of
work plus a permanent tail (currency normalisation, refunds, proration, store schema changes,
Apple root-CA rotation, Play API deprecations) to reproduce what a subscription aggregator sells.
RevenueCat is free below 2 500 $ of monthly tracked revenue and charges 1 % above — a cost that
only starts once there is revenue to look at — and now ingests Paddle natively, so one aggregator
can cover every channel Cymbra will ever have. The 2026-08 decision reverses design D7/D8 of
`add-premium-subscription` ("no middleware, JWS verified in-house"), knowingly.

## What Changes

- **RevenueCat becomes the single store aggregator** for the App Store (iOS + macOS) and Google
  Play channels: the app talks to the stores through the RevenueCat SDK (`purchases_flutter`)
  behind the existing `StoreClient` seam; RevenueCat verifies receipts, tracks renewals, grace,
  billing issues, cancellations, refunds, product changes and restores.
- **One notification endpoint** `POST /billing/revenuecat/webhook` (shared-secret `Authorization`
  header, event-id idempotency) replaces `/billing/apple/notifications` and `/billing/google/rtdn`.
  A pure mapper turns RevenueCat event types into the existing ledger transitions; rows keep
  `source ∈ {apple, google, web}` (mapped from the event's `store`) so `GetMyPlan`,
  `can_purchase_here`, "managed on <channel>" and the paywall are **untouched**.
- **The Cymbra ledger stays the truth for access**; RevenueCat is the truth for *store facts* and
  for *revenue analytics*. `ReportStorePurchase` (payload-carrying) and the app-side
  `RestorePurchases` RPC are replaced by a payload-less **`SyncStorePlan`**: after a purchase or a
  restore the app asks the server to pull its RevenueCat customer and reconcile the ledger from it
  (same code path as the reconciliation job). **BREAKING** for the unreleased `PlanService` proto
  (no client shipped; channels are dark).
- **Retired**: `billing/apple.rs` (x5c/ES256 verifier, App Store Server API client, bundled Apple
  root CAs), `billing/google.rs` (Play Developer API client, RTDN OIDC verification), the
  Apple/Google arms of `billing/reconcile.rs`, their env variables and secrets, the `x509-parser`
  / `p256` / `p384` dependencies. `in_app_purchase` is dropped from the app.
- **Web (Paddle) channel, phased**: the direct Paddle path (`CreateWebCheckout`, `/billing/web/webhook`,
  portal, cancel-on-erasure) stays as is in this change; a final, explicitly deferred section
  routes Paddle subscriptions into RevenueCat as well (manual purchase tracking from our existing
  HMAC-verified webhook: forward `sub_…` + Cymbra user id) so the dashboards cover Windows / Linux
  when that channel opens. `site-checkout` / `music-plan-web-api` are not modified.
- **Identity, privacy, erasure**: the RevenueCat `app_user_id` is the Cymbra account id and
  nothing else (no attributes, no email, no IDFA/Search-Ads collection); RevenueCat's restore
  behaviour is set to *keep with original app user id* so a receipt can never migrate to another
  Cymbra account; account erasure deletes the RevenueCat customer; RevenueCat is named as a
  sub-processor in the privacy policies (docs/legal + site) and its DPA is signed.
- **Visibility**: RevenueCat Overview / Charts (active subscriptions, MRR, revenue with month-to-date,
  trials, churn, by store / product / country) is the owner's dashboard; the back-office plan
  console gains a "open in RevenueCat" link on the account lookup. No in-house revenue chart.
- **Docs**: `apps/music/store/SUBSCRIPTIONS.md` and `backend/plans/README.md` rewritten for the
  RevenueCat setup (ASC In-App Purchase key + notification URL and Play service account +
  Pub/Sub topic are now configured **in RevenueCat**, not in our environment);
  `add-premium-subscription` design D7/D8 get a "superseded by" note.

## Capabilities

### New Capabilities

_None — the plan, paywall and billing capabilities exist; this change alters how the store
channels are fulfilled._

### Modified Capabilities

- `music-subscription-billing`: "every purchase is verified server-side" becomes "verified by
  the aggregator, ledger reconciled from the aggregator's customer state, never from the app";
  the per-provider notification endpoints become one aggregator webhook with the same
  idempotency, authentication and disabled-channel semantics; restore/reconciliation re-read
  the aggregator instead of each provider; a new requirement pins identity + erasure at the
  aggregator (opaque id, no attributes, delete on erasure, no cross-account transfer).
- `music-premium-paywall`: "restore purchases and refresh" now goes through the aggregator SDK
  and settles from its result (no timed wait); the account-settings *manage* action for store
  rows is unchanged; the "receipt bound to another account" outcome is produced by the
  aggregator's restore policy rather than by a server refusal.

_Note — ordering_: both capabilities exist today only as delta specs of the in-flight
`add-premium-subscription`; that change MUST be archived (or its specs synced with
`/opsx:sync`) before this one is archived, so the deltas below apply on top of them.

## Impact

Products: **Music** (app iOS/macOS/Android + backend `cymbra-plans` / `cymbra-server` /
`cymbra-worker`) consumes RevenueCat; **back-office** gets one link; **site** unchanged in this
change (Paddle→RevenueCat forwarding is the deferred last section); **ID** untouched.

- Backend: `backend/plans/src/billing/{apple,google,reconcile,env}.rs` (removed / rewritten),
  new `billing/revenuecat.rs` (webhook auth + pure event mapper + REST customer client),
  `backend/server/src/billing.rs` routes, `backend/plans/proto/plans.proto` (`SyncStorePlan`
  replaces `ReportStorePurchase`), `grpc.rs` / `service.rs`, `billing_events.provider` CHECK
  gains `revenuecat`, worker reconciliation + erasure (`RevenueCatCustomerEraser`), Cargo deps,
  `.env.example`, coverage ignore list.
- App: `pubspec.yaml` (`purchases_flutter` in, `in_app_purchase` out), `lib/services/store_client.dart`
  (`RevenueCatStoreClient`, identity hooks), `lib/state/plan_notifier.dart` (`PurchaseFlow`
  → `SyncStorePlan`, restore settles synchronously), regenerated Dart stubs, tests.
- Back office: `plans` store + view (one deep link), regenerated client.
- Ops/legal: RevenueCat project + apps + entitlement `premium` + products + webhook + API keys,
  DPA, privacy policies (fr/en), store checklist, README runbook; secrets rotated out of the
  environment (`CYMBRA_APPLE_ASC_*`, `CYMBRA_GOOGLE_SA_*`, `CYMBRA_GOOGLE_RTDN_*`) and in
  (`CYMBRA_REVENUECAT_*`).
- Not affected: plan model, trials / betas / codes, admin grants, withdrawal on lapse, flags
  `billing.apple.enabled` / `billing.google.enabled` (still gate the purchase button and the
  webhook's per-store acceptance), Caddy (`/billing/*` already routed to Axum).
