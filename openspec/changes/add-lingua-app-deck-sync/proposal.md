# add-lingua-app-deck-sync — the Cymbra Lingua app synchronises its own reviews

## Why

Dogfood on an iPhone (2026-10-14): the Cymbra Lingua app reviews a snapshot of the Safari
extension's deck and hands its answers back to the extension through the App Group. They reach
Cymbra's server — and the reader's other devices — only when Safari's extension next wakes and
synchronises. Until then the Mac's side panel or the e-ink reader still shows the cards just
reviewed in the app as due, and the app itself reviews Safari's last snapshot rather than the
reader's latest deck. Guillaume: « pas top ».

Since `add-lingua-review-translation-fallback` (#948) the app holds a Cymbra session of its own, for
online translation only. That session can carry the deck too.

## What Changes

- **Signed in inside the app, the app synchronises its deck with the server directly**, with the
  extension's own synchronisation code (one implementation, a host seam for the app):
  - **pull on opening and on return to the foreground**, before the review starts — so it reviews
    the server's latest merged with Safari's snapshot and its own pending changes, under
    last-write-wins; the pull waits a bounded time and never blocks the review;
  - **push after answers** (debounced), at the end of a session and when the app goes to the
    background — only what the app's openings changed, never Safari's snapshot on Safari's behalf;
  - **still hands its changes to the extension** through the App Group, as today. Both paths carry
    the same card and status operations; the server and the extension apply them under
    last-write-wins, which replaces a card's state with the same state — nothing is applied twice.
- **Offline in the app**: changes stay in the App Group's pending operations, as today; the next
  opening re-applies them and pushes them. No new queue.
- **One account only**: the app synchronises only when the extension on the same device is signed in
  to the same Cymbra account. On another account, or signed out, the app does not merge, says so in
  one quiet line, and keeps today's relay alone. The relay gains a digest of the extension's account
  id (never a token) so the app can compare.
- **An erasure is respected**: the relay carries the erasure mark the extension's snapshot acted on;
  when the server's is newer, the app drops the snapshot and its pending changes and reviews what the
  server holds.
- **Statistics keep one writer per device**: the app does not push daily statistics; its review
  counts keep reaching the extension through the relay, which already counts each opening once and
  pushes them under its device id. The day's allowance of new words is derived from the merged deck,
  so the pull makes it count the cards introduced elsewhere.
- **The app is a synchronising device of its own**: its own random device id, its own pull cursors,
  kept in the page's storage (losing them only re-pulls). **No server change**: the server identifies
  the account by the bearer and uses the device id as a last-write-wins tie-break only.
- **Sign-in is offered on opening the app**, not only at the first « Traduction en ligne » press: a
  quiet line (French UI copy: « Connecte-toi pour synchroniser tes révisions et utiliser la
  traduction en ligne. — Se connecter »). The signed-in line and the sign-in sheet's lede say that
  the session serves synchronisation and online translation (fr, en, es).
- Not signed in inside the app: today's behaviour, unchanged.
- **The privacy annex** (French, English, Spanish) says that the app, once the reader signs in inside
  it, synchronises the deck itself under its own session.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-apple-app`: the app obtains the deck from the server as well when signed in to the
  extension's account; it synchronises its reviews, offers sign-in on opening, refuses another
  account, respects an erasure, and leaves the device's statistics to the extension; « No learning
  state in the app » says what the app keeps.
- `lingua-sync`: the container app's session serves its deck synchronisation as well as online
  translation; the app synchronises as a device of its own with no server change.
- `lingua-privacy`: the disclosures say the app synchronises the deck under its own session.

Archive order: « The app reviews the deck the extension keeps », « Apple and Google sign-in on Safari
through the host app » and « Lingua's privacy disclosures match what it collects » are modified in
their wording from the unarchived `add-lingua-review-translation-fallback`, and « No learning state
in the app » is added by the unarchived `add-lingua-apple`: both are listed in `archiveAfter`
(design D12). `add-lingua-app-email-signin` (in flight) touches the app's sign-in; whichever of the
two archives second rebases its deltas (task 1.2).

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The extension (`apps/lingua-extension`): the sync engine gains a
  host seam, the relay gains two fields, the app page synchronises; the Apple container app
  (`apps/lingua-apple`): a background hook and the sign-in sheet's lede.
- **Cymbra Lingua backend — consumed unchanged.** `DeckService`, `KnownWordsService`,
  `LinguaDataService` as they are; no proto change, no migration, no configuration change
  (`lingua-app://app` is already an allowed origin).
- **Cymbra ID — consumed.** The app's existing session (`SignInOidc`, audience `lingua`) and
  `UserService.GetAccount` for the account id. Nothing redeclared from `id-*` or `platform-*`.
- **Public site — content only.** `apps/site`: the Lingua annex of the privacy policy (fr, en, es).
- Music, Live and the back office are untouched. The agent plugin (`apps/lingua-agent`) stays out of
  synchronisation.

**Code.** `apps/lingua-extension/src/sync/sync.ts`, `src/state/app-relay.ts`,
`src/state/review-relay.ts`, `src/background.ts`, `src/app/*` (a new `src/app/deck-sync.ts`),
`src/i18n/*/app.ts`; `apps/lingua-apple/Shared (App)/ViewController.swift`,
`LinguaSignIn/Sources/LinguaSignIn/SignInFlow.swift`; `apps/site/src/pages/confidentialite.md`,
`en/privacy.md`, `es/privacidad.md`.

**Data.** No new server data: the app's operations are the extension's shapes, under the same
account. A new device id appears in the account's card and status rows' tie-break column.

**CI.** `lingua-extension-check`, `lingua-apple-build`, `rust` (the backend convergence test) and
`site-check` already watch these paths; no new unit.
