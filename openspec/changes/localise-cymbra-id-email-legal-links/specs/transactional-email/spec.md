## MODIFIED Requirements

### Requirement: Localized footer legal links
Every transactional email footer SHALL carry Terms-of-Service and Privacy-Policy
links pointing to the `cymbra.app` legal pages, resolved from the email's locale:
French SHALL use the French pages, Spanish SHALL use the Spanish pages, and every
other locale SHALL fall back to the English pages.

#### Scenario: French footer links
- **WHEN** an email is rendered for a French recipient
- **THEN** the footer Terms link is `https://cymbra.app/cgu/` and the Privacy link
  is `https://cymbra.app/confidentialite/`

#### Scenario: Non-French footer links
- **WHEN** an email is rendered for a locale other than French and Spanish (including English and Italian)
- **THEN** the footer Terms link is `https://cymbra.app/en/terms/` and the Privacy
  link is `https://cymbra.app/en/privacy/`

#### Scenario: Spanish footer links
- **WHEN** an email is rendered for a Spanish recipient
- **THEN** the footer Terms link is `https://cymbra.app/es/terminos/` and the Privacy
  link is `https://cymbra.app/es/privacidad/`

## ADDED Requirements

### Requirement: An e-mail declares its language
A transactional email's HTML SHALL declare the language it is written in — the resolved locale, English when the requested one is unsupported — so that a reader's software reads it in that language.

#### Scenario: A French e-mail
- **WHEN** an email is rendered for a French recipient
- **THEN** its HTML declares `lang="fr"`

#### Scenario: An unsupported locale
- **WHEN** an email is rendered for a recipient whose locale is German
- **THEN** it is written in English and its HTML declares `lang="en"`
