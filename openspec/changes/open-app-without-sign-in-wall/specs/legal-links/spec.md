## MODIFIED Requirements

### Requirement: Consent notice at account entry

The app SHALL display, on every surface where a user signs in or creates an account — the account
entry screen and the contextual sign-in surface alike — a consent notice stating that continuing
implies acceptance of the Terms of Service and Privacy Policy, with both references tappable to
open the resolved URLs in an external browser.

#### Scenario: Consent notice is visible on entry
- **WHEN** the account entry screen is shown
- **THEN** a notice referencing the Terms of Service and the Privacy Policy is displayed, with each reference tappable

#### Scenario: Consent notice on the contextual sign-in surface
- **WHEN** the contextual sign-in surface is shown, from the welcome, the paywall or any account-gated action
- **THEN** a notice referencing the Terms of Service and the Privacy Policy is displayed, with each reference tappable

#### Scenario: Tapping a reference opens the page
- **WHEN** the user taps the Terms or Privacy reference in the consent notice
- **THEN** the app launches the corresponding locale-resolved URL in an external browser via the injected launcher
