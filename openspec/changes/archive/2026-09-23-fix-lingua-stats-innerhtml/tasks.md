## 1. Chart (`stats/chart.ts`)

- [x] 1.1 Rewrite `barChartSvg(values, fill, label): string` as `barChartElement(values, fill, label): SVGSVGElement`, using `document.createElementNS("http://www.w3.org/2000/svg", ...)` and `setAttribute` for `viewBox`/`role`/`aria-label`/geometry, matching today's output byte-for-byte.
- [x] 1.2 Update `test/stats.spec.ts`'s `barChartSvg` tests to assert on the returned `SVGSVGElement` (`.outerHTML` for the full-shape assertions, `.querySelectorAll("rect")`/attribute checks where a test targets specific bars).

## 2. Ladder (`stats/ladder.ts`)

- [x] 2.1 Rewrite `vocabularyHtml(est, hasLevels): string` as `vocabularyView(est, hasLevels): DocumentFragment | null` (or an empty-safe equivalent for the `est.universe === 0` case, replacing today's `return ""`).
- [x] 2.2 Rewrite `ladderHtml(rows, declared): string` as `ladderView(rows, declared): HTMLElement`.
- [x] 2.3 Update `test/stats.spec.ts`'s `vocabularyHtml`/`ladderHtml` tests to assert on the returned node's `.outerHTML`/`.textContent` per the design's guidance (whole-shape assertions → `outerHTML`; wording-presence assertions → `textContent`/query).

## 3. Stats view (`stats/view.ts`)

- [x] 3.1 Rewrite the local `seedControlHtml(): string` as `buildSeedControl(): HTMLElement`, constructing the `<div>`/`<select>`/`<input>`/`<button>` structure directly (no reparse-then-`pick()` round-trip).
- [x] 3.2 Rewrite `mountStats`'s root skeleton (`root.innerHTML = ...` at the top of the function) to build the fixed slots (`vocab-slot`, `ladder-slot`, `seed-slot`, `marked-slot`, `topline`/`ranges` from `RANGES`, `cards`, the trailing note) via `createElement`/`append`, mounted with `root.replaceChildren(...)`.
- [x] 3.3 Update `renderLadder` to mount `vocabularyView`/`ladderView`'s returned nodes into `.vocab-slot`/`.ladder-slot` via `replaceChildren`, including the two "no CEFR data" / "no estimate yet" static-note branches (build these as elements too, not leftover template strings).
- [x] 3.4 Update the "seed a level" block to mount `buildSeedControl()` into `.seed-slot` via `replaceChildren`.
- [x] 3.5 Update `renderCards`'s per-metric card loop (`card.innerHTML = ...`) to build the card's `<div class="metric">`/`<b class="mtotal">`/`<div class="chart">` structure via `createElement`/`append`, mounting `barChartElement(...)`'s returned SVG node directly instead of interpolating its serialized string.

## 4. Verification

- [x] 4.1 `yarn typecheck` in `apps/lingua-extension`.
- [x] 4.2 `yarn test` in `apps/lingua-extension` — all specs green, including the updated `stats.spec.ts` assertions.
- [x] 4.3 `yarn lint` in `apps/lingua-extension`.
- [x] 4.4 `yarn build` (all 3 targets) in `apps/lingua-extension`; grep `dist-firefox/*.js` and `dist-safari/*.js` for `.innerHTML` — confirm no remaining dynamic assignment (a literal empty-string clear, if any remains, is not a linter concern).
- [x] 4.5 Manual smoke check, done as a measured before/after on both hosts rather than by eye. **Chromium side panel** (real extension, driven over CDP: declare B1, seed 20 A1 words, open Stats): the screenshots of the two builds are the same file, byte for byte, and the rendered DOM matches once two serialisation-only differences are set aside — `style="width:0%"` normalised to `style="width: 0%;"`, and the seed count's `value` now a DOM property rather than an attribute (the field still shows 20). **In-page drawer** (`__REVIEW_IN_PAGE__`), exercised on Safari iOS rather than Firefox — the same bundle logic and the same host — by reinstalling over the same simulator so the data was unchanged and only the code differed: **0 differing pixels over 2 486 250**. The daily charts were at zero in both views, so the bars were not exercised there; `barChartSvg`/`barChartElement` were compared directly on `[2, 0, 5, 1, 9, 3]` instead, and every attribute matches to the hundredth.
