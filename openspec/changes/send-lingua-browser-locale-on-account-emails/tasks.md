# Tasks

## 1. The order (the owner)

- [ ] 1.1 [manual] Release order (D4): the owner deploys `prefer-account-locale-for-emails`'s backend (`backend-deploy`) and runs its check from outside (its task 3.6) before this change's implementation pull request merges. Merged only then, no Lingua release — the extension's or the Apple host app's, this change's or an unrelated one — carries it to a server that records a resend's or a reset's locale over the account's; released before, a device where nothing is chosen would move a language chosen on another device again.
- [x] 1.2 [manual] Settled by the owner on 2026-10-10 (in session), each as recommended: the browser's tag is sent even when Cymbra writes no e-mail in its language (Q1); the whole tag, not a bare code (Q2). The owner settles Q1 (a browser language Cymbra's e-mails are not written in) and Q2 (the whole tag or a bare code) before the implementation merges; the design follows each recommendation.

## 2. The extension (apps/lingua-extension/src/account)

- [x] 2.1 `locale.ts`: `AccountLanguage` loses `keepAccountLocale`; `accountLanguage` returns the browser's whole tag (`fr` when it gives none) as `locale` and `deletion` until the reader has chosen, and the account locale and the interface language once they have, as today; the module and function comments state the rule with 17b's server (D1, D2).
- [x] 2.2 `flow.ts`: `AccountFlowDeps.keepAccountLocale` and `overwritingLocale()` removed; `resend` and `requestReset` send `this.deps.locale`, as `signUp` and `setPassword` do; the `locale` comment says the four requests carry it (D1). `page.ts`: no flag passed; its comments (D1).
- [x] 2.3 Tests (D5): in `test/account-locale.spec.ts`, the fake Cymbra ID answering as 17b's server (stored → request → English; a request's locale recorded only when none is stored; `setLocale` for Music's setting) and today's fake kept under another name for case (e); the not-chosen rows (`en-GB`, `fr-FR`, `it-IT` and `fr` on all four requests); the two-device cases (a)–(e); the `lingua-account` test, the resend and the reset carrying the sign-up's locale in both states. In `test/account-page.spec.ts`, the resend of *an English interface in an Italian browser, nothing chosen* carries `it-IT`. `account-flow`, `account-host`, `session.net`, `account-copy`, `account-view`, `account-handle`, `onboarding`, `onboarding-level-row` and `level-choice` pass unchanged.

## 3. The gates

- [x] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`. `git diff --stat origin/main` names only `src/account/{locale,flow,page}.ts`, the two spec files, this change and the programme's row: no copy, catalogue, page, manifest, pack, golden or snapshot, nothing under `backend/` or `crates/`.
- [x] 3.2 `openspec validate send-lingua-browser-locale-on-account-emails --strict` passes, and `python3 scripts/openspec_archive_order.py send-lingua-browser-locale-on-account-emails` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); row 17c of `docs/lingua/language-matrix-programme.md` is marked done.
- [ ] 3.3 [manual] After the release, from outside: on a test account whose language is English, a password reset asked from Lingua in a French browser where nothing is chosen carries `fr-FR` (the request in the browser's tools), arrives in English, and Cymbra Music still shows English after signing in again.
