## Context

The app has an account entry screen (`entry_screen.dart`) and a settings end-drawer (`_SettingsDrawer` in `player_screen.dart`), plus a gen_l10n setup with four locales (`fr`, `en`, `es`, `it`). `url_launcher: ^6.3.2` is already a dependency but is only wired for desktop OAuth today — there is no general "open an external URL" seam, and no legal links anywhere. CLAUDE.md mandates Riverpod 2 + provider-injected dependencies (no constructor injection) and a ≥80% Flutter coverage gate that excludes native FFI, so any browser launch must sit behind an injectable seam that tests can fake.

## Goals / Non-Goals

**Goals:**
- One reusable, locale-aware source of truth for the CGU/Privacy URLs.
- Legal links reachable from settings (store compliance) and at account entry (RGPD).
- Fully testable without opening a real browser.

**Non-Goals:**
- No in-app rendering of the legal pages (external browser only).
- No blocking/consent-gate that prevents continuing (informational notice, not a required checkbox).
- No changes to the existing upload authorship attestation.
- No localized `es`/`it` legal pages (they fall back to `/en/`).

## Decisions

- **Locale→URL mapping in a pure helper.** A small pure function maps the active `AppLanguage`/locale to the four URLs (`fr` → French pages, else English). Pure and host-testable, matching the repo's `*_core` convention. Alternative — hardcoding at each call site — rejected: duplicates the fallback rule and drifts.
- **Launcher behind a Riverpod provider seam.** Introduce a `legalLinkLauncherProvider` (a thin wrapper over `launchUrl(..., LaunchMode.externalApplication)`) overridden with a fake in tests, mirroring the existing `midiServiceProvider` seam pattern. Alternative — calling `url_launcher` directly in widgets — rejected: breaks the coverage gate and is untestable. Reuse the existing `appLocaleProvider` to read the active locale rather than passing it down.
- **Two surfaces, one helper.** Settings gets a `Legal` section (two `ListTile`s under the category list, below a divider); entry gets a `RichText`/`Text.rich` consent line with two tappable spans. Both call the same resolver + launcher.
- **Strings via `.arb`.** Add `legalSectionTitle`, `legalTerms`, `legalPrivacy`, and an entry consent string. The entry consent uses two separate tappable labels rather than an interpolated sentence with embedded links, to keep translation and tap-target wiring simple across all four locales.

## Risks / Trade-offs

- [External pages must exist in all linked locales] → French and English pages are confirmed published; `es`/`it` intentionally point at `/en/`. If localized `es`/`it` pages are added later, extend the resolver only.
- [`launchUrl` can fail silently if no browser handler] → the seam swallows/ignores failure without crashing the UI (same posture as OAuth launch); no error surface needed for a legal link.
- [Consent notice is informational, not a hard gate] → acceptable: acceptance-by-continuing is standard and the upload flow already has an explicit authorship attestation for the higher-risk UGC action.

## Migration Plan

Additive, no schema or API change. Ship behind normal release; nothing to roll back beyond reverting the widget/string additions.

## Open Questions

- None blocking. If a full consent checkbox is later required at sign-up, it can be layered on the same resolver/launcher.
