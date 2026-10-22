## MODIFIED Requirements

### Requirement: Every indexable page names its canonical address
Every page of the site that is not marked `noindex` SHALL carry a canonical link and an `og:url` naming the address the host serves for it: `https://cymbra.app` followed by its path with a trailing slash, the root being `https://cymbra.app/`. Every `hreflang` link SHALL name its translation in the same form. A page marked `noindex` SHALL carry no canonical link.

#### Scenario: The Lingua page
- **WHEN** `/lingua/` is built
- **THEN** its canonical link and `og:url` are `https://cymbra.app/lingua/`, and its `hreflang` links name `https://cymbra.app/lingua/`, `https://cymbra.app/en/lingua/`, `https://cymbra.app/es/lingua/` and `https://cymbra.app/it/lingua/`

#### Scenario: A legal page
- **WHEN** `/en/privacy/` is built
- **THEN** its canonical link is `https://cymbra.app/en/privacy/` and its French translation is named `https://cymbra.app/confidentialite/`

#### Scenario: A personal page
- **WHEN** `/account/` is built
- **THEN** it is marked `noindex` and carries no canonical link
