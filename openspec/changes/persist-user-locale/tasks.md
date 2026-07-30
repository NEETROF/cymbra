## 1. Storage

- [ ] 1.1 Add migration `backend/user/migrations/0007_locale.sql`: `ALTER TABLE users ADD COLUMN locale TEXT;` (nullable; user_account schema).

## 2. UserPort + repo

- [ ] 2.1 Add to `UserPort` (`backend/user-port/src/lib.rs`): `set_locale(&self, user_id: &str, locale: &str) -> Result<()>` (upsert; no-op when `locale` is empty) and `locale(&self, user_id: &str) -> Result<Option<String>>`. Keep `#[automock]` intact.
- [ ] 2.2 Implement the repo methods in `backend/user/src/repo.rs` (trait) + `backend/user/src/pg.rs` (SQL: `UPDATE users SET locale = $2, updated_at = now() WHERE id = $1`; `SELECT locale FROM users WHERE id = $1`).
- [ ] 2.3 Implement them in `UserModule` (`backend/user/src/module.rs`): `set_locale` skips the write on empty input; `locale` returns the stored value.

## 3. Auth wiring (effective-locale precedence)

- [ ] 3.1 In `sign_up_local` (`backend/auth/src/module.rs`): after `resolve_or_provision`, call `user.set_locale(uid, locale)` when the request locale is non-empty.
- [ ] 3.2 In `resend_verification`: inside the existing "cred exists & unverified" branch, resolve the user id and `set_locale` when the request carries one; compute the effective locale (request → `user.locale(uid)` → English) and render with it.
- [ ] 3.3 In `request_password_reset`: inside the existing "account exists" branch (keep the uniform response), `set_locale` when carried, and render with the effective locale (request → stored → English). No lookup outside that branch.
- [ ] 3.4 Add a small helper to resolve `SupportedLocale` from `(request_locale, stored_locale)` so the precedence is defined once.

## 4. Tests

- [ ] 4.1 UserModule tests: `set_locale` writes and is a no-op on empty; `locale` reads back the stored value (mockall-backed repo per the rust-testing skill).
- [ ] 4.2 Auth tests: sign-up records the locale; a later resend with no request locale renders in the stored locale; request locale overrides stored; no stored + no request → English.
- [ ] 4.3 Enumeration test: `request_password_reset` returns the same uniform response for existing vs non-existing accounts with the stored-locale lookup in place.
- [ ] 4.4 Keep Rust line coverage ≥ 80% (`cargo llvm-cov --workspace --fail-under-lines 80 ...`).

## 5. Finalize

- [ ] 5.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 5.2 `openspec validate persist-user-locale --strict` passes.
