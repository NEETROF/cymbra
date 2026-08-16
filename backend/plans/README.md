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
wired channel at runtime. Adapters live in `src/billing/`:

| Channel | Verifies | Route | Reconciliation read |
|---|---|---|---|
| Apple | signed transaction / notification JWS: `x5c` chain to the pinned Root CA G3, ES256, bundle id, environment | `POST /billing/apple/notifications` (ASN v2) | App Store Server API (JWT with the ASC key) |
| Google | Play Developer API `subscriptionsv2.get` (+ server-side acknowledge) | `POST /billing/google/rtdn` (Pub/Sub push, Google OIDC token) — state is re-read from the API, never trusted from the body | same API |
| Web (Paddle) | HMAC `Paddle-Signature` over `ts:body`, replay-bounded | `POST /billing/web/webhook` | `GET /subscriptions/{id}` |

Every notification is applied at most once (`billing_events`, provider event id);
an unmappable one (no account token, no known row) is acknowledged and logged; a
disabled channel answers 200 and ignores (no provider retry storm).

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
- Rotate a webhook / API secret: change the env value and restart — the plans
  crate holds no key material of its own; the Apple root CA is pinned in
  `certs/AppleRootCA-G3.cer` (expires 2039).
- Refund path: the provider refunds (store / Paddle portal); the notification
  ends the row and, past grace, the next sweep withdraws plan-only content.
- Cohort export: back office → Plans → campaign → members → CSV.
- Erasure: `PlanService::purge_user` (called by the account erasure job); an
  active `web` row is cancelled on the provider first.
