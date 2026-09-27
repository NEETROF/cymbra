## 1. Record the apps (ID)

- [x] 1.1 Migration `backend/user/migrations/0011_account_apps.sql`: `account_apps(user_id, app, first_seen_at, last_seen_at)`, PK `(user_id, app)`, FK to `users` `ON DELETE CASCADE`, `CHECK (app IN ('music','live','lingua'))`, index on `(app, user_id)` (design D1)
- [x] 1.2 Drift test: the migration's app list equals `cymbra_platform::APP_SCOPES`
- [x] 1.3 `UserRepo::touch_app(user_id, app)` — Postgres upsert throttled to one write per hour (design D2) + the in-memory fake
- [x] 1.4 `UserPort::record_app_use(user_id, audience)`: records app scopes, ignores `back-office` / `web`; unit tests for both
- [x] 1.5 `AuthModule`: record the app in `issue` and `refresh`, logging and swallowing a failure; tests that a sign-in/refresh records the audience and that a failing record still returns tokens

## 2. Directory API

- [x] 2.1 `user.proto`: `AccountApp { app, last_seen_at }`, `AccountRow.apps = 5`, `ListAccountsRequest.apps = 6` (additive)
- [x] 2.2 `AccountSummary.apps` and `AccountFilter.apps` in `user-port`; the default `list_accounts_filtered` refuses an apps filter like an id filter
- [x] 2.3 Postgres directory query: AND-filter by apps in the shared `WHERE`, fetch the page's apps in one query; fake repo mirrors it
- [x] 2.4 gRPC: validate app values (`INVALID_ARGUMENT`), map the apps onto `AccountRow`; tests for the filter, the rows and the refusal

## 3. Back office

- [x] 3.1 Regenerate the gRPC-web stubs (`yarn gen`)
- [x] 3.2 Roles store: an `apps` criterion carried in `params`, passed to `listAccounts`
- [x] 3.3 `UsersView.vue`: App column with the Music / Lingua icons (accessible name + latest-sign-in tooltip, dash when none) and the App filter (any / Music / Lingua / both)
- [x] 3.4 en/fr strings, aligned
- [x] 3.5 e2e seam: accounts carry apps; `listAccounts` honours the filter
- [x] 3.6 Unit test (store passes the filter) + Playwright: icons on the rows, filtering by Lingua and by both

## 4. Gates

- [x] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov` ≥ 80 %
- [x] 4.2 Back office: lint, type-check, vitest, Playwright
- [x] 4.3 `openspec validate add-directory-app-usage --strict`
