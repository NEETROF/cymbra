# add-lingua-review-translation-fallback — design

## Context

See proposal.md for the why. What exists:

- **The review page** (`apps/lingua-extension/src/review/review-page.ts`, `view.ts`) is one
  implementation mounted by four hosts: the in-page drawer, Chrome's side panel, the EPUB reader
  (through its drawer) and the Cymbra Lingua app (`src/app/app.ts`). `mountReview` takes an
  optional `translator: TranslatorSource`; `review/translation.ts` builds the request for a card
  without a gloss — its sentence, the word marked where the front marks it (`markWord`), an
  expression its sentence does not hold sent alone, a single word in that case not sent — bounded by
  `TRANSLATION_WAIT_MS`. `view.ts` `missingGloss()` shows the translation, « Traduction en cours… »
  or the no-translation note. A card with a gloss is never translated (D1 of
  `add-lingua-review-translation`).
- **The app mounts the review without a translator**: `mountReview($("review-root"), port, store,
  { now, prefs, interfaceLanguage })`. Its build (`build.mjs`, `dist-app/`) sets
  `__TRANSLATION_HOST__` to `"none"`. The page is served by `ViewController` under
  `lingua-app://app/` (`AppPageResource`: the page from the app, the packs from the embedded
  extension). It reaches the App Group through the `lingua` script handler (`app/bridge.ts`):
  `review.state`, `review.pendingOps`, `review.writeOps`, `review.unavailable`. The relay
  (`src/state/app-relay.ts`) carries the backup and `RELAYED_PREFS` — « never the session or the
  account » — one way, and the app's card and status ops back, applied by the extension under
  last-write-wins.
- **The engine** runs in a classic worker (`src/translate/host/engine-worker.ts`) on models read
  from the IndexedDB database `lingua-model`, filled by `model-download.ts` with bytes verified
  against the catalogue the package carries (`model-manifest.ts`). `create-port.ts` gives a
  translator only when « Traduction étendue » is on for this device and every model of the pair's
  route is there. On Safari the engine runs in the extension's own web data store, which the app
  cannot read.
- **Online translation** (`src/translate/online.ts`): the selection card sends a runtime message to
  the background, which owns the session and calls `OnlineTranslationService.Translate` with the
  sentence, the selection's UTF-16 span and the pair. Offered when the per-device setting
  `ONLINE_TRANSLATION_KEY` is ticked and the account's effective flag
  `lingua.online_translation.enabled` is on; the server reads the same flag before asking any
  provider. A single word is excluded on the selection card. The RPC places no constraint on the
  selection's length in words.
- **The session.** The extension signs in over gRPC-web with a bearer `TokenPair`; access tokens
  live 15 minutes (`CYMBRA_ACCESS_TOKEN_TTL`), refresh tokens rotate, and a reused refresh token
  revokes its family. On Safari the container app runs the Apple and Google sheets
  (`Shared (App)/SignInView.swift`, `LinguaSignIn`) and hands the id_token to the extension through
  the App Group; `lingua-sync` holds « The container app SHALL hold neither a session nor learning
  state ».
