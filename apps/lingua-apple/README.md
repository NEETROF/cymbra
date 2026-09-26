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
  file in the extension is never silently left out. It does not delete: a file that left
  `dist-safari` stays in the `.appex` of an incremental build. After switching a local build
  between variants, delete its DerivedData; a release builds clean.
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
#    It carries the pinned translation engine (add-lingua-translation-safari): fetch it once.
#    The bundle is copied as built: for a device or a release, point it at production,
#    otherwise it calls http://localhost:50051 and every sign-in fails.
cd apps/lingua-extension && yarn fetch:engine
LINGUA_GRPC_WEB_URL=https://api.cymbra.app yarn build:safari

# 2. The app — Xcode, or from the command line:
cd ../lingua-apple
xcodebuild -project "Cymbra Lingua.xcodeproj" -scheme "Cymbra Lingua (iOS)" \
  -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
xcodebuild -project "Cymbra Lingua.xcodeproj" -scheme "Cymbra Lingua (macOS)" CODE_SIGNING_ALLOWED=NO build
```

A build without step 1 fails with an explicit message from the copy phase, and so does a
device build or an archive whose bundle still calls `localhost` (simulator and Mac builds
may keep a local backend).

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

## Release (App Store / TestFlight)

`lingua-apple-release` builds the production extension — real EN→FR pack,
`https://api.cymbra.app` — archives both platforms, signs them for the App Store and keeps the
`.ipa` and `.pkg` as workflow artifacts. It runs two ways:

- **On a `lingua-apple-v*` tag**, pushed when release-please's "Release PR" is merged: it
  builds that tag and **delivers** both packages. That is what a release is.
- **On a dispatch**, from the branch you dispatch from: it builds and signs, and uploads
  nothing unless you tick **deliver**. This is how a source change is validated before it is
  tagged.

Build numbers are `run×10` (iOS) and `run×10+1` (macOS): the two platforms share one App Store
Connect record, whose build numbers must increase across both.

**The shipped version is `version.txt`**, which release-please maintains, stamped onto both
archives as `MARKETING_VERSION` beside the build number. `MARKETING_VERSION` in
`project.pbxproj` is never what ships — CI already rewrites that file for signing, and a
second CI-owned edit there is a conflict waiting for the next signing change. On a tag,
`version.txt` and the tag must agree or the run stops before building
(`tool/release_version.py`, tested by `test_release_version.py`).

The TestFlight builds of the dogfooding pass were delivered under version `1.0`, when the
number was hand-written in the project. The first tagged release creates `1.0.0`.

Signing is switched to manual in CI only (`tool/ci_release_signing.py install`), and every
export is checked (`… verify`): Apple Distribution, a profile on each bundle, the App Group,
the App Sandbox on macOS and Sign in with Apple on the app. The certificates and the App Store
Connect key are the ones `music-release` uses; Lingua adds four **manually created** profiles
(Xcode-managed « Team Store » profiles are refused by manual signing), each with the Apple
Distribution certificate:

| Secret | Profile |
|---|---|
| `LINGUA_IOS_APP_PROFILE_BASE64` | App Store Connect, iOS, `com.cymbra.lingua` |
| `LINGUA_IOS_EXTENSION_PROFILE_BASE64` | App Store Connect, iOS, `com.cymbra.lingua.Extension` |
| `LINGUA_MAC_APP_PROFILE_BASE64` | Mac App Store Connect, `com.cymbra.lingua` |
| `LINGUA_MAC_EXTENSION_PROFILE_BASE64` | Mac App Store Connect, `com.cymbra.lingua.Extension` |

```bash
base64 -i profile.mobileprovision | gh secret set LINGUA_IOS_APP_PROFILE_BASE64
```

The App Store Connect record « Cymbra Lingua » (iOS + macOS, bundle `com.cymbra.lingua`) must
exist before a delivery. The iOS app icon is the opaque, full-bleed render of
`tool/gen_icons.sh` (App Store Connect refuses transparency); the macOS sizes keep the rounded
mark.

## App Store privacy (Confidentialité de l'app)

The answers for App Store Connect, from what Lingua actually sends
(`openspec/changes/add-lingua-privacy-controls`). Page text, reading exposures and a card's
page address never leave the device, so **Browsing History is not declared**. Nothing is
used for tracking.

| Apple data type | What it is in Lingua | Linked to the user | Purposes |
|---|---|---|---|
| Contact Info → Email Address | the Cymbra account's e-mail (sign-up, verification) | yes | App Functionality |
| Identifiers → User ID | the Cymbra account and its handle | yes | App Functionality, Analytics |
| Identifiers → Device ID | the random installation id used by sync | yes | App Functionality |
| User Content → Other User Content | word statuses, level, deck (word, sentence, gloss, review state) | yes | App Functionality |
| Usage Data → Product Interaction | daily counts (words learned, reviews, words met) | yes | App Functionality, Analytics (aggregated back-office figures) |

Not collected: browsing history, search history, location, contacts, purchases, diagnostics,
financial or health data. Privacy policy: `https://cymbra.app/confidentialite/`
(`/en/privacy/`, Annex B). Account deletion: `https://cymbra.app/suppression-compte/`,
linked from the extension's account page, next to « Effacer mes données Lingua ».

## Known issue — popup title on iOS 27

On iOS 27 in light mode, Safari draws the extension popup sheet's native title
(« Cymbra Lingua ») in black over the dark popup; iOS 26.5 draws it white. The title is
Safari's own chrome: a `theme-color` meta, an empty `<title>`, a canvas following the
system `color-scheme` and an empty `action.default_title` were all tried on a device and
changed nothing. It needs a Safari fix (Apple Feedback).

## Known issue — Google on macOS with Chrome as the default browser

macOS runs the Google sign-in in the default browser. After a Chrome session it could not
match (`SafariLaunchAgent`: « Received response for unrecognized request »), the system
queued every later attempt without opening anything until Chrome was quit completely. The
sheet now keeps « Annuler » active and says so; quitting Chrome and retrying works.
