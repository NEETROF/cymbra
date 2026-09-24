# Cymbra Lingua — browser extension (Chromium, Firefox, Safari)

The first user-facing Lingua surface: read the English web **in place**, with unknown
words highlighted, an honest per-page percentage, an offline word popup, phrase capture
and deck creation — **no account, no network**. Framework-free TypeScript, MV3, Yarn
Berry. It consumes the shared `lingua-core` analysis engine compiled to WASM
(`crates/lingua-wasm`).

## Develop

```bash
cd apps/lingua-extension
corepack enable
yarn install
yarn gen:wasm     # build the wasm bindings from crates/lingua-wasm  → src/wasm/pkg/ (gitignored)
yarn gen:proto    # build the gRPC-web/protobuf stubs (auth + sync)  → src/gen/ (gitignored)
yarn gen:pack     # build the EN→FR data pack                        → assets/pack.lingua (gitignored)
yarn build        # bundle every browser variant → dist-chromium/, dist-firefox/, dist-safari/
```

## Account sync (Cymbra ID)

Signing in (icon popup → **Continuer avec Google / Apple** or email/password) turns on
cross-device sync of the deck, word statuses and stats; signed out, the extension stays
purely local. Account creation, email verification (emailed code) and password reset live
on the **account page** (`account.html`, a tab opened from the popup or onboarding — the
popup is destroyed when the reader leaves for their mailbox). The transport is gRPC-web
bearer (Connect-ES), tokens split by volatility — access in `chrome.storage.session`,
refresh in `chrome.storage.local`. The backend origin and the provider client ids come
from build-time env vars, all defaulted for a local dogfooding build:

```bash
LINGUA_GRPC_WEB_URL="http://localhost:50051" \
LINGUA_GOOGLE_CLIENT_ID="<web-oauth-client-id>" \
LINGUA_APPLE_CLIENT_ID="<apple-services-id>" \
  yarn build     # build.mjs also grants the origin in the manifest host_permissions
```

An empty client id hides that provider's button; so does a browser without
`identity.launchWebAuthFlow` (Firefox for Android), where email is the only method.

