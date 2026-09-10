# lingua-browser-extension — Safari variant (delta)

## ADDED Requirements

### Requirement: Safari variant
The extension SHALL ship on Safari macOS and iOS as the `safari` variant of the multi-target build (the same source as the chromium and firefox variants), converted with `safari-web-extension-converter` and hosted by the Apple container app: analysis SHALL run natively over nativeMessaging behind the `AnalyzerPort` (no WASM in any Safari extension context), and the injected panel (drawer) SHALL carry in-browser review on its own, Safari having no panel API. Tier-3 channels (Edge Canary Android, curated stores, Chromium forks) SHALL NOT be promised or tested.

#### Scenario: One build, three artefacts
- **WHEN** the release build runs
- **THEN** it produces the chromium, firefox and safari variants from the same source, differing only in the manifest and the `AnalyzerPort` implementation

#### Scenario: Review in Safari with no panel API
- **WHEN** the user opens review in Safari
- **THEN** the injected drawer (shadow DOM) carries the review session, on the same local state as the rest of the extension
