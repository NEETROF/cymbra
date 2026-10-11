## MODIFIED Requirements

### Requirement: Account stores a preferred locale
The system SHALL persist a preferred locale on the shared account, record a request's non-empty locale on an account only when the account has none, and replace a stored locale only through the account's own language setting (`SetLocale`). Sign-up SHALL record the locale its request carries on the account it creates. A resend, a password-reset request and a set-password request that carry a non-empty locale SHALL record it when the account has no stored locale, and SHALL leave a stored one unchanged. `SetLocale`, called by the account's owner from an authenticated client, SHALL replace a stored locale, last writer wins. An account with no recorded locale SHALL have its e-mails rendered in the request's locale, or in English when the request carries none.

#### Scenario: Locale recorded at sign-up
- **WHEN** a user signs up and the request carries the locale `fr`
- **THEN** the account's stored preferred locale is `fr`

#### Scenario: Later request updates the stored locale
- **WHEN** the account's owner later calls `SetLocale` with the locale `es`
- **THEN** the account's stored preferred locale becomes `es`

#### Scenario: Empty locale does not overwrite
- **WHEN** a request for that account carries an empty locale
- **THEN** the stored preferred locale is left unchanged

#### Scenario: An e-mail request does not replace a stored locale
- **WHEN** a password reset or a resend is requested with the locale `en` for an account whose stored locale is `fr`
- **THEN** the stored preferred locale stays `fr`

#### Scenario: An e-mail request fills an absent locale
- **WHEN** the owner of an account that has no stored locale adds a password with a set-password request carrying `es`
- **THEN** the account's stored preferred locale becomes `es`

### Requirement: Stored locale is the email-localization fallback
Every account e-mail — the sign-up, resend, password-reset and set-password codes — SHALL be rendered in the account's stored preferred locale when it has one, in the request's locale when the account has none and the request carries one, and in English otherwise. A stored locale the e-mails are not written in SHALL still be the one selected, so the e-mail is rendered in English by the rendering layer's fallback. This SHALL NOT change the rendering layer — only which locale is selected.

#### Scenario: Request locale wins over stored
- **WHEN** the owner of an account whose stored locale is `fr` sets the locale `it` with `SetLocale`, and an email is then triggered by a request carrying `fr`
- **THEN** the email is rendered in `it`

#### Scenario: An e-mail request's locale does not override the stored one
- **WHEN** an email is triggered by a request carrying `it` for an account whose stored locale is `fr`
- **THEN** the email is rendered in `fr`

#### Scenario: Stored locale used when request carries none
- **WHEN** an email is triggered with no locale for an account whose stored locale is `fr`
- **THEN** the email is rendered in `fr`

#### Scenario: Request locale used when the account has none
- **WHEN** a password reset carrying `fr-FR` is requested for an account that has no stored locale
- **THEN** the email is rendered in `fr`

#### Scenario: English when neither is present
- **WHEN** an email is triggered with no locale for an account that has no stored locale
- **THEN** the email is rendered in English

#### Scenario: Set-password follows the account
- **WHEN** the owner of an account whose stored locale is `es` adds a password with a request carrying no locale, or carrying `en`
- **THEN** the verification email is rendered in `es`

#### Scenario: A stored locale the e-mails are not written in
- **WHEN** an email is triggered by a request carrying `fr` for an account whose stored locale is `de-DE`
- **THEN** the email is rendered in English

### Requirement: Stored-locale lookup preserves enumeration safety
Reading the stored locale, and recording a request's locale on an account that has none, during the resend and password-reset flows SHALL NOT change the uniform response that hides whether an account exists.

#### Scenario: Reset response is uniform regardless of account existence
- **WHEN** a password reset is requested for an address that has an account and for one that does not
- **THEN** both requests return the same uniform success response, and the stored-locale lookup and write for the existing account produce no externally observable difference

#### Scenario: Resend response is uniform regardless of account existence
- **WHEN** a verification code is resent for an address that has an unverified account and for one that has no account
- **THEN** both requests return the same uniform success response, and only the existing account's stored locale is read or written
