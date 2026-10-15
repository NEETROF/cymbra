# add-lingua-app-deck-sync — design

## Context

See proposal.md for the why. What exists:

- **The relay** (`apps/lingua-extension/src/state/app-relay.ts`, `review-relay.ts`). The Safari
  extension's background writes a `SharedState` to the App Group after each save of its backup
  (debounced 3 s): the backup, `RELAYED_PREFS`, the daily-count store keys, `savedAt`, the
  online-access flag — « never the session or the account ». The app writes back, after every
  answer, an `AppOps` payload: the card and status operations its engine holds dated from `since`
  (`appOpsSince(port, since)`, the engine's sync shape), plus `daily`, the review counts of each
  opening by a random opening id. The extension collects it when its background wakes
  (`review.readOps`), applies it under last-write-wins on the latest backup (`applyAppOps` →
  `applyStatusChanges` / `applyCardOps`), adds each opening's counts once (`applyAppDaily`,
  remembering per opening what it already added), and clears it (`review.clearOps`).
- **The app's page** (`src/app/session.ts`, `app.ts`) holds no deck across openings: each opening
  restores the engine from the shared backup, re-applies the pending `AppOps` not yet collected, and
  mounts the extension's Révision page on an in-memory store whose every backup save rewrites the
  pending payload. Back in the foreground (`ViewController.swift` → `window.linguaForeground`) the
  page reloads unless a session is under way.
- **The app's session** (`src/app/account.ts`, #948): the extension's `Session` over a Keychain area
  (`auth.readRefresh` / `auth.writeRefresh` / `auth.clear`), `SignInOidc` for the `lingua` audience
  through the app's Apple / Google sheet (`auth.signIn`), the access token in memory, direct
  gRPC-web from `Origin: lingua-app://app` (allowed in production). It is opened on the first press
  of « Traduction en ligne » or from the account line, which shows only where online translation is
  offered.
- **The sync engine** (`src/sync/sync.ts`, `SyncEngine`): per exchange, it reads the account's
  erasure mark (`LinguaDataService.GetDataState`; a store older than the mark empties itself),
  restores a dedicated engine from the stored backup, pushes **every** status, declared level and
  card the engine exports and the daily statistics, then pulls statuses and cards after its cursors,
  applies them on the latest backup, saves it, and only then advances the cursors. Each op carries
  `deviceId`, a random UUID generated once per install (`getOrCreateDeviceId`). Cursors, the
  erasure mark, the languages of the last sync and the labels mark live in the storage area it is
  given.
- **The server** (`backend/lingua`): the user comes from the bearer; `device_id` is the
  last-write-wins tie-break of cards (`pg_deck.rs`: newer `updated_at`, or equal and greater
  `device_id`), statuses and levels, and part of the daily-statistics key (`user, day, language,
  device_id`, replace-by-key); `GetStats` sums across devices. Ops dated before the account's erasure
  mark are refused (`tests/convergence.rs`).
- **New words per day** (`refine-lingua-review-session` D2): the allowance is `per_day` less the
  cards first answered since the reader's local midnight (`Deck::introduced_since`: `reps == 1` and
  `last_review >= day_start`) — derived from the deck, not counted apart.
- **Requirements in force.** `lingua-apple-app` « The app reviews the deck the extension keeps »
  (« never through a network request of its own »; the fallback change lists the app's only
  requests: models, sign-in, online translation, « none SHALL carry the deck »); `lingua-sync`
  « Apple and Google sign-in on Safari through the host app » (main: « The container app SHALL hold
  neither a session nor learning state »; the fallback change: a session « only of its own, for
  online translation in its review »); `add-lingua-apple`'s « No learning state in the app » (« It
  SHALL hold no account session either »), added by a change never archived.
- **Sibling.** `add-lingua-app-email-signin` (in flight) adds email and password to the app's
  sign-in. This change does not depend on it: whichever sheet the app runs opens the session this
  change uses.

## Goals / Non-Goals

**Goals:**
- A review done in the app reaches the server, and the reader's other devices, without Safari
  running.
- The app reviews the reader's latest deck, not only Safari's last snapshot.
- One synchronisation implementation; no server change.
- Not signed in inside the app: exactly today's behaviour.