- **The card** (`crates/lingua-core/src/decks/card.rs`): `gloss: Option<String>`,
  `gloss_language` (absent means `fr`), `updated_at`. `reviewCurrent` (`crates/lingua-wasm`) shows
  the card's own gloss when it is in the engine's native language, else `readable_gloss` (the pack
  of the native language first, the card's text after). `addCard` **replaces** an existing card,
  gloss included, with the pack's or the surface's. Sync: `CardOp` fields 1–12 (12 is
  `gloss_language`); the server upserts a whole card when the incoming `updated_at` is newer
  (`pg_deck.rs`: `gloss = excluded.gloss`), and `Deck::apply_card_lww` does the same on the client,
  keeping the local page address when the pulled one is empty. Migrations 0001–0007 exist.
- **`lingua-translation` holds « A machine translation is never stored »**, which also covers the
  online translation.
- **Siblings.** `add-lingua-card-sentence-translation` (docs merged #694, not implemented) keeps the
  whole sentence's translation at capture in a new card field, never as the gloss; its D3 names
  `CardOp` fields 12 and 13 and migration 0006, all since taken or about to be. `add-lingua-card-edit`
  (upcoming) lets the reader edit a card's gloss. `fix-lingua-en-fr-modal-glosses` (upcoming) fixes
  the pack's English modals.

## Goals / Non-Goals

**Goals:**
- Every review card has a way to an answer wherever the reader has a model or online access — in
  the app as in the extension.
- An answer the reader accepts is asked once, ever, on every device.
- A reader past a bad pack gloss can replace it, and the replacement wins over the pack.
- E-ink stays one repaint per reveal; anything more is the reader's tap.

**Non-Goals:**
- Editing a gloss by hand outside the keep action, or going back to the dictionary's gloss:
  `add-lingua-card-edit`, which shares D5's mark.
- Keeping the sentence's translation: `add-lingua-card-sentence-translation`.
- Fixing pack data: `fix-lingua-en-fr-modal-glosses`.
- Changing the engine, the catalogue, the online service, its providers, its limit or its access
  rule.
- Syncing the deck from the app over the network: the app keeps reading the App Group.

## Decisions

### D1 — The order: the card's gloss, the engine on its own, then the reader's tap

A revealed card answers, in order:
1. **The card's gloss** — one the reader kept, else the current pack's, else the one stored at
   capture (D5). With a gloss, nothing is asked on
   its own: the engine costs nothing on glossed cards, as before.
2. **Without a gloss, the engine on its own**, as `add-lingua-review-translation` does today, now in
   the app as well (D6).
3. **The reader's tap.** Online translation (D2) once revealed, when offered; and on a card **with**
   a gloss, a small link (French UI copy: « Traduire la phrase ») that asks the engine for that card
   (when a model is ready) and shows the online button under its rule. The link is shown only when
   at least one of the two can answer; tapping it replaces the link with the translation block.

*Alternative.* Translating every glossed card on its own: an engine call per card on the e-ink
reader, a second answer competing with the gloss being tested (D1 of the archived change). The bad
gloss is the exception; the tap costs one gesture on it, nothing on the others.

### D2 — Online translation in review follows the selection card's rule, single words included

The button (French UI copy: « Traduction en ligne ») is shown on a revealed card if and only if the
setting is ticked on this device and the reader's account has access — the rule of
`lingua-translation`, read the same way (the effective flag; a stale snapshot can show the button,
never grant a translation). It sends the card's sentence with the word's span — the span D3 of the
archived change computes — and the card's language and the native language, through the unchanged
`Translate` RPC. An expression its sentence does not hold is sent alone, with the whole text as the
selection; a single word its sentence does not hold is not offered the button. One press per card
per review; a late answer for an answered card is dropped; « unavailable » and « limit » read as on
the selection card.

**Single words are allowed here**, unlike the selection card, because the reasons for excluding them
do not hold: on a page, a single word has its dictionary card, which is the better answer and costs
nothing; in review the button stands where the dictionary had nothing (no gloss) or where the reader
judged the gloss useless and tapped past it. The word is still never sent alone: its sentence goes
with it, marked by position, so the provider translates it in context. Each press counts against
the same daily limit.

The review gets an `online` source in its options beside `translator`. In the extension the drawer,
the side panel and the EPUB reader hand it the runtime-message client the selection card uses — the
background owns the session. In the app it is the page's own client (D7).

*Alternative.* A button on every card, glossed or not: the same competition with the gloss as in D1.

### D3 — What is kept: the marked word's translation, shown before it is kept

Under a translation (engine or online) the review shows one action. Its candidate is:
- for a translation with marks, the marked text — the marks in range, in order, joined by « … »
  (« remettait … à plus tard »);
- for an expression translated alone, the whole translation.

When the candidate is non-empty, at most 80 UTF-16 code units and drawn from a single mark (or is
an expression translated alone), the action names it (French UI copy: « Garder « se relâcher » »)
and one tap keeps it: the reader sees exactly what is kept, so a mark the engine placed badly is
visible before it is kept. Otherwise — no mark (the engine dropped the tag, or a pair whose marks
are not measured), several marks (a split translation, or a spurious second mark: on the iPhone
spike Bergamot answered « Ce phénomène est <b>facilitamment appelé</b> une ruée <b>vers</b> le
sucre » for « colloquially referred to as »), marks out of range, a candidate too long — the
action reads « Garder une traduction… » and opens a one-line field, prefilled with the candidate
when there is one, empty otherwise, with « Enregistrer » disabled while the trimmed field is empty;
the same 80-unit bound applies. The field is also reachable from the one-tap action's neighbour
« Modifier », so a candidate that is close but wrong is corrected rather than kept as is.

Keeping writes the text as the card's gloss, `gloss_language` the engine's native language,
`gloss_by_reader` set (D5), and bumps `updated_at`; the answer space shows the new gloss in place
and the action disappears. The sentence's translation itself is never kept by this change.

*Alternative.* Keeping the whole sentence: that is `add-lingua-card-sentence-translation`'s field,
and a sentence is not a gloss — the front tests a word. *Alternative.* Disabling keep when the mark
is missing: the slacken case on a pair whose marks are not measured would never be fixable.

### D4 — Kept, it is never asked again; not kept, nothing is stored

A kept gloss makes the card a glossed card whose gloss comes first (D5): D1 step 1 answers it, the engine is not asked, and the
« Traduire la phrase » link still lets the reader replace it. A translation not kept is discarded
with the card, as today: nothing in the backup, nothing synced. The answer memory of the page stays
in memory only.

### D5 — One mark for a gloss the reader chose: `gloss_by_reader`, shared with `add-lingua-card-edit`

The mark is the one `add-lingua-card-edit` defines (its D4, D5), with exactly its name and shape:
`Card.gloss_by_reader: bool`, `#[serde(default, skip_serializing_if = "is_false")]` — left out of
the backup and of the card op when `false`, so every backup and every export without a chosen gloss
keeps its bytes, and the backup's schema version does not move. Keeping a machine translation sets
it to `true`, as writing a translation does there; withdrawing it is that change's (an emptied
field). **Whichever of the two changes is implemented first introduces the field, the `CardOp`
field number and the migration (D6); the other reuses them** — no second marker, no second
migration. A new binding `keepGloss(lemma, gloss, language, now)` writes the gloss, the engine's
native language as its label, the mark, and bumps `updated_at`.

Precedence, everywhere a gloss could be replaced:
- **At review, for every card** (display only, the card never rewritten): (1) a gloss the reader
  chose, as written, whatever its language; (2) the current pack's gloss for the lemma, read as
  `readable_gloss` reads it; (3) the gloss stored at capture. Today `reviewCurrent` shows the stored
  gloss whenever it is in the engine's native language and reaches the pack only for other
  languages, so a pack fix — such as the curated « would » of `fix-lingua-en-fr-modal-glosses` —
  never reaches cards already in decks. This change makes the pack win over the stored text for
  every card; `fix-lingua-selection-card-punctuation` (its D3) already routes cards **without** a
  gloss through `readable_gloss` (trimmed lemma) and is not redone here. `reviewCurrent`'s keys do
  not change.
- `addCard` on a lemma whose card has a reader's gloss keeps that gloss, its label and its mark (a
  re-capture refreshes the sentence and form, not the reader's choice).
- Any rewrite of stored glosses from a newer pack skips a reader's gloss.

*Trade-off.* A pack regression now shows on old cards too, not only on new captures; the reader can
always pin their own gloss (keep, or `add-lingua-card-edit`), which the pack never overrides.
*Alternative.* A three-state source (not said / pack / reader) with a server-side sticky rule: it
would let an explicit return to the dictionary travel and guard against older clients' ops, but
`add-lingua-card-edit` settled on the boolean and a pass-through server, and two markers for one
fact is the thing to avoid. *Alternative.* Reading the mark off `gloss_language` or the text:
invisible to other devices.

### D6 — Sync carries the mark, the server passes it through

As `add-lingua-card-edit` D5 states it: `CardOp` gains `bool gloss_by_reader = 13`; `lingua.cards`
gains `gloss_by_reader BOOLEAN NOT NULL DEFAULT false` (migration `0008`, idempotent,
schema-qualified like 0001–0007); the server stores it and returns it as stored, with no logic. The
server already updates `gloss` on a newer op (`pg_deck.rs`), so a kept gloss travels with the field
alone. `buf breaking` (`FILE`) passes. `exportCardOps` emits the mark only when set; `applyCardOps`
reads it; `apply_card_lww` keeps the local mark when the incoming card carries none and its gloss is
the same text (the echo of a server or a client that predates the field). The app's ops carry it
through the relay unchanged (the relay is the `CardOp` shape) and the extension applies them under
the same rule.

*Alternative.* No proto change, the gloss alone: a kept gloss would sync, but every other device
would show the pack's gloss over it (D5 precedence), and a restore from the server would lose the
mark.

### D7 — Extended translation in the app: its own engine worker, its own copy of the models

The app page gets the engine the Safari extension runs, in a classic worker the page owns — the
page's painting thread never runs it (the placement lint is extended to the app's entry). The engine
artefacts are served from the embedded extension by `AppPageResource`, as the packs are, so they
are in the app bundle once. The models cannot be shared: the Safari extension keeps them in Safari's
web data store, out of the app's reach, and native messaging is no channel for tens of megabytes. So
the app keeps its own copy, in the IndexedDB database `lingua-model` of its page's origin, downloaded
by the extension's own download code from `models.cymbra.app` and verified against the catalogue the
build carries — the same bytes, the same checks.

