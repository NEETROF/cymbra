## 1. Directory API (ID)

- [x] 1.1 `user.proto`: `AccountRow.created_at = 6` (unix seconds, additive)
- [x] 1.2 `user-port`: `AccountSummary.created_at`
- [x] 1.3 Postgres directory query selects `extract(epoch FROM created_at)`; fake repo mirrors it
- [x] 1.4 gRPC maps `created_at` onto `AccountRow`; tests on the repo fake and the gRPC mapping

## 2. Back office

- [x] 2.1 Regenerate the gRPC-web stubs
- [x] 2.2 Date helper in `src/lib/` (unix seconds → locale date) + `lastSignIn(apps)`; the app tooltip reuses it; unit tests
- [x] 2.3 `UsersView.vue`: Signed up and Last sign-in columns (dash when none)
- [x] 2.4 `UserDetailView.vue`: both dates under the title
- [x] 2.5 en/fr strings, aligned
- [x] 2.6 e2e seam: accounts carry `createdAt`; Playwright asserts both dates in the table and on the account page

## 3. Sorting

- [x] 3.1 `user.proto`: `ListAccountsRequest.sort = 7`, `descending = 8`; `AccountSort` + `AccountFilter.sort` / `descending` in `user-port` (the default port path refuses a non-default order)
- [x] 3.2 Postgres: whitelisted `ORDER BY` per key, `NULLS LAST`, ties by creation then id; fake repo mirrors it; gRPC refuses an unknown key; tests for every key, both directions, and the refusal
- [x] 3.3 Back office: `sort` / `descending` in the roles store + `sortBy` (flip on same column, text A→Z, dates newest first, back to page 1); sortable headers with `aria-sort`; e2e seam sorts; vitest + Playwright

## 4. Gates

- [x] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, user crate tests
- [x] 4.2 Back office: lint, type-check, vitest, Playwright
- [x] 4.3 `openspec validate add-directory-account-dates --strict`
