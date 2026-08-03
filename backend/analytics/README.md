# cymbra-analytics — first-party feature-usage telemetry

The end-to-end pipeline that answers two product questions on our own
infrastructure (no third-party SDK, EU/RGPD-first): **how many unique users, by
platform and device class, over a period**, and **which features were used**,
filterable in any combination. See `openspec/changes/add-feature-usage-analytics`
for the full proposal/design.

## Shape (data flow)

```
Flutter app                     backend/server            analytics schema (Postgres)      cymbra-worker (daily)
─────────────                   ──────────────            ───────────────────────────      ────────────────────
UsageTrackingNotifier.record()  UsageService.ReportEvents  usage_events (raw, TTL'd)         usage_rollup  → usage_action_daily
  → offline buffer (prefs)  ──▶  validate + bucket    ──▶                              ──▶                   usage_user_daily
  → periodic best-effort flush   insert batch              usage_action_daily (perm.)        usage_purge   → deletes raw > TTL
                                                            usage_user_daily   (perm.)
back-office "Usage" screen ◀── UsageService.Get* (aggregates) ◀──────────────────────────────┘
```

## Taxonomy (client-owned, shape-validated string — NOT a proto enum, design D7)

The single source of truth is the Flutter constant
`apps/music/lib/analytics/usage_actions.dart` (`UsageActions` + `UsageVariants`).
An action only ever originates from an instrumented call site there, so it can
never reach the backend without a client release — governance is that registry +
code review. The server validates only the **shape** (`^[a-z][a-z0-9_]{0,63}$`)
and accepts any well-formed action, even one it has never seen; the back-office
filter list is derived from `SELECT DISTINCT action` in the aggregates. Two
optional context fields (design D8): `subject_id` (high-cardinality, e.g. a score
UUID — kept in **raw only**) and `variant` (low-cardinality qualifier — the play
mode, or a settings **category**, never a setting value; participates in the
action aggregate grain). 12 actions ship now; `guest_session_start` is deferred
(design D9).

## Pseudonymous identity (period-salted `user_bucket`, design D2)

`salt(month) = HMAC(CYMBRA_ANALYTICS_BUCKET_SECRET, "YYYY-MM")`, then
`user_bucket = HMAC(salt, user_id)` (hex). The raw `user_id` is **never** stored
on a usage row and there is no FK to `users`. Distinct users are exact **within** a
month; the salt rotates monthly, so the same user is unlinkable across months (a
range spanning months may count a two-month-active user twice — surfaced as a
caveat on the back-office screen). Nothing is stored to make this work; it is
reproducible within a month from the master secret alone (Option A).

## Storage tiers (design D3, dedicated `analytics` schema — design D10)

- `usage_events` — raw, one row per action, all dimensions. Retention is the
  `data.retention.usage_events_days` **feature-flag** (default 180 ≈ 6 months);
  powers free-form cross-filtering over the window.
- `usage_action_daily` — `(day, action, variant, platform, device_class,
  app_version, locale) → event_count`. Permanent, tiny (all grain columns are
  low-cardinality). Widen the grain here to add a new *historical* filter dimension.
- `usage_user_daily` — `(day, user_bucket, platform, device_class)` presence.
  Permanent; makes `COUNT(DISTINCT user_bucket)` over any window exact (daily
  distinct counts **cannot** be summed into a period distinct count).

## Worker jobs (design D4)

Two daily jobs on the **ordered** `analytics.maintenance` channel:
`usage_rollup` (03:10 UTC) folds each closed day into the two aggregates via
idempotent upserts; `usage_purge` (03:40 UTC) deletes raw older than the retention
window read from the flag. Rollup is strictly before purge (ordered channel +
earlier cron), so purge only ever removes already-aggregated days.

## Runtime controls (feature flags)

- `analytics.collection.enabled` — global kill-switch, **defaults on** (opt-out
  audience measurement). Off ⇒ clients stop emitting on their next flag refresh.
- `data.retention.usage_events_days` — raw retention window (sensitive; BO-editable
  without a redeploy; the purge job reads it each run).

Plus a **per-user consent** toggle in the app (default on, opt-out), distinct from
the operator kill-switch — either one off suppresses emission.

## Wiring / config

The `UsageService` is wired in `backend/server` only when **both**
`CYMBRA_ANALYTICS_DATABASE_URL` (role `analytics_svc`, own schema) **and**
`CYMBRA_ANALYTICS_BUCKET_SECRET` are set (a missing secret must never fall back to
a guessable key). The worker reads the retention flag via `CYMBRA_FLAGS_DATABASE_URL`
(defaults-only, i.e. 180 days, when unset). Provision the schema/role on a live DB
with `backend/deploy/provision-analytics-role.sql`.

## Testing

- Pure logic (`usage_core`: bucketing determinism within/across periods,
  validation, timestamp clamping) is host-unit-tested.
- The `ReportEvents` + reporting handlers are unit-tested with `mockall` doubles
  (auth required, partial batch, admin gating).
- Rollup idempotency, distinct-user exactness, and purge-keeps-aggregates are
  covered by the `#[ignore]` DB integration test
  (`cargo test -p cymbra-analytics --test rollup_it -- --ignored`, needs a live DB
  with the `analytics` schema).
