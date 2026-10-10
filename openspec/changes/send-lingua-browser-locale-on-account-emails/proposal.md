# send-lingua-browser-locale-on-account-emails — a resend and a reset carry the browser's language again, now that no request can overwrite an account's

## Why

Change 17b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`prefer-account-locale-for-emails`, implemented on main in #859) left one question to Lingua, its
open question Q1, which the owner settled on 2026-10-10 (in session): **once 17b's server is
deployed, Lingua sends the browser's language again on « resend the code » and « reset the
password », even when the reader has not chosen a language on this device.**

Change 17 (`localise-lingua-account-onboarding`, on main in #786, its design D2) made those two
requests carry **no locale** until the reader has chosen their native language on the device. The
server then recorded a non-empty locale over the account's, last writer wins, so a laptop where the
reader had chosen nothing would have written its browser's language over a language chosen on
another device, or given by Cymbra Music. An empty locale was the only way to keep the account's.

17b removed that risk on the server: every account e-mail is written in the account's stored
language, else the request's, else English, and a request's locale is recorded only on an account
that has none (`backend/auth/src/module.rs`, `email_locale` and `AuthModule::account_email_locale`).
The empty locale now protects nothing, and costs one thing: an account with no stored language —
created with Google or Apple, its password set before 17b's deploy, never opened in Cymbra Music —
gets its reset e-mail in English, whatever the reader's browser speaks.

## What Changes

- **One locale per state, on all four requests.** Until the reader has chosen their native
  language on this device, sign-up, resend, reset and set-password all carry the browser's whole
  tag (`navigator.language`, `fr` when it gives none) — sign-up and set-password already did; resend
  and reset carried none. Once they have chosen, nothing changes: the account locale of change 17's
  D2 on all four (design D1).
- **`keepAccountLocale` goes**, with the flow's `overwritingLocale`: `AccountLanguage` is the locale
  and the deletion link's language; the flow's `resend` and `requestReset` send `deps.locale`, as
  `signUp` and `setPassword` do (D1).
- **The deletion link does not move**: by the browser's tag until the reader chooses, by the
  interface language after (`lingua-privacy`, unchanged).
- **Released after 17b's backend deploy, never before** (D4): on the old server, a resend or a reset
  carrying a locale records it over the account's, and a laptop where nothing is chosen would again
  move the language another device chose. An explicit owner task gates the merge and the release.
- **Tests** against a fake Cymbra ID that answers as 17b's server does, on two devices; one test
  against the server before 17b, which pins the release order's reason (D5).
- **Nothing visible moves**: no copy, no catalogue key, no page, no pack, no golden, no snapshot; the
  French interface byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: MODIFIED *The account's e-mails follow the interface language* —
  until the reader has chosen, the browser's tag on all four requests instead of none on a resend and
  a reset; what Cymbra ID does with it since 17b stated; its scenarios kept by name, *A reader who has
  not chosen* and *A choice made on another device* reworded, *A Spanish-native reader*'s deletion
  link named by `lingua-privacy` (D6); *An account with no language* and *A choice does not move an
  account's language* added. Added by change 17 and modified by `add-site-spanish-locale` (change
  29), both open: this change archives after both, and its block, which replaces the requirement
  whole, carries 29's Spanish deletion page by reference (D6).
- `lingua-account`: MODIFIED *Email verification by code* and *Password reset by code* — the resend
  and the reset request carry the locale the sign-up carries, recorded only on an account that has
  none; every scenario kept, unchanged. Added by `add-lingua-account-parity` and modified by change
  17, both open: this change archives after both.

*Account creation by email from the extension* (sign-up already sends the browser's language until
the reader chooses), `lingua-privacy`'s deletion link, and 17b's `user-locale-preference` and
`transactional-email` are consumed, not modified.

## Impact

- **Cymbra Lingua** (changed): `apps/lingua-extension/src/account/{locale,flow,page}.ts` and their
  tests (`account-locale.spec.ts`, `account-page.spec.ts`). No `src/i18n` file, no `.html`, no
  manifest, no pack, no `crates/lingua-*`, no `scripts/lingua-data`. The Apple host app carries the
  extension and so this change, under the same release order.
- **Cymbra ID** (consumed, not changed): 17b's rule. No `.proto` moves: the four requests' `locale`
  fields (`auth.proto`: `SignUpLocalRequest` 3, `ResendVerificationRequest` 2,
  `RequestPasswordResetRequest` 2, `SetLocalCredentialRequest` 3) carry a value where two of them
  carried an empty string.
- **Cymbra Music** (consumes nothing new): an account with no language may now get the browser's
  whole tag from a reset, as a Lingua sign-up or set-password already gives it; Music adopts only a
  whole code of its own, so its interface does not move (D3).
- **Back office, site, Cymbra Live**: nothing.
- **Order**: implemented after change 17 (on main) and 17b (on main); merged and released only after
  17b's backend deploy (D4); archived after `add-lingua-account-parity`, change 17, 17b and change
  29 (D6).
- **Not here**: Lingua calling `SetLocale` when a signed-in reader chooses (17b's Q3, settled: a
  Lingua choice does not move the account's language); normalising stored tags (`fr-FR` → `fr`).
