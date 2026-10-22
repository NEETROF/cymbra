# Spec Delta

## ADDED Requirements

### Requirement: The Italian catalogue writes as Italian does, before it is shown
The interface's Italian catalogue SHALL address the reader with « tu », never « Lei »; SHALL name the level scale « QCER »; SHALL write numbers, percentages and dates as Italian does (« 20.000 », « 96% », « 4 ottobre 2026 »), counts by `Intl.PluralRules` for Italian; SHALL elide and contract its articles and prepositions where a message is built from parts (« dall'inglese », « dallo spagnolo », « dal francese »); and SHALL address the reader without assuming a gender. Until the Italian interface is shown, no surface SHALL read the Italian catalogue: the interface languages stay French, English and Spanish, and every surface and request is what it was.

#### Scenario: Italian numbers and percentages
- **WHEN** the Italian catalogue formats 20000, 0.96 as a percentage, and 4 October 2026
- **THEN** it writes « 20.000 », « 96% » and « 4 ottobre 2026 »

#### Scenario: A contraction built from parts
- **WHEN** the Italian note on levels borrowed from another language names English, Spanish or French
- **THEN** it reads « dall'inglese », « dallo spagnolo » or « dal francese »

#### Scenario: The level scale
- **WHEN** the Italian catalogue names the level scale
- **THEN** it says « QCER »

#### Scenario: Not shown yet
- **WHEN** a reader whose browser is in Italian uses a build with the Italian catalogue and no pair glossed in Italian
- **THEN** every surface is in French, English or Spanish as before, and no Italian text is shown

## MODIFIED Requirements

### Requirement: The interface's copy is kept in one typed catalogue per language
The interface's copy SHALL be kept in one catalogue per language, one module per surface, the French module the source and the English, Spanish and Italian modules typed after it, so that a key missing in a language does not compile; a surface moved to the catalogue SHALL read every text it shows from it; a sentence built from parts SHALL be one message taking its parts, and a count SHALL take plural forms chosen by `Intl.PluralRules` for the interface language. A French string literal outside the catalogue SHALL fail a lint, except in a file the lint's baseline names; the baseline SHALL fail when it names a file holding no literal. No English, Spanish or Italian entry SHALL be empty or equal to its French one unless the text is the same in every language, or the list of texts an Italian draft shares with the French names it.

#### Scenario: A key missing in Spanish
- **WHEN** a key is added to the French catalogue and not to the Spanish one
- **THEN** the build does not compile

#### Scenario: A French literal outside the catalogue
- **WHEN** a French string is written in a surface's module that the baseline does not name
- **THEN** the lint fails, naming the file and the line

#### Scenario: A baseline that went stale
- **WHEN** a surface's module moved its copy to the catalogue and still stands on the baseline
- **THEN** the lint fails, naming the file to remove

#### Scenario: A count in each language
- **WHEN** the review's remaining-cards message is rendered from the catalogue for 1 and for 3
- **THEN** the French reads « 1 carte(s) à revoir » and « 3 carte(s) à revoir », the English "1 card to review" and "3 cards to review", the Spanish « 1 tarjeta por repasar » and « 3 tarjetas por repasar »

#### Scenario: The drafts are whole
- **WHEN** the parity test runs
- **THEN** every English, Spanish and Italian entry is present and non-empty, every slot of a French message is taken by its translations, and no translation equals the French text outside the list of texts that are the same in every language

#### Scenario: A count in Italian
- **WHEN** the review's remaining-cards message is rendered from the Italian catalogue for 1 and for 3
- **THEN** it reads « 1 carta da ripassare » and « 3 carte da ripassare »
