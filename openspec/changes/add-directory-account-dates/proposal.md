## Why

An operator looking at an account in the back office cannot tell how old it is or whether
the person still uses Cymbra. Both answers already exist in the database: the account's
creation time (`users.created_at`) and the last time each app minted it a token
(`account_apps.last_seen_at`, change `add-directory-app-usage`). Only the per-app last use
is visible today, buried in the tooltip of each app icon.

## What Changes

- `ListAccounts` returns each account's **sign-up date** (`AccountRow.created_at`, unix
  seconds). Additive field — `buf breaking` passes.
- The Users table gains two columns: **Signed up** and **Last sign-in**. The account
  page shows the same two dates under its title.
- **Last sign-in** is the most recent `last_seen_at` among the account's apps, already on
  the row: no new storage and no new field. It is coarsened to the hour and counts token
  refreshes, so it reads "last signed-in activity"; an account with no app record shows a
  dash.
- The Users table can be **sorted** by handle, name, sign-up date or last sign-in, both
  ways, by clicking the column header. The directory is paginated, so `ListAccounts`
  sorts: it gains `sort` (`handle` | `display_name` | `created_at` | `last_sign_in`,
  empty = handle, anything else `INVALID_ARGUMENT`) and `descending`. Accounts with no
  value for the key come last in both directions. Apps, roles, plan and betas are not
  sortable: they are sets, or live in another service.
- No backfill. Accounts created before `users.created_at` existed (2026-06-29) carry the
  migration time as their sign-up date; they are few and accepted as is. `account_apps`
  started on 2026-09-27, so an account idle since then shows no last sign-in until it
  comes back.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `admin-account-directory` (legacy `admin-*`): the directory exposes each account's
  sign-up date; the Users page and the account page show sign-up and last sign-in.

## Products impacted

- **ID** — new: `created_at` on the directory row (`user-port` `AccountSummary`, the
  Postgres query, `user.proto`). Consumed: `users.created_at` and `account_apps`,
  unchanged.
- **Back office** — new: two columns and sortable headers on `/admin/users`, two dates on
  `/admin/users/{id}`. Consumed: `ListAccounts`.
- **Music / Lingua / Live / Site** — not affected.

## Impact

- `backend/user-port`: `AccountSummary.created_at`, `AccountFilter.sort` / `descending`
  (`AccountSort`); `user.proto` `AccountRow.created_at = 6`, `ListAccountsRequest.sort = 7`
  and `descending = 8`.
- `backend/user`: directory query selects `created_at` and orders by the requested key;
  fake repo mirrors both; gRPC maps and validates.
- `apps/back-office`: regenerated stubs, `UsersView.vue`, `UserDetailView.vue`, a shared
  date helper, the e2e seam, en/fr strings, tests.
