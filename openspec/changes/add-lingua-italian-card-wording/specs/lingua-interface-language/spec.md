## MODIFIED Requirements

### Requirement: The interface's copy is kept in one typed catalogue per language
The interface's copy SHALL be kept in one catalogue per language, one module per surface, the French module the source and the English and Spanish modules typed after it, the Italian ones too, `grammar` among them, so that a key missing in a language does not compile; a surface moved to the catalogue SHALL read every text it shows from it; a sentence built from parts SHALL be one message taking its parts, and a count SHALL take plural forms chosen by `Intl.PluralRules` for the interface language. A French string literal outside the catalogue SHALL fail a lint, except in a file the lint's baseline names; the baseline SHALL fail when it names a file holding no literal. No English, Spanish or Italian entry SHALL be empty or equal to its French one unless the text is the same in every language, or the list of texts that language's draft shares with the French names it.

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
- **THEN** every English, Spanish and Italian entry is present and non-empty, every slot of a French message is taken by its translations, and no translation equals the French text outside the lists of texts that are the same in every language or that the language shares with the French

#### Scenario: A count in Italian
- **WHEN** the review's remaining-cards message is rendered from the Italian catalogue for 1 and for 3
- **THEN** the plural forms chosen are `one` and `other`, and the counts are written « 1 » and « 3 »
