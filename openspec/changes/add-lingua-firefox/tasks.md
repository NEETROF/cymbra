# Tasks — add-lingua-firefox

## 1. Firefox port (desktop + Android)

- [ ] 1.1 Day-one spike: WASM in a Firefox content script (CSP) — verdict documented; the design assumes the event-page fallback whatever the outcome
- [ ] 1.2 Manifest-variant system introduced in the build: a `firefox` variant generated alongside `chromium` (event page `background.scripts` declared alongside `service_worker`, explicit `wasm-unsafe-eval` CSP, `browser_specific_settings`)
- [ ] 1.3 Event-page `AnalyzerPort` impl: WASM loaded in the event page, batched requests, memoisation per word form on the content-script side
- [ ] 1.4 Optional permissions at install (`permissions.contains` detection + prompt) and the panel via `sidebar_action` (the same page as the side panel)
- [ ] 1.5 Manual Firefox desktop + Android pass (`web-ext run` / adb) documented; AMO publication (desktop + Android, the same zip)
