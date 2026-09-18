---
name: browser-extension-architecture
description: Framing + hard constraints for the MV3 browser extensions in this repo (apps/lingua-extension, Chromium + Firefox from one source). Use BEFORE building or changing ANY extension UI surface — a content-script HUD, the popup, the side panel / sidebar, an injected drawer, or where an action "opens" a panel — and whenever wiring content-script ↔ background messaging, content-script injection/registration, or per-browser (__TARGET__) behaviour. Encodes the platform truths that are expensive to rediscover (who can open the side panel / sidebar, Firefox GeckoView injection, staying in the reading page, one-impl-two-hosts) plus the scoping questions to settle first.
metadata:
  author: cymbra
  version: "1.0"
---

# Browser-extension architecture & framing (Cymbra Lingua)

One source builds two variants (`build.mjs`, `__TARGET__` = `"chromium" | "firefox"`):
`dist-chromium/` and `dist-firefox/`. Firefox ships desktop **and** Android from the same
zip. These rules exist because getting them wrong causes real rework — settle them **before**
writing UI code, and prefer confirming a UX fork with the user over guessing.

## Frame first — questions to answer before building a surface

1. **Where does this action open, per entry point?** List every entry point (the in-page
   HUD pill, the toolbar popup, a keyboard command) and make the same action land in the
   **same place** from all of them. Inconsistent destinations read as broken.
2. **Does it leave the page being read?** The product principle: **stay in the reading page**
   (except where the platform forces otherwise). Rank surfaces: **in-page drawer** (overlay,
   never leaves) ≥ **native side panel / sidebar** (docks alongside) ≫ **a tab** (leaves the
   page — last resort).
3. **Who is triggering it — a page element or an extension surface?** This decides what you
   can even open (see the truth table). A content-script/HUD click and a popup click have
   *different* powers.
4. **Is the view already built once?** If review/stats/settings exist in one host, render the
   **same module** in the new host — do not duplicate (see "one impl, N hosts").
5. **Per-browser divergence is expected.** Chromium and Firefox will not always use the same
   surface; that is fine as long as rule 1 (consistency *within* a browser) and rule 2
   (stay-in-page) hold.

## The platform truth table (hard-won — do not relitigate by trial)

**Opening the lateral panel:**
- Chromium **Side Panel** (`chrome.sidePanel.open`): needs a user gesture. It **works from
  the popup**, and **works from an in-page element** *if* the background calls it
  **synchronously inside `runtime.onMessage`** using `sender.tab.id` — Chrome propagates the
  content-script click's activation across that one message. Any `await` before the call, or
  querying the tab first, spends the gesture → it fails.
- Firefox **sidebar** (`sidebarAction.open`): needs a user gesture that **does NOT survive
  the content-script → background message**. So the popup can open the sidebar, but **a page
  element (HUD) cannot**. A keyboard `commands.onCommand` handler keeps its gesture (call
  `sidebarAction.open()` synchronously there — not inside a `tabs.query` callback).
- Consequence: to keep the **same** destination from the HUD *and* the popup on Firefox while
  staying in-page, both use the **in-page drawer**, not the sidebar. On Chromium both use the
  Side Panel. (Lingua ships exactly this.)
- A tab of `sidepanel.html` is the universal fallback but it **leaves the page** — use it only
  where nothing else works (and question whether the drawer fits instead).

**Injecting the reader (content script):**
- Chromium is **activeTab-first**: no static content script; inject via the popup's
  `scripting.executeScript`, or register dynamically with `scripting.registerContentScripts`
  after an `<all_urls>` grant (persists across sessions there).
- Firefox ships a **static `content_scripts` on `<all_urls>`, always** (`build.mjs`). MV3
  dynamic registration does **not** reliably fire on GeckoView / Firefox for Android, and
  `browser.contentScripts.register()` is torn down when the non-persistent event page unloads
  (~30 s) — so only the browser-level static injection runs on every load **and** reload. The
  global "Surlignage activé" toggle is the off switch; the reader is 100 % local. See
  `syncReaderRegistration` (no-op on Firefox) and the memory `lingua-firefox-android-injection`.
- On Firefox the analyzer engine runs in the **background event page** (its content-script CSP
  blocks WASM); the content script reaches it over the `AnalyzerPort` messaging seam. On
  Chromium the engine runs in the content script. Don't assume the engine is local to a surface.

## Which context am I in? (the family of bugs that compiles and does nothing)

Three incidents in one dogfooding week shared one shape: **code valid in one context, running
in another**. Each compiled, threw nothing, and silently did not work — no test saw it,
because the wiring lived in the background, which no test covers.

