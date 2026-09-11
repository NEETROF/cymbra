# Cymbra Lingua — browser extension (Chromium)

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
yarn gen:pack     # build the EN→FR data pack                        → assets/pack.lingua (gitignored)
yarn build        # bundle the loadable extension                    → dist/
```

Then load it unpacked: Chrome → `chrome://extensions` → Developer mode → **Load
unpacked** → pick `apps/lingua-extension/dist`.

Checks (what CI runs):

```bash
yarn lint && yarn format:check && yarn typecheck && yarn test
```

## How it works

- **Analysis** runs in the content script's isolated world via the `AnalyzerPort`
  seam (`src/analyzer/`). The Chromium implementation is the WASM module
  (`WasmAnalyzerPort`); Firefox and Safari will implement the same port differently
  without touching the reading code.
- **Highlighting** uses the CSS Custom Highlight API — two registries
  (`cymbra-lingua-unknown`, `cymbra-lingua-learning`), **zero DOM mutation**
  (`src/reading/highlight.ts`, `blocks.ts`). Dynamic pages are re-analysed per mutated
  subtree, visible content first (`observer.ts`).
- **State** — statuses, the captured deck, calibration — lives in
  `chrome.storage.local` under a versioned schema with forward migration
  (`src/state/`). A gesture in one tab repaints every other via `storage.onChanged`.
- **Permissions** — `activeTab` by default (the popup's _Analyser cette page_),
  `<all_urls>` optional (_Toujours surligner_, granted once). No network requests at
  all; the pack and glosses are local assets.
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
  hosts over one logic. On Safari (no panel API) this becomes the sole review surface.

State authority: the WASM engine holds lingua-core's whole `LinguaState` (knowledge +
deck + FSRS); it is persisted as its lossless backup string in `chrome.storage.local`,
so a gesture or a graded card in one context repaints every other via
`storage.onChanged`, and the backup file is a byte-for-byte export of the same thing.

## Scope

Reading + review ship here. The Firefox/Safari manifest variants (the Firefox sidebar
reuses the side-panel page; Safari leans on the drawer), sync and accounts, and the
agent plugin are later changes in the stack.
