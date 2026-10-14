## 1. Offered on Firefox for Android

- [x] 1.1 `background.ts`: `offered` is true wherever the engine is packaged; the `getPlatformInfo` check goes; `ModelStatus.offered` and `NOT_OFFERED` stay (unreachable background, no engine); tests
- [x] 1.2 The settings row, the download, the deletion and every state behave as on Firefox desktop under an `android` platform; tests

## 2. Warming the engine

- [x] 2.1 Move the idle period (`ENGINE_IDLE_MS`) to `translate/port.ts`, where a surface may read it; `channel.ts` imports it; `lint-translator-placement` stays green
- [x] 2.2 Wire: a `warm` message type with its guard, distinct from `lingua-translate`, the analyser's `lingua-rpc` and the keep-warm ping; tests
- [x] 2.3 `EngineAccess.warm()`: `EngineChannel.warm()` starts the worker and loads the model without translating, and arms the idle release; a failed start leaves nothing behind; tests with the fake clock
- [x] 2.4 Chromium: `OffscreenEngine.warm()` ensures the document and forwards a `warm` op; `serveOffscreen` answers it; tests
- [x] 2.5 Background: answers `warm` only when `model.ready()`; never loads anything otherwise; tests

## 3. The signals that warm it

- [x] 3.1 `SelectionWatcher` gains `onBegin`, called on the first usable `selectionchange` of a gesture, once, re-armed by a collapsed selection; tests
- [x] 3.2 `ReadingSession` answers `onBegin` with `warm` only when its translator source has a translator (setting on, model ready); nothing is sent otherwise; tests
- [x] 3.3 `keepWarm` remembers when its page last asked for a translation and sends `warm` on `visibilitychange` → visible within the idle period; nothing for a page that never asked, or asked longer ago; tests with a fake clock and document

## 4. The page's answers

- [x] 4.1 An answer memory around the port a surface gets: key = sentence + selection span, 32 most recent, translations only, per page, never stored; tests (hit, different span, `unavailable` not kept, eviction)

## 5. Measured on the device

- [x] 5.1 SM-P610, Firefox release, a build of this change: first selection of a visit, a selection after a one-minute return from another app, handles adjusted over one sentence, memory over a ten-minute read; record the times against the 2026-09-26 baseline (cold 4.1–4.7 s) in design.md
- [x] 5.2 Firefox desktop and Chrome: the same pass, headless (`e2e546` harness), nothing regressed; warming with no model sends nothing

## 6. Copy and documentation

- [x] 6.1 `STORE-LISTING.md`: the AMO description (fr + en) names Firefox for Android among the places extended translation is offered
- [x] 6.2 `TRANSLATION.md`, `README.md`, `REVIEWERS.md`: no longer "hidden on Firefox for Android"; the warm signals and the answer memory described where the engine's lifetime is

## 7. Gates

- [x] 7.1 `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (coverage ≥ 80 %), `yarn check:variants`
- [x] 7.2 `openspec validate add-lingua-translation-android --strict`
