# Cymbra Music — Interactive Piano POC

POC validating the **Flutter (UI / CustomPainter) ↔ flutter_rust_bridge ↔ Rust (low-level MIDI engine)** architecture.

Scope: piano, real-time USB-MIDI input, dual rendering (standard staff / Synthesia waterfall), basic Wait Mode. Out of scope: LEDs, server, dashboard, WebRTC, audio.

## Architecture

```
rust/src/api/
  score.rs   # Note/Measure/Score structs + demo_score()
  midi.rs    # midir listening, NoteOn/NoteOff stream to Flutter
lib/
  theme/cymbra_theme.dart        # "Sonic Luminescence" palette
  state/player_state.dart       # activeNotes, score, mode, wait-mode, elapsedMs
  painters/
    piano_layout.dart           # shared X-axis keyboard ↔ waterfall
    piano_keyboard_painter.dart # keyboard (active keys highlighted)
    synthesia_painter.dart      # waterfall cascade (Ticker)
    staff_painter.dart          # staff + bar lines + notes
  screens/player_screen.dart    # UI, transport, keyboard capture, Ticker
  src/rust/                     # GENERATED Dart bindings (do not edit)
```

The `Note`/`Measure`/`Score`/`MidiEvent` types are defined **once in Rust**; the Dart classes are generated.

## Run

```bash
# Desktop (primary POC target)
flutter run -d macos

# Regenerate the bridge after any Rust API change
flutter_rust_bridge_codegen generate
```

## Usage

- **Mode toggle** (top bar): switch Synthesia ⇄ Staff.
- **Play/Pause**: starts the cascade. **Wait**: freezes the cascade until the expected note is held.
- **Without a MIDI keyboard**: computer-keyboard fallback (piano-style row):
  `a w s e d f t g y h u j k o l` → `C4 C#4 D4 D#4 E4 F4 F#4 G4 G#4 A4 A#4 B4 C5 C#5 D5`.
- **With a USB MIDI keyboard**: plug it in at any time (before or after launch).
  A Rust thread watches the ports and connects automatically to the first one detected;
  the **indicator in the top-right** shows the state:
  - 🟢 green + device name = connected;
  - 🟦 amber = detected, connecting;
  - ⚪ gray = no device.
  Hot plug/unplug is handled (auto-reconnect). Click the indicator to pick a specific device.

## Mobile targets

Real MIDI on **all** targets via `midir 0.11`:
- **iOS**: CoreMIDI (frameworks linked in `rust_builder/ios/rust_lib_music.podspec`).
- **Android**: AMidi via NDK (**minSdk 29**, set in `android/app/build.gradle.kts`).
  The `JavaVM` is provided at runtime by `JNI_OnLoad` ([rust/src/lib.rs](rust/src/lib.rs)),
  which initializes `ndk_context` — that's how midir's AMidi backend (via
  `jni-min-helper`) finds the Android context. Deps: `jni`, `ndk-context`.
  **Important**: `JNI_OnLoad` is only called if the lib is loaded by the JVM. frb
  loads it via `dlopen` (which does not trigger `JNI_OnLoad`), so a
  `System.loadLibrary("rust_lib_music")` is required in `MainActivity` ([MainActivity.kt](android/app/src/main/kotlin/org/cymbra/music/MainActivity.kt)).
  Without it: `PanicException(android context was not initialized)`.

To test on Android: connect the USB MIDI keyboard via a **USB-OTG adapter**;
the indicator turns green with the keyboard name.

## Hot-plug detection (macOS / iOS)

On CoreMIDI, plug/unplug notifications are only delivered to the **main run loop**.
A process that merely re-enumerates from a background thread (like our watcher) will
therefore **not** see devices connected after startup. Fix: a small CoreMIDI
"refresher" client created on the main thread in `AppDelegate` (macOS [AppDelegate.swift](macos/Runner/AppDelegate.swift),
iOS [AppDelegate.swift](ios/Runner/AppDelegate.swift)) — empty notification block,
its mere presence keeps the process's MIDI view up to date. The Rust enumeration
then sees hot-plug changes. (Linux/Windows re-enumerate natively, nothing to do.)

