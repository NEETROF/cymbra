# Design — prefer-account-locale-for-emails

## Context

See proposal.md (Why). Every account e-mail Cymbra ID sends, and how its language is chosen today:

| E-mail | Request → handler | Locale chosen | Stored locale | Sent by |
|---|---|---|---|---|
| Sign-up code | `SignUpLocal` → `AuthGrpc::sign_up_local` (`backend/auth/src/grpc.rs`) → `AuthModule::sign_up_local` | `SupportedLocale::parse(Some(locale))` | `set_locale(uid, locale)` after `resolve_or_provision("local", email)` | rendered here, enqueued as a `verification_email` job in the verification write's transaction; the worker only sends it (`backend/worker/src/handlers.rs`) |
| Resent code | `ResendVerification` → `resend_verification` (unverified credential only) | `effective_locale(locale, stored)`: request → stored → English | `set_locale` (last writer wins), then `locale` | inline, `EmailSender` (SMTP) |
| Reset code | `RequestPasswordReset` → `request_password_reset` (existing credential only) | `effective_locale(locale, stored)` | `set_locale`, then `locale` — inside the account-exists branch (`persist-user-locale` D5) | inline |
| Set-password code | `SetLocalCredential` (authenticated; the caller's own account) → `set_local_credential` | `SupportedLocale::parse(Some(locale))` | neither read nor written | inline, after the submission is parked (`PendingCredentialStore`) |

`effective_locale` has exactly two callers, resend and reset (the code graph's `graphify explain
"effective_locale"`: `auth/src/module.rs` L354 and L485). The templates are
`cymbra_platform::email_template::{verification_email, password_reset_email}`, which take one
`SupportedLocale` (`En`, `Es`, `Fr`, `It`; `parse` reads the primary subtag, English for anything
else). No other account e-mail exists: `VerifyEmail` and `ResetPassword` send none; `DeleteAccount`
enqueues `purge_user`, which sends none; there is no e-mail-change flow; the browser surface
`/web/auth/*` (`backend/server/src/web_auth.rs`) only signs in, refreshes and logs out; Discord
announcements and push reminders are not account e-mails.

**Which pool reads the stored locale.** `AuthModule.user` is the `UserPort` wired in
`backend/server/src/main.rs`: `UserModule::new(PgUserRepo::new(user_pool))`, where
`user_pool = db::connect(&cfg.user_database_url, 5)` — `CYMBRA_USER_DATABASE_URL`, the role
`user_svc`, whose `search_path` is `user_account` (`backend/db/init/roles.sql.tpl`). So
`PgUserRepo::locale`'s `SELECT locale FROM users` and `set_locale`'s `UPDATE users SET locale` read
and write `user_account.users.locale` (migration `0007_locale.sql`). Whether the account exists and
is verified comes from `CredentialRepo` on `auth_pool` (`CYMBRA_AUTH_DATABASE_URL`, `auth_svc`,
`auth.local_credentials`). The auth module never touches the `user_account` schema itself; this
change keeps both calls on the same port and pool and adds no SQL.

Who writes the stored locale: these paths (sign-up, resend, reset) and `SetLocale`
(`backend/user/src/grpc.rs`, authenticated, the caller's own account), which Cymbra Music and the
back office call when their user changes language signed in, and after sign-in when the account has
none (`account-language-sync`). Lingua calls `SetLocale` nowhere. Who reads it: these paths, `GetAccount`
(`UserAccount.locale`, `user.proto` field 7: Music and the back office adopt it after sign-in when
it is one of their codes, whole), and Music's practice reminders (`backend/music/src/pg_streak.rs` reads `u.locale`; untouched here).

## Goals / Non-Goals

**Goals:**
- The four e-mails in the account's stored language, else the request's, else English.
- No request but `SetLocale` replaces an account's language.
- Set-password follows the account, and gives an account with none its first language.

**Non-Goals:**
- Any `.proto` change, or a new RPC.
- A Lingua change (Q1, Q3).
- Normalising stored tags (`fr-FR` stays `fr-FR`), or migrating stored values.
- A new e-mail, or a change to the templates.

## Decisions

### D1 — One rule, one function

`email_locale(stored: Option<&str>, request: &str) -> SupportedLocale` replaces `effective_locale`: a
non-empty stored tag, else a non-empty request tag, else English, parsed by `SupportedLocale::parse`.
A stored tag the e-mails are not written in (`de-DE`) still wins and renders in English, as
`parse` gives it, even when the request carries `fr`: the account's language is the account's, and a
device's never stands in for it (Q2 offers the other reading). One helper applies it on all four
paths:

`AuthModule::account_email_locale(user_id, request) -> Result<SupportedLocale>` reads
`UserPort::locale`, records the request's locale with `set_locale` when none is stored and the
request's is non-empty (D2), and returns `email_locale(stored, request)`.

### D2 — A request records its locale only on an account that has none

Resend, reset and set-password record a non-empty request locale only when the account has none;
sign-up's is the first write, on the account it has just created, so it records as today. A stored
locale is replaced, last writer wins, only by `SetLocale`. Two reasons:

- **Coherence.** Writing the request's locale over the stored one and then reading the stored one
  first is the old rule under another name.
- **A request is not the owner's setting.** A resend or a reset needs only an address: today anyone
  who knows it can switch an account's e-mails — and Music's language at its next sign-in — by
  requesting a reset in Italian. `SetLocale` is authenticated and comes from a language setting.

The write is a read then a write through the two existing port methods, no new port method or SQL.
Two first requests racing on an empty account may both write; the last one stays, and either is a
locale that account's requests carried — harmless. *Alternative:* a `set_locale_if_absent` port
method over `UPDATE users SET locale = $2 WHERE id = $1 AND locale IS NULL` — atomic, but a new
method on `UserPort`, `UserRepo`, the fake and the Postgres adapter for a race with no harm.

### D3 — Set-password records the request's locale when the account has none

`set_local_credential` is authenticated (the caller's own account, from the token), and it is often
the first locale-carrying request an account created with Google makes. Recording there is what
`persist-user-locale` meant (`0007_locale.sql`: « set on any authenticated call that carries a
non-empty locale »), and
it shrinks case 1: from the deploy on, a password set from Lingua or Music gives the account the
language that request carried, so a later reset from a device that sends none is written in it. The
write happens after the up-front refusals (a second password, a weak password, an address already
taken) and after the submission is parked, so a refused request writes nothing; an abandoned one
leaves the language on the account, which is the account's anyway. *Alternative:* read only — case
2's e-mail follows the account, but case 1's accounts keep no language until Music or the back
office gives them one.

**What Music sees.** Music adopts the account's locale after sign-in when it is one of its codes,
whole (`AppLanguage.fromCode`), pushes its own when there is none, and leaves an undisplayable one
alone. A set-password from Lingua before the reader has chosen carries the browser's whole tag
(`fr-FR`, change 17's D2), which Music leaves alone and no longer replaces with its own — exactly what
a Lingua sign-up already does today. A set-password from Music carries Music's code, which Music
adopts.

### D4 — Sign-up keeps its behaviour

`sign_up_local` calls the helper on the account `resolve_or_provision` just created: nothing is
stored yet, so the request's locale is recorded and used, English when it carries none — today's
result, with one read more. The job payload (rendered subject, HTML, text) is unchanged.

### D5 — Enumeration safety holds

Resend and reset keep the helper inside the branch that already exists only for a known account
(`creds.get(email)`), so an unknown address calls neither `locale` nor `set_locale` and both get the
same `Ok(())`. The requirement, which named the reset alone, now names the resend and the write.

### D6 — Contracts unchanged

No `.proto` file changes: the `locale` fields of `SignUpLocalRequest` (3), `ResendVerificationRequest`
(2), `RequestPasswordResetRequest` (2) and `SetLocalCredentialRequest` (3), and `UserAccount.locale`
(`user.proto`, 7), keep their numbers, types and names; the `proto` workflow's `buf breaking` (rule
set `FILE`) has nothing to report. The fields still mean what their comment says — the client's
interface locale; what changes is how the server weighs it. `auth.proto`'s comment « empty falls back
to English », already partial since `persist-user-locale`, is left for the next change that edits the
file, so this one leaves every `.proto` byte for byte. The `AuthPort` and `UserPort` signatures are
unchanged; their doc comments state the rule. `0007_locale.sql`'s comment, which states the old
precedence, is not edited: sqlx refuses to start when an applied migration's checksum changes.

### D7 — The spec deltas

`user-locale-preference`'s three requirements are MODIFIED with their names, which the archive
matches. *Stored locale is the email-localization fallback* keeps its name although the stored
locale is now the first choice — its text, not its name, carries the rule. Every scenario keeps its
name too (`openspec validate` refuses a MODIFIED block that drops one): *Request locale wins over
stored* now shows the one request whose locale does replace the stored one, `SetLocale`, and the
e-mail that follows it; *An e-mail request's locale does not override the stored one* says the rest.
`transactional-email`'s *Localized content with English fallback* is MODIFIED for one
scenario, *Optional locale is backwards compatible*, which said English for every request without a
locale. No open change holds any of the four requirements (`grep -rl "### Requirement: <name>"
openspec/changes/*/specs`), and no rule in this change's specs builds on change 17's, so there is no
`archiveAfter`.

### D8 — Tests

In `backend/auth/src/module.rs`, mockall by default (`cymbra_user_port::MockUserPort`, already a
dev-dependency with the `mock` feature): the interactions — `set_locale` never called when a locale
is stored, called once with the request's when none is, never for an unknown address or a refused
set-password. The harness over `UserModule` and `FakeUserRepo` (the behavioural in-memory repo the
module's tests already use) for the end-to-end cases: case 1 and case 2 as in the proposal, and the
inverted `request_locale_overrides_stored_locale`. `email_locale` has a table test. No file is added
to the coverage exclusions.

## Risks / Trade-offs

- **A language changed signed out no longer reaches the e-mails** (Music, before sign-in) → the
  account's language wins, as Music's sign-in already makes it win over the app's; changing it
  signed in moves both.
- **A stored language Cymbra does not write gives English** even when the request carries one it
  does (D1) → Q2.
- **Lingua's chosen language no longer moves the account through a resend or a reset**, which change
  17's D2 expected → Q3.
- **A whole tag recorded by set-password** (`fr-FR`) keeps Music from pushing its own code → the same
  as a Lingua sign-up today; change 17 sends bare codes once the reader has chosen.
- **The read-then-write race** (D2) → harmless.

## Migration Plan

One backend deploy (`backend-deploy`, the owner's). Nothing is migrated: stored locales stay as
they are, and an account with none gets one from its next sign-up, resend, reset or set-password
that carries one. Rolling back restores the old precedence with no data to undo. A sign-up e-mail
queued before the deploy keeps the language it was rendered in.

## Open Questions

**Settled by the owner on 2026-10-10 (in session), each as recommended:** Q1 yes, as a small Lingua
follow-up released after this deploy (not folded into change 17, whose release may come first); Q2
English, one language per account (D1); Q3 kept so, a Lingua choice does not move the account's
language.


- **Q1 — Should Lingua send the browser's language again on a resend and a reset, before the reader
  has chosen?** Change 17 sends none there so as not to overwrite the account; after this change a
  request cannot overwrite it, and a locale is used only when the account has none. *Example:* Léa's
  account was created with Google in Lingua and her password set before this deploy, so it has no
  language; she asks for a reset from Lingua on her French browser. With change 17 as is, the e-mail
  is in English; if Lingua sends `fr-FR`, it is in French and the account keeps French from then on.
  Only accounts whose password was set before this deploy and that were never opened in Music are
  concerned (D3). *Recommendation:* yes, as a small Lingua follow-up released after this deploy (or
  folded into change 17 if it is not merged by then, its release still after this deploy).
- **Q2 — A stored language the e-mails are not written in.** *Example:* Hans signed up in Lingua on a
  German browser (stored `de-DE`); on his laptop he chose French in Lingua and asks for a reset there,
  carrying `fr`. With D1 the e-mail is in English, as every one of his e-mails; the other reading —
  the first of the stored and the request's languages that Cymbra writes — would make this one
  French and the next one, from another device, English. *Recommendation:* D1 (English), for one
  language per account.
- **Q3 — Choosing a language in Lingua no longer changes the account's.** *Example:* Ana's account is
  French (set in Music); in Lingua she chooses Spanish and asks for a password reset there. With
  change 17 on today's server, the account, its e-mails and Music would turn Spanish; after this
  change they stay French until she changes Music's language. *Recommendation:* keep it so; if a Lingua choice should move the account, a later Lingua
  change calls `SetLocale` when a signed-in reader chooses, as Music does.
