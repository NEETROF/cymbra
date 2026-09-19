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
   The published extension id is stable (manifest key); an unpacked dev build's id is
   shown on `chrome://extensions`.
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
  touching the reading code.
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

The version lives in **`package.json`**, which release-please bumps from the Conventional
Commits touching this app; the same run mirrors it into `manifest.json`, the single manifest
`build.mjs` folds into all three variants. Nothing here is edited by hand: `yarn check:version`
(a gate on every pull request) fails when the two disagree, and when the version is not three
plain integers — a store refuses a `-rc.1` at upload, long after the tag is pushed.

Merging the "Release PR" pushes a `lingua-extension-v*` tag, which runs
`lingua-extension-release`:

1. the production build of every variant — the real EN→FR pack, `https://api.cymbra.app`;
   a package that still calls a local backend, or whose manifest reports another version,
   fails the run;
2. `cymbra-lingua-chromium-<version>.zip` and `cymbra-lingua-firefox-<version>.zip` attached
   to the GitHub Release;
3. the Chromium package submitted to the Chrome Web Store, the Firefox one to
   addons.mozilla.org with the source archive Mozilla requires (see [REVIEWERS.md](REVIEWERS.md)).

The **safari** variant is built here but published nowhere: it ships inside the Apple host
app, which `lingua-apple-release` builds from the same commit under its own version.

A **dispatch** does all of that except publishing, keeping the packages as workflow artifacts.
It is the only way to validate a source change before tagging it.

**Published means submitted.** Both stores review a new version, and a first submission is
read by a human. The run stops at the store accepting the upload and says so; a rejection
arrives by email days later and is answered in the dashboard, not by re-running the workflow.

Secrets a tag run needs (it stops and names the ones that are missing):

| Secret                                                    | What it is                                                                                    |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `CWS_EXTENSION_ID`                                        | the Chrome Web Store item id — the listing must already exist                                 |
| `CWS_CLIENT_ID`, `CWS_CLIENT_SECRET`, `CWS_REFRESH_TOKEN` | OAuth client with the Chrome Web Store API enabled, for the account that owns the item        |
| `AMO_JWT_ISSUER`, `AMO_JWT_SECRET`                        | addons.mozilla.org API credentials, for the account that owns the `lingua@cymbra.app` listing |

## Scope

Reading + review + the Chromium/Firefox/Safari variants ship here. The agent plugin is a
separate app (`apps/lingua-agent`).