## Running on device (reminders)

- **iOS debug**: launch from Xcode (CLI JIT debug fails on recent iOS); for hot
  reload: Run in Xcode then `flutter attach -d <id>`.
- **iOS standalone** (home screen): `flutter run --release -d <id>`.
- **Android**: `flutter run -d <id>` or `flutter build apk --debug`.

## Build commands after changes

| Change | Action |
|---|---|
| Dart | `r` / `R` (hot reload/restart) in `flutter run` |
| Rust internals | restart `flutter run` (cargokit rebuilds the lib) |
| Rust API | `flutter_rust_bridge_codegen generate` then `flutter run` |
| Native (Swift/Kotlin/gradle/podspec) | restart `flutter run` (+ `pod install` if podspec) |

On this Mac, `pod install` requires a UTF-8 locale: `LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 pod install`.

## Cymbra ID account layer (dev setup)

The app talks to the **Cymbra ID** backend (`backend/`) over native gRPC for
sign-in, sessions, and the unique handle. Everything sits behind injectable
Riverpod seams (`lib/services/auth_service.dart`, `account_service.dart`,
`token_store.dart`, `oidc_token_source.dart`), so unit/widget tests run with
fakes and never touch a channel or platform plugin.

### gRPC stub codegen

The Dart client stubs are generated from the backend protos into
`lib/src/grpc/` (gitignored, like `lib/src/rust/`) and excluded from analysis +
coverage. Regenerate after a proto change:

```bash
melos run gen-grpc          # wraps apps/music/tool/gen_grpc.sh
```

Requires `protoc` on PATH (`brew install protobuf` / `apt install
protobuf-compiler`); the script installs the pinned Dart plugin
(`protoc_plugin 22.5.0`, matching the `protobuf 4.x` runtime — a newer plugin
emits code for an incompatible runtime). CI runs this before analyze/test.

### Backend endpoint

The gRPC endpoint defaults to plaintext `localhost:50051` (dev). Override the
`cymbraEndpointProvider` for staging/production (TLS). Bring the backend up with
`backend/docker-compose.yml` (`CYMBRA_GRPC_ADDR=0.0.0.0:50051`).

`localhost` works for desktop builds and simulators. To test on a **physical
device** against a backend on your dev machine, point the host at the machine's
LAN IP at build time:

```bash
flutter run -d <device> \
  --dart-define=GOOGLE_CLIENT_ID=<id>.apps.googleusercontent.com \
  --dart-define=CYMBRA_GRPC_HOST=<dev-machine-ip>   # e.g. 192.168.1.32; CYMBRA_GRPC_PORT defaults to 50051
```

Find the IP with `ipconfig getifaddr en0` (macOS Wi-Fi). Requirements:

- device and dev machine on the **same Wi-Fi/LAN**;
- the backend already listens on `0.0.0.0:50051` (reachable on the LAN);
- **iOS** prompts for *Local Network* access on first connect → **Allow** (the
  `NSLocalNetworkUsageDescription` key is set; toggle later under Settings →
  Privacy → Local Network);
- if the connection **times out**, allow incoming connections to `cymbra-server` in
  the **macOS firewall** (or disable it for dev). Plaintext gRPC over the LAN is
  fine for dev — it's a raw socket, so iOS ATS does not apply.

Changing a `--dart-define` requires a full rebuild (not hot reload).

### Google / Apple sign-in (platform config — tasks 6.3/6.4)

Email/password and guest work against the local backend with **no extra config**.
The Google/Apple buttons are **hidden until configured**, so an unconfigured build
never invokes the native SDK (which would crash with `GIDClientID is set in
Info.plist`). Configuration is supplied at build time with `--dart-define`:

