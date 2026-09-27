## Why

The back-office Users directory lists every Cymbra ID account, but nothing on a row says
which product the person actually uses. Since Lingua shipped, one account can be a Music
player, a Lingua reader, both, or neither (an account created on the site and never used
in an app). An operator answering "is this a Lingua user?" — for support, for a beta
invitation, before revoking sessions — has no way to tell, and no way to list "the
accounts that use Lingua".

Nothing durable records it today. `auth.sessions` carries the app audience, but a session
row is deleted on sign-out, on revocation and 30 days after its last refresh, so it
answers "is signed in right now", not "uses this app".

## What Changes

- Cymbra ID records, per account, **which apps it has signed in to** — one row per
  (account, app) with the first and the last use — written whenever an access token
  is minted for an app audience (`music`, `lingua`, `live`): at sign-in and at every
  refresh. `back-office` and `web` (the site) are not apps and are not recorded.
- `ListAccounts` returns each account's apps and accepts an **apps filter** (accounts that
  used every listed app), applied in the same query as the handle/email search so the
  total and the pagination stay exact.
- The Users page shows an **icon per app** on each row (Music, Lingua) with its last use in the
  tooltip, and gains an **App** filter: any / Music / Lingua / Music and
  Lingua.
- No historical backfill: the table fills as clients sign in or refresh. Every live
  session refreshes the next time its app opens, so an active user gets their icon on
  first use after the deploy; an account idle for longer has no live session to backfill
  from anyway.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `user-account` (legacy `id-*`): an account records the apps it has signed in to.
- `admin-account-directory` (legacy `admin-*`): the directory exposes and filters by the
  apps an account uses; the Users page shows them as icons and offers the filter.

## Products impacted

- **ID** — new: the per-account app record (table in `user_account`, a `UserPort`
  method, the write from the auth module). Consumed: the existing session/audience flow,
  unchanged.
- **Back office** — new: the icon column and the App filter on `/admin/users`. Consumed:
  `ListAccounts`.
- **Music / Lingua / Live** — nothing changes client-side; their existing sign-in and
  refresh calls are what records the usage.
- **Site** — not affected (`web` is not recorded).

## Impact

- `backend/user`: migration `0011_account_apps.sql`, repo + Postgres adapter, the
  `record_app_use` port method, the directory query (column + filter).
- `backend/user-port`: `UserPort::record_app_use`, `AccountSummary.apps`,
  `AccountFilter.apps`; `user.proto` gains `AccountRow.apps` and
  `ListAccountsRequest.apps` (additive — `buf breaking` passes).
- `backend/auth`: `issue` and `refresh` record the app, best effort (a failure is logged
  and never blocks a sign-in).
- `apps/back-office`: regenerated stubs, roles store, `UsersView.vue`, the e2e seam, en/fr
  strings.
- Write cost: at most one small upsert per (account, app) per hour — the write is skipped
  while the stored last use is under an hour old.
