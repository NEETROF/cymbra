## ADDED Requirements

### Requirement: The site is published in French, English and Spanish
The public site SHALL publish its pages in French at the root, in English under `/en/` and in Spanish under `/es/`, each language with its own slugs. Every page SHALL name the address of each translation it has; its language switch SHALL offer those translations, and its `hreflang` SHALL name each of them, with `x-default` the English page when there is one and the French page otherwise.

#### Scenario: A legal page's translations
- **WHEN** a visitor opens `/confidentialite/` and switches to English
- **THEN** `/en/privacy/` opens, and the page names `/en/privacy/` and `/es/privacidad/` as its translations

#### Scenario: A page in one language
- **WHEN** a page exists in French alone
- **THEN** its switch offers no other language, and its `hreflang` names it alone

### Requirement: The Spanish pages readers of Spanish are sent to exist
The site SHALL publish in Spanish the privacy policy, the terms, the support page and the account deletion page, as translations of the French pages, and an unknown address under `/es/` SHALL answer the Spanish not-found page.

#### Scenario: A Spanish reader deletes their account
- **WHEN** a reader whose interface is Spanish follows the extension's deletion link
- **THEN** `/es/eliminar-cuenta/` opens in Spanish and lets them sign in and delete their account

#### Scenario: An unknown Spanish address
- **WHEN** a visitor opens `/es/pagina-que-no-existe/`
- **THEN** the Spanish not-found page answers
