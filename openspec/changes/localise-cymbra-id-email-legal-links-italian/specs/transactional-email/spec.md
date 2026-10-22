## MODIFIED Requirements

### Requirement: Localized footer legal links
Every transactional email footer SHALL carry Terms-of-Service and Privacy-Policy
links pointing to the `cymbra.app` legal pages, resolved from the email's locale:
French SHALL use the French pages, Spanish SHALL use the Spanish pages, Italian SHALL
use the Italian pages, and every other locale SHALL fall back to the English pages.

#### Scenario: French footer links
- **WHEN** an email is rendered for a French recipient
- **THEN** the footer Terms link is `https://cymbra.app/cgu/` and the Privacy link
  is `https://cymbra.app/confidentialite/`

#### Scenario: Non-French footer links
- **WHEN** an email is rendered for a locale other than French, Spanish and Italian (including English)
- **THEN** the footer Terms link is `https://cymbra.app/en/terms/` and the Privacy
  link is `https://cymbra.app/en/privacy/`

#### Scenario: Spanish footer links
- **WHEN** an email is rendered for a Spanish recipient
- **THEN** the footer Terms link is `https://cymbra.app/es/terminos/` and the Privacy
  link is `https://cymbra.app/es/privacidad/`

#### Scenario: Italian footer links
- **WHEN** an email is rendered for an Italian recipient
- **THEN** the footer Terms link is `https://cymbra.app/it/termini/` and the Privacy
  link is `https://cymbra.app/it/privacy/`
