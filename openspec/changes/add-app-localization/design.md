## Context

The app is English-only. `MaterialApp` (`CymbraApp` in [main.dart](apps/music/lib/main.dart:73))
sets only `title`, `theme`, and `home` — no `locale`, `localizationsDelegates`,
or `supportedLocales`. There is no `flutter_localizations`/`intl` dependency, no
`l10n.yaml`, and no `.arb` files. Every visible string is a hardcoded literal
(~200+ across `player_screen.dart`, `library_screen.dart`, the `auth/` screens,
and `auth_messages.dart`).

There is also **no local settings persistence**: `flutter_secure_storage` exists
but is reserved for auth tokens (`TokenStore`), and player settings
(`keyboardRange`, `selectedHands`, metronome) live only in in-memory Riverpod
state and reset on restart. So this change introduces both the localization
plumbing *and* the first general local-preferences seam — the latter deliberately
generic so future settings (e.g. `piano-sound-selection`) reuse it.

State follows the mandated pattern (Riverpod 2 + Freezed, codegen; dependencies as
overridable providers — see [player_notifier.dart](apps/music/lib/state/player_notifier.dart)).
The settings UI is a right end-drawer with a master/detail, radio-style list
(dropdowns flicker on iPad — see the `player-settings-drawer` note), which the
language picker composes into.

## Goals / Non-Goals

**Goals:**
- Four UI languages (`en`/`fr`/`it`/`es`) via Flutter's standard `gen_l10n`/ARB.
- Default to the device locale when supported; English otherwise.
- A language picker on the main screen (settings drawer), switching **hot** (no
  restart) by driving `MaterialApp.locale` from a provider.
- Persist the choice locally; restore on launch; graceful fallback for an
  unsupported persisted value.
- A generic, injectable local-preferences seam, testable with a fake, reusable by
  future settings.
- Keep state/widgets ≥80% testable with fakes; no native lib needed for tests.

**Non-Goals:**
- **Not** translating 100% of strings in this change. The plumbing lands and the
  visible main-screen/settings strings are localized; the large `auth/` surface
  can be migrated incrementally afterward (the framework makes it mechanical).
- No RTL languages, no locale-specific number/date/currency formatting beyond what
  `intl` gives for free, no per-score/content translation (scores are data).
- No cloud sync of the preference across devices (local only).
- No migration of existing ephemeral player settings to persistence here (that is
  its own change; this only adds the seam they *could* later use).
- No Rust/engine changes.

## Decisions

### D1: Flutter `gen_l10n` (ARB) rather than a third-party i18n package
Use the first-party `flutter gen_l10n` toolchain: `flutter_localizations` (SDK) +
`intl`, `generate: true` in `pubspec.yaml`, an `l10n.yaml`, and `app_en.arb`
(template) + `app_fr/it/es.arb`. Generated `AppLocalizations` is accessed via
`AppLocalizations.of(context)`.

*Why:* zero extra runtime deps, IDE/CI support, ICU plurals/placeholders built in,
and it is the idiomatic Flutter answer the team already uses codegen for
(`build_runner`). *Alternatives:* `easy_localization`/`slang` (runtime key
lookup, but adds a dependency and diverges from the codegen convention);
hand-rolled `Map` lookups (loses plural/placeholder handling and tooling). ARB
wins on being standard and generated like the rest of the app.

### D2: `MaterialApp.locale` driven by a `@riverpod` locale notifier → hot switch
Add a keep-alive `@riverpod` notifier (e.g. `AppLocale`) whose state is the
selected `Locale?` (null = "follow device/default"). `CymbraApp` becomes a
`ConsumerWidget` that `ref.watch`es it and passes it to `MaterialApp.locale`.
Because the locale is provider state, selecting a new language rebuilds
`MaterialApp` and Flutter re-resolves every `AppLocalizations.of(context)` — an
immediate, restart-free switch.

*Why:* this is exactly how Flutter is designed to localize reactively, and it fits
the Riverpod mandate. *Alternative:* an `InheritedWidget`/`ValueNotifier` at the
root — rejected as it reintroduces non-Riverpod app state (against
`state-management`).

### D3: Startup resolution order — persisted → device → English
On startup the locale notifier reads the persisted language via the preferences
seam. Resolution: **persisted supported value** wins; else the **device locale** if
its language is supported (resolved against `supportedLocales`); else **English**.
An unsupported persisted value is treated as absent and the fallback is
re-persisted, so state self-heals.

*Why:* matches the proposal (user override beats device default) and keeps a
single source of truth. Restoring is async (preferences read); the notifier starts
from a synchronous best guess (device/English) and updates once the stored value
loads, avoiding a blocking gate at launch. `MaterialApp.locale = null` already
means "use the platform locale," so the pre-restore frame is correct-by-default,
not a flash of the wrong language for the common case.

