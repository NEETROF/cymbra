## 1. Resolver + launcher seam

- [x] 1.1 Add a pure `legalLinks` resolver (host-testable, `*_core` style) mapping the active locale to the four URLs: `fr` → `https://cymbra.app/cgu/` + `https://cymbra.app/confidentialite/`; else → `https://cymbra.app/en/terms/` + `https://cymbra.app/en/privacy/`
- [x] 1.2 Add a `legalLinkLauncherProvider` seam wrapping `launchUrl(uri, mode: LaunchMode.externalApplication)`, injectable/overridable in tests (mirror `midiServiceProvider` pattern)
- [x] 1.3 Unit-test the resolver for each locale (`fr`, `en`, `es`, `it`)

## 2. Localized strings

- [x] 2.1 Add `legalSectionTitle`, `legalTerms`, `legalPrivacy`, and the entry consent strings to `app_en.arb` (template) with `@` metadata
- [x] 2.2 Mirror the new keys in `app_fr.arb`, `app_es.arb`, `app_it.arb`
- [x] 2.3 Run `flutter gen-l10n` (or build_runner) and confirm `AppLocalizations` exposes the new getters

## 3. Account-menu Legal entries

- [x] 3.1 Add Terms and Privacy `PopupMenuItem`s (below a divider, near sign out / delete) to the signed-in account menu in `account_menu.dart`
- [x] 3.2 Wire each entry to resolve the URL from the active locale and call the launcher seam
- [x] 3.3 Widget test: overriding the launcher fake, opening the menu and tapping each entry records the correct locale-resolved URL

## 4. Entry consent notice

- [x] 4.1 Add a consent notice with tappable Terms and Privacy references to `entry_screen.dart`
- [x] 4.2 Wire the tappable references to the resolver + launcher seam
- [x] 4.3 Widget test: the notice renders and tapping each reference records the correct URL via the fake launcher

## 5. Verify

- [x] 5.1 `melos run analyze` + `dart format` clean; `dart run custom_lint` passes
- [x] 5.2 `flutter test --coverage` passes with line coverage ≥ 80%
- [x] 5.3 `openspec validate add-legal-links --strict` passes
