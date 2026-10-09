## ADDED Requirements

### Requirement: The home page and Cymbra Music's page exist in Spanish
The site SHALL publish its home page at `/es/` and Cymbra Music's page at `/es/music/`, as translations of the French pages. Every Spanish page SHALL link them as its home and as Cymbra Music, and their French and English twins SHALL name them as their Spanish translations. The account, code redemption and checkout pages SHALL stay in French and English, and the Spanish pages SHALL link their English pages.

#### Scenario: A Spanish visitor opens Cymbra Music
- **WHEN** a visitor on `/es/privacidad/` follows the navigation's « Music » link, or the brand
- **THEN** `/es/music/`, or `/es/`, opens in Spanish

#### Scenario: A French visitor switches to Spanish
- **WHEN** a visitor on `/music/` switches to Spanish
- **THEN** `/es/music/` opens, and the page names `/music/`, `/en/music/` and `/es/music/` as its translations

#### Scenario: The account stays in English for a Spanish visitor
- **WHEN** a visitor on `/es/` follows « Cuenta »
- **THEN** `/en/account` opens, and its language switch offers French and no Spanish

### Requirement: The Spanish home page describes Lingua for the readers it serves
The Spanish home page SHALL describe Cymbra Lingua from the pairs it ships. Once a pair glossed in Spanish ships, it SHALL name the languages read with a Spanish gloss and link the Spanish Lingua page. Until then, it SHALL name the languages Lingua reads and the speakers it is made for, and link the English Lingua page; it SHALL NOT say that Lingua explains words in Spanish.

#### Scenario: Before any pair glossed in Spanish
- **WHEN** every shipped pair is glossed in French or English
- **THEN** the Spanish home page's Lingua card names the languages read and the speakers of each native language as the readers it is made for, and links `/en/lingua`

#### Scenario: Spanish speakers learning English
- **WHEN** en-es ships
- **THEN** the card names English as the language read and no audience, and links `/es/lingua`
