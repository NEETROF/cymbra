## ADDED Requirements

### Requirement: Account stores a preferred locale
The system SHALL persist a preferred locale on the shared account. The identity
system SHALL set it when an authenticated request carries a non-empty locale, and
SHALL update it last-writer-wins on any later request that carries one. An account
with no recorded locale SHALL be treated as English.

#### Scenario: Locale recorded at sign-up
- **WHEN** a user signs up and the request carries the locale `fr`
- **THEN** the account's stored preferred locale is `fr`

#### Scenario: Later request updates the stored locale
- **WHEN** an authenticated request for that account later carries the locale `es`
- **THEN** the account's stored preferred locale becomes `es`

#### Scenario: Empty locale does not overwrite
- **WHEN** a request for that account carries an empty locale
- **THEN** the stored preferred locale is left unchanged

### Requirement: Stored locale is the email-localization fallback
Transactional email SHALL be rendered in the effective locale resolved with the
precedence: the request's locale when non-empty, otherwise the account's stored
preferred locale, otherwise English. This SHALL NOT change the rendering layer —
only which locale is selected.

#### Scenario: Request locale wins over stored
- **WHEN** an email is triggered by a request carrying `it` for an account whose stored locale is `fr`
- **THEN** the email is rendered in `it`

#### Scenario: Stored locale used when request carries none
- **WHEN** an email is triggered with no locale for an account whose stored locale is `fr`
- **THEN** the email is rendered in `fr`

#### Scenario: English when neither is present
- **WHEN** an email is triggered with no locale for an account that has no stored locale
- **THEN** the email is rendered in English

### Requirement: Stored-locale lookup preserves enumeration safety
Reading the stored locale during the password-reset flow SHALL NOT change the
uniform response that hides whether an account exists.

#### Scenario: Reset response is uniform regardless of account existence
- **WHEN** a password reset is requested for an address that has an account and for one that does not
- **THEN** both requests return the same uniform success response, and the stored-locale lookup for the existing account produces no externally observable difference
