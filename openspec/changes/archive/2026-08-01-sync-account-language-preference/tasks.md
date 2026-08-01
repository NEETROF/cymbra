## 1. Proto + codegen

- [x] 1.1 In `backend/user-port/proto/user.proto`: add `optional string locale = 7;` to `message Account`; add `message SetLocaleRequest { string locale = 1; }`; add `rpc SetLocale(SetLocaleRequest) returns (Account);` to `service UserService`.
- [x] 1.2 Regenerate the gRPC clients: `melos run gen-grpc` (Dart `apps/music/lib/src/grpc/user.pbgrpc.dart` + Vue `apps/back-office/src/gen/user_pb`).

## 2. Backend — DTO + repo + gRPC

- [x] 2.1 Add `locale: Option<String>` to the `Account` DTO in `backend/user-port/src/lib.rs`.
- [x] 2.2 In `backend/user/src/repo.rs` (`FakeUserRepo`) + `backend/user/src/pg.rs`: `get_account` also returns `locale` (`SELECT ... locale ...`); populate the DTO field. Update the Fake's `account()` helper.
- [x] 2.3 In `backend/user/src/grpc.rs`: carry `locale` in `to_proto` (+ the inline `get_account` mapping); implement `set_locale(req)` — resolve the caller from the token identity, call `port.set_locale(&id.user_id, &req.locale)`, then return the updated account via `port.get_account`.
- [x] 2.4 Ensure `UserModule::get_account` surfaces the stored locale (it delegates to the repo — verify the DTO round-trips the value).

## 3. Backend — tests

- [x] 3.1 UserModule/grpc tests: `set_locale` writes the caller's account and `get_account` returns it; empty locale is a no-op (mockall/Fake per the rust-testing skill).
- [x] 3.2 Enumeration/authz sanity: `set_locale` targets only the caller's identity (not a body-supplied id).
- [x] 3.3 Keep Rust line coverage ≥ 80% (`cargo llvm-cov --workspace --fail-under-lines 80 ...`); `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.

## 4. Flutter (`apps/music`) — account service + sync

- [x] 4.1 In `services/account_service.dart`: add `locale` to the `Account` model and `Future<void> setLocale(String locale)` to `AccountService`.
- [x] 4.2 In `services/grpc_client.dart` (`GrpcAccountService`): map the new proto `locale` in `_toAccount`; add a `setLocale` adapter (via `_authed`, mirroring `updateHandle`).
- [x] 4.3 In `state/app_locale.dart`: `select(AppLanguage)` pushes `AccountService.setLocale` when a session is authenticated (persist-local first); add `applyFromAccount(AppLanguage)` that persists locally **without** pushing (reconcile path).
- [x] 4.4 Add a dedicated **language-sync listener** widget near the top of the app subtree that `ref.listen`s `sessionProvider`: on authenticated, reconcile per design D2/D4 (set+displayable → `applyFromAccount`; unset → push current; undisplayable → no-op). Include the last-server-value echo guard.
- [x] 4.5 Point the language selector UI (`widgets/language_selector.dart`) at the unchanged `select(...)` (no direct service call from the widget).

## 5. Flutter — tests

- [x] 5.1 Notifier/widget tests (mockito fakes via provider overrides — flutter-testing skill): `select` pushes `setLocale` when authenticated and stays local when signed out; login reconcile applies a set+displayable server locale over the local choice and persists it; unset server locale keeps local and pushes it; undisplayable server locale leaves UI + stored value alone; cold start does no pre-auth read.
- [x] 5.2 `melos run analyze` + `dart run custom_lint` clean; coverage gate (very_good_coverage) ≥ 80%.

## 6. Back-office (`apps/back-office`) — store + wiring

- [x] 6.1 Add `stores/locale.ts` (mirroring `stores/roles.ts`): `choose(locale)` → `api().user.setLocale(...)` then `setLocale(locale)`; `reconcile()` → `api().user.getAccount()` and apply via `setLocale` when the returned locale is in `SUPPORTED_LOCALES`. Request state as one `Async<T>` union (`lib/async.ts`), matched exhaustively.
- [x] 6.2 `App.vue`: language buttons call the store's `choose(...)` instead of `i18n` `setLocale` directly (keep `i18n/index.ts::setLocale` pure).
- [x] 6.3 Trigger `reconcile()` after login from `stores/auth.ts` `bootstrap()`/`refresh()`.
- [x] 6.4 Extend the Playwright fake-client seam (`lib/e2e-seam.ts`) with `setLocale` + `getAccount` returning a `locale`.

## 7. Back-office — tests

- [x] 7.1 Store unit tests behind `setClientsForTest`: `choose` calls `setLocale` RPC + updates i18n; `reconcile` applies a supported server locale and ignores an unsupported one.
- [x] 7.2 Playwright: selecting a language persists across a simulated re-login (server value reconciled). `yarn` typecheck/lint clean.

## 8. Finalize

- [x] 8.1 `openspec validate sync-account-language-preference --strict` passes.
- [x] 8.2 Ran `flutter_rust_bridge_codegen generate` if any Rust **public API** changed (N/A — gRPC only; confirm) and `melos run gen-grpc` committed.
