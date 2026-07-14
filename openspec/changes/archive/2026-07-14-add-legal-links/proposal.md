## Why

The app ships no in-app link to Cymbra's Terms of Service (CGU) or Privacy Policy. Both the Apple App Store and Google Play require a reachable privacy-policy link inside the app, and the RGPD requires the CGU and privacy notice to be accessible — especially now that users upload their own scores (UGC) and attest authorship on upload. Without these links the app is not store-compliant and exposes NEETROF to avoidable legal risk.

## What Changes

- Add two outbound legal links — **Terms of Service (CGU)** and **Privacy Policy (Confidentialité)** — that open the corresponding page on `cymbra.app` in an external browser.
- Surface them in **two places**:
  - Two entries in the signed-in **account menu**, grouped near the account actions (sign out / delete account) — satisfies the store "reachable from inside the app" requirement, kept close to the user profile.
  - A **consent line** on the account entry / sign-up screen ("En continuant, vous acceptez…") with the two links tappable (RGPD best practice at account creation), reachable to guests.
- **Locale-aware URLs**: `fr` → `https://cymbra.app/cgu/` + `https://cymbra.app/confidentialite/`; every other locale (`en`, `es`, `it`) → `https://cymbra.app/en/terms/` + `https://cymbra.app/en/privacy/`.
- Add the supporting localized strings to the four `.arb` files.

No breaking changes. No new dependency (`url_launcher` is already present).

## Capabilities

### New Capabilities
- `legal-links`: In-app access to the Terms of Service and Privacy Policy — locale-aware external links surfaced in settings and at account entry.

### Modified Capabilities
<!-- None: no existing spec's requirements change. -->

## Impact

- **App (Flutter)**: `apps/music/lib/screens/auth/account_menu.dart` (account-menu Legal entries), `apps/music/lib/screens/auth/entry_screen.dart` (consent line), a small shared launcher helper (`services/legal_links.dart`), and the four `apps/music/lib/l10n/app_*.arb` files.
- **Dependencies**: reuses existing `url_launcher: ^6.3.2`; no new packages.
- **External**: relies on the four legal pages already published on `cymbra.app`.
- **Tests/coverage**: new widget tests for the settings Legal section and entry consent line; the launcher is placed behind an injectable seam so the ≥80% Flutter coverage gate holds without opening a real browser.
