## Why

The Cymbra music app ships English-only: every user-facing string is a hardcoded
literal and `MaterialApp` sets no `locale`, `localizationsDelegates`, or
`supportedLocales`. Cymbra targets learners across Europe, so the app should speak
the user's language — starting from their device locale and letting them switch on
demand — to feel native rather than "an English app I have to tolerate."

## What Changes

- Add **in-app localization for four languages**: English (`en`), French (`fr`),
  Italian (`it`), Spanish (`es`), via Flutter's standard `flutter_localizations` +
  `intl`/ARB codegen (`gen_l10n`), wiring `localizationsDelegates` and
  `supportedLocales` on the root `MaterialApp`.
- **Default to the user's device locale** on first launch: if the device locale
  is one of the four supported languages, use it; otherwise fall back to English.
- Add a **language picker on the main screen** (in the existing settings
  end-drawer) so the user can override the language at any time.
- **Apply the chosen language hot** — the UI re-renders in the new language
  immediately, with no app restart — by driving `MaterialApp.locale` from a
  Riverpod provider.
- **Persist the language choice locally** and restore it on the next launch; a
  stored language that is no longer supported falls back to the device
  locale/English.
- **Introduce a local key-value preferences seam** (the app has no settings
  persistence today — only `flutter_secure_storage` for auth tokens), exposed as
  an injectable Riverpod provider so state/widgets stay testable with a fake.
- **Externalize existing hardcoded UI strings** into ARB resources for the four
  languages (incrementally; the localization plumbing does not require every
  string to move at once, but the visible main-screen/settings strings are
  translated as part of this change).

## Capabilities

### New Capabilities
- `app-localization`: present the app in one of several supported languages
  (`en`/`fr`/`it`/`es`), defaulting to the device locale, switchable at runtime
  from the main screen with an immediate (hot) UI update, and persisted locally
  across launches — with graceful fallback to a supported language.
- `local-preferences`: a local, injectable key-value store for small user
  preferences (starting with the selected language), persisted on-device and
  restored on launch, behind a testable provider seam.

### Modified Capabilities
<!-- No existing capability's requirements change; this adds new capabilities and
     follows state-management (Riverpod 2 + Freezed codegen) without altering it. -->

## Impact

- **`pubspec.yaml`**: add `flutter_localizations` (SDK) and `intl`; enable
  `flutter: generate: true`; add an `l10n.yaml` (ARB dir, template, output). Add a
  small persistence dependency for the preferences seam (`shared_preferences`) —
  none exists today.
- **`lib/main.dart` (`CymbraApp`)**: set `localizationsDelegates`,
  `supportedLocales`, and drive `locale` from a new `localeProvider`; the app is
  wrapped so the locale notifier can restore the persisted value before/at first
  frame.
- **New `l10n/` ARB files**: `app_en.arb` (template) + `app_fr.arb`,
  `app_it.arb`, `app_es.arb`; generated `AppLocalizations` (gitignored generated
  output, produced by `flutter gen-l10n`/build).
- **New state** (`lib/state/`): a `@riverpod` locale notifier holding the selected
  language, backed by the preferences seam, restored at startup; a Freezed/enum
  model for the supported languages.
- **New service seam** (`lib/services/`): an injectable `PreferencesService`
  (abstract + real `shared_preferences` impl + fake) exposed via a provider —
  reused for future settings (e.g. composes with `piano-sound-selection`'s
  persisted selection).
- **UI**: a language picker row in the settings end-drawer (a selectable
  list/radio layout, not a flyout dropdown — per the iPad-flicker note); the
  visible main-screen and settings strings switch to `AppLocalizations`.
- **CI/coverage**: `flutter gen-l10n` runs before analyze/test (alongside
  `build_runner`); generated `l10n` output is excluded from coverage like other
  generated sources; state/widgets stay ≥80% testable via the fake
  `PreferencesService` and `ProviderScope` overrides (no native lib needed).
- **No Rust/engine changes** — this is a Flutter-only change.
