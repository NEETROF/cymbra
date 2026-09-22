## Context

`apps/lingua-extension/src/stats/view.ts` (`mountStats`) renders the stats/review view shared by the Chromium side panel and the Firefox/Safari in-page drawer. Some of it already builds DOM nodes directly (`markedRow`, `markedSection` — the parts that render a marked word's `lemma`, i.e. text a reader chose, not a fixed vocabulary). The rest — the root skeleton, the vocabulary estimate, the CEFR ladder, the "seed a level" control, and the per-metric daily-count cards — is built as HTML template strings assigned to `.innerHTML`. `ladder.ts`'s `vocabularyHtml`/`ladderHtml` and `chart.ts`'s `barChartSvg` are documented as "pure, HTML/SVG string out," unit-tested by asserting on the returned string (`test/stats.spec.ts`).

None of these `innerHTML` sinks are actually exploitable — every interpolated value is a number, a fixed CEFR level, or a hardcoded French label from a `const` — but addons-linter (AMO's static analyzer) flags any `innerHTML` assignment with a non-literal right-hand side regardless of the data's actual provenance, and it does so once per bundle that contains the compiled code (3 bundles here, since both the drawer and the side panel import `stats/view.ts`). Fixing the flagged pattern once, at the source, clears the warning everywhere it is duplicated.

## Goals / Non-Goals

**Goals:**
- Eliminate every dynamic `.innerHTML =` assignment in `stats/view.ts` and its two helper modules (`ladder.ts`, `chart.ts`), replacing them with DOM (`createElement`/`append`/`textContent`) or SVG (`createElementNS`) construction.
- Preserve the exact current markup: same element structure, class names, attributes, and text — this is a mechanical rendering-layer swap, not a UI change.
- Keep `vocabularyHtml`/`ladderHtml`/`barChartSvg` easily unit-testable without a real browser (the project's `vitest.config.ts` already runs under `jsdom`, so this needs no new dependency).

**Non-Goals:**
- No visual, UX, or behavioral change to the stats view.
- No change to `markedRow`/`markedSection`, which already use DOM APIs correctly.
- No change to any `lingua-browser-extension` spec requirement — purely an implementation detail (see proposal's Capabilities section).
- Not attempting to silence the warning by other means (e.g. a linter suppression, `DOMPurify`-style sanitization, or `textContent`-only rendering that would lose the intentional HTML structure like `<b>`/`<span>` wrapping) — the point is to render the same structure without a static-HTML sink the linter cannot reason about.

## Decisions

- **Rename returned-value contract from HTML string to DOM node.** `vocabularyHtml(est, hasLevels): string` becomes `vocabularyView(est, hasLevels): DocumentFragment | HTMLElement` (or returns `null`/an empty fragment for the `est.universe === 0` case, mirroring today's `return ""`). Same for `ladderHtml` → `ladderView`, and `barChartSvg(values, fill, label): string` → `barChartElement(values, fill, label): SVGSVGElement`, built with `document.createElementNS("http://www.w3.org/2000/svg", ...)`. The `Html`/`Svg` suffix is renamed to `View`/`Element` to signal the shape change to any future caller — keeping the old name with a new return type would be a silent trap.
- **Callers use `replaceChildren`/`append`, not `innerHTML`, to mount a returned node into a slot.** E.g. `pick(".vocab-slot").replaceChildren(vocabularyView(...))`. `mountStats`'s own root skeleton (currently one big template-string assignment at `view.ts:100`) is rebuilt with a handful of `createElement` + `className` + `append` calls — it is fully static (fixed slots + range buttons from the constant `RANGES` array), so this is mechanical.
- **`seedControlHtml` becomes `buildSeedControl(): HTMLElement`**, constructing the `<select>`/`<input>`/`<button>` structure directly instead of concatenating a string then having `pick()` re-query it — this also removes the current indirection where the control is stringified, reparsed by the browser, then queried back out by id.
- **Tests assert on the constructed node's shape/text, not a raw string.** Existing `.toContain(substring)` assertions on HTML become either `.textContent` substring checks, `outerHTML` checks (a DOM node's `.outerHTML` is a valid, equivalent assertion surface post-construction, and keeps most existing test bodies close to their current form), or targeted attribute/child assertions where a test currently inspects specific markup (e.g. the "no confirmed mention" case). Prefer `outerHTML` for tests that are essentially snapshotting the whole returned markup, and `textContent`/DOM queries for tests that only care about the presence or absence of specific wording — this avoids either rewriting every test's intent or losing precision.
- **`barChartElement`'s SVG children use `setAttribute`, not string concatenation**, for `viewBox`, `role`, `aria-label`, and the `<rect>`/`<line>` geometry — SVG has no shorthand DOM-construction sugar like HTML's `className`, so this is more verbose than the HTML side but mechanically the same idea.

## Risks / Trade-offs

- **[Risk] A subtle markup difference slips in during the rewrite (e.g. a missing class, a swapped attribute order that some CSS selector depended on) → not caught by string-based tests that no longer run.** Mitigation: keep tests asserting on `outerHTML` for the bulk of cases (byte-for-byte equivalent to today's string assertions), and manually diff a rendered snapshot (side panel + drawer, via `yarn build:chromium`/`build:firefox` and the existing dogfood flow) before merging.
- **[Risk] The AMO linter warning is not actually silenced by this change** (e.g. if some other pattern also trips "unsafe innerHTML", or if a warning persists due to caching/version comparison on AMO's side) **→ the fix looks complete locally but the next submission still shows warnings.** Mitigation: grep the built Firefox/Safari bundles for `.innerHTML =` with a non-literal right-hand side before merging (`rtk proxy grep -n "\.innerHTML" dist-firefox/*.js`) to get a local proxy signal; full confirmation still requires the next AMO submission's validation report (out of this change's control — flagged in the proposal's Impact section).
- **[Trade-off] More verbose code for the SVG chart and the seed control** (DOM construction is inherently wordier than a template string) in exchange for removing the untyped "build a string, let the browser reparse it, then re-query it by selector" round-trip that `innerHTML` assignment does today — a net simplification for anything that already needs to attach event listeners afterward (e.g. `seedControlHtml` today builds a string then immediately `pick()`s `#seed-go`/`#seed-level` back out of it).

## Migration Plan

No runtime migration — this is a build-time/source-only change with no persisted state, no API, and no user-visible behavior difference. Land it as one PR:
1. Rewrite `chart.ts::barChartSvg` → `barChartElement`.
2. Rewrite `ladder.ts::vocabularyHtml`/`ladderHtml` → `vocabularyView`/`ladderView`.
3. Rewrite `view.ts` (`seedControlHtml`, the root skeleton in `mountStats`, and the per-metric card loop) to consume the above and mount via `replaceChildren`/`append`.
4. Update `test/stats.spec.ts` assertions for the new return shapes.
5. `yarn build` (all 3 targets) + `yarn typecheck` + `yarn test` + `yarn lint`; grep `dist-firefox/*.js` and `dist-safari/*.js` for remaining `.innerHTML =` assignments.
6. Manual smoke check: open the stats view in both the Chromium side panel and the Firefox in-page drawer (dogfood build) and compare against the pre-change screenshot.

Rollback is a plain revert — no data or contract to unwind.

## Open Questions

- None blocking. The one external unknown (whether AMO's linter actually stops flagging these sites) is a verification step in the next submission, not a design decision — tracked in the proposal's Impact section rather than gating this change.