**Non-Goals:**
- Making the app usable without the extension (it still needs the extension's shared state to open
  a review: packs, profile, preferences).
- Syncing Safari's account-free local state into an account from the app (the extension's first
  sign-in does that, `lingua-sync` « Local store merged at first sign-in »).
- Handing the extension's session to the app, or the app's to the extension (`add-lingua-review-translation-fallback` D8).
- Pushing the app's daily statistics itself (D9).
- Email sign-in in the app: `add-lingua-app-email-signin`.

## Decisions

### D1 — One sync engine, a host seam for the app

`SyncEngine` stays the only implementation. It gains three options, defaulted to today's behaviour so
the extension's call site does not move:
- `ops?: () => Promise<{ cards: CardOp[]; statuses: StatusOp[] }>` — what to push. Default: every op
  the engine exports, as today. The app passes the ops it relays (D3).
- `pushStats?: boolean` (default `true`) and `pushLevels?: boolean` (default `true`) — the app passes
  `false` (D9; it never changes a declared level).
- Two phases beside `sync()`: `push()` (erasure check, then push — no restore, no apply, no cursor
  move) and `sync()` (the full exchange, as today). The app uses `push()` while a session runs and
  `sync()` only when none does (D5).

The app's `storage` area is the page session's store (`openAppSession`'s in-memory area holding the
backup — whose saves already rewrite the pending payload) layered with a small bookkeeping area in
the page's `localStorage` for the device id, cursors, erasure mark, languages and labels mark (D10).
Its `port` is the review page's own engine: the app runs one engine (memory on iPhones, measured at
~290 MiB with the translation engine in `add-lingua-review-translation-fallback` D7), and the full
exchange restores and applies on it only between sessions. The new module `src/app/deck-sync.ts`
owns the triggers (D5), the account check (D7) and the erasure guard (D8); it reaches the clients
through the account's transport (`createClients(createTransport(() => account.token()))`).

*Alternative.* A second, app-only sync client: two implementations of LWW, cursors and erasure that
would drift — exactly what `lingua-sync` avoids between browsers. *Alternative.* A second engine for
sync, as the extension's background does: twice the packs in the web content process for no gain,
since the review and the sync never need to run on different states.

### D2 — The state the app reviews: Safari's snapshot, its pending changes, then the server, under last-write-wins

At opening the app builds its state as today — the shared backup, then the pending `AppOps` — and,
when it may synchronise (D7), runs one full exchange **before the review is mounted**, showing its
loading line meanwhile. The three sources merge under the engine's own last-write-wins (per card
`updated_at`, per status `updated_at`, deletions included): whichever wrote a card last wins, as on
any device. Nothing is preferred by origin.

The exchange is bounded: `APP_PULL_WAIT_MS` (3 s). Past it, the review opens on the snapshot; the
exchange finishes in the background and its result is applied as soon as no session runs (at once
if the reader has not started one; else at the session's end), after which the review refreshes its
due count. A failure (offline, server unreachable, refused session) opens the review on the snapshot
with no message beyond the account line's (D6).

The foreground return keeps reloading the page when no session runs; the reload is an opening, so
it pulls again. Openings and foreground returns are throttled like a surface the reader opened in
the extension: at most one exchange every 10 s, counted from the last successful one
(`lingua-sync` « Synchronisation keeps up with the reader »).

*Alternative.* Reviewing the server's state alone when signed in: a card captured in Safari a moment
ago and not yet pushed would be missing; the snapshot is the freshest local truth, the server the
freshest remote one, and LWW is how every device already combines them.

### D3 — What the app pushes: what its openings changed, the same ops it relays

The app pushes exactly the operations it writes to the App Group: `appOpsSince(port, since)`, where
`since` is the oldest pending opening's (or this opening's second). It never pushes the rest of
Safari's snapshot: those are the extension's to push, under its own device id and erasure checks,
and re-asserting them from the app would only rewrite equal rows on the server.

Pulled ops dated at or after `since` (a card the reader reviewed on the Mac a minute ago) are in that
set too: pushed back, they are a no-op or an equal-content rewrite on the server; relayed, the
extension applies them as it would a pull. Either is harmless (D4).

