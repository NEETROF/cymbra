# Design — add-lingua-extension-reading

## Context

Sixth change in the Lingua stack. The core is already there: deterministic analysis
(`add-lingua-analysis`), the lemma-first knowledge model (`add-lingua-knowledge-model`),
decks/FSRS and Anki export (`add-lingua-decks-review`), the EN→FR pack
(`add-lingua-data-pack`), the WASM bindings and native/WASM parity (`add-lingua-wasm`).
This change industrialises the extension half of the **working preshot**
(`~/workspace/lingua-preshot`, a plain-JS MV3 extension) that proved the pieces on real
pages: CSS Custom Highlight API rendering, the popup → statuses → repaint loop, slider
calibration, selection capture.

Monorepo constraints inherited: Yarn for JS, no business logic in the shells. Product
decisions already made: never a reading silo, never "lemma" on screen, local-only by
default, honest per-lemma counting.

## Goals / Non-Goals

**Goals:**
- A user (the founder first) reads the English web in Chrome/Edge with unknown words
  highlighted, an honest per-page percentage, an offline word popup, phrase capture and
  card creation — **with no account and no network**.
- Lay the `AnalyzerPort` seam that will make Firefox (WASM in an event page) and Safari
  (nativeMessaging) possible without touching the content script.

**Non-Goals:**
- Review surfaces inside the extension — side panel, drawer, Anki export
  (`add-lingua-extension-review`).
- The Firefox and Safari build/manifest variants (`add-lingua-firefox`,
  `add-lingua-apple`).
- Multi-device sync, accounts, backend (`add-lingua-backend`,
  `add-lingua-connected-clients`); the agent plugin (`add-lingua-agent`).

## Decisions

The core decisions (the `(language, lemma)` key without POS, the lemmatisation cascade,
the pack format, FSRS scheduling, the WASM target) are inherited from the upstream changes
in the stack and are not re-decided here.

### D1 — Rendering: CSS Custom Highlight API, interaction via `caretRangeFromPoint`
Two registries (`lingua-unknown`, `lingua-learning`), zero DOM mutation (no war with
React/hydration — proven by the preshot). Click: `caretRangeFromPoint` → a sorted token
index (portable, Safari-compatible later); `highlightsFromPoint` (Chromium 140+) as
progressive enhancement. No `<span>` fallback in v1 (every MVP target supports the API).
A debounced MutationObserver re-scans **per mutated subtree** (the preshot re-scanned
everything — good enough to judge, not good enough for Gmail).

### D2 — Where the WASM lives: in the content script
Chrome's isolated world allows `wasm-unsafe-eval`; the module (~1 MB of wasm code + an EN
pack ≤ 5 MB) is instantiated per tab, with zero IPC to analyse. The service worker keeps
only orchestration (badge, commands). Per-form results are memoised on the JS side.
Rejected alternative: WASM in the SW (a round trip per page, and the SW is killed at 30 s).
Analysis is still exposed behind an **`AnalyzerPort`** over messages — that is the seam
that will let Firefox (WASM in an event page) and Safari (nativeMessaging) work without
touching the content script.

### D3 — Permissions: `activeTab` + `optional_host_permissions <all_urls>`
Install with no scary warning; "highlight this page" works immediately; "always highlight"
asks for the grant once. It aligns Chrome with the model Firefox/Safari impose (later
targets in the stack) and de-risks the store review. The percentage badge works in both
modes.

### D4 — Extension state: `chrome.storage.local`, versioned schema
Statuses (a compact lemma→status map), cards (JSON), calibration, preferences — under a
versioned root key with migration. The pack is an extension asset, not storage. The
extension and the future agent plugin (`add-lingua-agent`) will each have their own local
store; **reconciliation belongs to the sync changes** (`add-lingua-backend` /
`add-lingua-connected-clients`), not to this one — a fake local sync would be thrown-away
work.

### D5 — Visual identity: shared Cymbra tokens
Source of truth = `CymbraColors` ("Sonic Luminescence",
`apps/music/lib/theme/cymbra_theme.dart`). The extension embeds a `tokens.css` that mirrors
it — exactly the back-office precedent (`apps/back-office/src/styles.css` already mirrors
the Flutter theme). Surfaces the extension owns = full identity (Midnight Navy, primary
violet, 12/18 radii); surfaces injected into third-party pages = the same tokens but
legibility first (host pages are light or dark). A useful coincidence: the palette already
contains amber (`handLeft`, semantically "pending") and coral (`error`) — they become the
"learning"/"unknown" highlight tints, making the highlighting natively Cymbra. A lint
forbids any hex outside `tokens.css`; the review surfaces
(`add-lingua-extension-review`) will consume the same sheet.

### D6 — Monorepo: `apps/lingua-extension`, framework-free TS
Framework-free TS (injected DOM does not need Vue), vitest for the TS logic, a Yarn build;
the WASM module is the one produced by `add-lingua-wasm` (wasm-pack `--target web`). No
Flutter, no Tauri in this change.

## Risks / Trade-offs

- [Re-scan performance on heavy SPAs (Gmail, virtualised feeds)] → per-subtree re-scan +
  IntersectionObserver (analyse what is visible first) + a per-frame budget; a pathological
  page degrades gracefully (partial highlighting), never with jank attributable to the
  extension.
- [WASM + pack (~6 MB) instantiated per tab] → per-form memoisation, lazy instantiation (on
  the first English block detected), a single module shared by the main frame; measure
  before optimising (target < 50 ms of init).
- [Chrome store review (optional `<all_urls>`, reading page text)] → the D3 posture plus a
  privacy policy stating "the text you read never leaves the device" — true by construction
  (the "No network requests" requirement).
