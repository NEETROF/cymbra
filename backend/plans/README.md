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

## Operations

- Connection: `CYMBRA_PLANS_DATABASE_URL` (role `plans_svc`, search_path `plans`).
- Flags: `plans.enabled`, `plans.grace_days`, `plans.premium.products`,
  `plans.soundfont_library.max_fonts.{free,premium}`, `plans.scores.*`,
  `billing.{apple,google,web}.enabled`.
- Disable everything: `plans.enabled = false` (no redeploy).
- Disable one purchase channel: `billing.<channel>.enabled = false` — its
  notification endpoint acknowledges and ignores (logged), the paywall hides it.
- Sweep: `plans_withdraw` (daily, after `plans_reconcile`).
- Erasure: `PlanService::purge_user` (called by the account erasure job); an
  active `web` row is cancelled on the provider first.
