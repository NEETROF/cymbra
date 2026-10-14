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
- **Activation page** — `Shared (App)/Resources/Base.lproj/Main.html` (French in the
  markup), driven by `Shared (App)/ViewController.swift`: iOS shows the steps (no state API
  there); macOS shows the real state (`SFSafariExtensionManager`) and opens Safari's
  settings on the extension (`SFSafariApplication.showPreferencesForExtension`). Its copy
  is one table per language, `Resources/copy.js` (fr byte for byte the markup's, en and es
  drafts), which fills the page on `DOMContentLoaded` in the language `ViewController`
  injects before load: the extension's interface language once the extension has run
  (its native handler keeps it in the App Group, `interface.language`), the device's
  before — among the declared languages only, English once English ships and French
  otherwise when neither is declared. A French page is
  left untouched; `apps/lingua-extension/test/apple-activation-page.spec.ts` pins the
  French and the fill (`openspec/changes/localise-lingua-apple-host`, D2).
- **Languages** — the app declares the native languages of the packs the Safari build
  carries (`dist-safari/assets/packs/<studied>-<native>.lingua`), nothing else: the
  _Declare Lingua languages_ phase (`tool/app_localizations.sh`, last on both app targets,
  after the copy on both extension targets) writes `CFBundleLocalizations` — what the App
  Store's "Languages" line shows — and `CFBundleDevelopmentRegion` (`en` once English
  ships, `fr` otherwise) into the processed plist, only when it holds something else; a
  native is 2–3 lowercase letters, or the build fails. While every shipped native is
  French, the plist is never touched — left as Xcode generated it: the source plist's
  `[fr]` on the apps, none on the extensions, and `en` — the project's development
  language, which Xcode writes over the source's `fr` — as the region. So the Swift side
  counts as declared `CFBundleLocalizations` alone (`SignInLanguage.offered(in:)`), never
  `Bundle.localizations`, which adds that `en` and the `.lproj` folders: a device in
  English keeps a French page and a French sheet, and a link naming English a French
  sheet, until English ships. Tested by `tool/test_app_localizations.sh` and `swift test`
  (a bundle built on disk as Xcode builds it), and `lingua-apple-build` checks every built
  plist and each app's `CFBundleLocalizations` (D1).
- **Minimum OS** — iOS 17.2 / macOS 12: the reader paints with the CSS Custom Highlight
  API, which Safari ships from 17.2.
- **Apple and Google sign-in** (`openspec/changes/add-lingua-connected-clients`, D6) — Safari
  has no `identity.launchWebAuthFlow`, so the extension opens this app on
  `cymbra-lingua://signin?provider=apple|google&lang=fr|en|es` (`CFBundleURLTypes` in both
  apps). The app shows `Shared (App)/SignInView.swift`, runs the provider's native sheet and
  leaves the id_token in the App Group; the extension's `SafariWebExtensionHandler` answers
  `auth.takeIdToken` with it once, within five minutes. The Cymbra session stays in the
  extension. The logic lives in the local package [`LinguaSignIn`](LinguaSignIn), linked by
  all four targets. The sheet's copy (`SignInCopy(language:)`, fr byte for byte, en and es
  drafts) is in the link's `lang` — the extension's interface language — when the app
  declares that language, the device's when declared, English once English ships and
  French otherwise; `lang` is read from a closed list (`SignInLink.language(from:)`), never
  shown or passed on, and an older link naming none is shown the same way without it.
  Apple's button keeps the label the system gives it (D3).

- **The review** (`openspec/changes/add-lingua-apple-review`) — once the extension has shared its
  state, the app opens on the deck it keeps: the extension's own Révision and Statistiques, the `app` build of `apps/lingua-extension`
  (`dist-app/`), copied into the app bundle (`LinguaApp/`) by a _Copy Lingua app page_ phase on
  both app targets. `ViewController` serves it under `lingua-app://app/` (`AppPageResource`: the
  page from the app, the packs from the embedded extension, so they are in the app once) and
  answers its `lingua` script handler (`AppBridgeMessage`). The relay lives in the App Group as
  two files (`ReviewRelayStore`): the extension's state, written by `SafariWebExtensionHandler`
  (`review.saveState`), and the app's changes, written after every answer and taken back by the
  extension (`review.readOps`, `review.clearOps`). Until a state is shared — or when the page cannot
  read it — the app shows the activation page, as before. On a Mac the window becomes resizable
  and grows to a review column.

