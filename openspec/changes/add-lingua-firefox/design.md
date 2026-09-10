# Design — add-lingua-firefox

## Context

The Chromium extension exists and is complete (`add-lingua-extension-reading` +
`add-lingua-extension-review`): a single MV3 WebExtension source, analysis consumed
by the content script through the `AnalyzerPort` (the seam laid by
`add-lingua-wasm`, Chromium impl = WASM in the content script), and a review side
panel. This change ports it to Firefox desktop + Android and introduces the
manifest-variant system. Every upstream decision (Highlight API highlighting,
versioned storage, `activeTab` + optional permissions, the Cymbra design language)
is inherited as-is from the earlier changes in the stack.

## Decisions

### D1 — One artefact, several build variants (the Firefox half of D12 in the source design)

One MV3 WebExtension source; the build produces manifest variants — `chromium` and
`firefox` with this change (the `safari` variant joins the matrix with
`add-lingua-apple`). What differs stays confined behind two seams: the
**`AnalyzerPort`** (Chrome/Edge = WASM in the content script; Firefox = WASM in the
event page — its CSP blocks WASM in a content script, day-one spike) and the
**panel surface** (Side Panel API on Chromium; `sidebar_action` on Firefox — the same
extension page in both cases). Firefox: `background.scripts` (event page) declared
alongside `service_worker`, optional host permissions at install (prompt), the same
zip published to AMO for desktop and Android.

## Risks / Trade-offs

- [WASM in Firefox content scripts (CSP)] → day-one spike of the port; the fallback
  is already the design (WASM in the event page + messages through the
  `AnalyzerPort`) — the spike's verdict does not change the architecture, only
  whether a later optimisation is on the table.
- [Event page killed between two requests] → batched requests + memoisation per word
  form on the content-script side; re-instantiating the module is idempotent.
- [Two variants for a solo dev] → a single artefact + seams; the implementation order
  stays Chromium → Firefox → Apple; a manual pass per browser before each release.