**Sign in with Apple** asks Apple for **no scope** (`response_type=code id_token`,
`response_mode=fragment`): Apple only allows the fragment without email/name scopes, and
Cymbra ID resolves the account by `(apple, sub)` without reading the email — so the same
Apple ID lands on the same account as in Cymbra Music. The client id is the site's
**Services ID** (`PUBLIC_APPLE_CLIENT_ID`, already in the backend's `CYMBRA_APPLE_AUDIENCE`);
add the extension redirect URLs (`https://<extension-id>.chromiumapp.org/` and Firefox's
`identity.getRedirectURL()`) as return URLs on it.

Two one-time manual steps make **real** sign-in work (the code + email/password path ship
regardless; Google needs the client, and the backend must allow the extension origin):

1. **Google OAuth client** (Web type) with the redirect
   `https://<extension-id>.chromiumapp.org/` — its client id goes into
   `LINGUA_GOOGLE_CLIENT_ID` here and into the backend's `CYMBRA_GOOGLE_AUDIENCE` CSV.
   An unpacked dev build's id is pinned by the committed `LINGUA_EXT_KEY`
   (`figfjglfdiffocldficbimecjnhnkhkh`); the **published** id is a different one, assigned
   by the store, because a store package may not carry `key` at all (see Release).
2. **Backend CORS**: add the extension origin `chrome-extension://<extension-id>` to
   `CYMBRA_ALLOWED_WEB_ORIGINS` in the **dev** environment only. (Firefox's
   `moz-extension://<uuid>` is per-install; use a fixed origin via
   `browser_specific_settings` when wiring Firefox sync.)

One source, three build variants (`yarn build:chromium` / `build:firefox` /
`build:safari` build just one). Load unpacked:

- **Chrome/Edge** → `chrome://extensions` → Developer mode → **Load unpacked** → pick
  `apps/lingua-extension/dist-chromium`.
- **Firefox** (desktop or Android) → `yarn start:firefox` (`web-ext run`, uses
  `dist-firefox`), or `about:debugging` → Load Temporary Add-on.
- **Safari** (macOS) → Safari Settings → Advanced → _Show features for web developers_,
  then Developer → _Allow unsigned extensions_, and Develop → _Add Temporary Extension…_
  → pick `apps/lingua-extension/dist-safari`. For iOS, and for a signed macOS build, use
  the host app in `apps/lingua-apple`.

**Reader injection differs by browser** (`build.mjs`). Chromium is **activeTab-first**:
no static content script; the reader is injected on demand (popup → _Analyser cette page_)
or, after _Toujours surligner_ grants `<all_urls>`, by a dynamic
`scripting.registerContentScripts`. **Firefox and Safari always ship the reader as a
static `content_scripts` on `<all_urls>`** — MV3 dynamic registration doesn't reliably
fire on GeckoView / Firefox for Android (the highlight worked once via _Analyser cette
page_ but not across a reload), and `browser.contentScripts.register()` dies with the
non-persistent event page. The browser-level static injection is the only thing that runs
on every load **and** reload. The global _Surlignage activé_ toggle (default on) is the
off switch, and the reader is entirely local (no network), so always-on is an acceptable
trade there.

Dogfooding — build the variant and launch it (no flag needed on Firefox now):

```bash
yarn dogfood:firefox-android   # = build:firefox + web-ext run --target firefox-android
# or yarn dogfood:firefox on desktop; web-ext live-reloads on each rebuild
```

`LINGUA_ALL_URLS=1` remains a **Chromium-only dev** escape hatch to force the static
script there too (to test the always-on path on Chrome); never ship it for Chromium, whose
model is activeTab-first. `content.ts` self-guards against running twice.

Checks (what CI runs):

```bash
yarn lint && yarn format:check && yarn typecheck && yarn test
```

## How it works

- **Analysis** goes through the `AnalyzerPort` seam (`src/analyzer/`): the WASM module
  runs in the content script's isolated world on Chromium (`WasmAnalyzerPort`), and in
  the background event page on Firefox and Safari (`MessagingLinguaPort`), without
  touching the reading code. `analyse(blocks)` reads a page behind its gates (language,
  token count); `phraseGloss(text)` reads a selection without them, and returns every
  token with its dictionary form, its status and its gloss whatever that status, plus the
  pack's **expressions** it finds in the text — the longest run of dictionary forms that
  matches a key, over at most five tokens.
- **What a selection opens** (`src/reading/selection-card.ts`, where every such decision
  lives, so `content.ts` stays a thin caller): an expression the pack knows is the answer,
  keyed by its dictionary form — `gave up` and `give up` are one card — and stored on the
  card it creates, being dictionary data. Failing that, the card shows a word-by-word
  gloss, labelled as not being a translation of the selection: only the words you do not
  know, no function words, six rows at most, the first sense of each, and never stored. A
  card that has to wait for the engine opens with no action and gains them with its
  answer, so nothing you can press moves under your finger.
- **Highlighting** uses the CSS Custom Highlight API — two registries
  (`cymbra-lingua-unknown`, `cymbra-lingua-learning`), **zero DOM mutation**
  (`src/reading/highlight.ts`, `blocks.ts`). Only the blocks within about one viewport of
  the visible area are painted, following the scroll: WebKit re-evaluates every registered
  range on each rendering update, and ~15 000 ranges on a long article stalled Safari for
  seconds. Dynamic pages are re-analysed per mutated subtree, visible content first
  (`observer.ts`).
- **Gestures** — a plain click on a highlighted (unknown/learning) word opens the popup
  (`Je connais` / `+ Deck` / `Ignorer`). A word you already marked isn't highlighted, so
  to change your mind **Alt/Option-click** it: the popup reopens with the status-aware
  actions (an ignored word offers `Remettre à apprendre` to un-ignore it). A plain click
  never intercepts a non-highlighted word, so the page's own click handling is untouched.
  `Alt+L` captures a multi-word selection as a phrase card.
- **Selection** — selecting text opens a card, on every pointer. One word (a hyphenated
  compound included) opens the word card of its page token, or — in a block the page
  analysis skipped — of the dictionary form the analyser finds in it, so the card and the
  status are keyed by that form, never by the text as written; a name outside the lexicon
  keeps the card under the text as written. Several words open the expression card: with
  nothing better to offer, it lists the pack glosses of the words you don't know — function
  words left out, a few rows at most — under a line saying it is not a translation of the
  expression. Those rows are a reading aid, never stored on a card. A card that needs the
  engine opens **pending** (headword, waiting line, no action) and completes exactly once:
  with the answer, or with a fallback after 3 s / on failure that offers actions only where
  the key is known without the answer (`src/reading/selection-card.ts` owns these decisions;
  `content.ts` only wires them).
- **Read-aloud** (`src/reading/speech.ts`) — the card offers `▶ Mot` / `▶ Sélection` (the
  selection as seen on the page, `ran` not `run`) and `▶ Phrase` (its sentence, left out when
  the selection is the whole sentence); the speaking button becomes `■ Arrêter`. It uses the
  page's Web Speech API from the content script — `speak()` runs inside the click, which is
  the user activation Chrome and Safari on iOS require — with **on-device voices only**:
  `localService: true` in the studied language, the voice always named on the utterance.
  Chrome's desktop "Google …" voices synthesise on Google's servers and are never used, even
  as the default; where only those exist (Chrome on Linux or ChromeOS without a system
  voice), the row is simply absent — install an English system voice to get it. The automatic
  choice trusts a default voice only when it is the only one marked (Safari marks them all),
  and never lands on Apple's novelty, Eloquence or legacy voices (`Bubbles`, `Eddy`, `Fred`…)
  while an ordinary one exists; Réglages lists them apart under _Autres voix_ and keeps the
  reader's choice per device (`cymbra-lingua-voice`, never synced). Closing the card, opening
  another word or leaving the tab stops the speech. The ranking is tested on voice lists
  captured from real browsers (`test/fixtures/voices/`). An iPhone set to French names the
  novelty voices in French (`Bulles`, `Murmure`) — they are recognised by their identifier, not
  their name — and lists some voices twice in two qualities, shown once. On iOS the ring/silent
  mode mutes the voice and it resumes when silent mode is turned off; nothing the extension can
  change. **Firefox for Android** reports every voice of Android's engine (Samsung TTS, Google
  TTS…) as not local, in three-letter codes (`eng-GBR-default`): it cannot tell where that engine
  synthesises. There the row is absent until the reader switches on _Utiliser la voix d'Android_
  in Réglages, which says the text may then leave the device depending on Android's engine
  (`cymbra-lingua-android-voices`, per device). Chrome on Android runs no extension.
