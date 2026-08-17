## 1. Logger + sinks (D1, D2, D3)

- [ ] 1.1 `pubspec.yaml`: add `logging`; `lib/services/app_log.dart`: root `Logger('cymbra')`, `logFor(area)`, `installAppLogging({required bool isDebug, PlatformLogChannel channel})` (listener: console in debug, platform channel `INFO`+ in release, ring buffer `INFO`+ always), `AppLogBuffer` (500 lines, ordered), `redact()` (bearer / JWT / e-mail / access-code shapes), `FlutterError.onError` + `PlatformDispatcher.onError` → `SEVERE`
- [ ] 1.2 `lib/services/platform_log_channel.dart`: `PlatformLogChannel` seam (`write(level, category, message)`), `MethodChannel('app.cymbra.music/log')` impl (never throws, pre-ready buffering), `FakePlatformLogChannel` for tests, `platformLogChannelProvider`
- [ ] 1.3 Native handlers: iOS `AppDelegate.swift` + macOS `MainFlutterWindow.swift`/`AppDelegate.swift` (`os_log` subsystem `app.cymbra.music`, category = area, level mapping), Android `MainActivity.kt` (`Log.println`, tag `cymbra.<area>`); Windows/Linux fall back to stderr in Dart
- [ ] 1.4 Unit tests: level routing per mode, redaction fixtures, ring buffer capacity/order, channel failure swallowed
- [ ] 1.5 `main.dart`: `installAppLogging` before `runApp`

## 2. Migration + lint gate (D5)

- [ ] 2.1 Replace every `debugPrint(` in `lib/` with the area logger (auth / plans / offline cache / push / audio / notation / streak / listeners), `catch (_)` that swallow → `WARNING`/`FINE` with cause; keep messages identifier-only
- [ ] 2.2 `analysis_options.yaml`: `avoid_print` on; `.github/workflows/flutter.yml`: fail on `debugPrint(` under `lib/` (allow-list: `app_log.dart`); `flutter analyze` + `custom_lint` clean

## 3. Diagnostic journal (D4)

- [ ] 3.1 Help screen: "Diagnostic log" section — header (version/build, platform/OS, locale, plan kind), buffered lines, **Copy**, **Share** if `share_plus` is available (else clipboard only); l10n en/fr/es/it
- [ ] 3.2 Widget test: journal shown, Copy puts header + lines on the clipboard, no e-mail/user id in the header

## 4. Docs + verification

- [ ] 4.1 `apps/music/README.md` "Diagnostics": how to read the app log per platform (`log stream --predicate 'subsystem == "app.cymbra.music"'`, `adb logcat -s cymbra.*`), what the journal contains, redaction rules; `flutter-testing` skill: fake channel default
- [ ] 4.2 `openspec validate add-app-diagnostic-logging --strict`; `flutter analyze`, `custom_lint`, tests + coverage ≥ 80 %
- [ ] 4.3 Manual: TestFlight iOS + macOS build — trigger a handled failure (Google cancelled / offline), read it in Console.app; Help → Copy journal on device