### D4 — Both paths converge, nothing is applied twice

A card operation carries the card's whole state — gloss, FSRS state, `updated_at`, deletion — not a
delta. Applying it twice, in any order, through any path, leaves the same card: the server keeps the
row with the later `updated_at`, the extension's `apply_card_lww` the same; a grade is not
re-applied because a grade is not what travels, its resulting state is. Statuses are the same.

The extension, collecting ops the app already pushed:
1. applies them on its latest backup — LWW, so a card it changed later keeps its own change;
2. at its next sync pushes its whole deck (as today) — those cards carry the app's `updated_at`; the
   server compares: equal `updated_at`, so the row is rewritten only when the extension's device id
   sorts above the app's (same content, a new sequence number), else left as is;
3. pulls — gets back what it already holds; applying it changes nothing.

The equal-content rewrite in step 2 makes the other devices pull the same card once more; it already
happens between any two devices today, converges on the greatest device id, and costs one row per
card the app changed. It is accepted rather than fixed server-side (no server change).

The one thing that is an increment — the day's review counts — never takes both paths (D9).

### D5 — When the app synchronises

- **Opening and foreground**: a full exchange (D2).
- **After an answer**: a push, debounced `APP_PUSH_DEBOUNCE_MS` (5 s) after the last save of the
  backup, so a quick run of answers is one request.
- **End of a session**: a push, then a full exchange (the session no longer holds the engine).
- **Background / close**: `ViewController` calls a new `window.linguaBackground` on
  `UIApplication.didEnterBackgroundNotification` (iOS) and `NSApplication.willResignActiveNotification`
  (macOS), wrapped on iOS in `beginBackgroundTask` so the page's pending push can finish; the page
  flushes the debounced push at once. If the system suspends the page first, nothing is lost (D6).

Every trigger is single-flight: a push asked during an exchange runs after it. No exchange or push
runs while the app is signed out, or when D7 or D8 says no.

### D6 — Offline and failures: the App Group is already the queue

The app writes its pending `AppOps` to the App Group after every answer (today's behaviour) and
rebuilds from it at every opening. That payload is the app's outbox: a push that fails, or never
runs (no network, suspended page), leaves it in place, and the next opening re-applies it and pushes
it — the push being the whole set since `since`, idempotent under LWW. When the extension collects
and clears it, the changes are in the extension's backup, hence in the next snapshot, and its own
sync pushes them. No new queue, no new storage.

Failures are categorised as in the extension (`lingua-sync`: never a raw message). The account line
says it once, quietly: (French UI copy) « Synchronisation impossible pour l'instant : tes révisions
partiront plus tard. ». A refused session (the refresh token revoked) signs the app out, as the
online translation path already does.

### D7 — One account: the app synchronises only on the extension's account

Merging the app's state into an account requires the deck the app holds to belong to that account.
The app's state starts from Safari's snapshot, so the app synchronises only when **the extension on
the same device is signed in to the same Cymbra account as the app**.

- The extension records its account's id when it signs in (from `UserService.GetAccount`, which it
  already calls for the profile) and clears it when it signs out. The relay's `SharedState` gains
  `account: string | null`: `SHA-256("cymbra-lingua-relay:" + user_id)` in hex, `null` when signed
  out — a digest, not a credential, so the comment's rule becomes « never a token ». An older
  extension writes no field.
- The app computes the same digest from its own session's `GetAccount` after sign-in or resume.
- Equal: the app synchronises. Different: it does not, and its account line says (French UI copy)
  « Safari est connecté à un autre compte : tes révisions passeront par Safari. ». Safari signed out:
  « Connecte aussi Safari à ce compte pour synchroniser depuis l'app. ». Field absent (an older
  extension): « Mets à jour l'extension Safari pour synchroniser depuis l'app. ». In all three the
  relay works as today and online translation is unaffected.

Why refuse when Safari is signed out: the snapshot is then an account-free deck. Synchronising it
from the app would push it into the account without the extension's sign-in — whose first-sign-in
merge is the explicit, documented moment for that — and would relay the account's cards back into a
signed-out extension, which `lingua-sync` says no longer synchronises.

