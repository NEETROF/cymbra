## 1. Contract (proto + port)

- [x] 1.1 Add to `backend/user-port/proto/user.proto`: `ListAccountsRequest { uint32 limit = 1; uint32 offset = 2; string query = 3; }`, `AccountRow { string id = 1; string handle = 2; string display_name = 3; repeated string roles = 4; }`, `ListAccountsResponse { repeated AccountRow accounts = 1; uint32 total = 2; }`, and `rpc ListAccounts(ListAccountsRequest) returns (ListAccountsResponse);`.
- [x] 1.2 Regenerate the user-port Rust types and confirm they build (`cargo build -p cymbra-user-port`).

## 2. Backend listing (host-testable core first)

- [x] 2.1 Add a `list_accounts(query, limit, offset) -> Result<AccountPage, RepoError>` method to the user repo trait (`repo.rs`), with `AccountRow { id, handle, display_name, roles }` and `AccountPage { rows, total }`.
- [x] 2.2 Implement it in `pg.rs`: paginated select over `users` LEFT JOIN `user_roles` (aggregate roles at `scope='music'`), ordered by `handle` (nulls last) then `created_at`; optional filter `handle_key LIKE normalize(query) || '%'` OR a `local` identity `lower(subject) = lower(query)`; plus a `COUNT(*)` for `total`.
- [x] 2.3 Add a fake/in-memory impl of the trait method for module/grpc tests.
- [x] 2.4 Unit-test the listing core: page/limit/offset + total, handle filter (case-insensitive, prefix), email filter (local identity), empty result, and role aggregation per account.

## 3. gRPC handler (admin-guarded)

- [x] 3.1 Implement the `ListAccounts` handler in `grpc.rs` behind `require_admin`; clamp `limit` to a sane max; map non-admin → `permission_denied`/`unauthenticated`.
- [x] 3.2 Test the handler: admin lists + filters; non-admin is refused; page/total shape is correct.

## 4. Codegen

- [x] 4.1 Regenerate the Flutter gRPC stubs (`melos run gen-grpc`).
- [x] 4.2 Regenerate the back-office gRPC-web stubs (`cd apps/back-office && yarn gen`).

## 5. Back-office directory page

- [x] 5.1 Add a `list(query, limit, offset)` action to `apps/back-office/src/stores/roles.ts` returning `Async<{ accounts, total }>` (store-only API call; errors folded into the union). After a successful `grant`/`revoke`, re-list the current page so rows refresh.
- [x] 5.2 Rework `RolesView.vue` into a paginated table: filter box (handle/email), columns handle + display name + role badges, per-row grant/revoke for moderator & admin, and pagination controls. Remove the manual UUID field. Keep the per-account audit history (row action). Empty/error via `humanError` (localized, no raw code).
- [x] 5.3 Add/adjust the i18n strings (en + fr) for the filter, columns, per-row actions, pagination, and empty state.

## 6. Coverage & checks

- [x] 6.1 Unit-test the roles store: `list` success populates the union with accounts+total; a grant/revoke triggers a re-list; error → user-facing message.
- [x] 6.2 Add an e2e spec: admin sees a paginated directory, filters by handle, grants a role on a row and the badge updates; empty filter shows the localized "no accounts" message with no raw gRPC code in the DOM.
- [x] 6.3 Run the gates: backend `cargo test`/`clippy` + `cargo llvm-cov` (≥ 80%), and back-office `yarn lint && yarn format:check && yarn typecheck && yarn test:coverage && yarn e2e`.
- [x] 6.4 `openspec validate add-admin-account-directory --strict` passes.
