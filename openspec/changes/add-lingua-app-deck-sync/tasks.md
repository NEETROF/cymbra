# Tasks

## 1. Coordination

- [ ] 1.1 (D12) Before archiving: check that `add-lingua-review-translation-fallback` and `add-lingua-apple` are archived (both are in `archiveAfter`); if `add-lingua-apple` is still open with only its manual-pass and store tasks left, archive it first. Verify with `python3 scripts/openspec_archive_order.py add-lingua-app-deck-sync` exiting 0.
- [ ] 1.2 (D11, D12) When `add-lingua-app-email-signin` reaches `main`: reconcile the two changes — the sign-in sheet's lede, the account line's copy and their deltas of « Apple and Google sign-in on Safari through the host app »; whichever archives second rewrites its delta on the first's archived text, keeping both additions, and lists the other in `archiveAfter`. Verify with `openspec validate add-lingua-app-deck-sync --strict` and, after both archives, by reading `openspec/specs/lingua-sync/spec.md`: every scenario of both changes is present.

## 2. Sync engine host seam (extension)

- [ ] 2.1 (D1) `src/sync/sync.ts`: options `ops`, `pushStats`, `pushLevels` (defaults: today's behaviour) and a `push()` phase (erasure check then push; no restore, no apply, no cursor move); `sync()` unchanged for the extension. Verify with `test/sync.spec.ts`: the existing cases pass unchanged; with `ops` given only those ops are pushed; with `pushStats: false` no `upsertDailyStats` call; `push()` moves no cursor and applies nothing.
- [ ] 2.2 (D8) The engine accepts a recorded erasure mark from its host (the snapshot's) and, when the server's is newer, resets and reports it to the host instead of only wiping its storage. Verify with `test/sync.spec.ts`: a host mark older than the server's resets the engine and pushes nothing from before; an equal one changes nothing.

## 3. Relay fields (extension)

- [ ] 3.1 (D7) The extension records a digest of its account id (`SHA-256("cymbra-lingua-relay:" + user_id)`, hex) at sign-in from `UserService.GetAccount`, refreshes it at sync when absent, and clears it at sign-out; `SharedState` gains `account: string | null`. Update the `app-relay.ts` header comment (« never a token »). Verify with `test/app-relay.spec.ts` (encoded and parsed; an older payload reads as absent, distinct from `null`) and `test/review-relay.spec.ts` (shared after sign-in, `null` after sign-out, never a token in the payload).
- [ ] 3.2 (D8) `SharedState` gains `erasedAt` (the extension's `ERASED_AT_KEY`, 0 when none). Verify with `test/app-relay.spec.ts` and `test/review-relay.spec.ts`: the mark is shared, an older payload reads 0.

## 4. The app's synchronisation (page)

- [ ] 4.1 (D1, D10) `src/app/deck-sync.ts`: builds a `SyncEngine` over the review page's port, the session store layered with a bookkeeping area in `localStorage` keyed by account digest (device id, cursors, languages, labels mark, erasure mark), `ops` = the relayed ops (`appOpsSince(port, since)`), `pushStats: false`, `pushLevels: false`, the clients over the account's transport. Verify with `test/app-deck-sync.spec.ts`, the sync clients test doubles recording their calls: the device id is generated once and differs from any extension id given; another account's bookkeeping is dropped at sign-in; a lost bookkeeping area pulls from cursor 0.
- [ ] 4.2 (D2) Opening: one full exchange before mounting the review, bounded by `APP_PULL_WAIT_MS`; past it the review opens on the snapshot and the result is applied once no session runs, then the review refreshes. Verify with `test/app-deck-sync.spec.ts` (fake timers): a quick pull is applied before mounting; a slow one after; one arriving during a session waits for its end; a failure opens on the snapshot.
- [ ] 4.3 (D3, D5) Triggers: push debounced `APP_PUSH_DEBOUNCE_MS` after a backup save; push then full exchange at a session's end; full exchange on opening/foreground throttled to one per 10 s; `window.linguaBackground` flushes the push; single-flight. Verify with `test/app-deck-sync.spec.ts`: three answers in 2 s give one push carrying exactly the relayed ops; no `sync()` while `review.reviewing()`; a background call pushes at once.
- [ ] 4.4 (D7) Account rule: sync only when the app's digest equals the shared `account`; otherwise no request carrying the deck and the right line (other account, Safari signed out, older extension). Verify with `test/app-deck-sync.spec.ts` for the three scenarios of « The app synchronises only on the extension's account », online translation still offered.
- [ ] 4.5 (D8) Erasure guard: a server mark newer than the shared `erasedAt` resets the engine, writes an empty pending payload (`review.writeOps`), and reviews the server's state. Verify with `test/app-deck-sync.spec.ts` for « Erased on the Mac, Safari not run since ».
- [ ] 4.6 (D6, D9) Offline and statistics: a failed push leaves the pending payload intact and the next opening pushes it; the page never calls `upsertDailyStats`; the relayed `daily` is unchanged. Verify with `test/app-deck-sync.spec.ts` and `test/app-session.spec.ts` (the `daily` payload as before).
- [ ] 4.7 (D11) Account line and copy: the opening line, « Plus tard » (30 days in the page's storage), the signed-in-and-synchronising line, D6's and D7's lines; `src/app/view.ts` `renderAccountLine` and `src/i18n/{fr,en,es}/app.ts`; all text nodes. Verify with `test/app-view.spec.ts` for each state and the i18n alignment test.
- [ ] 4.8 Gates in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass, coverage of `src/app/deck-sync.ts` and the changed sync code at least 80 %.

## 5. Integration

- [ ] 5.1 (D4) `test/app-deck-sync.integration.spec.ts`: an in-memory `DeckService` / `KnownWordsService` / `LinguaDataService` double applying the server's rule (newer `updated_at`, or equal and greater device id; erasure mark), and three hosts on it — Safari's extension (relay + its own sync), the app (relay + its sync) and a Mac extension. Scenario « Reviewed in the app, seen on another device without opening Safari »: the app reviews ten cards and pushes; the Mac syncs and holds none of them due; then Safari collects the relay and syncs, and every host converges with the cards' states unchanged and no grade applied twice. Plus « A change the extension made afterwards still wins » and « Erased on the Mac ». Verify with `yarn test`.
- [ ] 5.2 (D4, D10) `backend/lingua/tests/convergence.rs`: a third device id pushing a subset already pushed by another device converges, and an equal-content re-push changes no card's content. Verify with `cargo test -p cymbra-lingua --test convergence`.

## 6. The Apple app

- [ ] 6.1 (D5) `Shared (App)/ViewController.swift`: call `window.linguaBackground` on `UIApplication.didEnterBackgroundNotification` (inside `beginBackgroundTask`, ended on the script's reply or expiry) and `NSApplication.willResignActiveNotification`. Verify with the `lingua-apple-build` workflow and in dogfood 7.1.
- [ ] 6.2 (D11) `LinguaSignIn/Sources/LinguaSignIn/SignInFlow.swift`: the `appLede` copy in fr, en, es. Verify with `swift test` in `LinguaSignIn` (the copy test, if any, updated) and by reading the three.
- [ ] 6.3 `apps/lingua-apple/README.md` and the review/relay section of `apps/lingua-extension/README.md`: the app's synchronisation, its account rule, its statistics rule. Verify by reading them against D2–D10.

## 7. Privacy

- [ ] 7.1 (D13) The Lingua annex in `apps/site/src/pages/confidentialite.md`, `en/privacy.md` and `es/privacidad.md`: the synced row says the app synchronises statuses and cards under its own session once the reader signs in inside it, daily stats still through the extension. Verify with `site-check` and by reading the three against « Lingua's privacy disclosures match what it collects ».
- [ ] 7.2 The App Store privacy answers recorded with the Apple app: check the declared categories still cover the app's own synchronisation (user content, product interaction, device ID, all linked). Verify by noting the result in the pull request.

## 8. Dogfood

- [ ] 8.1 iPhone: sign in inside the app from the opening line (same account as Safari); review ten cards with Safari closed; close the app; open Chrome's side panel on the Mac: the ten are no longer due. Then airplane mode, review five, reopen with network: the five reach the Mac. Verify by noting the results in the pull request.
- [ ] 8.2 Mac app: review in the app, then open Safari on the same Mac: Safari's due count matches, the day's statistics count the app's reviews once. Verify by noting the results in the pull request.
- [ ] 8.3 Chrome (Mac): review three cards in the side panel, then open the app on the iPhone without opening Safari: those three are not due in the app. Sign the app into another account: the « autre compte » line shows and nothing syncs. Verify by noting the results in the pull request.