*Alternative.* Synchronise anyway on the app's account: a reader with two accounts would mix them.
*Alternative.* Relay the raw user id: no need; the digest compares as well and the App Group is the
two processes' only, but a digest keeps an identifier out of a shared container.

### D8 — An erasure is respected

`SharedState` gains `erasedAt`: the erasure mark the extension's store last acted on
(`ERASED_AT_KEY`, epoch millis; 0 when none). The app's sync engine starts each exchange with that
mark as its recorded one (its own bookkeeping value only when newer). When the server's mark is
newer than the snapshot's, the snapshot predates an « Effacer mes données Lingua » made elsewhere:
the app resets its engine (as `wipeLocal` does), **replaces the pending `AppOps` with an empty
payload** — so the extension cannot collect pre-erasure ops after its own wipe and re-date them past
the mark — and reviews what the server returns. The extension wipes itself at its next sync, as
today. An older extension (no field) is read as 0: any erasure the account ever had makes the app
start from the server, the safe side.

### D9 — Statistics keep one writer per device; the allowance follows the deck

The day's review counts are increments, and the server sums rows across devices. If the app pushed
its counts under its own device id while the extension added the same counts to its own row (as
`applyAppDaily` does), every review in the app would count twice. So the app does **not** push daily
statistics (`pushStats: false`); the relay keeps carrying `daily`, the extension keeps adding each
opening's counts once and pushing them under its device id. Per-language counts follow, since the
relay's counts are per studied language. Trade-off: another device's Statistiques show the app's
reviews once Safari has run — the deck itself (due counts, cards) is current at once.

« Nouveaux mots par jour » needs no counter of its own: the allowance is derived from the merged deck
(cards first answered since local midnight). The pull before the review makes the app count the cards
introduced today on other devices; the extension, applying the app's ops, counts the app's. Two
sessions running at the same moment on two devices can both introduce up to the allowance — as two
extension devices can today; accepted.

*Alternative.* The app pushes its counts and the extension excludes them from its own row: the
extension would have to split its daily store into its own and the app's, per account, and an app on
another account (D7) would complicate it further — for a statistics lag only.

### D10 — The app is a device of its own; no server change

The app generates its own device id (`crypto.randomUUID()`, once) and keeps it, its two pull cursors,
its last-sync languages, its labels mark and its erasure mark in the page's `localStorage` under one
key per account digest. That storage may not be durable under a custom scheme (the reason the refresh
token is in the Keychain), and that is acceptable here: losing it means a new device id (a tie-break
participant, nothing else) and a full pull — idempotent. Signing in to another account, or out,
drops the previous account's bookkeeping.

The server needs nothing: it identifies the account by the bearer, takes the device id as an opaque
tie-break, and never sees which client a device is. A device id per install is already the rule; the
app is one more install. No proto field, no migration, no configuration (the origin is listed since
#948). The extension's cursors and device id are untouched.

### D11 — Sign-in is offered on opening

Signing in inside the app now serves two things, so it is offered as soon as the app opens, not only
at the first « Traduction en ligne » press. The account line (French UI copy):
- not signed in in the app: « Connecte-toi pour synchroniser tes révisions et utiliser la traduction
  en ligne. » with « Se connecter » — shown whenever the extension is signed in, whatever the online
  setting; quiet text, no dialog, dismissed for 30 days with « Plus tard » (kept in the page's
  storage);
- signed in and synchronising: « Connecté à Cymbra : tes révisions se synchronisent » with « Se
  déconnecter »;
- signed in but not synchronising: D7's or D6's line, with « Se déconnecter ».

The sign-in sheet's lede becomes (French UI copy) « Connecte-toi pour synchroniser les révisions
faites dans l'app et utiliser la traduction en ligne. Utilise le même compte que dans Safari. »
(`SignInFlow.swift` `appLede`, fr / en / es). The first press of « Traduction en ligne » still opens
the sheet when not signed in. `add-lingua-app-email-signin` (merged first) replaced the note for an
email-only account: the sheet now offers « Continuer avec un email », and « Se connecter » in the
opening line opens that same sheet. Its named signed-in line (« Connecté en tant que @handle … »)
carries over to the synchronising line, and the account's id comes from the `GetAccount` that names
it — no second read.

### D12 — The deltas, and the archive order

- `lingua-apple-app` « The app reviews the deck the extension keeps »: its text in force is
  `add-lingua-review-translation-fallback`'s (the app's only network requests). This delta rewrites
  that text — the deck may also come from the server, over the app's own session, on the extension's
  account — and keeps every scenario.
