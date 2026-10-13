# Tasks

## 1. The app build of the extension's code
- [x] 1.1 `build.mjs`: an `app` target — entry page with Révision and Statistiques, `WasmAnalyzerPort` in the page, a storage adapter over the bridge, no `chrome.*` (D1). Verify with `yarn build` and `check_variants` covering `app`.
- [x] 1.2 The bridge adapter: load the shared state, write the review's ops after every answer (D3). Verify with specs on a fake bridge.

## 2. The extension's side of the relay
- [x] 2.1 The background sends its saved state to the native handler (debounced) and applies the app's pending ops at startup and on a visible page (D3). Verify with specs on a fake native port.
- [x] 2.2 `SafariWebExtensionHandler.swift`: write the state, read and clear the ops, in the App Group. Verify with Swift tests in `LinguaSignIn`'s package style.

## 3. The app
- [x] 3.1 `ViewController.swift`: the scheme handler serving the `app` build and packs, the script bridge to the App Group, the review home; the explanation kept when no state is shared (D2). Verify on the iOS simulator and macOS.
- [x] 3.2 macOS App Group entitlement and provisioning. Verify a macOS build reads the state.

## 4. Integration and dogfood
- [x] 4.1 Gates: the extension's `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`; the Swift tests. Verify all pass.
- [ ] 4.2 Dogfood on an iPhone, an iPad and a Mac: capture in Safari, review in the app, back in Safari the due count follows. Verify by noting it in the pull request.
