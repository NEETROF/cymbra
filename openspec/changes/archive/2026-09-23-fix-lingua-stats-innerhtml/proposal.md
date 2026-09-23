## Why

The AMO validation of Cymbra Lingua 1.0.2 reports 15 "Unsafe assignment to innerHTML" warnings (0 errors — not blocking, but flagged on every future submission). All 15 trace back to a single source file, `apps/lingua-extension/src/stats/view.ts`, whose compiled output is duplicated across three bundles (`stats.js`, `content.js` via the Firefox/Safari in-page drawer, `sidepanel.js` via the Chromium side panel) because both surfaces import it. The interpolated data is already trusted (CEFR levels, numeric counts, fixed labels) — there is no injectable content — but addons-linter cannot prove that statically, and the file's own convention elsewhere (`markedRow`/`markedSection`, same file) already builds trusted+untrusted content via DOM APIs instead of HTML strings. This closes the gap for the remaining sites.

## What Changes

- Replace the 6 HTML/SVG string-template builders that back `innerHTML` assignments in `stats/view.ts` with DOM-API construction (`createElement`/`append`/`textContent`, `createElementNS` for the SVG chart), matching the pattern `markedRow`/`markedSection` already use in the same file.
- Affected builders: `mountStats`'s own root skeleton and per-metric card rendering (`stats/view.ts`), `vocabularyHtml` and `ladderHtml` (`stats/ladder.ts`), `barChartSvg` (`stats/chart.ts`), and the local `seedControlHtml` (`stats/view.ts`).
- No visual or behavioral change: same markup shape, same classes, same text, same SVG output — this is a rendering-mechanism swap, not a feature or UX change.
- Existing unit tests for `ladderHtml`/`vocabularyHtml`/`barChartSvg` (`test/stats.spec.ts`), which currently assert on returned HTML/SVG strings, are adapted to assert on the returned DOM/SVG nodes instead (via the `jsdom` environment already configured for this project's tests — no new dependency).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: adds a new requirement codifying that the stats view renders without dynamic `innerHTML` — a hardening posture (alongside the existing "Minimal permission posture" requirement in the same spec), not a change to any user-facing behavior. No existing requirement's text or scenarios change.

## Impact

- **Products**: Cymbra Lingua only (`apps/lingua-extension`) — no impact on Music, ID, Live, back-office, or site.
- **Code**: `src/stats/view.ts`, `src/stats/ladder.ts`, `src/stats/chart.ts`, `test/stats.spec.ts`.
- **Consumed vs new**: consumes the existing `stats/model.ts` data shapes unchanged; introduces no new dependency (DOM APIs and `jsdom` are already used/available in this codebase).
- **Build/CI**: verified by re-running `yarn build:firefox` / `yarn build:safari` in `apps/lingua-extension` and confirming the built bundles contain no more dynamic `.innerHTML =` assignments (the addons-linter warning source); full confirmation only lands after the next AMO submission's validation report.