- **Translation in the app's review** (`openspec/changes/add-lingua-review-translation-fallback`,
  D7–D9; measured in `spike-1.4.md` of that change on a real iPhone 15 Pro Max, the simulators and a
  Mac).
  - *Engine.* The extension's own engine worker and model worker run as classic workers of the page.
    Their files — `engine-worker.js`, `model-worker.js`, `engine/` and `model-manifest.json` — are
    the Safari extension's, served by `AppPageResource` from the embedded `.appex` under
    `lingua-app://app/`, so they are in the app once and `dist-app` carries none. Measured on the
    iPhone: cold load 107–172 ms, first translation 129–136 ms, then 25–30 ms; the web content process
    peaks near 347 MiB and settles near 290 MiB with one model.
  - *Models.* The app keeps its own copy in the page's IndexedDB (`lingua-model`), downloaded by the
    extension's download code from `models.cymbra.app` and checked against the bundled catalogue. It
    follows the extension's « Traduction étendue » on the device (relayed): on, the review offers the
    download once, with its size, and fetches nothing before the reader accepts; off, the app deletes
    its copy at its next opening. WebKit refuses `navigator.storage.persist()` to the custom scheme,
    so the copy is evictable: a pair downloaded once and missing since is offered again. The copy
    survives relaunch and a device restart. **`models.cymbra.app` must keep answering
    `access-control-allow-origin: *`**: the page reads the models cross-origin.
  - *Session.* « Traduction en ligne » in the app uses a Cymbra session of the app's own, never the
    extension's. The first press opens `AppSignInView` (Apple first, then Google) through the page's
    `auth.signIn`; the page exchanges the id_token through `SignInOidc` (`lingua` audience) and keeps
    the access token in memory, the refresh token in the Keychain (`KeychainRefreshTokenStore`, the
    page's `auth.readRefresh` / `auth.writeRefresh` / `auth.clear`, `AppSessionMessage`). A line under
    the views shows the session and ends it, or offers to sign in. *Email accounts*
    (add-lingua-app-email-signin): the sheet's third entry, « Continuer avec un email », closes it and
    answers `{method: "email"}` (`AppSignInChoice.email`); the page then shows the extension's own
    email sign-in (`AccountFlow` + `renderAccount` in their app scope, `src/app/email-panel.ts`) —
    sign-in, the code step of an unverified address, « Mot de passe oublié ? » — over the app's
    session (`SignInLocal`, `lingua` audience). No account creation in the app (a new reader uses Apple,
    Google, or the extension's account page), no handle step. The password is typed in the page and
    sent only to Cymbra ID: it never crosses the bridge and is never stored — only the refresh token,
    in the Keychain. The app does not reload its page on returning to the foreground while the panel
    is open, so a code read in Mail finds its step. The page calls the server directly over gRPC-web: WebKit sends `Origin: lingua-app://app`,
    which production must list in `CYMBRA_ALLOWED_WEB_ORIGINS`.

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
yarn build:app   # the review page the app shows (add-lingua-apple-review)

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
exist before a delivery. Every icon here is rendered from `apps/lingua-extension/assets/icon.svg`
by `tool/gen_icons.sh` — never edit the PNGs by hand. The iOS app icon is the opaque, full-bleed
render (App Store Connect refuses transparency); the macOS sizes keep the rounded mark.

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
| User Content → Other User Content | word statuses and level, each under the studied language it belongs to; deck (word, sentence, gloss and the language of that gloss, review state), each card under its studied language | yes | App Functionality |
| Usage Data → Product Interaction | daily counts (words learned, reviews, words met), per day and studied language, with the native language of the day | yes | App Functionality, Analytics (aggregated back-office figures) |

Not collected: browsing history, search history, location, contacts, purchases, diagnostics,
financial or health data. The language Lingua sends to the Cymbra account (the interface's or
the browser's, kept as the account's language) maps to no Apple data type, so the categories
stay unchanged. Privacy policy: `https://cymbra.app/confidentialite/` (`/en/privacy/`,
`/es/privacidad/`, Annex B). Account deletion: `https://cymbra.app/suppression-compte/`
(`/en/delete-account/`, `/es/eliminar-cuenta/`); the extension's account page links the
French page for a French browser and the English one otherwise, next to « Effacer mes données
Lingua ». `/es/eliminar-cuenta/` is not linked from the extension yet: that link waits for
`add-site-spanish-locale`'s task 3.2.

Reviewed for « Traduction en ligne » (add-lingua-online-translation, 2026-10-11): **no new data
type**. The sentence a reader sends with its button goes to Cymbra and on to the translation
provider, is translated and kept by no one: Apple counts as collected only data accessible for
longer than it takes to service the request in real time, which this is not. The per-account
count of the day's online translations (erased after a day) and the daily totals per provider
and pair (no account) fall under Usage Data → Product Interaction, already declared.

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
