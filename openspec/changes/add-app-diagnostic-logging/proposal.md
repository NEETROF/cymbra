## Why

The music app has no logging: 46 `debugPrint` call sites and several `catch (_)` blocks
that swallow the cause. `debugPrint` shows in `flutter run` and **nowhere in a release
build** — the macOS TestFlight Google sign-in failure (2026-08-17) took a full
differential rebuild to diagnose because the app said "Something went wrong" and left no
trace. With a community beta about to start, "it doesn't work" reports will be the norm,
and a tester cannot attach a Flutter console. The engine already writes to the
platform log (`api/platform_log.rs`, Android liblog); the Dart side needs the same
discipline, plus a way for a tester to hand us the last minutes of the app's own
journal — without a third-party crash service (project constraint: no paid / % / data
third parties).

## What Changes

- **One logger for the app**: `package:logging` (Dart team, no transitive deps) — a root
  `cymbra` logger, hierarchical child loggers per area (`cymbra.auth`, `cymbra.plans`,
  `cymbra.audio`, …), levels `FINE` (debug only) / `INFO` / `WARNING` / `SEVERE`; every
  `catch` that survives logs at `WARNING`+ with the exception and stack. `debugPrint` /
  `print` are banned in `lib/` (lint gate).
- **A native sink in release**: a small method channel `app.cymbra.music/log` → `os_log`
  on iOS/macOS (subsystem `app.cymbra.music`, category = logger name), `Log.println` on
  Android (tag = logger name), stderr on Windows/Linux — visible in Console.app /
  `log stream` / logcat on TestFlight and store builds. Debug builds keep the console.
- **A diagnostic journal**: an in-memory ring buffer of the last N lines (default 500,
  `INFO`+), shown on the Help screen ("Diagnostic log") with **Copy** and **Share**, plus
  device / app version header — what a beta tester pastes into Discord.
- **Redaction by construction**: the sink redacts bearer tokens, JWT-shaped strings and
  e-mail addresses before writing anywhere; access codes are never logged in clear (rule
  + test).
- **Not** in scope: remote crash/log shipping, backend correlation ids, the Rust engine's
  own log path (already native; unifying the subsystem is a follow-up).

## Capabilities

### New Capabilities
- `music-app-logging`: the app-side logging contract — one hierarchical logger, levels,
  release sink to the platform log, redaction rules, the ban on ad-hoc prints, and the
  user-visible diagnostic journal (ring buffer, Help screen copy/share).

### Modified Capabilities
- (none — the Help screen gains an entry; no existing requirement changes)

## Impact

- **Cymbra Music (app)** — `lib/services/app_log.dart` (logger + sink + ring buffer +
  redaction), `lib/services/platform_log_channel.dart` (method channel seam, fake in
  tests), native handlers: `ios/Runner/AppDelegate.swift`, `macos/Runner/AppDelegate.swift`
  (or `MainFlutterWindow`), `android/.../MainActivity.kt`; Help screen section;
  migration of every `debugPrint` in `lib/`; `analysis_options.yaml` (`avoid_print`) +
  a CI grep for `debugPrint(` in `lib/`.
- **Tests** — unit: sink levels / redaction / ring buffer; widget: Help screen journal
  copy; the method channel is doubled with a fake.
- **Docs** — `apps/music/README.md` "Diagnostics" (how to read logs on each platform,
  what the journal contains), the `flutter-testing` skill note (fake sink).
- **Privacy** — the journal contains no token, no e-mail, no code; the user chooses to
  share it. Mention in the privacy policy is not required (no automatic transmission).
