## ADDED Requirements

### Requirement: The pages Lingua sends Italian readers to exist in Italian
The site SHALL publish in Italian the privacy policy, the terms, the support page and the account deletion page, as translations of the French pages. The Italian privacy policy, terms and account deletion page SHALL carry, directly under their title, « Traduzione di cortesia: fa fede la versione francese », linking their French page; each SHALL add nothing to its French page and leave nothing out but that notice, and SHALL carry its French page's date of last update and its sections. `/it/privacy/`, `/it/termini/` and `/it/elimina-account/` SHALL be pinned routes from the day they are published.

#### Scenario: Deleting an account in Italian
- **WHEN** a visitor opens `/it/elimina-account/`
- **THEN** the page is in Italian and lets them sign in and delete their account

#### Scenario: The French prevails
- **WHEN** a visitor opens `/it/privacy/`, `/it/termini/` or `/it/elimina-account/`
- **THEN** the page carries, under its title, « Traduzione di cortesia: fa fede la versione francese », linking `/confidentialite/`, `/cgu/` or `/suppression-compte/`

#### Scenario: A French edit not mirrored
- **WHEN** the French privacy policy's date of last update moves and the Italian's does not
- **THEN** the site check fails, naming the Italian page

#### Scenario: A pinned Italian page dropped
- **WHEN** a build no longer produces `/it/elimina-account/`
- **THEN** the site's route check fails, naming the path

### Requirement: The home page and Cymbra Music's page exist in Italian
The site SHALL publish its home page at `/it/` and Cymbra Music's page at `/it/music/`, as translations of the French pages, with Music's terms as its Italian app writes them. Every Italian page SHALL link them as its home and as Cymbra Music, and their French, English and Spanish twins SHALL name them as their Italian translations. The account, code redemption and checkout pages SHALL stay in French and English, and the Italian pages SHALL link their English pages.

#### Scenario: An Italian visitor opens Cymbra Music
- **WHEN** a visitor on `/it/privacy/` follows the navigation's « Music » link, or the brand
- **THEN** `/it/music/`, or `/it/`, opens in Italian

#### Scenario: The account stays in English for an Italian visitor
- **WHEN** a visitor on `/it/` follows « Account »
- **THEN** `/en/account` opens, and its language switch offers French and no Italian

### Requirement: The Italian home page describes Lingua for the readers it serves
The Italian home page SHALL describe Cymbra Lingua from the pairs it ships. Once a pair glossed in Italian ships, it SHALL name the languages read with an Italian gloss and link the Italian Lingua page. Until then, it SHALL name the languages Lingua reads and the speakers it is made for, and link the English Lingua page; it SHALL NOT say that Lingua explains words in Italian.

#### Scenario: Before any pair glossed in Italian
- **WHEN** no shipped pair is glossed in Italian
- **THEN** the Italian home page's Lingua card names the languages read and the speakers of each shipped native language as the readers it is made for, and links `/en/lingua`

#### Scenario: Italian speakers learning English
- **WHEN** en-it ships
- **THEN** the card names English as the language read and no audience, and links `/it/lingua`

## MODIFIED Requirements

### Requirement: Each page names its translations
The public site SHALL publish its pages in French at the root, in English under `/en/`, in Spanish under `/es/` and in Italian under `/it/`, each language with its own slugs. Every page SHALL name the address of each translation it has; its language switch SHALL offer those translations, and its `hreflang` SHALL name each of them.

#### Scenario: A legal page's translations
- **WHEN** a visitor opens `/confidentialite/` and switches to English
- **THEN** `/en/privacy/` opens, and the page names `/en/privacy/`, `/es/privacidad/` and `/it/privacy/` as its translations

#### Scenario: A page in one language
- **WHEN** a page exists in French alone
- **THEN** its switch offers no other language, and its `hreflang` names it alone

### Requirement: The home page and Cymbra Music's page exist in Spanish
The site SHALL publish its home page at `/es/` and Cymbra Music's page at `/es/music/`, as translations of the French pages. Every Spanish page SHALL link them as its home and as Cymbra Music, and their French and English twins SHALL name them as their Spanish translations. The account, code redemption and checkout pages SHALL stay in French and English, and the Spanish pages SHALL link their English pages.

#### Scenario: A Spanish visitor opens Cymbra Music
- **WHEN** a visitor on `/es/privacidad/` follows the navigation's « Music » link, or the brand
- **THEN** `/es/music/`, or `/es/`, opens in Spanish

#### Scenario: A French visitor switches to Spanish
- **WHEN** a visitor on `/music/` switches to Spanish
- **THEN** `/es/music/` opens, and the page names `/music/`, `/en/music/`, `/es/music/` and `/it/music/` as its translations

#### Scenario: The account stays in English for a Spanish visitor
- **WHEN** a visitor on `/es/` follows « Cuenta »
- **THEN** `/en/account` opens, and its language switch offers French and no Spanish
