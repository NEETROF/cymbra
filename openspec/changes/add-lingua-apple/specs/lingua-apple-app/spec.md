# lingua-apple-app — Safari container app (iOS + macOS)

## ADDED Requirements

### Requirement: An app with functions of its own, one universal listing
The container app SHALL be an app in its own right (decks, FSRS review, settings, LingQ import) rather than a shell around the extension, and SHALL ship under **a single universal App Store listing covering iOS and macOS** (universal purchase, one Xcode project). It SHALL embed the data packs in its own bundle — never in extension storage.

#### Scenario: The app lives without the extension
- **WHEN** the user opens the app without having enabled the Safari extension
- **THEN** they can browse their decks, review their due cards, and start calibration or a LingQ import

### Requirement: Guided extension activation
The app SHALL guide activation of the Safari extension, which arrives disabled by default: on iOS, a step-by-step walkthrough (the OS offers neither a prompt nor a state API) with activation detected through an **App Group heartbeat** (the extension writes a beat on every run, the app reads it); on macOS, a button that opens Safari's Extensions pane directly (`SFSafariApplication.showPreferencesForExtension`) plus the real state through `SFSafariExtensionManager`. The home screen SHALL reflect the activation state.

#### Scenario: First launch on iOS
- **WHEN** the user opens the app for the first time on iOS with no active extension
- **THEN** the activation walkthrough shows first, and dismisses itself once the extension's heartbeat is observed

#### Scenario: Two-click activation on macOS
- **WHEN** the user clicks « Activer dans Safari » on macOS (the shipping UI is French)
- **THEN** Safari's Extensions pane opens on the right entry, and the app shows « extension active » as soon as the state API confirms it

### Requirement: Native analysis shared with the extension
The Safari extension SHALL obtain its analysis over nativeMessaging (event page → `SafariWebExtensionHandler`), the handler running `lingua-core` compiled natively — no WASM SHALL be instantiated in any Safari extension context. Requests SHALL be batched (per viewport), memoised per word form on the content-script side, and a failure of the bridge SHALL degrade gracefully (no highlighting) without blocking the page.

#### Scenario: Analysing a page in Safari
- **WHEN** an English page is loaded in Safari with the extension active
- **THEN** highlighting appears, the analysis having gone through the native handler, and no WASM module was loaded in the extension

### Requirement: Per-device local state, ready for sync
Each device SHALL keep its own local state (statuses, cards, calibration) under the same versioned schema as the Chromium/Firefox extension, seeded by calibration or a LingQ import; no synchronisation SHALL exist in this change, and `lingua-core`'s shared types SHALL guarantee that a later merge (the sync change) is mechanical.

#### Scenario: iPhone and Mac stay independent
- **WHEN** the user marks a word as known on their Mac
- **THEN** their iPhone's state is unchanged (there is no sync in this change), with no error and no schema divergence