- **State** — statuses, the captured deck, calibration — lives in
  `chrome.storage.local` under a versioned schema with forward migration
  (`src/state/`). A gesture in one tab repaints every other via `storage.onChanged`.
- **Permissions** — on Chromium, `activeTab` by default (the popup's _Analyser cette
  page_) with `<all_urls>` optional (_Toujours surligner_, granted once); on Firefox and
  Safari the reader is a static content script on every page (see the injection note
  above). No network requests at all; the pack and glosses are local assets.
- **Identity** — one token sheet (`src/styles/tokens.css`) mirrors the Cymbra
  "Sonic Luminescence" palette; no colour literal lives anywhere else (lint-enforced).

## The data pack (important)

Real EN→FR packs are **never committed** and are built by `scripts/lingua-data`. In this
checkout the real-source fetch is still a stub, so `yarn gen:pack` builds from the tiny
committed **testdata** sources (`scripts/lingua-data/testdata/en-fr`): a four-word
lexicon (`run`, `city`, `seldom`, `conundrum`). That is enough to exercise the whole
pipeline end-to-end, but it means a real article will show almost everything as unknown
until the full pack is wired. `test/fixtures/en-fr.testdata.lingua` is the committed
fixture the tests load.

## Manual verification (pending, task 1.4)

The dynamic-content path (per-subtree re-scan + visible-first prioritisation) needs an
on-device pass on heavy SPAs (Gmail, an infinite feed, a docs site, a news site, a
chat app): confirm inserted paragraphs get highlighted, scrolling stays smooth, and the
extension never janks a page. This can't run in CI; do it against a real pack.

## Review (side panel + drawer)

Reading builds the deck; review runs it, on the same local state:

- A native **side panel** (`sidepanel.html`, `Alt+Shift+S`, the page is pushed and the
  panel survives navigation): deck summary, an FSRS review session (answer hidden until
  revealed, `À revoir`/`Difficile`/`Correct`/`Facile`, `Je connais`), lossless
  **backup** (download) / **restore** (re-import), and a Sources & confidentialité
  section (the pack's NOTICE + "nothing leaves the device").
- An injected **drawer** (`Alt+Shift+D`, closed shadow DOM) for micro-reviews without
  leaving the page — the same `ReviewController` + `renderReview` as the side panel, two
  hosts over one logic. On Firefox and Safari it is the review surface.

State authority: the WASM engine holds lingua-core's whole `LinguaState` (knowledge +
deck + FSRS); it is persisted as its lossless backup string in `chrome.storage.local`,
so a gesture or a graded card in one context repaints every other via
`storage.onChanged`, and the backup file is a byte-for-byte export of the same thing.

