## 1. Resolver + launcher seam

- [ ] 1.1 Add a pure `legalLinks` resolver (host-testable, `*_core` style) mapping the active locale to the four URLs: `fr` → `https://cymbra.app/cgu/` + `https://cymbra.app/confidentialite/`; else → `https://cymbra.app/en/terms/` + `https://cymbra.app/en/privacy/`
- [ ] 1.2 Add a `legalLinkLauncherProvider` seam wrapping `launchUrl(uri, mode: LaunchMode.externalApplication)`, injectable/overridable in tests (mirror `midiServiceProvider` pattern)
- [ ] 1.3 Unit-test the resolver for each locale (`fr`, `en`, `es`, `it`)

## 2. Localized strings

- [ ] 2.1 Add `legalSectionTitle`, `legalTerms`, `legalPrivacy`, and the entry consent strings to `app_en.arb` (template) with `@` metadata
- [ ] 2.2 Mirror the new keys in `app_fr.arb`, `app_es.arb`, `app_it.arb`
- [ ] 2.3 Run `flutter gen-l10n` (or build_runner) and confirm `AppLocalizations` exposes the new getters

## 3. Settings Legal section

- [ ] 3.1 Add a Legal section (divider + Terms and Privacy `ListTile`s) to the bottom of the settings drawer master view in `player_screen.dart`
- [ ] 3.2 Wire each tile to resolve the URL from the active locale and call the launcher seam
- [ ] 3.3 Widget test: overriding the launcher fake, tapping each tile records the correct locale-resolved URL

## 4. Entry consent notice

- [ ] 4.1 Add a consent notice with tappable Terms and Privacy references to `entry_screen.dart`
- [ ] 4.2 Wire the tappable references to the resolver + launcher seam
- [ ] 4.3 Widget test: the notice renders and tapping each reference records the correct URL via the fake launcher

## 5. Verify

- [ ] 5.1 `melos run analyze` + `dart format` clean; `dart run custom_lint` passes
- [ ] 5.2 `flutter test --coverage` passes with line coverage ≥ 80%
- [ ] 5.3 `openspec validate add-legal-links --strict` passes
