# Tasks

Size: about **10 ideal days** — core and binding 1.5, background and badge 2.5, popup and review
figure 1.5, rating request 1.5, Apple app 1, copy and privacy annex 0.5, gates and dogfood 1.5.

## 1. The core and its binding

- [ ] 1.1 `Deck::reviewable_count_for(now, languages, allowance)` in `crates/lingua-core/src/decks/review.rs` (D1). Verify with `cargo test -p lingua-core`: equal to the queue length of `ReviewSession::start_with` with no `limit` over decks mixing reviewed, never-reviewed, retired and two languages, with and without an allowance, before and after cards introduced today.
- [ ] 1.2 The outlook (D2): per studied language `due`, `upcoming` (at most 100 due times after `now`, ascending, retired excluded), `fresh`, `introduced` since `dayStart`; counted in the core beside 1.1. Verify with `cargo test -p lingua-core`: the count worked out from the outlook at `now`, at a later time crossing upcoming due times, and on the next local day equals `reviewable_count_for` at those times.
- [ ] 1.3 `crates/lingua-wasm`: the engine's `reviewableCount(now, languages, newPerDay, dayStart)` and the standalone function `reviewOutlook(backup, now, dayStart)` returning JSON (as `reprofileBackup` does), erroring on a backup it cannot read. Verify with `cargo test -p lingua-wasm` and `cargo clippy --workspace --all-targets -- -D warnings`.

## 2. One figure in the extension

- [ ] 2.1 `LinguaPort.reviewableCount` through `WasmAnalyzerPort`, the messaging port and the fake port; the review page's summary (`refreshSummary`) and the reading session's `statsMessage` use it with the reader's allowance and `localDayStart` (D1). Verify with `test/engine.spec.ts` (real WASM: the figure excludes new words beyond the allowance) and the review page's specs.
- [ ] 2.2 A pure module `src/state/review-outlook.ts`: the outlook's type and version, `countAt(outlook, languages, now, dayStart, perDay)` (D2). Verify with a new `test/review-outlook.spec.ts`, including a cross-check against the engine's `reviewableCount` on the same deck at three times.

## 3. The badge

- [ ] 3.1 The background computes the outlook after each write of `ROOT_KEY` (hooked beside `onReaderDataChanged`), at start when the key is missing or of another version, never while the setting is off; a write during a run marks it dirty and one more run follows, no timer (D2). Verify with a background-wiring spec on fakes, and measure one run on a 2,000-card backup (desktop Chrome cold worker, the Boox on Firefox for Android); record both in the pull request.
- [ ] 3.2 A pure module `src/state/badge.ts`: `formatCount` (0 → nothing, 99, « 99+ »), the global badge (text, colour per colour preset, title per language), a tab's badge from its `stats` message (percentage; `null` text and the count's colour for nothing to count or switched off) (D4, D5, D6). Verify with a new `test/badge.spec.ts` covering every state, the e-ink presets and the setting off.
- [ ] 3.3 Wire it in `background.ts`: the global badge and title, the `stats` listener's new per-tab rules, the events of D3 (`runtime.onStartup`, `runtime.onInstalled`, `tabs.onActivated`, `windows.onFocusChanged` guarded, `chrome.storage.onChanged` for the allowance, the setting and the colour preset), the last applied values in `chrome.storage.session`. Verify with `test/manifests.spec.ts` asserting no new permission (no `alarms`, `idle`, `notifications`, `tabs`) in any variant.
- [ ] 3.4 The background's start no longer syncs nor refreshes the flags by itself; `runtime.onStartup` and `runtime.onInstalled` do; Safari's handed-token collection and the Apple relay stay on every start (D9). Verify with the scheduler's and background's specs.
- [ ] 3.5 The setting « Afficher les cartes à réviser sur l'icône » and its note in Réglages › Langue › « Rythme de révision », on by default, a device key in `chrome.storage.local` (D7). Verify with the settings view's specs and `test/lint-settings-hosts.spec.ts`.

## 4. The popup

- [ ] 4.1 « Deck (en cours) » and « Réviser (N) » shown on every tab, highlighting on or off; away from an analysed page N comes from the outlook for the language the review would open in; « Réviser » opens the drawer, or `sidepanel.html` in a tab where the drawer cannot open on Firefox and Safari (D8). Verify with `test/popup.spec.ts` in the three interface languages.

