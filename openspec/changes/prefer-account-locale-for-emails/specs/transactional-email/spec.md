## MODIFIED Requirements

### Requirement: Localized content with English fallback
Transactional email subject and body SHALL be localized to the flagship app's supported locales (English, Spanish, French, Italian), selected from the recipient's locale as `user-locale-preference` resolves it: the account's stored preferred locale, else the request's locale, else none. When no locale is resolved, or the resolved locale is not supported, the backend SHALL render the email in English. The recipient's locale SHALL be accepted as an optional, additive field on the relevant requests such that omitting it does not break existing clients.

#### Scenario: Renders in the requested supported locale
- **WHEN** an email is rendered for a recipient whose locale is French
- **THEN** the subject and body are in French

#### Scenario: Falls back to English for unknown or missing locale
- **WHEN** an email is rendered with no locale, or with an unsupported locale
- **THEN** the subject and body are rendered in English
- **AND** the email is still fully branded and valid

#### Scenario: Optional locale is backwards compatible
- **WHEN** a client sends a request without the locale field
- **THEN** the request succeeds and the email is rendered in the account's stored preferred locale, or in English when the account has none
