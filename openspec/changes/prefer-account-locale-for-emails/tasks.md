# Tasks

## 1. The rule (backend/auth)

- [ ] 1.1 In `backend/auth/src/module.rs`, `email_locale(stored: Option<&str>, request: &str) -> SupportedLocale` replaces `effective_locale`: a non-empty stored tag, else a non-empty request tag, else English, each parsed by `SupportedLocale::parse`; its doc comment names this change and the precedence (D1).
- [ ] 1.2 `AuthModule::account_email_locale(user_id, request) -> Result<SupportedLocale>`: reads `self.user.locale(user_id)`, calls `self.user.set_locale(user_id, request)` only when nothing non-empty is stored and `request` is non-empty, and returns `email_locale(stored, request)` (D1, D2).
- [ ] 1.3 `sign_up_local` calls it on the account `resolve_or_provision` returns, in place of its `set_locale` and `SupportedLocale::parse(Some(locale))`; the job payload is unchanged (D4).
- [ ] 1.4 `resend_verification` and `request_password_reset` call it inside their account-exists branch, in place of `set_locale`, `locale` and `effective_locale` (D2, D5).
- [ ] 1.5 `set_local_credential` calls it on the caller's `user_id` after its refusals and after the submission is parked, and renders the verification e-mail in what it returns (D3).
- [ ] 1.6 Doc comments state the rule: `UserPort::set_locale` and `UserPort::locale` (`backend/user-port/src/lib.rs`), `AuthPort::sign_up_local`'s `locale` (`backend/auth-port/src/lib.rs`), and the module comments of the four paths. No `.proto`, no migration (`0007_locale.sql` included), no template is edited (D6).

## 2. Tests (backend/auth, mockall by default)

- [ ] 2.1 `email_locale` table: (`fr`, `it`) → French; (none, `it`) → Italian; (none, ``) → English; (``, `es`) → Spanish; (`de-DE`, `fr`) → English; (`FR_ca`, ``) → French.
- [ ] 2.2 With `MockUserPort`: a resend and a reset for an account whose stored locale is `fr`, carrying `it`, never call `set_locale` and send a French e-mail; with no stored locale and `fr-FR`, `set_locale(uid, "fr-FR")` is called once and the e-mail is French; with neither, `set_locale` is never called and the e-mail is English.
- [ ] 2.3 With `MockUserPort`: set-password for an account whose stored locale is `es`, carrying `` and then `en`, sends Spanish e-mails and never calls `set_locale` (case 2); for an account with none, carrying `fr`, records `fr` once and sends French; a refused set-password (a second password, a weak password, an address already taken) calls neither `locale` nor `set_locale`.
- [ ] 2.4 Enumeration, with `MockUserPort`: a reset and a resend for an unknown address call neither `locale` nor `set_locale` and return the same `Ok(())` as for a known one (D5).
- [ ] 2.5 Through the harness over `UserModule` and `FakeUserRepo`: `request_locale_overrides_stored_locale` becomes `stored_locale_wins_over_request_locale` (sign-up `fr`, resend `it` → French, stored still `fr`); case 1 — a Google account, a password set and verified before any locale, a reset carrying `fr-FR` → French and stored `fr-FR`, a later reset carrying `en` → French; `SetLocale` to `it`, then a reset carrying `fr` → Italian. The other locale tests pass unchanged.

## 3. Gates

- [ ] 3.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 3.2 `cargo test -p cymbra-auth -p cymbra-auth-port -p cymbra-user-port` (the crates this change edits).
- [ ] 3.3 `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes, with the new lines of `backend/auth/src/module.rs` covered and nothing added to the exclusions.
- [ ] 3.4 `git diff --stat origin/main -- 'backend/*/proto'` is empty, so `buf breaking` (the `proto` workflow) is unaffected (D6).
- [ ] 3.5 `openspec validate prefer-account-locale-for-emails --strict` passes, and `python3 scripts/openspec_archive_order.py prefer-account-locale-for-emails` exits 0.
- [ ] 3.6 [manual] The owner deploys the backend (`backend-deploy`) and checks it from outside: on a test account whose language is French (set in Cymbra Music), a password reset requested from a client showing English arrives in French, and Music still shows French after signing in again.
- [ ] 3.7 Row 17b of `docs/lingua/language-matrix-programme.md` is marked done, naming the pull request.
