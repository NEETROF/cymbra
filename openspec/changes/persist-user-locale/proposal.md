## Why

Transactional emails are now localized (change: `template-backend-emails`), but the
language is only ever taken from the **request** that triggers the mail. That works
for the three client-triggered flows (sign-up, resend-verification, password-reset),
yet the moment an email is initiated **server-side** — a job or notification with no
client request context — there is no locale to render with, so it silently falls
back to English. Persisting each user's preferred locale on their account gives
those future flows a correct language, and hardens the current ones when a client
omits the field.

## What Changes

- Store a **preferred locale** on the shared account (`user_account.users`), owned
  by the identity system so it applies to every audience and to OIDC accounts, not
  just local credentials.
- **Write it via the `UserPort`** from the auth module (never a direct cross-schema
  write — design D0): set at sign-up and **refreshed last-writer-wins** on any later
  authenticated call that carries a non-empty locale.
- **Read it as the email-localization fallback.** Render precedence becomes
  **explicit request locale → stored account locale → English**. The
  `template-backend-emails` rendering layer is unchanged; only the auth callers pick
  the effective locale.
- Keep the password-reset flow's **uniform anti-account-enumeration** response — the
  stored-locale lookup is internal and must not leak whether an account exists.
- Tests + keep Rust line coverage ≥ 80%.

## Capabilities

### New Capabilities
- `user-locale-preference`: the account persists a preferred locale, written by the
  identity system (set on provisioning, updated last-writer-wins) and consulted as
  the fallback language for transactional email when a request carries none.

### Modified Capabilities
<!-- None as a delta spec: `template-backend-emails` (transactional-email) is still
     in-flight and not archived, so its behavior is extended here via the new
     capability's requirements rather than a delta against an unarchived spec. -->

## Impact

- **DB**: new migration `backend/user/migrations/0007_locale.sql` — add `locale TEXT`
  (nullable) to `user_account.users`.
- **Code**:
  - `backend/user-port/src/lib.rs` — `UserPort` gains `set_locale(user_id, locale)`
    (upsert; no-op on empty) and `locale(user_id) -> Option<String>`; regenerate the
    mockall mock.
  - `backend/user/src/repo.rs` + `pg.rs` — repo methods + SQL.
  - `backend/auth/src/module.rs` — after resolving the user, `set_locale` when the
    request carries one; in `resend_verification` / `request_password_reset`, resolve
    the effective locale (request → stored → English) before rendering.
  - `backend/user/src/grpc.rs` / callers — unaffected (no new RPC).
- **No** proto/API change, no client change (the apps already send their locale from
  `template-backend-emails`).
- **Depends on** `template-backend-emails` (the `email_template` layer + the request
  `locale` plumbing) being present.
