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

### Google / Apple sign-in (platform config — TODO before shipping OIDC)

Email/password and guest work against the local backend with no extra config.
The Google/Apple buttons need native credentials wired up first:

- **Google**: register OAuth client IDs (iOS, Android, macOS), add the iOS URL
  scheme / Android intent filter for `google_sign_in`, and set the backend's
  `CYMBRA_GOOGLE_AUDIENCE` to the registered client ID(s).
- **Apple**: enable the "Sign in with Apple" capability and set
  `CYMBRA_APPLE_AUDIENCE`.
- **Local dev**: the compose `mock-oidc` profile stands in for Google/Apple —
  start it with `docker compose --profile oidc up` (see
  `CYMBRA_DEV_OIDC_ISSUER`).

Until those client IDs exist the OIDC buttons will fail with `UNAUTHENTICATED`;
this is tracked as tasks 6.3/6.4 of the `add-music-account-access` change.

## License

Free and **open source** under the [Apache License 2.0](../../LICENSE) — use, modify
and redistribute it, keeping the copyright/license notices and stating your changes
(Apache 2.0 §4).

Apache 2.0 (§6) does **not** grant the **brand**: the name **"Cymbra"** and its logo
are trademarks of **NEETROF** — a fork must ship under its own name and logo. See
[TRADEMARKS.md](../../TRADEMARKS.md).

Contributions welcome under the same terms — see [CONTRIBUTING.md](../../CONTRIBUTING.md).

Copyright 2026 NEETROF.
