# `cymbra-notifications` — the push platform

Server-driven push for **iOS, Android and macOS** through a single provider (FCM
HTTP v1; Apple platforms via FCM→APNs bridging). Windows and Linux have no
reliable app-closed push path: their clients never register a token and are never
targeted — features keep those users informed in-app.

This crate is **infrastructure, not a notification**. It ships no concrete
notification type; the section below is the contract a feature implements to add
one.

## What lives where

| Piece | Module | Covered? |
| --- | --- | --- |
| `NotificationService` gRPC (register/unregister/prefs/timezone) | `grpc.rs` | excluded (thin adapter) |
| Registry port (`PushRegistry`) + value types | `repo.rs` | yes |
| Postgres registry over `user_account.*` | `pg.rs` | excluded (I/O glue) |
| Recipient selection — **every** consent/flag/hour gate | `select_core.rs` | yes |
| Send port (`PushSender`), `PushMessage`, `SendOutcome` | `sender.rs` | yes |
| FCM HTTP v1 client | `fcm.rs` | excluded except `classify` |
| Selection → send → prune loop | `dispatch.rs` | yes |

Storage lives in the **`user_account` schema** (migration
`backend/user/migrations/0008_push_notifications.sql`): `push_tokens`,
`notification_prefs`, and `users.timezone`. Device tokens and preferences are
account data with an FK to `user_account.users`, so account erasure cascades them
away — no cross-schema cleanup job.

## Adding a notification type

A *type* is a category id plus four things the platform asks of you. None of them
touch this crate.

### 1. Pick a category id

A stable snake_case identifier, e.g. `practice_streak`. It is the key of the
user's preference row and of both feature flags.

### 2. Declare the two flags

In `cymbra-feature-flags`'s `registry::builtin()`, add the keys built by
`registry::category_enabled_key` / `registry::category_hour_key`. Declare them
under `APP_ALL` — they are server-side send configuration, evaluated by the worker
which has no app context:

```rust
flag(
    "notifications.category.practice_streak.enabled",
    APP_ALL,
    false, // safe state: a category ships disabled and is turned on from the BO
    false,
    "Evening practice-streak reminder.",
),
cfg(
    "notifications.category.practice_streak.hour",
    APP_ALL,
    FlagValue::Int(20), // the LOCAL hour the reminder fires
    false,
    "Local hour (0–23) the practice-streak reminder fires.",
),
```

Both keys are hot-reloadable and appear automatically in the back-office
notifications panel, which lists everything under the `notifications.` prefix. An
**undeclared** category resolves to `enabled = false`, so a half-landed feature
sends nothing.

Omitting the hour key makes the send **event-triggered**: no hour gate applies and
the dispatch fires whenever it is enqueued.

### 3. Bring your candidate query

The platform does not know who is eligible — you do. Resolve your user ids in your
own module (e.g. `streak > 0 AND last_played_date < today`) and pass them as the
job's `user_ids`. Omit `user_ids` to target every user with a registered device.
The platform then loads their tokens/timezones/preferences and applies the shared
gates.

### 4. Enqueue the dispatch

Scheduled — one `jobs.schedules` row for the `push_dispatch` job. Run it **hourly**
and let the selection core's local-hour gate pick out the users for whom it is
currently the configured hour:

```sql
INSERT INTO jobs.schedules (name, kind, cron_expr, timezone, payload, enabled, missed_run_policy)
VALUES (
  'practice_streak_reminder', 'push_dispatch', '0 * * * *', 'UTC',
  '{"category":"practice_streak","title":"Ta série","body":"Joue aujourd''hui pour la garder !"}',
  true, 'skip'
);
```

Event-triggered — enqueue it from your module through the usual
`cymbra_jobs::EnqueueRequest` / `jobs.enqueue` seam with the same payload plus
`user_ids`.

Payload fields (`PushDispatchJob` in `backend/worker/src/handlers.rs`):

| Field | Required | Meaning |
| --- | --- | --- |
| `category` | yes | your category id |
| `title` / `body` | yes | the **already-localized** copy — the platform does not translate |
| `data` | no | routing payload delivered with the notification, e.g. `{"route": "/practice"}` |
| `user_ids` | no | your candidate set; absent ⇒ every registered device |
| `default_pref` | no | what an absent user preference means for this category (default `true` = opt-in) |

### 5. Client toggle (optional but expected)

The app renders a per-category switch by calling `SetNotificationPref(category,
enabled)`. Nothing platform-side needs to change; the toggle writes the row your
category's selection reads.

## The gates, in order

`select_recipients` applies these and nothing else decides who gets a message:

1. **kill-switch** `notifications.enabled` off ⇒ nobody, any category.
2. **category enable** off ⇒ nobody for that category, whatever users chose.
3. **token present** — a blank token is skipped; duplicates collapse.
4. **user preference** — explicit `false` skips; absent falls back to the
   category's `default_pref`.
5. **local hour** — when a schedule hour is configured, the user's local hour must
   match. Unknown/unparsable timezone falls back to `DEFAULT_TIMEZONE`.

Every gate fails **closed**: an unreadable flag store, an undeclared category or an
unconfigured Firebase project all result in zero sends.

## Configuration

| Variable | Where | Effect |
| --- | --- | --- |
| `CYMBRA_FCM_SERVICE_ACCOUNT_JSON` | worker | Firebase service-account key JSON. Unset ⇒ `push_dispatch` completes as a no-op. |

The registry RPCs need no configuration — clients can register tokens and set
preferences before a Firebase project exists.