## 5. The rating request

- [ ] 5.1 `src/state/store-links.ts`: the three listings and each build target's review page (D12). Verify with a vitest that reads `apps/site/src/lib/stores.ts` and compares the Chrome Web Store id, the AMO slug and the App Store id.
- [ ] 5.2 A pure module `src/review/rating.ts`: the state, `afterSession`, `ratingDue`, `answer` (D10). Verify with a new `test/rating.spec.ts`: not before the fifth session nor the third day; shown at the qualifying end; « Plus tard » and no answer → ten more sessions; twice at most; « Noter » and « Ne plus demander » end it; practice counts; « Rien à réviser » does not.
- [ ] 5.3 The review page counts each « Séance terminée » and asks; the view renders the request under the end's actions; « Noter » goes through `openPage` to the background; no request in the Safari build nor with `__APP__` (D10, D11). Verify with `test/view.spec.ts` and the review page's specs (the request absent, shown, answered; « Encore 10 » first; no network call before « Noter »).
- [ ] 5.4 Réglages › Données: « Noter Cymbra Lingua » under « Sources et confidentialité », in every build, through the background (D13). Verify with the settings view's specs.

## 6. The Cymbra Lingua app

- [ ] 6.1 The page posts `review.sessionFinished` at each « Séance terminée » with `__APP__` (`src/app/bridge.ts`); `AppBridgeMessage` answers it. Verify with `test/app-session.spec.ts` and the `LinguaSignIn` tests.
- [ ] 6.2 `ReviewRequestPolicy` in `LinguaSignIn` (count in the app's `UserDefaults`, fifth session, once per `CFBundleShortVersionString`); `ViewController` asks two seconds later through `AppStore.requestReview(in:)`, or `SKStoreReviewController.requestReview()` where the minimum platform lacks it (D11). Verify with `swift test` in `apps/lingua-apple/LinguaSignIn` and a Debug build on iPhone, iPad and Mac (StoreKit shows its prompt in Debug).

## 7. Copy

- [ ] 7.1 Catalogue entries in `src/i18n/{fr,en,es}/`: the setting and its note (`settings`), the icon's title with its plural forms and per-language list (a new `badge` module), the request's sentence and three actions (`review`), the Réglages link and its note (`settings`); the English and Spanish drafts are whole. Verify with `test/i18n.spec.ts`, `test/lint-copy.spec.ts` and `test/lint-lemma.spec.ts`.
- [ ] 7.2 Annex B of the privacy policy, « Ce qui reste sur votre appareil »: the count of review sessions and the answer to the rating request stay on the device, the store page opens only when the reader chooses — in `apps/site/src/pages/confidentialite.md`, `en/privacy.md` and `es/privacidad.md`. Verify with the site's checks (`site-check`).
- [ ] 7.3 `apps/lingua-extension/REVIEWERS.md`: the badge (no permission, no timer) and the one-time rating request (no incentive, at most twice, a link opened on request). Verify by reading it in the pull request.

## 8. Gates and dogfood

- [ ] 8.1 Gates: `cargo fmt --all --check`, clippy, `cargo llvm-cov` ≥ 80 %; the extension's `yarn gen:wasm && yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`; `swift test`. Verify all pass.
- [ ] 8.2 Chrome and Firefox desktop: the count on a new tab, on a page with nothing to count, on a page switched off; the percentage on an analysed page and gone after following a link (if a percentage outlives its page on Chromium, reset it on `tabs.onUpdated` `loading`); the count down after a review; no exchange in the network log on a tab switch after a minute of quiet, and the time from that switch to the badge on a cold service worker. Verify by noting it in the pull request.
- [ ] 8.3 Safari on the Mac, iPhone and iPad: where the count shows, if at all; if iOS and iPadOS show no badge text, the setting is hidden there. Verify by noting it in the pull request.
- [ ] 8.4 The Boox (Firefox for Android, e-ink preset): the count's legibility in the menu, if Firefox for Android shows it. Verify by noting it in the pull request.
- [ ] 8.5 The rating request reached on a test profile (sessions and days set in `chrome.storage.local`): the Chrome Web Store reviews tab and the Firefox Add-ons listing open; the app's StoreKit prompt in a Debug build. Verify by noting it in the pull request.
