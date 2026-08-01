## Context

`template-backend-emails` renders transactional email in the recipient's language,
but the locale is only ever the one carried by the **request** that triggers the
mail (`sign_up_local` / `resend_verification` / `request_password_reset`, each
passing the app's `l10n.localeName`). There is no per-account memory of language, so
any **server-initiated** email (a future job/notification with no client request)
has nothing to render with and falls back to English.

Current state relevant to storage:
- The shared account lives in `user_account.users` (`backend/user/migrations/0001_init.sql`),
  which already has a `preferences JSONB NOT NULL DEFAULT '{}'` column and a
  `updated_at`/`version` pair.
- Accounts are created by `UserModule::resolve_or_provision(provider, subject)`
  (`backend/user/src/module.rs:89`), which today takes **no** locale.
- The auth module reaches the user schema **only through `UserPort`** (design D0 —
  auth never writes `user_account` directly).

## Goals / Non-Goals

**Goals:**
- Persist a preferred locale per account, written by the identity system.
- Make it the fallback language for transactional email: precedence
  **request locale → stored locale → English**.
- Zero disruption to the current client-triggered flows (request locale still wins).
- Preserve the password-reset flow's account-enumeration safety.

**Non-Goals:**
- No user-facing "email language" setting (UI + RPC) — that is the future escalation
  for deterministic multi-device control (see Open Questions), not this change.
- No change to the `email_template` rendering layer, no new email types.
- No proto/API/client changes — the apps already send their locale.
- No backfill of existing accounts (they simply have `NULL` = English until their
  next locale-carrying call).

## Decisions

### D1 — A dedicated `locale TEXT` column, not `preferences` JSONB
Add a nullable `locale TEXT` to `user_account.users` rather than storing it under the
existing `preferences` JSONB.
*Rationale:* the locale is a **system-relevant** field the backend reads on the mail
path (cheap `SELECT locale`, indexable, clear `NULL` semantics), whereas `preferences`
is a client-oriented bag; keeping the two apart avoids write-path coupling between the
auth-set locale and any client-owned preference writes. The migration is a trivial
`ALTER TABLE ... ADD COLUMN`.
*Alternative:* `preferences->>'locale'` — no migration, but mixes a backend-owned
field into client-owned JSON and complicates queries. Rejected.

### D2 — Write via a dedicated `set_locale`, not by changing `resolve_or_provision`
`UserPort` gains `set_locale(user_id, locale)` (upsert; a no-op when the locale is
empty) rather than threading `locale` through `resolve_or_provision`. The latter is
called from several auth paths (`sign_up_local`, `sign_in_oidc`, the reset paths); a
new signature would churn all of them for a concern only some care about. Auth calls
`set_locale` right after resolving the user, only when the request carries a locale.

### D3 — Last-writer-wins refresh
Every authenticated call that carries a non-empty locale overwrites the stored value.
For multi-device users the **request** locale still wins for client-triggered mail
(so each device's mail is correct regardless of the stored value); the stored value
only feeds server-initiated mail, where "the most recently used device's language" is
the best available heuristic. A user-controlled override is the future escalation.

### D4 — Render precedence: request → stored → English
The auth callers compute the effective locale: use the request locale if non-empty,
else the stored account locale, else English. The `email_template` layer keeps taking
a single `SupportedLocale` — it is unaware of the fallback.

### D5 — Enumeration-safe reads
`request_password_reset` keeps its uniform response. The stored-locale lookup happens
**inside** the existing "account exists" branch (the same branch that already sets the
reset token), so it adds no new observable difference between existing and
non-existing accounts.

## Risks / Trade-offs

- **Extra read on the mail path.** → One indexed `SELECT` per resend/reset (already
  DB-bound flows); negligible, and skipped entirely when the request carries a locale.
- **Multi-device stored value "flaps".** → By design (D3) it never affects
  client-triggered mail; only the server-initiated fallback, where any single value is
  a heuristic. Documented escalation: explicit preference.
- **JSONB vs column bikeshed at review.** → D1 records the rationale.

## Migration Plan

1. Add `0007_locale.sql` (`ALTER TABLE users ADD COLUMN locale TEXT`). Backward
   compatible; existing rows are `NULL` (= English).
2. Add `UserPort::set_locale` + `locale` (+ repo/pg SQL + mock), deploy.
3. Wire auth (set on provisioning/carrying calls; fallback on render).
*Rollback:* additive column + additive port methods; revert code, the column can stay
harmlessly or be dropped.

## Open Questions

- **Explicit "email language" preference** (UI + a setter RPC) for deterministic
  multi-device control — deferred; this change makes it a clean follow-up (the column
  is already there; only the client control + precedence-above-request would be new).
- Should the reset flow ignore the request locale and always use the stored one (so a
  reset always arrives in the account's known language even from a shared device)?
  Default: keep request-wins for consistency with the other flows.
