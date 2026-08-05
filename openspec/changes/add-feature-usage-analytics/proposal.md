## Why

We have no way to answer basic product questions: how many people actually use
Cymbra over a given period, on which platforms and device classes, and which
features they exercise. Today the only signals are incidental server logs and
feature-specific tables (play data, downloads), which cannot be sliced freely and
say nothing about purely client-side actions. We need real usage measurement — but
Cymbra's whole positioning is EU/RGPD-first with data on OVH, so shipping a
third-party SDK (Firebase/Google Analytics) is both a residency problem (US
transfer, CNIL exposure) and a coverage problem (FlutterFire analytics does not
support Windows/Linux desktop, which we ship). The answer is a small first-party
telemetry pipeline that stays on our own infrastructure.

## What Changes

- Introduce a **curated action taxonomy** — a single client-owned registry of
  trackable feature action names (a shared constant, not scattered literals nor a
  freeform free-for-all) — emitted from the Flutter app for both backend-mediated
  and purely client-side actions. The `action` travels as a **shape-validated
  string** (not a frozen proto enum), so new actions ship with normal client
  releases without a coordinated backend deploy; the back-office filter list is
  derived from the actions actually seen in the data.
- Add a batched **`ReportEvents` ingestion RPC**. The Flutter client buffers
  events in a local offline queue and flushes them periodically, best-effort.
  Each event carries `action`, `platform`, `device_class` (phone/tablet/desktop),
  `app_version`, `locale`, `occurred_at`, plus two optional context fields:
  `subject_id` (e.g. the score UUID for play/favorite/upload actions) and `variant`
  (a low-cardinality qualifier such as the play mode or the settings *category* —
  never a setting value).
- Identify users **pseudonymously via a hash of `user_id` salted per period**
  (salt rotates monthly): unique users are countable within a period but a user
  **cannot** be linked across periods — no long-term behavioural profiling.
- Persist a **three-tier store**:
  - `analytics.usage_events` — raw, one row per action, all dimensions. Retention TTL
    is a **config value in `cymbra-feature-flags`** (default 6 months), adjustable
    from the back-office without redeploy. Powers free-form cross-filtering over
    the retention window.
  - `analytics.usage_action_daily` — `(day, action, variant, platform,
    device_class, app_version, locale) → event_count`. Permanent archive for
    long-range trends. (`subject_id` is high-cardinality and stays in raw only.)
  - `analytics.usage_user_daily` — `(day, user_bucket, platform, device_class)`
    presence rows. Permanent. Makes exact `COUNT(DISTINCT)` over any window
    possible (daily distinct counts cannot be summed into a period distinct count).
- Add two **recurring worker jobs** (reusing the existing `job-infrastructure`):
  a daily **rollup** job that folds each closed day into the two `_daily` tables,
  and a daily **purge** job that deletes raw events older than the configured TTL.
  Rollup runs before purge; the multi-month margin makes ordering safe.
- Add a remote **kill-switch flag** in `cymbra-feature-flags` to disable
  collection entirely without a client release.
- Add a back-office **"Usage" screen**: freely combinable filters (date range,
  platform, device class, action) plus a unique-users-by-platform view, reading
  the aggregate tables (and raw within the retention window for max flexibility).
- Add a **consent posture**: collection defaults to **opt-out** (on by default) as
  a first-party audience-measurement regime, with a user-facing toggle. *(This is
  the one open decision flagged for review — opt-out vs explicit opt-in; see
  design.md.)*

## Capabilities

### New Capabilities
- `feature-usage-analytics`: first-party capture, ingestion, pseudonymous
  identification, tiered storage, retention/purge, and daily aggregation of
  product-usage events — the end-to-end pipeline from the Flutter client through
  the `ReportEvents` RPC to the Postgres tables and worker rollups, plus the
  user consent control.
- `usage-analytics-console`: the back-office reporting surface that queries the
  aggregates to answer "how many unique users, by platform/device, over a period"
  and "which actions were performed, filtered in any combination".

### Modified Capabilities
<!-- None. This change consumes the existing runtime-feature-flags and
     job-infrastructure capabilities without altering their spec-level behaviour. -->

## Impact

- **Flutter app** (`apps/music`): new usage-tracking service behind an injectable
  seam + Riverpod provider, offline event buffer, `device_class`/`platform`
  derivation, consent toggle in settings, and taxonomy call sites at tracked
  actions.
- **Proto / API** (`backend/music/proto`): new batched `ReportEvents` RPC on a
  **dedicated `UsageService`** with an event message whose `action` is a
  shape-validated string; regenerated gRPC bindings.
- **Rust backend** (`backend/`, `cymbra-music`): ingestion handler, period-salted
  hashing, host-testable core + repos over the new tables (mockall-doubled).
- **Database**: new migration adding `analytics.usage_events`, `analytics.usage_action_daily`,
  `analytics.usage_user_daily` and their indexes.
- **Worker** (`cymbra-worker`): two new recurring jobs (rollup, purge) reading the
  TTL from `cymbra-feature-flags`.
- **Feature flags** (`cymbra-feature-flags`): new config values — retention TTL and
  a collection kill-switch — surfaced in the back-office flags panel.
- **Back-office** (`apps/back-office`): new "Usage" screen, Pinia store behind the
  injectable client seam, `ts-pattern` `Async<T>` state, Playwright e2e.
- **Privacy/legal**: pseudonymous, period-salted, first-party, retention-bounded;
  consent posture to confirm with the RGPD/CNIL framing.