| Context | Runs where | May call | May NOT call |
|---|---|---|---|
| **Content script** (`content.ts`, `reading/`, `review/`, `stats/`) | the **visited page's** origin | `runtime.sendMessage/connect`, `storage.*`, `i18n` | `tabs`, `action`, `identity`, `sidePanel`, **the extension's `indexedDB`** (that one is the *site's*) |
| **Extension page** (popup, side panel, account, onboarding) | the extension origin | the above **plus** `tabs`, `action`, `identity`, `sidePanel`, the extension's `indexedDB` | — |
| **Background** (service worker / event page) | the extension origin, no DOM | everything, and owns the reader's store | `document`, `window`, dynamic `import()` on Chromium |

The three that cost a build each:
1. **`chrome.tabs.create` from the in-page drawer** — the API does not exist there, so the
   link did nothing at all. Anything a page-context module needs from the extension goes
   through a message: `state/open-page.ts` (`openPage`).
2. **Two WASM engines racing** — the background hosts the reading engine *and* the sync
   engine on one glue module; `init()` only short-circuits once it has *finished*, so a
   concurrent second init replaced the module's memory and left the first engine reading out
   of bounds. One initialisation per module, shared (`analyzer/engine.ts`).
3. **A trigger watching a key that moved** — the sync fired "2 s after a mutation" by
   listening for the backup key in `chrome.storage.local`. The data moved to IndexedDB; the
   listener kept compiling and never fired again. **When data moves, hunt everything that
   listened at its old address.**

Two lint specs now fail the build on 1 and 3 (`test/lint-page-context.spec.ts`): a
page-context module may not name `tabs`/`action`/`identity`/`sidePanel`, and nothing may
watch a moved key through `storage.onChanged`. Extend the lists when you add either.

## Where the reader's data lives (and how a surface follows it)

- **The reader's data** — engine backup, daily statistics, sync cursors, device id — lives in
  **IndexedDB, owned by the background** (`state/store.ts`). Surfaces read and write it
  through the same `AsyncStorageArea` seam, whose calls become messages (`messagedArea`).
  Never open the database from a surface, even one that could (an extension page): two
  writers is the echo-suppression problem this design removes.
- **Preferences, tokens and transient marks** stay in `chrome.storage.local`: the highlight
  and pill toggles, the session tokens, the last-sync time, the lost-session mark. They are
  small and must be readable before any round-trip.
- **Following a change**: the owner bumps a marker (`STORE_CHANGED_KEY`) after every write;
  surfaces subscribe with `watchStore` / `watchBackup`. Not a port — a port dies with the
  background page Safari suspends constantly; `storage.onChanged` reaches every context,
  needs no permission, and survives.
- **Every write by the owner goes through `ownerArea`**, so the two consequences of a
  mutation — telling the surfaces, and scheduling the sync — can never be forgotten separately.

## Safari is a third target, and the harshest

`build.mjs` also produces `dist-safari/`, bundled by the host app (`apps/lingua-apple`).
Treat it as the worst case for everything above:
- its background page is **suspended whenever idle**, so anything that must survive a
  suspension cannot be a port, an in-memory cache or a pending timer;
- a message whose response is still pending **keeps the page alive** — that is why a sync
  request is answered only once its exchange is over;
- `chrome.identity` does not exist: Apple and Google sign-in run in the host app and hand an
  id_token back through the App Group (see `state/native-signin.ts`);
- the reading engine runs in the background there, as on Firefox.

## One impl, N hosts — never duplicate a view

Review, stats and settings each have **one** builder, rendered into whatever host needs it —
the native side panel **and** the in-page drawer:
`mountReview` (`src/review/review-page.ts`), `mountStats` (`src/stats/view.ts`),
`mountSettings` (`src/reading/settings-view.ts`), plus `renderReview` for the widget itself.
Each builds its own DOM into a passed container and returns a `refresh()`. When you add a view
or a host, extend/reuse these — do **not** copy markup or logic into a second file, and put the
shared CSS where every host loads it (`review.css` / `settings.css`, imported into the drawer's
shadow via `content.ts` and `<link>`ed by `sidepanel.html`). If content differs between two
hosts, that is the bug.

## Injected-surface mechanics (checklist)

- **Closed shadow DOM** for anything injected into the page (`attachShadow({mode:"closed"})`),
  styled by `tokens.css` + the view CSS passed as one string. Mirrors `WordPopup` / `Drawer`.
- **`data-cymbra-lingua-skip`** on the host, and append it to `document.documentElement` (not
  `body`), so the reader never analyses its own UI (honoured by `blocks.ts` / `observer.ts`).
- **`[hidden]` loses to `display:flex`/`grid`** (equal specificity, author rule wins): a
  toggled-`hidden` flex row needs an explicit `.x[hidden]{display:none}` or it never hides.
- **Colours only from `tokens.css`** — no colour literal anywhere else (lint-enforced); both
  `:root` and `:host` get the tokens.
- **Guard double-init**: a content script can be injected twice (static + activeTab); keep the
  `window.__cymbraLinguaReading` guard, and mount injected hosts idempotently (clear an orphan
  host by id; mount only after a successful first paint).
- **Preserve the page selection**: a toolbar button's `mousedown` collapses the document
  selection — `preventDefault()` on `mousedown` if the click reads `window.getSelection()`.

## Process (this repo)

- **Confirm UX forks with the user before building** (surface, position, which buttons) — a
  wrong guess costs an on-device rebuild. See the memory `no-auto-merge-without-ok` and
  `batch-fixes-one-pr`: never merge without an explicit OK, and iterate fixes on the **same**
  PR rather than one PR per correction.
- **Verify per target**: `yarn typecheck && yarn lint && yarn test && yarn format:check`, then
  `yarn build` (both variants) and grep the built `dist-*/` bundles to confirm the right code
  landed per browser (`__TARGET__` folds constant conditions). Dogfood with
  `yarn dogfood:firefox[-android]`.
- **TS won't catch a bad `*.css` import path** (the `*.css` module is a wildcard) — esbuild
  will; always run `yarn build`, not just `typecheck`.
