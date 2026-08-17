## ADDED Requirements

### Requirement: The site signs users in through the existing web sign-in surface

The site SHALL offer a sign-in island (email + password, "Continue with Google", "Continue with
Apple") that calls the backend web sign-in (`/web/auth/signin`) with audience `web`, keeps the
returned access token **in memory only**, and relies on the HttpOnly refresh cookie for
persistence. A Google or Apple button SHALL be hidden when its client id is not configured; email
sign-in SHALL always be available. Errors SHALL be localized (fr/en), never raw.

#### Scenario: Email sign-in

- **WHEN** a user submits a valid email and password
- **THEN** the island holds an access token in memory, no token is written to web storage, and the hosting page proceeds

#### Scenario: Provider button hidden without configuration

- **WHEN** the Apple client id is not configured
- **THEN** the Apple button is absent and Google / email remain

### Requirement: Sessions persist across pages by refresh, and sign-out revokes

On mount, an island SHALL call `/web/auth/refresh`: a valid cookie yields a fresh access token
without any UI, an absent or expired one shows the sign-in form. Sign-out SHALL call
`/web/auth/logout` and clear the in-memory token.

#### Scenario: Returning visitor is signed in silently

- **WHEN** a user with a valid refresh cookie opens `/account`
- **THEN** the plan is shown without a sign-in form

#### Scenario: Sign-out

- **WHEN** the user signs out
- **THEN** the cookie is cleared server-side and the island returns to the sign-in form
