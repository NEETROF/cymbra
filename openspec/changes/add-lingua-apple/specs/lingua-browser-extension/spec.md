# lingua-browser-extension — Safari variant (delta)

## ADDED Requirements

### Requirement: Safari variant
The extension SHALL ship on Safari for macOS and iOS as the `safari` variant of the multi-target build, produced from the same source as the chromium and firefox variants and hosted by the Apple container app. The variant SHALL run the WASM analysis in a non-persistent event page consumed by the content script through the `AnalyzerPort`, inject the reader through a static content script, use the in-page drawer as its only review surface (Safari has no panel API), and declare no permission Safari does not support (`sidePanel`, `identity`). When no level is declared, the page SHALL offer the level choice, since Safari opens no first-run page. Tier-3 channels (Edge Canary Android, curated Edge/Samsung stores, Chromium forks) SHALL NOT be promised or tested.

#### Scenario: One build, three artefacts
- **WHEN** the release build runs
- **THEN** it produces the chromium, firefox and safari variants from the same source, differing only in their manifests and build-time target

#### Scenario: Reading on an iPhone
- **WHEN** an English page is loaded in Safari on an iPhone with the extension enabled
- **THEN** unknown words are highlighted, analysed by the WASM engine in the extension's event page, with no native component involved

#### Scenario: Engine back after a suspension
- **WHEN** the user returns to Safari after two minutes in another app and adds a highlighted word to the deck
- **THEN** the action completes without perceptible delay and a reloaded page is highlighted again

#### Scenario: Review in Safari
- **WHEN** the user opens review from the in-page pastille or from the extension popup in Safari
- **THEN** the in-page drawer carries the review session, on the same local state as the rest of the extension

#### Scenario: First run with no declared level
- **WHEN** the user enables the extension in Safari and opens an English page without having declared a level
- **THEN** the in-page pastille offers to choose a level

### Requirement: Touch-primary devices
The extension's own surfaces SHALL adapt when the primary pointer is coarse (Safari on iPhone and iPad, Firefox for Android): the popup SHALL use the full available width, and no link to a browser keyboard-shortcut editor SHALL be shown, since no such page exists there.

#### Scenario: Popup on a phone
- **WHEN** the user opens the extension popup on a phone
- **THEN** the popup content spans the sheet's width instead of a narrow desktop column

#### Scenario: No dead shortcut link
- **WHEN** the user opens the shortcuts section of the settings on a touch-primary device
- **THEN** the shortcuts are listed without a "configure the browser's shortcuts" link

## MODIFIED Requirements

### Requirement: In-place highlighting without DOM mutation
The extension SHALL highlight unknown words (and, distinctly, "learning" words) through the
CSS Custom Highlight API, without wrapping words in elements or otherwise modifying the
page DOM. Dynamic content (SPAs) SHALL be re-analysed per mutated subtree, debounced. The
content script SHALL consume analysis exclusively through a message port (`AnalyzerPort`),
whose implementation depends on the variant — the WASM module in the content script on
Chromium, the WASM module in the event page on Firefox and Safari — with no change to the
content script. Highlights SHALL be registered only for the blocks within about one viewport
of the visible area and SHALL follow scrolling, so that a long page never registers
thousands of ranges at once (WebKit re-evaluates every registered range on each rendering
update); hit-testing a click SHALL remain available for every analysed word.

#### Scenario: English page highlighted
- **WHEN** an English article page is loaded with the extension active
- **THEN** unknown words appear highlighted and the page DOM contains no node added by the extension (outside the extension's own UI hosts)

#### Scenario: Dynamically inserted content
- **WHEN** an SPA inserts a new English paragraph
- **THEN** that paragraph is analysed and highlighted without re-analysing the whole page

#### Scenario: Long page stays responsive
- **WHEN** a long article with more than 10 000 unknown words is highlighted in Safari
- **THEN** clicking a word opens its popup without perceptible delay, and scrolling reveals the highlights of newly visible blocks
