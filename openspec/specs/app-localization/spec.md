# app-localization Specification

## Purpose
TBD - created by archiving change add-app-localization. Update Purpose after archive.
## Requirements
### Requirement: Supported Languages

The app SHALL support presenting its user interface in four languages — English
(`en`), French (`fr`), Italian (`it`), and Spanish (`es`) — wired through
`flutter_localizations` with `supportedLocales` and `localizationsDelegates`
declared on the root `MaterialApp`. English SHALL be the ultimate fallback
language when no other supported language applies.

#### Scenario: Supported languages are advertised
- **WHEN** the app is built
- **THEN** the root `MaterialApp` declares `en`, `fr`, `it`, and `es` in
  `supportedLocales` and includes the app's localization delegates

#### Scenario: Localized strings resolve for a supported language
- **WHEN** the active language is one of `en`/`fr`/`it`/`es`
- **THEN** user-facing strings that have been localized render in that language

### Requirement: Default To Device Locale

On first launch (no language has been persisted yet), the app SHALL adopt the
device's locale when that locale's language is one of the supported languages;
otherwise it SHALL fall back to English as the default. English SHALL always be
the fallback whenever the resolved locale is not supported.

#### Scenario: Device language is supported
- **WHEN** no language has been persisted and the device locale is Italian
- **THEN** the app starts in Italian

#### Scenario: Device language is not supported
- **WHEN** no language has been persisted and the device locale is German (not a
  supported language)
- **THEN** the app starts in English

### Requirement: In-App Language Selection

The app SHALL let the user choose the interface language from the main screen,
presented in the existing settings drawer as a selectable list (not a flyout
dropdown). Each supported language SHALL be represented by its flag, rendered as a
Unicode regional-indicator emoji (🇬🇧 `en`, 🇫🇷 `fr`, 🇮🇹 `it`, 🇪🇸 `es`), with the
active language marked. Each flag SHALL carry an accessible/semantic label so the
control is usable by screen readers. The language list SHALL be exposed through
app state so the selection can be driven and observed.

#### Scenario: Language picker lists supported languages as flags
- **WHEN** the user opens the settings drawer's language section
- **THEN** it shows the four supported languages as flags (🇬🇧 English, 🇫🇷 French,
  🇮🇹 Italian, 🇪🇸 Spanish) with the active language marked

#### Scenario: User selects a language
- **WHEN** the user taps a language other than the active one
- **THEN** that language becomes the active language

### Requirement: Hot Language Switching

Changing the language SHALL take effect immediately, re-rendering the visible UI
in the newly selected language without restarting the app, by driving
`MaterialApp.locale` from app state.

#### Scenario: UI updates without restart
- **WHEN** the user selects a different language in the drawer
- **THEN** the already-visible screen re-renders its localized strings in the new
  language without an app restart

### Requirement: Persisted Language Choice

A language chosen by the user SHALL be persisted locally and restored on the next
launch, taking precedence over the device locale. If the persisted language is no
longer supported, the app SHALL fall back to the device locale (or English) and
persist that fallback. The language selection SHALL be held in an injectable
provider backed by the local preferences store so tests can drive it with fakes.

#### Scenario: Language survives a restart
- **WHEN** the user selects Spanish and later relaunches the app
- **THEN** the app starts in Spanish regardless of the device locale

#### Scenario: Persisted language chosen over device locale
- **WHEN** a supported language has been persisted and differs from the device
  locale
- **THEN** the persisted language is used at startup

#### Scenario: Unsupported persisted language falls back
- **WHEN** the persisted language is no longer among the supported languages
- **THEN** the app resolves to the device locale (or English) and persists that
  as the new choice

