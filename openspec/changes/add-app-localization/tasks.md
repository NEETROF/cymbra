## 1. Dependencies & l10n toolchain

- [ ] 1.1 Add `flutter_localizations` (SDK) and `intl` to `pubspec.yaml`; add `shared_preferences` for the local preferences store.
- [ ] 1.2 Enable `flutter: generate: true` in `pubspec.yaml` and add an `l10n.yaml` (`arb-dir: lib/l10n`, `template-arb-file: app_en.arb`, `output-localization-file: app_localizations.dart`).
- [ ] 1.3 Ensure generated `l10n` output is gitignored (like other generated sources) and excluded from Flutter coverage (`very_good_coverage` config).
- [ ] 1.4 Wire `flutter gen-l10n` into the pre-analyze/test step (melos generate + CI) so `AppLocalizations` always exists before analyze/test.

## 2. ARB resources

- [ ] 2.1 Create `lib/l10n/app_en.arb` as the template with the visible main-screen/settings strings (keys + English values + placeholder/`@` metadata).
- [ ] 2.2 Create `app_fr.arb`, `app_it.arb`, `app_es.arb` mirroring every key from the template with translations.
- [ ] 2.3 Run `flutter gen-l10n`; confirm `AppLocalizations` generates and placeholders validate against the template.

## 3. Local preferences seam

- [ ] 3.1 Add an abstract `PreferencesService` (get/set/remove string) in `lib/services/` with a `SharedPreferencesService` production impl over `shared_preferences`.
- [ ] 3.2 Expose it via `preferencesServiceProvider` (`@riverpod`); add an in-memory `FakePreferencesService` in test support.
- [ ] 3.3 Unit-test the fake and the seam contract: written value round-trips, missing key reports absent, value survives a simulated relaunch (new service over same backing map).

## 4. Language model & locale state

- [ ] 4.1 Add an `AppLanguage` enum (`en`/`fr`/`it`/`es`) with `toLocale()` and a `flag` — a Unicode regional-indicator emoji (🇬🇧 `en`→GB, 🇫🇷 `fr`→FR, 🇮🇹 `it`→IT, 🇪🇸 `es`→ES) — plus a semantic/accessibility label per language; include `supportedLocales` derivation.
- [ ] 4.2 Add a keep-alive `@riverpod` `AppLocale` notifier holding the selected `Locale?`, depending on `preferencesServiceProvider`.
- [ ] 4.3 Implement startup resolution (pure/host-testable): persisted supported value → device locale (if supported) → English; an unsupported persisted value is treated as absent and the fallback re-persisted.
- [ ] 4.4 Implement `select(language)` on the notifier: update state and persist via the preferences seam; support clearing the override back to device-follow (`locale = null`) if the "System default" option is adopted.

## 5. MaterialApp wiring (hot switch)

- [ ] 5.1 Make `CymbraApp` a `ConsumerWidget`; `ref.watch(appLocaleProvider)` and pass it to `MaterialApp.locale`.
- [ ] 5.2 Set `localizationsDelegates` (`AppLocalizations.delegate` + the three `GlobalMaterial/Widgets/CupertinoLocalizations` delegates) and `supportedLocales` (en/fr/it/es) on `MaterialApp`.
- [ ] 5.3 Verify a locale change rebuilds `MaterialApp` and re-resolves `AppLocalizations` live (no restart).

## 6. Settings-drawer language picker

- [ ] 6.1 Add a "Language" section to the settings end-drawer in `player_screen.dart`, following the existing master/detail radio-style pattern (selectable list, not a flyout dropdown — iPad flicker).
- [ ] 6.2 Render each supported language as its flag emoji (with a `Semantics`/tooltip label for accessibility), mark the active one, and call `appLocaleProvider.select(...)` on tap.
- [ ] 6.3 (Optional per design) add a "System default" entry that clears the override.

## 7. String migration (visible slice)

- [ ] 7.1 Replace hardcoded strings on the main player screen (title/subtitle, settings labels: MIDI device / Keyboard size / Hand, view-mode labels, back-to-library, etc.) with `AppLocalizations.of(context)` lookups.
- [ ] 7.2 Replace hardcoded strings on the library screen and practice-level labels used in the visible slice.
- [ ] 7.3 Confirm the `auth/` string bulk is left as tracked follow-up (out of scope here); note it in the change/PR.

## 8. Tests

- [ ] 8.1 Locale resolution/fallback unit tests: device supported/unsupported, persisted-wins-over-device, unsupported persisted falls back and re-persists.
- [ ] 8.2 Persistence round-trip test via `FakePreferencesService`: a selected language is restored on a simulated relaunch.
- [ ] 8.3 Widget test: pumping `CymbraApp` (or a localized harness) with `ProviderScope` overrides shows strings in the resolved language and updates live when the language provider changes (hot switch).
- [ ] 8.4 Widget test: the settings-drawer language picker shows the four language flags (with semantic labels), marks the active one, and switching updates state + persistence.
- [ ] 8.5 Run `flutter test --coverage` and confirm Flutter line coverage stays ≥ 80% (generated `l10n` excluded).

## 9. Pre-PR verification

- [ ] 9.1 `melos run analyze` + `dart format` clean; `dart run custom_lint` passes (riverpod_lint).
- [ ] 9.2 `openspec validate add-app-localization --strict` passes.
- [ ] 9.3 Manually verify on a running app: first launch follows device locale, switching in the drawer updates the UI immediately, and the choice survives a restart.