```bash
flutter run -d macos \
  --dart-define=GOOGLE_CLIENT_ID=<id>.apps.googleusercontent.com \
  --dart-define=APPLE_SIGN_IN_ENABLED=true
```

**Google** — once you have an OAuth client ID (the reversed client ID and the
client ID are **never committed**; supply them at build time):
1. Pass it via `--dart-define=GOOGLE_CLIENT_ID=...` (no `GIDClientID` Info.plist
   entry needed — it's passed in Dart).
2. **macOS**: the callback URL scheme `com.googleusercontent.apps.$(GOOGLE_OAUTH_CLIENT_SUFFIX)`
   in `macos/Runner/Info.plist` is resolved from a build setting. Copy
   `macos/Runner/Configs/Secrets.example.xcconfig` → `Secrets.xcconfig` (gitignored)
   and set `GOOGLE_OAUTH_CLIENT_SUFFIX` to the part of your client ID before
   `.apps.googleusercontent.com`. In CI the `macos` release job writes this file
   from the `GOOGLE_CLIENT_ID` secret. (iOS still uses the literal placeholder in
   `ios/Runner/Info.plist`; wire it the same way when iOS builds land. Android uses
   a `serverClientId`, not a URL scheme.)
3. Set the backend's `CYMBRA_GOOGLE_AUDIENCE` to the same (full) client ID.

**Google on desktop (Windows/Linux)** — the native `google_sign_in` plugin has no
Windows/Linux implementation, so those platforms use a browser-loopback OAuth flow
(RFC 8252: authorization code + PKCE, redirect captured on a local `127.0.0.1`
port). It's hidden until configured:

```bash
flutter run -d windows \
  --dart-define=DESKTOP_GOOGLE_CLIENT_ID=<client>.apps.googleusercontent.com
# add --dart-define=DESKTOP_GOOGLE_CLIENT_SECRET=<secret> only for a Web client
```

1. In Google Cloud, pick the OAuth client for the flow and register the loopback
   redirect URIs `http://127.0.0.1` and `http://localhost` on it.
2. **Client type (design D3):** a **Desktop app** client is Google's supported
   loopback client. Note Google **still requires its `client_secret` at the token
   exchange** even with PKCE, so pass `DESKTOP_GOOGLE_CLIENT_SECRET` (the desktop
   secret is not treated as confidential). Its `aud` is the desktop client id, so
   **add that id to the backend's `CYMBRA_GOOGLE_AUDIENCE`** (now comma-separated,
   accepts multiple):
   `CYMBRA_GOOGLE_AUDIENCE=<web-client>...,<desktop-client>...`.
   (A **Web** client keeps a single audience but ships the more-sensitive web
   secret and doesn't support dynamic-port loopback cleanly — not recommended.)
3. No URL-scheme / Info.plist entries are needed — desktop uses only the dart-defines.

**Apple** — enable `--dart-define=APPLE_SIGN_IN_ENABLED=true`, add the
**"Sign in with Apple"** capability in Xcode (writes the
`com.apple.developer.applesignin` entitlement — this **requires a development
certificate**, like all capability entitlements), and set `CYMBRA_APPLE_AUDIENCE`.

**Local dev**: the compose `mock-oidc` profile stands in for Google/Apple — start
it with `docker compose --profile oidc up` (see `CYMBRA_DEV_OIDC_ISSUER`).

**Production build**: point the app at the TLS-fronted backend (Caddy terminates
HTTPS on 443) with dart-defines:

```bash
flutter build ... \
  --dart-define=CYMBRA_GRPC_HOST=api.<your-domain> \
  --dart-define=CYMBRA_GRPC_PORT=443 \
  --dart-define=CYMBRA_GRPC_SECURE=true
```

## Release builds (mobile)

### Config via `--dart-define-from-file`

All build-time config (OIDC client IDs + gRPC endpoint) lives in per-environment
JSON under [`config/`](config/) — one source of truth instead of scattered flags.
The values are **public** (OAuth client IDs and the API host are embedded in the
shipped binary anyway), so both files are committed:

- [`config/dev.json`](config/dev.json) — plaintext `localhost:50051`;
- [`config/prod.json`](config/prod.json) — `api.cymbra.app:443` over TLS.

```bash
flutter build appbundle --release --dart-define-from-file=config/prod.json   # Android (Play)
flutter build ipa       --release --dart-define-from-file=config/prod.json \
  --export-options-plist ios/ExportOptions.plist                             # iOS (TestFlight)
```

For a physical device on the LAN, copy `config/dev.json` and set
`CYMBRA_GRPC_HOST` to the dev machine's IP (see above).

### Android signing (Play App Signing)

Release builds use an **upload key**; Google holds the final app-signing key
(Play App Signing). Without `android/key.properties` the release build falls back
to the shared debug key, so this stays optional for local/CI smoke builds.

```bash
keytool -genkeypair -v \
  -keystore apps/music/android/app/upload-keystore.jks \
  -keyalg RSA -keysize 2048 -validity 10000 -alias upload
cp apps/music/android/app/key.properties.example apps/music/android/key.properties
# then edit key.properties with the store/key passwords you just chose
```

`key.properties`, `*.jks` and `*.keystore` are gitignored. For Google sign-in to
work on Play builds, register **both** SHA-1s in the Android OAuth client: the
upload key's, and the Play **app-signing** key's (from Play Console → App
integrity, after the first upload).

**CI** (`.github/workflows/release-build.yml`, `android` job) rebuilds
`key.properties` from repo secrets and signs the AAB + APK:
`ANDROID_KEYSTORE_BASE64` (`base64 -i upload-keystore.jks`),
`ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`.

### iOS signing

`ios/ExportOptions.plist` targets `app-store` with automatic signing under team
`VMFJ6KRW77`. A local archive needs an Apple Distribution cert + an App Store
provisioning profile for `com.cymbra.music` (Xcode → Signing & Capabilities, be
signed in to the Apple Developer account). Sign in with Apple is already in
`Runner.entitlements`.

**CI** (`ios` job) imports the cert + profile into a throwaway keychain and
exports with **manual** signing (a generated `ExportOptions-ci.plist`). Secrets:
`IOS_DIST_CERT_BASE64` (`base64 -i dist.p12`), `IOS_DIST_CERT_PASSWORD`,
`IOS_PROVISIONING_PROFILE_BASE64`, `IOS_PROVISIONING_PROFILE_NAME`, `IOS_TEAM_ID`,
and `GOOGLE_CLIENT_ID` (reversed-client-id URL scheme). The first tagged run may
need a tweak to the profile name — iOS signing is environment-sensitive.

**TestFlight upload** is automatic on a real release (tag push); manual
`workflow_dispatch` runs build + sign only. It uses an App Store Connect API key
(`altool`), skipped cleanly if its secrets are absent. Create the key in App
Store Connect → Users and Access → Integrations → App Store Connect API (role
*App Manager*), then set: `ASC_API_KEY_ID`, `ASC_API_ISSUER_ID`, and
`ASC_API_KEY_P8` (`base64 -i AuthKey_XXXX.p8`). `ITSAppUsesNonExemptEncryption`
is `false` in `Info.plist` (TLS-only), so no export-compliance prompt.

## License

Free and **open source** under the [Apache License 2.0](../../LICENSE) — use, modify
and redistribute it, keeping the copyright/license notices and stating your changes
(Apache 2.0 §4).

Apache 2.0 (§6) does **not** grant the **brand**: the name **"Cymbra"** and its logo
are trademarks of **NEETROF** — a fork must ship under its own name and logo. See
[TRADEMARKS.md](../../TRADEMARKS.md).

Contributions welcome under the same terms — see [CONTRIBUTING.md](../../CONTRIBUTING.md).

Copyright 2026 NEETROF.
