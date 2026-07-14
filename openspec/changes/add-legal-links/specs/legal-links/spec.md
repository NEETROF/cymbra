## ADDED Requirements

### Requirement: Locale-aware legal link resolution

The app SHALL resolve the Terms of Service (CGU) and Privacy Policy URLs from the active app locale. French SHALL use the French pages; every other supported locale SHALL fall back to the English pages.

- Terms of Service: `fr` → `https://cymbra.app/cgu/`; otherwise → `https://cymbra.app/en/terms/`
- Privacy Policy: `fr` → `https://cymbra.app/confidentialite/`; otherwise → `https://cymbra.app/en/privacy/`

#### Scenario: French locale resolves French URLs
- **WHEN** the active locale is `fr`
- **THEN** the Terms link resolves to `https://cymbra.app/cgu/` and the Privacy link to `https://cymbra.app/confidentialite/`

#### Scenario: Non-French locale falls back to English URLs
- **WHEN** the active locale is `en`, `es`, or `it`
- **THEN** the Terms link resolves to `https://cymbra.app/en/terms/` and the Privacy link to `https://cymbra.app/en/privacy/`

### Requirement: Legal links reachable from the account menu

The app SHALL expose a Terms of Service entry and a Privacy Policy entry in the signed-in account menu, grouped near the account actions (sign out / delete account). Tapping an entry SHALL open the resolved URL in an external browser.

#### Scenario: Open Terms from the account menu
- **WHEN** a signed-in user opens the account menu and taps the Terms of Service entry
- **THEN** the app launches the locale-resolved Terms URL in an external browser via the injected launcher

#### Scenario: Open Privacy from the account menu
- **WHEN** a signed-in user opens the account menu and taps the Privacy Policy entry
- **THEN** the app launches the locale-resolved Privacy URL in an external browser via the injected launcher

### Requirement: Consent notice at account entry

The account entry / sign-up screen SHALL display a consent notice stating that continuing implies acceptance of the Terms of Service and Privacy Policy, with both references tappable to open the resolved URLs in an external browser.

#### Scenario: Consent notice is visible on entry
- **WHEN** the account entry screen is shown
- **THEN** a notice referencing the Terms of Service and the Privacy Policy is displayed, with each reference tappable

#### Scenario: Tapping a reference opens the page
- **WHEN** the user taps the Terms or Privacy reference in the consent notice
- **THEN** the app launches the corresponding locale-resolved URL in an external browser via the injected launcher

### Requirement: Launcher is behind an injectable seam

The URL launch SHALL be performed through a provider-injected seam so that widgets and state remain testable without opening a real browser or the native `url_launcher` plugin.

#### Scenario: Tests drive the seam with a fake
- **WHEN** a widget test overrides the launcher provider with a fake
- **THEN** tapping a legal link records the requested URL through the fake instead of invoking the native browser