### D4: Generic `PreferencesService` seam over `shared_preferences`
Introduce an abstract `PreferencesService` (get/set string, remove) with a
`SharedPreferencesService` production impl and an in-memory fake, exposed via
`preferencesServiceProvider`. Add `shared_preferences` (not
`flutter_secure_storage`, which stays for secrets). The locale notifier depends on
the provider, overridden with the fake in tests.

*Why:* `shared_preferences` is the standard, dependency-light choice for small
non-secret prefs; a generic seam (rather than a locale-specific one) means the
next setting reuses it instead of adding another store. *Alternatives:* reuse
`flutter_secure_storage` (overkill/slow for a non-secret, and conflates concerns);
write our own file — needless. Modeling the seam as an interface keeps tests off
native channels (mirrors `midiService`/`audioService`).

### D5: Localize the visible surface now; leave a typed enum for languages
Add a small `AppLanguage` enum (`en`/`fr`/`it`/`es`) with `toLocale()` and a
**flag** for the picker, rendered as a Unicode regional-indicator emoji rather
than a language name: 🇬🇧 `en`, 🇫🇷 `fr`, 🇮🇹 `it`, 🇪🇸 `es` (each an ISO-3166
region pair — `en`→GB, `fr`→FR, `it`→IT, `es`→ES). Migrate the strings the user
actually sees on the main/settings path in this change; track the `auth/` bulk as
follow-up so the change stays reviewable and coverage stays green.

*Why:* bounds the diff, delivers a visibly working feature, and avoids a 250-string
mechanical churn blocking the plumbing. The enum gives type-safety for the picker
and persistence key. Flags are compact and language-neutral (the picker never
renders in a language the user can't read). *Caveat:* flags denote countries, not
languages (🇬🇧 for English is a convention, not a truth), so each flag keeps an
accessible/semantic label (`Semantics`/tooltip) for screen readers and clarity.

## Risks / Trade-offs

- **[Partial translation looks inconsistent]** → Localize a coherent slice (the
  main screen + settings drawer, where the picker lives) so the feature is
  demonstrably real; file the `auth/` remainder as explicit follow-up rather than
  shipping half-translated auth. Flag emoji make the picker itself language-neutral
  (never rendered in a language the user can't read), with a semantic label backing
  each flag for accessibility.
- **[Async preference restore → brief wrong-language frame]** → For the default
  (no override) case, `locale=null` already yields the device locale, so no flash.
  For an override, the notifier resolves the stored value before first meaningful
  interaction; worst case is a sub-frame device-locale render, not a restart.
- **[Coverage dips from new generated l10n + widget strings]** → Exclude generated
  `l10n` output from coverage like other generated sources; unit-test the locale
  notifier and `PreferencesService` fake, and the resolution/fallback logic (pure,
  host-testable) to keep ≥80%.
- **[CI forgets `gen-l10n`]** → Wire `flutter gen-l10n` into the same pre-analyze/
  test step as `build_runner` (melos), so generated `AppLocalizations` always
  exists; missing generated files fail fast locally and in CI.
- **[Placeholder/ICU mistakes across four ARBs]** → `gen_l10n` validates
  placeholders against the template at generation time; keep `app_en.arb` as the
  single template and mirror keys in the others.

## Migration Plan

1. Add deps (`flutter_localizations`, `intl`, `shared_preferences`), `generate:
   true`, `l10n.yaml`, and the four ARB files (English template first).
2. Land the `PreferencesService` seam + provider + fake (no behavior change yet).
3. Add the `AppLocale` notifier + `AppLanguage` enum; wire `MaterialApp.locale`
   and delegates in `CymbraApp`.
4. Add the language picker to the settings drawer; localize the visible
   main-screen/settings strings.
5. Tests: resolution/fallback, persistence round-trip, hot-switch widget test,
   picker behavior — all with the fake preferences + `ProviderScope` overrides.

**Rollback:** the feature is additive and behind the new providers; reverting the
`MaterialApp` wiring restores English-only behavior. Persisted preference keys are
namespaced and ignored if the feature is removed.

## Open Questions

- Should the language picker also expose an explicit **"System default"** entry
  (clears the override, `locale=null`), or only the four concrete languages?
  (Leaning yes — it is the clean way to return to device-follow after an override.)
- Which exact strings constitute the "visible slice" to translate now vs. defer —
  confirm the main-screen + settings-drawer boundary during `/opsx:apply`.
- Do we localize the app **`title`** (task-switcher label) per-language, or keep
  the brand "Cymbra Music" fixed? (Brand likely stays fixed.)