## Browser variants

The build produces one artefact per browser from a single source. What differs is chosen
at build time: `__TARGET__` plus capability defines (`__ENGINE_IN_EVENT_PAGE__`,
`__REVIEW_IN_PAGE__`, `__STATIC_READER__` — see `capabilities` in `build.mjs`), so each
bundle folds away the other variants' branches:

- **Chromium** (`chromium`): WASM engine in the content script; panel via the Side Panel
  API; service-worker background.
- **Firefox** (`firefox`, desktop + Android): Firefox's CSP blocks WASM in a content
  script, so the engine runs in the **event page** and the content script reaches it over
  a messaging `AnalyzerPort` (`rpc.ts` / `messaging-port.ts` / the `rpc-host` in
  `background.ts`); review in the in-page drawer; an add-on id in
  `browser_specific_settings`. The engine self-hydrates from storage on event-page wake.
  Published to AMO (desktop + Android, the same zip).
- **Safari** (`safari`, macOS + iOS): the Firefox path with a **non-persistent** event
  page (iOS refuses a persistent one), no `sidePanel` / `identity` permission, and no
  add-on id. Distributed inside the host app `apps/lingua-apple`, whose Xcode project
  bundles `dist-safari/`.

## Release

The version lives in **`package.json`** and nowhere else: release-please bumps it from the
Conventional Commits touching this app, and `build.mjs` stamps it onto each variant's manifest
at build time. `manifest.json` carries no `version` at all — a mirror is a second source that
drifts, and the one we tried (release-please's `extra-files`) rewrote the whole file rather
than the one value, expanding every array until the Prettier gate refused it.

`yarn check:version`, a gate on every pull request, holds what only an upload would otherwise
reveal: the version is three plain integers (a store refuses a `-rc.1` at upload, long after
the tag is pushed), `manifest.json` has not grown a `version` back, and its `description` fits
**112 characters** — Apple's limit, validated when the signed archive reaches App Store
Connect. Chrome allows 132; calibrating on Chrome is how `lingua-apple-v1.1.0` failed after a
full build.

**A release is not a deployment.** Merging the "Release PR" pushes a `lingua-extension-v*` tag,
which runs `lingua-extension-release` and does everything except reach a store:

1. the production build of every variant — the real EN→FR pack, `https://api.cymbra.app`; a
   package that still calls a local backend, or whose manifest reports another version, fails
   the run;
2. `cymbra-lingua-chromium-<version>.zip` and `cymbra-lingua-firefox-<version>.zip` attached to
   the GitHub Release.

**Submitting to the stores is a separate, deliberate act**: dispatch the same workflow with
that tag in `tag` and `publish` ticked. It checks the tag out, rebuilds it, and submits the
Chromium package to the Chrome Web Store and the Firefox one to addons.mozilla.org with the
source archive Mozilla requires (see [REVIEWERS.md](REVIEWERS.md)). The shape is
`lingua-apple-release`'s: what reaches readers is never a side effect of a merge, and a merge
never goes red because store credentials are missing.

**Each store is answered on its own credentials.** One whose keys are missing is skipped with a
warning and named as _not submitted_ in the run summary; the other still receives the version.
Only a run that can reach neither store fails. The two are never ready at the same moment, so
requiring both would make the slower one gate the faster one for ever. To catch the skipped
store up, dispatch the same tag again once its keys exist — the packages are rebuilt from that
tag, so it receives bytes identical to the other store's.

**Name that store in `stores`** (`both`, `chrome`, `firefox`) when you do. Keys say which
stores a run _can_ reach, never which it _should_: the store that already holds the version
would refuse a second submission of it, and the summary would then report it as a store without
the version — false, and the loudest line in the report. A submission that did not refuse would
be worse, replacing a package that is under review. A store left out is named as _not part of
this submission_, distinctly from one whose keys are missing, and raises no warning: leaving it
out was a decision. A run still fails when none of the stores it names can be reached.

The summary also names **Safari**, which this act never reaches — that variant ships inside the
Apple host app, on its own tag. An extension-only fix bumps `lingua-extension` and never
`lingua-apple`, so Safari readers stay behind with nothing failing to say so; dispatch
`lingua-apple-release` with `deliver` to send the same bytes to App Store Connect under the
current marketing version.

The summary reports what each submission **did** — accepted, refused, or never attempted — not
whether its credentials existed. A store can hold every key and still refuse: the Chrome Web
Store rejects a publish until the dashboard's Privacy practices tab is filled, and the upload
succeeds first, so only the outcome tells you whether the store has the version.

AMO refuses a listed version that names no **licence** and an add-on that names no
**category**, so the lane sends both with `--amo-metadata`: `all-rights-reserved`, which grants
nothing, matching a repository that carries no `LICENSE`, and the `language-support` shelf.
Expect the store to reveal such requirements one at a time — each is refused only once the
previous one is satisfied. It is written by the workflow rather than committed beside the manifest,
because this job checks out the _tag_ — a file added to `main` would be missing from every
older tag. Change it in the AMO dashboard, or there, when it becomes a decision.

A **dispatch with no tag** builds the branch you dispatched from and publishes nothing — the
only way to validate a source change before tagging it.

The **safari** variant is built here but published nowhere: it ships inside the Apple host app,
which `lingua-apple-release` builds from the same commit under its own version.

**Published means submitted.** Both stores review a new version, and a first submission is
read by a human. The run stops at the store accepting the upload and says so; a rejection
arrives by email days later and is answered in the dashboard, not by re-running the workflow.

A store build differs from a production build by three environment variables, and each one
fails quietly if forgotten — the release lane sets all three and then proves it from the
built bundles:

| Variable                                            | Value                    | What forgetting it does                                                                                                  |
| --------------------------------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| `LINGUA_GRPC_WEB_URL`                               | `https://api.cymbra.app` | every sign-in and sync calls localhost                                                                                   |
| `LINGUA_EXT_KEY`                                    | **empty**                | the manifest keeps the unpacked dev id pin, and the Chrome Web Store rejects the upload ("the key field is not allowed") |
| `LINGUA_GOOGLE_CLIENT_ID`, `LINGUA_APPLE_CLIENT_ID` | repository variables     | an empty id hides that sign-in button, so the extension ships with Google or Apple missing and no error anywhere         |

Because `LINGUA_EXT_KEY` is empty for the store, **the published extension's id is assigned by
the store** and is not the dev id `figfjglfdiffocldficbimecjnhnkhkh`. Once the item exists, its
id has to reach two other places or signing in fails for every reader: the Google OAuth client
needs `https://<id>.chromiumapp.org/` as a redirect URI, and the backend needs
`chrome-extension://<id>` in `CYMBRA_ALLOWED_WEB_ORIGINS`. The published id is
`lodgdmkjlbpieomelpdkfaifdbipfncd`.

The listings themselves are filled once by hand — CI only uploads versions of an item that
already exists. The copy for both dashboards, the permission justifications and the data
disclosures live in [STORE-LISTING.md](STORE-LISTING.md), so they change in the same pull
request as the behaviour they describe.

What a tag run needs (it stops and names whatever is missing). Repository **variables**, for
the public values, as `PUBLIC_GOOGLE_CLIENT_ID` already is:

| Variable                  | What it is                                                                                                                    |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `CWS_EXTENSION_ID`        | the Chrome Web Store item id, `lodgdmkjlbpieomelpdkfaifdbipfncd` — it is in the store URL, and the listing must already exist |
| `LINGUA_GOOGLE_CLIENT_ID` | the web OAuth client carrying the extension's redirect URI                                                                    |
| `LINGUA_APPLE_CLIENT_ID`  | the Apple Services ID, the same one the site uses (`com.cymbra.bo.web`)                                                       |

Repository **secrets**, for the credentials:

| Secret                                                    | What it is                                                                                    |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `CWS_CLIENT_ID`, `CWS_CLIENT_SECRET`, `CWS_REFRESH_TOKEN` | OAuth client with the Chrome Web Store API enabled, for the account that owns the item        |
| `AMO_JWT_ISSUER`, `AMO_JWT_SECRET`                        | addons.mozilla.org API credentials, for the account that owns the `lingua@cymbra.app` listing |

## Scope

Reading + review + the Chromium/Firefox/Safari variants ship here. The agent plugin is a
separate app (`apps/lingua-agent`).
