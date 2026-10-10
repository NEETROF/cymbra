# prefer-account-locale-for-emails — an account's e-mails in the account's language

## Why

Beside change 17 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`localise-lingua-account-onboarding`), outside the 57: the owner's decision of 2026-10-10, in
session — **Cymbra ID's account e-mails use the account's stored language first, and the request's
language only when the account has none.**

Cymbra ID sends four account e-mails, all from `AuthModule` (`backend/auth/src/module.rs`), and
picks each one's language per request:

| E-mail | Language today | What it writes on the account |
|---|---|---|
| Sign-up code (`SignUpLocal`) | the request's, else English | the request's locale, on the account it creates |
| Resent code (`ResendVerification`) | the request's, else the stored one, else English (`effective_locale`) | the request's locale, **over** any stored one |
| Password-reset code (`RequestPasswordReset`) | the request's, else the stored one, else English (`effective_locale`) | the request's locale, **over** any stored one |
| Set-password code (`SetLocalCredential`) | the request's, else English — the stored one is never read | nothing |

So an account's language follows whichever device asked last. A reset requested from a browser in
English moves a French account's e-mails, its Cymbra Music language at the next sign-in and its
practice reminders to English — and a reset needs nothing but the address. Change 17 works around
it from Lingua: until the reader has chosen a language on a device, its resend and reset carry no
locale (its design D2). Two cases are left that only the server can fix:

1. **An account with no stored locale** — created with Google, a password set later, never opened in
   Cymbra Music: a reset from a device that sends no locale is written in English.
2. **Set-password** writes in the request's language, English when it carries none, never in the
   account's; and it records nothing, so it leaves case 1's accounts without a language.

The rule the archived `persist-user-locale` chose (request first, last writer wins: its D3 and D4)
predates `account-language-sync`. It treated the stored locale as a guess — « the most recently used
device's language » — and named a user-controlled setting as the future escalation. That setting
exists now: Cymbra Music and the back office write the account's language through `SetLocale` when
its owner changes it. The stored locale is the owner's choice, so it comes first.

## What Changes

- **One rule for the four e-mails**: the account's stored locale, else the request's, else English —
  `email_locale(stored, request)` replaces `effective_locale`, and one helper applies it on every
  path (design D1).
- **A request's locale is recorded only on an account that has none.** Sign-up's write is the first
  (the account is new, so nothing changes there); a resend, a reset and a set-password fill an empty
  account and leave a stored locale alone. A stored locale is replaced only by `SetLocale`, the
  account's own language setting (D2).
- **Set-password reads the stored locale and records the request's when there is none** (case 2,
  and fewer case-1 accounts from the deploy on: D3).
- **Tests** in `cymbra-auth`, mockall's `MockUserPort` by default; the existing test
  `request_locale_overrides_stored_locale` is inverted (D8).
- **Unchanged**: every `.proto` (no field added, removed or renumbered — `buf breaking` has nothing
  to compare), the `AuthPort` and `UserPort` signatures, the e-mail templates, the pools and roles,
  and the response that hides whether an account exists (D5, D6).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `user-locale-preference` (Cymbra ID; a legacy name mapped to `id-*` in `openspec/config.yaml`,
  not renamed here): MODIFIED *Account stores a preferred locale* (recorded only on an account that
  has none; replaced only by `SetLocale`), *Stored locale is the email-localization fallback* (stored
  → request → English, on the four e-mails) and *Stored-locale lookup preserves enumeration safety*
  (the write too, and the resend). Requirement and scenario names kept — *Request locale wins over
  stored* now shows the one request whose locale does win, `SetLocale` —, scenarios added. No open
  change holds these requirements.
- `transactional-email` (Cymbra ID, legacy name): MODIFIED *Localized content with English
  fallback* — its scenario *Optional locale is backwards compatible* rendered every request without
  a locale in English, which the stored locale already contradicted; it now says the account's
  stored locale, English when it has none. No open change holds this requirement
  (`localise-cymbra-id-email-legal-links` modifies another one of this capability).

## Impact

- **Cymbra ID** (changed): `backend/auth/src/module.rs` (the rule, the helper, the four paths and
  their tests); doc comments in `backend/user-port/src/lib.rs` and `backend/auth-port/src/lib.rs`. The
  stored locale stays in `user_account.users.locale`, read and written through `UserPort` on the
  `user_svc` pool as today (design, Context).
- **Cymbra Music** (consumes): its four requests keep sending its interface language; its e-mails
  come in the account's language even when the app shows another, which happens only before
  sign-in (a language changed while signed out, a new device). A reset no longer moves the
  account's language, so Music's sign-in no longer adopts a language a reset wrote; its language
  setting (`SetLocale`) moves it as before. Its practice reminders (push) already read the stored
  locale alone.
- **Back office** (consumes): nothing changes — it sends none of the four requests; its own
  language sync is `SetLocale`/`GetAccount`, unchanged; the account directory does not show the
  locale.
- **Cymbra Lingua** (consumes): change 17's requests work as designed — a resend or reset with no
  locale keeps the account's. Once this is deployed, a resend or reset that carries one no longer
  moves the account either, so Lingua could send the browser's language on them again — that is
  open question Q1, not this change.
- **Site**: nothing; it sends none of these requests.
- **Order**: no dependency on change 17 or any open change, either way; no `archiveAfter`. The owner
  deploys the backend (`backend-deploy`).
- **Not here**: a Lingua client change (Q1, Q3); normalising stored tags (`fr-FR` → `fr`); a new
  e-mail.