The app follows the extension's choice on this device: the relay carries « Traduction étendue »'s
setting (not its model state). When it is on and the app has no models for a pair of the reader's
deck, the review shows once, quietly, an offer (French UI copy: « Traduire aussi dans l'app ?
Téléchargement de 26 Mo ») with « Télécharger »; a second download on the same device is the
reader's call, never silent. When the setting is off, the app deletes its copy at its next opening.
Until the models are there the app's review is today's.

**Feasibility gate (task 1.4) — passed; chosen: models in the page's IndexedDB, the engine (not
online-only).** The spike (branch `spike/lingua-app-engine`, its `spike-1.4.md`) ran on a Mac (M2
Max), the iOS simulators (iPhone 17e, iPad Air, iPhone 15 on iOS 17.2, the app's minimum) and a real
iPhone 15 Pro Max (8 GB, iOS 27.2), under `lingua-app://app/`:
- IndexedDB: 64 MiB written in 359–720 ms and the real en-fr model (24.6 MiB served, 35.0 MiB stored)
  downloaded, verified and stored in 1.0–1.3 s on Wi-Fi; both **survive a relaunch and a device
  restart**; quota ~9.8 GiB on iOS, no quota error even at 1 GiB. `navigator.storage.persist()` is
  **refused** (no prompt): the copy is evictable, so the app offers the download again when a pair it
  downloaded is missing.
- The extension's own `engine-worker.ts` and `model-worker.ts`, unchanged, run as classic workers
  (`importScripts`, Bergamot's WebAssembly `200 application/wasm` from `AppPageResource`). On the
  iPhone: cold load 107–172 ms, first translation 129–136 ms, warm 25–30 ms — the extension's own
  answers, marks included.
- The web content process peaks near **347 MiB** and settles near **290 MiB** with the engine and one
  model (Instruments, real iPhone); not killed. An older 4 GB iPhone is still to be watched in the
  dogfood.
- `models.cymbra.app` answers `access-control-allow-origin: *`, which the page needs to read the
  models cross-origin: **that header is now a dependency of the app** (the `cymbra-models` Pages
  project).

So: no native download, no models served by `AppPageResource`. The engine's files — its worker, the
download worker, `engine/` and `model-manifest.json` — are the Safari extension's, served by
`AppPageResource` from the embedded `.appex`; `dist-app` carries none of them. The page constructs the
two workers in one module (`src/app/workers.ts`), which the placement lint holds to.

*Alternative.* Apple's Translation framework in Swift: free and system-managed, but iOS 18 / macOS 15
only (the app supports iOS 17.2), another engine with other answers than the extension's, and no
marked span — so nothing to keep in one tap. *Alternative.* The extension translating on the app's
behalf: the Safari extension does not run while the app is in front.

### D8 — Online translation in the app: the app's own session, never the extension's

The app needs a bearer and the server's access check. Three ways were weighed:
- **Relay the extension's access token** — it lives 15 minutes, and the extension's background does
  not run while the app is in front: stale almost every time the app opens. Rejected.
- **Share the refresh token** — two holders rotating the same token: the first rotation makes the
  other's a reused token, which revokes the family and signs the reader out of Safari too. Rejected.
- **The app signs in on its own** — chosen. The first press of « Traduction en ligne » in an app
  without a session opens the app's existing Apple / Google sheet (`SignInView`); the id_token is
  handed to the page through the bridge (a reply to `auth.signIn`, not the App Group's five-minute
  hand-off), and the page exchanges it through `SignInOidc` for the `lingua` audience with the
  extension's own auth client (interceptors, single-flight refresh) — one implementation. The access
  token stays in the page's memory; the refresh token is kept in the app's Keychain through the
  bridge (`auth.readRefresh`, `auth.writeRefresh`, `auth.clear`), not in web storage whose
  durability for a custom scheme is not guaranteed. It is a session of its own family, revocable
  like any other; account deletion and sign-out everywhere end it. A quiet line under the app's
  views (French UI copy: « Connecté à Cymbra pour la traduction en ligne — Se déconnecter ») shows it
  and ends it. The email flows stay in the extension; an email-only account gets, in the app, a note
  that online translation in the app needs Apple or Google sign-in.

Before the app has a session, whether to show the button reads the relayed facts: the extension's
setting and its last snapshot of the access flag — never its session. After sign-in, access is read
from the app's own session's flags, like the extension's. A reader who signed out from the app is
offered no button until they sign in again from the app's line.

**Transport (task 1.4) — direct.** Measured on the Mac, the simulators and the real iPhone: every
request from the page carries **`Origin: lingua-app://app`** (never `null`), with `Sec-Fetch-Mode:
cors`; a gRPC-web POST is preflighted with that origin and passes CORS when the server echoes it.
Production refuses it today only because the origin is not listed: the server matches origins
exactly (`AllowOrigin::list`), and `lingua-app://app` is a valid header value. So the page calls the
server directly with the extension's own gRPC-web transport, and `lingua-app://app` is added to
`CYMBRA_ALLOWED_WEB_ORIGINS` (configuration, as for the extensions). No bridge transport.

This modifies `lingua-sync`'s « The container app SHALL hold neither a session nor learning state »:
the app may hold a session, for online translation only, and still no learning state of its own.

*Alternative.* A one-time hand-off code minted by the extension's session for the app (new auth
RPC): no second sign-in, but a new credential-minting surface in Cymbra ID for one feature. A
follow-up if the dogfood finds the second sign-in a burden.

### D9 — The relay carries two settings and one flag, still never the session

`RELAYED_PREFS` gains the « Traduction étendue » setting and `ONLINE_TRANSLATION_KEY`; the shared
state gains the extension's snapshot of the one access flag. A kept gloss made in the app reaches
the extension as a card op (D6), and the extension pushes it. Relay version unchanged: the new keys
are ignored by an older reader.

### D10 — Quiet on e-ink

Nothing appears on its own on a glossed card. The link, the online button and the keep action are
small text buttons in the answer space, sized from `--cymbra-lingua-ui-scale`, coloured from
`tokens.css`, with no transition. Every extra repaint follows a tap.

### D11 — The never-stored requirement, and its sibling

« A machine translation is never stored » is renamed « A machine translation is stored only when the
reader keeps it »: only the marked word's translation, only on the reader's explicit action, only as
the card's gloss, synced with the card; the online service still keeps nothing. The privacy annex
(French, English, Spanish) says it in the deck row and the two translation paragraphs, and says that
the app downloads models and may send a sentence when the button is pressed.

`add-lingua-card-sentence-translation` modifies the same requirement (its D7) and keeps the sentence
translation in a field of its own, never as the gloss — no overlap with this change's word gloss.
Its D3 names `CardOp` fields 12 and 13 and migration 0006: 12 is now `gloss_language`, 13 and
`0008` go to `gloss_by_reader` (D6), so when implemented it takes the next free numbers, writes its
delta against the renamed requirement, and archives after this change; task 1.1 records that in its
`.openspec.yaml`.

`add-lingua-card-edit` modifies three of the requirements this change modifies — « Review shows a
gloss the reader can read », « Complete card synchronisation, media excluded » and « A review made
in the app reaches the extension ». A MODIFIED delta replaces the whole requirement, so whichever of
the two archives second rewrites its delta on the first's archived text, keeping both additions
(task 1.2). The two deltas already agree on the mark and the precedence. Once both exist, a kept sentence translation's mark can feed this
change's keep action without asking any engine.

## Risks / Trade-offs

- [**The app's second sign-in**: a reader signed in to the Safari extension must sign in again in the
  app for online translation] → asked only on the first press, with the sheets the app already runs;
  the hand-off code (D8 alternative) is the follow-up if it hurts. An email-only account cannot sign
  in in the app (note shown).
- [**Engine in the app's WKWebView**: IndexedDB on a custom scheme, the worker, WebAssembly memory
  inside the app's web content process (iOS jetsam limits)] → measured (task 1.4): it runs on a real
  iPhone at ~290 MiB settled; an older 4 GB iPhone is watched in the dogfood.
- [**The app's models are evictable** (`persist()` refused)] → a pair downloaded once and found
  missing is offered again; never re-downloaded silently.
- [**`models.cymbra.app` stops answering `access-control-allow-origin: *`**] → the app could no longer
  read the models; the header is recorded as a dependency of the app.
- [**Two copies of the models on one device** (26–52 MB more)] → never silent: offered once, deleted
  when the setting is turned off.
- [A custom-scheme origin rejected by CORS] → measured: WebKit sends `lingua-app://app`, admitted once
  listed in `CYMBRA_ALLOWED_WEB_ORIGINS`; the deploy must list it before the app ships.
- [A badly placed mark kept as a gloss] → the action names what it keeps; « Modifier » corrects it;
  `add-lingua-card-edit` edits it later.
- [More text derived from a page leaves the device: the word's translation, synced] → only on an
  explicit tap, a few words, and the gloss slot already syncs the dictionary's text; disclosed.
- [A client that predates the mark, holding the dictionary's gloss, grades the card after the reader
  kept another one elsewhere: its newer op carries the old text, unmarked, and wins] → the window is
  the time before that client updates itself; the echo rule covers the common case (same text); the
  reader keeps again. Accepted with `add-lingua-card-edit`'s pass-through server.
- [The pack now wins over a stored gloss: a pack regression shows on old cards] → the baselines pin
  every pack's glosses; the reader can pin their own.
- [Single-word online requests use up the daily limit faster] → one press per card per review; the
  limit is the server's, unchanged.
- [Proto number collision with `add-lingua-card-sentence-translation`, which names 12/13] → 12 is
  `gloss_language`, 13 is `gloss_by_reader`; the renumbering is recorded (D11, task 1.1).

## Migration Plan

Deploy order: the migration and the server, unless `add-lingua-card-edit` already shipped them
(the column defaults to `false`; older clients are unaffected), then the extension (keep, online in review, the echo rule, relay keys), then the app
(engine, sign-in), after `lingua-app://app` is added to `CYMBRA_ALLOWED_WEB_ORIGINS` in production.
The privacy annex is published with the first client release that can keep a gloss.

Rollback: an extension or app without this change ignores `gloss_by_reader` and shows a kept gloss
as an ordinary gloss (in the engine's native language, its stored text). Rolling the server back
leaves the column unused and the value in place.