- `lingua-apple-app` « No learning state in the app »: added by `add-lingua-apple`, never archived,
  and already contradicted by #948 (« It SHALL hold no account session either »). This delta states
  what the app keeps (sync bookkeeping, pending changes) and that it holds no deck across openings.
- `lingua-sync` « Apple and Google sign-in on Safari through the host app »: rewritten on the fallback
  change's text — the app's session serves its deck synchronisation as well.
- `lingua-privacy` « Lingua's privacy disclosures match what it collects »: rewritten on the fallback
  change's text.
- New requirements: the app's synchronisation, its account rule, its erasure guard and its statistics
  rule (`lingua-apple-app`); the sign-in offered on opening (`lingua-apple-app`); the app as a device
  of its own (`lingua-sync`).
- « A review made in the app reaches the extension » is **not** modified: it stays true word for word,
  and `add-lingua-card-edit` and the fallback change already modify it.

`.openspec.yaml` lists `add-lingua-review-translation-fallback` and `add-lingua-apple` in
`archiveAfter`: a MODIFIED block replaces a requirement whole, so this change must archive after the
two changes whose text it rewrites (the archive-order script would also wait on `add-lingua-apple`'s
ADDED header by itself). `add-lingua-app-email-signin` is not on `main` yet; whichever of the two
archives second rewrites its delta of the sign-in requirement on the first's archived text (task 1.2).

### D13 — The privacy annex

The annex's « Synced for a signed-in reader » row says the deck and statuses are synchronised by the
extension and — once the reader signs in inside the Cymbra Lingua app — by the app too, under its own
session, with the same content (no page address); and that daily statistics still go through the
extension. Nothing new is collected: same data families, same account.

## Risks / Trade-offs

- [**iOS may suspend the page before the background push finishes**] → `beginBackgroundTask` gives it
  time; anything unpushed is in the App Group and leaves at the next opening (D6).
- [**Statistics lag on other devices** until Safari runs] → accepted for a single writer (D9); the
  deck and due counts are current at once.
- [**Equal-content rewrites** when the extension re-pushes cards the app already pushed (D4)] → one row
  per changed card, converging on the greatest device id; already the behaviour between devices.
- [**The 3 s wait at opening** on a slow network] → bounded; past it the review opens on the snapshot
  and the pull lands between sessions (D2).
- [**A reader with two accounts** sees « Safari est connecté à un autre compte »] → said plainly; the
  relay keeps working.
- [**An older extension** writes no account digest: the app does not synchronise] → it says so and
  asks to update Safari; the extension ships before the app (Migration Plan).
- [**The page's `localStorage` is not durable** under a custom scheme] → losing it costs a full pull
  and a new device id (D10).
- [**Concurrent sessions on two devices** can each introduce the day's allowance] → as between two
  extension devices today; the pull before the review narrows the window.
- [**The relay now holds an account digest**] → in the App Group, readable by the app and its
  extension only; not reversible to the account id without the id itself; never a token.
- [**`add-lingua-app-email-signin` lands with other copy**] → task 1.2 reconciles the sheet's lede and
  the sign-in requirement, whichever archives second.

## Migration Plan

1. Ship the extension: the relay's `account` and `erasedAt` fields (an older app ignores them), the
   sync engine's options (defaults unchanged).
2. Ship the app: the page's synchronisation, the sign-in line on opening, the background hook, the
   sheet's lede. Against an older extension it does not synchronise and says so.
3. Publish the privacy annex with the app release.

Rollback: an app without this change stops pushing and pulling; its relay is unchanged, so the
extension carries the reviews as before. The extension's new relay fields are ignored by an older
app. No server state to undo: the app's rows are ordinary rows of the reader's account.
