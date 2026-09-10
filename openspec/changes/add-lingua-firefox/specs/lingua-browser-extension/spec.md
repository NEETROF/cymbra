# lingua-browser-extension — Firefox variant (delta)

## ADDED Requirements

### Requirement: Firefox variant
The extension SHALL ship on Firefox (desktop and Android) as a build variant produced from the same source as the chromium variant: an event page (`background.scripts` declared alongside `service_worker`), WASM analysis loaded in the event page and consumed by the content script through the `AnalyzerPort`, optional host permissions requested at install, the panel via `sidebar_action` (the same page as the Chromium side panel), and AMO publication for desktop and Android from the same zip.

#### Scenario: One build, two artefacts
- **WHEN** the release build runs
- **THEN** it produces the chromium and firefox variants from the same source, differing only in the manifest and the `AnalyzerPort` implementation (the safari variant joins this build with `add-lingua-apple`)

#### Scenario: Firefox permissions at install
- **WHEN** the user installs the extension on Firefox
- **THEN** the extension works in its per-page mode (« surligner cette page » — the UI ships in French) and offers the global grant through the optional-permissions prompt
