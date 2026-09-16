# Cymbra Lingua — Apple host app (Safari extension, iOS + macOS)

Apple only distributes Safari extensions inside an app. This is that app, and nothing
more: it hosts the `safari` build of [`apps/lingua-extension`](../lingua-extension) and
guides its activation. Reading, statuses, decks, review and sign-in all live in the
extension (see `openspec/changes/add-lingua-apple`).

- **One Xcode project, one universal listing** — iOS and macOS app targets, each
  embedding its Safari web extension target. Bundle ids `com.cymbra.lingua` and
  `com.cymbra.lingua.Extension`.
- **The extension is not copied into this project.** A _Copy Lingua extension_ build
  phase on both extension targets copies `apps/lingua-extension/dist-safari/` into the
  `.appex` at build time, so the bundled extension is always the current build and a new
  file in the extension is never silently left out.
- **Activation page** — `Shared (App)/Resources/Base.lproj/Main.html` (French copy),
  driven by `Shared (App)/ViewController.swift`: iOS shows the steps (no state API
  there); macOS shows the real state (`SFSafariExtensionManager`) and opens Safari's
  settings on the extension (`SFSafariApplication.showPreferencesForExtension`).
- **Minimum OS** — iOS 17.2 / macOS 12: the reader paints with the CSS Custom Highlight
  API, which Safari ships from 17.2.
- **Apple and Google sign-in** (`openspec/changes/add-lingua-connected-clients`, D6) — Safari
  has no `identity.launchWebAuthFlow`, so the extension opens this app on
  `cymbra-lingua://signin?provider=apple|google` (`CFBundleURLTypes` in both apps). The app
  shows `Shared (App)/SignInView.swift`, runs the provider's native sheet and leaves the
  id_token in the App Group; the extension's `SafariWebExtensionHandler` answers
  `auth.takeIdToken` with it once, within five minutes. The Cymbra session stays in the
  extension. The logic lives in the local package [`LinguaSignIn`](LinguaSignIn), linked by
  all four targets.

The project was scaffolded by `xcrun safari-web-extension-converter` and then changed by
hand to replace its copied resources with the build phase above.

## Build

```bash
# 1. The extension (see apps/lingua-extension/README.md for gen:wasm / gen:proto / gen:pack).
#    The bundle is copied as built: for a device or a release, point it at production,
#    otherwise it calls http://localhost:50051 and every sign-in fails.
cd apps/lingua-extension && LINGUA_GRPC_WEB_URL=https://api.cymbra.app yarn build:safari

# 2. The app — Xcode, or from the command line:
cd ../lingua-apple
xcodebuild -project "Cymbra Lingua.xcodeproj" -scheme "Cymbra Lingua (iOS)" \
  -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
xcodebuild -project "Cymbra Lingua.xcodeproj" -scheme "Cymbra Lingua (macOS)" CODE_SIGNING_ALLOWED=NO build
```

A build without step 1 fails with an explicit message from the copy phase.

`LINGUA_GOOGLE_CLIENT_ID` (project build setting) is the Google OAuth **iOS** client of
`com.cymbra.lingua`; it must also be listed in the backend's `CYMBRA_GOOGLE_AUDIENCE`, and
`com.cymbra.lingua` in `CYMBRA_APPLE_AUDIENCE`. An empty value hides Google in the app and
in the Safari extension.

## Run

- **iOS simulator / iPhone** — run the `Cymbra Lingua (iOS)` scheme, then Settings → Apps
  → Safari → Extensions → Cymbra Lingua, and allow websites. On a device, the team
  (`DEVELOPMENT_TEAM`, NEETROF) signs automatically once an Apple account is added in
  Xcode → Settings → Accounts.
- **macOS** — run the `Cymbra Lingua (macOS)` scheme. An unsigned or locally signed build
  needs Safari → Settings → Developer → _Allow unsigned extensions_.

Tabs opened before the extension was enabled or updated need a reload to be highlighted.

## Known issue — popup title on iOS 27

On iOS 27 in light mode, Safari draws the extension popup sheet's native title
(« Cymbra Lingua ») in black over the dark popup; iOS 26.5 draws it white. The title is
Safari's own chrome: a `theme-color` meta, an empty `<title>`, a canvas following the
system `color-scheme` and an empty `action.default_title` were all tried on a device and
changed nothing. It needs a Safari fix (Apple Feedback).
