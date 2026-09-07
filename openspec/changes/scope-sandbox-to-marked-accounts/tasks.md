## 1. Store the mark

- [x] 1.1 Migration `backend/plans/migrations/0003_sandbox_accounts.sql`: `plans.sandbox_accounts(user_id uuid primary key, created_at timestamptz not null default now(), created_by text not null)`
- [x] 1.2 Add a `SandboxAccountRepo` port to `backend/plans/src/ports.rs`: `is_sandbox_account(user_id)`, **`sandbox_accounts_among(ids) -> Set`** (the directory decorates a whole page — a per-row read would be an N+1), `set(user_id, by)`, `clear(user_id)`
- [x] 1.3 Implement it over Postgres in `backend/plans/src/pg.rs` (thin I/O, excluded from the coverage gate like its neighbours)
- [x] 1.4 Mockall mock for the port, and a unit test that presence/absence round-trips through the service

## 2. Resolve the flag per account

- [x] 2.1 Rename the mappers' `allow_sandbox` parameter to say it is now per-account, and update their doc comments — the signatures and their existing tests do not change
- [x] 2.2 Webhook handler: resolve the mark from `ev.app_user_id` before calling `map_event`
- [x] 2.3 `sync_customer`: resolve the mark from its `user_id` before calling `map_customer`
- [x] 2.4 Test: a sandbox event for a marked account writes a row; the same event for an unmarked account is skipped as `SkipReason::Sandbox`
- [x] 2.5 Test: a **production** event writes a row for an unmarked account — the mark must not gate production
- [x] 2.6 Test: clearing the mark stops honouring new sandbox events and leaves rows already written untouched
- [x] 2.7 `TRANSFER`: resolve the mark over the union of `transferred_from`, `transferred_to` and `app_user_id`, and honour the event only when all are marked (design D3b)
- [x] 2.8 Test: a sandbox transfer between two marked accounts is applied; the same transfer onto an unmarked account is skipped and moves no row
- [x] 2.9 Test: a sandbox event whose `app_user_id` is not a Cymbra uuid is skipped — it now reports `Sandbox` rather than `MalformedUser`, since the sandbox guard runs first (design D3c)

## 3. Remove the environment flag

- [x] 3.1 Drop `allow_sandbox` from `RcConfig` and `RevenueCatEnv`, and its read in `BillingChannels::build` / `from_env`
- [x] 3.2 Remove `CYMBRA_REVENUECAT_ALLOW_SANDBOX` from `backend/.env.example`, `backend/plans/README.md` and `apps/music/store/SUBSCRIPTIONS.md` (two places: the setup section and the rollout order), documenting the mark in its place
- [x] 3.3 Reword task 7.9b of the still-open `swap-store-billing-to-revenuecat` change — it tells the reader to flip the flag before the first real purchase, which will no longer exist
- [x] 3.4 Check no other reference survives: `grep -rn ALLOW_SANDBOX --exclude-dir=target --exclude-dir=.git .` — `backend/server/src/main.rs` passes the field, and `target/` is full of stale binary matches that drown the real ones

## 4. Admin RPC

- [x] 4.1 `SetSandboxAccount(user_id, enabled)` in `backend/plans/proto/plans.proto`; add `sandbox_account` to `AccountPlanBadge` and `LookupAccountPlanResponse`; add `sandbox_accounts_only` to `ListAccountIdsByPlanRequest`
- [x] 4.2 Implement the RPC in `backend/plans/src/grpc.rs`, gated like `GrantPremium`, and write the change to the existing admin audit trail in both directions
- [x] 4.3 Populate `sandbox_account` in `LookupAccountPlan` and `GetPlansForAccounts`, and honour `sandbox_accounts_only` in `ListAccountIdsByPlan`
- [x] 4.4 Test: a caller without admin authority is refused — covered where it lives. The gate is the shared `guard::require_admin_in_scope(&id, "music")` every admin RPC on this service calls, and `platform/src/guard.rs::admin_in_scope_is_scope_matched` already asserts a `music/admin` is refused elsewhere and a `global/admin` passes. `plans/src/grpc.rs` has no RPC-level harness; building one for a single RPC would test the helper, not this call
- [x] 4.5 Test: setting and clearing both land in the audit trail with the actor
- [x] 4.6 Test: `ListAccountIdsByPlan` with `sandbox_accounts_only` returns only marked accounts, and composes with the plan filter

## 5. Back office

- [x] 5.1 Regenerate the gRPC-web stubs (`yarn gen`) after the proto change
- [x] 5.2 Account page: a "sandbox account" checkbox next to the plan actions, calling the store — the component never calls the API itself, and the request state is one `Async<T>` union
- [x] 5.3 Directory: a sandbox-account filter beside the existing Plan and Bêta filters, and the badge on the row so a filtered list shows why it matched
- [x] 5.4 Locale strings in both `en` and `fr`, aligned — no drift
- [x] 5.5 Component tests: the checkbox reflects the fetched state, toggling calls the store, and a failed call surfaces as the union's error rather than a thrown exception
- [x] 5.6 Directory test: the filter narrows the list

## 6. Ship it

- [x] 6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80`
- [x] 6.2 Back-office lint, typecheck and unit tests green
- [x] 6.3 Name the mark as a pre-submission step in `apps/music/store/README.md`, next to the demo-account notes (design's open question — resolve it here)
- [ ] 6.4 Deploy backend then back office; mark the two review accounts; confirm the filter returns exactly them
- [ ] 6.5 Delete `CYMBRA_REVENUECAT_ALLOW_SANDBOX` from the box's `.env`, roll `server` + `worker`, and confirm with `docker inspect` — `printenv` misreports on this stack
- [ ] 6.6 End-to-end before resubmitting to Apple: a sandbox purchase on a marked account writes an entitlement row, the same purchase on an unmarked account writes none
