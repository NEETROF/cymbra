# cymbra-plans

The plans module of the Cymbra backend (change: `add-premium-subscription`).

Two independent axes on an account, both decided server-side:

| Axis | What | Where |
|---|---|---|
| **Plan** `free` / `premium` | multi-source, expiring entitlement ledger (`apple`, `google`, `web`, `code`, `admin`); premium while any row is active; the premium unlock set is **fixed in code** | `plans.plan_entitlements` |
| **Beta memberships** | campaigns of kind `premium_trial` (premium N days from *each* tester's enrolment, default 90) or `feature` (early access via `beta:<key>` flag rollouts, closed by the operator) | `plans.beta_campaigns`, `plans.beta_memberships` |

Plus single-use **access codes** (hashed at rest, one redemption per account per
campaign, redeemed on the web only) and the **billing events** idempotency ledger.

## Rules that never bend

- **Identifiers only.** The schema stores user ids, provider references, dates
  and statuses. Never names, addresses, cards, invoices — the stores and the web
  merchant-of-record own those.
- **Consumers ask `grants(unlock)`**, never a plan name. Keys: `catalog.unlimited`,
  `soundfonts.library`, `soundfont_library.extended`, `scores.extended_quotas`,
  `offline.cache`.
- **A paid row is never created, shortened or ended by a campaign operation.**
- **Security guardrails ignore both axes** (`catalog-access-limits`, auth throttles).
- **Withdrawal on lapse (D13)** happens once, past grace, never on what the user
  owns: rows are claimed (`withdrawn_at`) then the offline cache secret rotates
  through the `CacheSecretRotator` seam.
- **Kill-switch** `plans.enabled` (default off) ⇒ everyone is `free`, no
  memberships, no side effect.

## Layout

```
src/model.rs    pure types (Plan, Unlock, Source, rows, campaigns, snapshot)
src/core.rs     pure decisions: active rows, governing row, memberships,
                enrolment refusals, withdrawal pending, snapshot
src/codes.rs    code generation (130-bit, typeable) + SHA-256 hashing
src/ports.rs    seams (repos, config, clock, rotator, PlanSource, issuer) — mockable
src/service.rs  PlanService: snapshot, redeem/enrol, grants, campaigns, codes,
                provider-event apply, withdrawal sweep, purge
src/pg.rs       Postgres adapters (coverage-excluded, #[ignore] integration tests)
migrations/     plans schema (role plans_svc)
```

`cymbra-music` and the flags service depend on this crate through
`ports::PlanSource` only; `plans` never depends on `music`.

## Purchase channels

Each channel is wired only when its environment block is present
(`backend/.env.example`, "purchase channels"); `billing.<channel>.enabled` gates a
wired channel at runtime. Adapters live in `src/billing/`. Since
`swap-store-billing-to-revenuecat` the **store** channels (App Store on iOS +
macOS, Google Play) go through one aggregator, **RevenueCat**: the app purchases
through its SDK, RevenueCat verifies with the store and tracks the lifecycle, and
Cymbra learns store facts two ways that end in the same forward-only upsert —
the webhook (push) and the customer API (pull: `SyncStorePlan` after a
purchase/restore, and the reconciliation sweep). Ledger rows keep the **store** as
`source` (`APP_STORE`/`MAC_APP_STORE` → `apple`, `PLAY_STORE` → `google`,
`PADDLE` → `web`); `billing_events.provider` is where the aggregator shows up.

| Channel | Authenticates | Route | Reconciliation read |
|---|---|---|---|
| RevenueCat (App Store, Play; Paddle once routed through it) | shared-secret `Authorization` header (constant-time) before the body is read | `POST /billing/revenuecat/webhook` — event id idempotency, per-store `billing.<channel>.enabled` skip, pure mapper (`revenuecat.rs`, D4 table) | `GET /v1/subscribers/{app_user_id}` (secret API key), same mapper rules |
| Web (Paddle, direct) | HMAC `Paddle-Signature` over `ts:body`, replay-bounded | `POST /billing/web/webhook` | through the aggregator once Paddle is routed there |

Every notification is applied at most once (`billing_events`, provider event id);
an unmappable one (unknown store, non-premium product, malformed account id,
sandbox in production, informational type) is acknowledged as a counted no-op; a
disabled channel answers 200 and ignores (no provider retry storm). The
`TRANSFER` event is applied defensively (source rows ended, destinations re-read)
even though the project's restore behaviour is "keep with original App User ID"
— a receipt bound to another account fails in the SDK (`receiptAlreadyInUse`)
instead of migrating.

**What RevenueCat knows** (D6): the opaque Cymbra account id (`app_user_id`),
platform/SDK metadata and store transaction facts (product, price, currency,
country, dates). Never a name, email, handle or attribute — the app's store seam
exposes no attribute API. RevenueCat is a US company; its DPA (SCCs) is signed
and it is listed as a sub-processor in the privacy policies. Revenue and
subscription analytics are read on its dashboards; nothing about amounts is
stored here (D5).

## Operations

- Connection: `CYMBRA_PLANS_DATABASE_URL` (role `plans_svc`, search_path `plans`).
- Flags: `plans.enabled`, `plans.grace_days`, `plans.premium.products`,
  `plans.soundfont_library.max_fonts.{free,premium}`, `plans.scores.*`,
  `billing.{apple,google,web}.enabled`.
- Disable everything: `plans.enabled = false` (no redeploy).
- Disable one purchase channel: `billing.<channel>.enabled = false` — its
  notification endpoint acknowledges and ignores (logged), the paywall hides it.
- Sweeps: `plans_reconcile` (daily 04:10 UTC — re-read paid rows ending within 3
  days) then `plans_withdraw` (04:40 — withdrawal on lapse), both on the ordered
  `plans.maintenance` channel; missed runs are skipped (idempotent).
- Rotate the RevenueCat webhook secret: set a new `Authorization` value on the
  webhook in the RevenueCat dashboard, put it in `CYMBRA_REVENUECAT_WEBHOOK_SECRET`,
  restart (RevenueCat retries the deliveries refused in between). Rotate the API
  key: issue a new v1 secret key in RevenueCat → `CYMBRA_REVENUECAT_API_KEY` →
  restart → revoke the old one. Paddle: change the env value and restart. The
  plans crate holds no key material of its own.
- Disable a store: `billing.apple.enabled` / `billing.google.enabled = false` —
  the paywall hides its button and the webhook acknowledges-and-ignores that
  store's events (not recorded: once re-enabled, the reconciliation sweep
  re-reads the accounts). Unset `CYMBRA_REVENUECAT_*` to unmount the route.
- Sandbox: `CYMBRA_REVENUECAT_ALLOW_SANDBOX=true` on staging only; production
  ignores (counts) `SANDBOX` events and subscriptions.
- Refund path: the store refunds (App Store / Play console, or Paddle portal);
  RevenueCat forwards it (`CANCELLATION` / `CUSTOMER_SUPPORT`) and the row ends
  now; past grace, the next sweep withdraws plan-only content. Support looks the
  account up in the back office → "open in RevenueCat" for the store facts.
- Where revenue is read: RevenueCat Overview / Charts (active subscriptions,
  MRR, revenue with month-to-date, trials, churn; by store / product / country).
- Cohort export: back office → Plans → campaign → members → CSV.
- Erasure: `PlanService::purge_user` (called by the account erasure job); the
  RevenueCat customer is deleted (`DELETE /v1/subscribers/{id}`, idempotent) and
  an active `web` row is cancelled on Paddle first — both outside the erasure
  transaction, so a provider failure retries the job.
