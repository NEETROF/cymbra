## 1. Pending set-password store (seam)

- [x] 1.1 Define a `PendingCredentialStore` trait in `backend/auth/src/` with `put(token, PendingLocalCredential{user_id,email,password_hash}, ttl)`, `take(token) -> Option<PendingLocalCredential>` (get-and-delete for single use), keyed by the verification token. Add `#[cfg_attr(test, automock)]` per the rust-testing skill.
- [x] 1.2 Implement it over `cymbra_platform::cache::Cache` (JSON-serialize the record; namespaced key e.g. `pending_setpw:<token>`; TTL = `verify_ttl`). Store the argon2 hash only — never plaintext.
- [x] 1.3 Wire the store into `AuthModule` construction (new `Arc<dyn PendingCredentialStore>` dependency) and its callers (`server/src/main.rs`, `server/src/lib.rs`, the module test harness).

## 2. Defer binding in `set_local_credential`

- [x] 2.1 In `backend/auth/src/module.rs::set_local_credential`, keep the eager validation (weak-password reject; reject if the account already has a `local` identity via `list_identities`).
- [x] 2.2 Replace `creds.insert` + `link_identity` with: hash the password, mint a UUID token, `pending_store.put(token, {user_id, email, hash}, verify_ttl)`.
- [x] 2.3 Send the verification email via the existing branded/localized path (`email_template::verification_email` + the enqueued job or direct send), carrying the token — content unchanged.
- [x] 2.4 (client, required) `apps/music` set-password flow: on submit success, navigate straight to the existing `OtpVerifyScreen` (email only, no password) so the user verifies in place while staying signed in — the deferred bind removes the old sign-in→`FailedPrecondition`→OTP route. Update the widget test.

## 3. Bind on verification in `verify_email`

- [x] 3.1 In `verify_email(token)`, first `pending_store.take(token)`: on a hit, insert the `local_credential` as **already verified** + `link_identity(user_id, "local", email)`; map an email-taken race to a clean `AlreadyExists` (pending already consumed, so nothing lingers).
- [x] 3.2 On a miss, fall through to the existing sign-up path (`creds.verify_by_token`) unchanged.
- [x] 3.3 Add the credential-store primitive to insert an already-verified credential (or insert+verify in one transaction) so the password is usable immediately; implement in the in-memory fake and the pg repo.

## 4. Tests

- [x] 4.1 Submitting a set-password does **not** create a `local_credentials` row or a `local` identity, and the email stays free (another account can still take it).
- [x] 4.2 Verifying the pending code creates a verified credential + `local` identity, and `sign_in_local` then succeeds (no `FailedPrecondition`).
- [x] 4.3 Contended email: two pending set-passwords for the same email → first verify binds, second verify returns `AlreadyExists` and leaves nothing bound.
- [x] 4.4 Abandoned/expired pending: a `take` miss (expired) leaves no credential/identity and no reservation.
- [x] 4.5 Eager rejects still fire on submit: weak password; account already has a `local` identity.
- [x] 4.6 The existing sign-up verification path (unverified `local_credentials` row) still verifies (regression).
- [x] 4.7 Keep Rust line coverage ≥ 80% (`cargo llvm-cov --workspace --fail-under-lines 80 ...`).

## 5. Finalize

- [x] 5.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `openspec validate verify-before-local-credential-link --strict` passes.
