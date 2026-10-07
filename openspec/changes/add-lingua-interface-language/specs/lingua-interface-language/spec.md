## ADDED Requirements

### Requirement: The interface language is the reader's native language, under its own key
The interface language SHALL be the reader's native language — `fr`, `en` or `es` — and SHALL be stored on the device under its own key, which the background SHALL write from the profile's native language whenever that is read or set; a device whose key is absent or unknown SHALL read `fr`. Every surface SHALL be able to read the interface language before its first paint, without an engine.

#### Scenario: Every installed reader
- **WHEN** a reader whose profile names no native language opens any surface
- **THEN** the surface reads `fr`, and the French interface is shown

#### Scenario: The key follows the profile
- **WHEN** the background reads or sets a profile whose native language is Spanish
- **THEN** the key holds `es`, and a surface opened afterwards reads `es`

#### Scenario: A surface before any engine
- **WHEN** the popup or the account page opens
- **THEN** it reads the interface language from the key, with no engine loaded

### Requirement: Copy lives in one typed catalogue per language
Every text the interface shows SHALL come from one catalogue per language, one module per surface, the French module the source and the English and Spanish modules typed after it, so that a key missing in a language does not compile; a sentence built from parts SHALL be one message taking its parts, and a count SHALL take plural forms chosen by `Intl.PluralRules` for the interface language. A French literal outside the catalogue SHALL fail a lint, except in a file the lint's baseline names; the baseline SHALL fail when it names a file holding no literal. No English or Spanish entry SHALL be empty or equal to its French one unless the text is the same in every language.

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
- **WHEN** the review shows how many cards remain, for 1 and for 3
- **THEN** the French reads « 1 carte(s) à revoir » and « 3 carte(s) à revoir », the English "1 card to review" and "3 cards to review", the Spanish « 1 tarjeta por repasar » and « 3 tarjetas por repasar »

#### Scenario: The drafts are whole
- **WHEN** the parity test runs
- **THEN** every English and Spanish entry is present and non-empty, every slot of a French message is taken by its translations, and no translation equals the French text outside the list of texts that are the same in every language

### Requirement: The French interface is byte for byte what it was
The French catalogue SHALL hold each text as the interface showed it before the catalogue existed — its spaces, apostrophes, guillemets, abbreviations and number formats — and no surface SHALL show a French text that differs from it by one byte once it reads the catalogue. A typography pass on the French, if any, SHALL be a change of its own.

#### Scenario: Every reader today
- **WHEN** a reader of French opens any surface after this change
- **THEN** every text is the one shown before, byte for byte

#### Scenario: A French format `Intl` would change
- **WHEN** the translation setting states « 25,8 Mo » or the sync says « il y a 3 min. »
- **THEN** the French catalogue holds those strings as they are, and the English and Spanish use the locale's formatting

### Requirement: Copy quoted in a Lingua requirement is the French interface's
When a requirement of a `lingua-*` capability quotes copy — a button, a label, a message — it SHALL be read as quoting the French interface's text, and every interface language SHALL carry the same message in its own words; such a requirement SHALL NOT be modified for a language added, and a new requirement MAY name a text by its meaning instead.

#### Scenario: A requirement that quotes « Réviser »
- **WHEN** a Spanish-native reader opens the surface the requirement describes
- **THEN** the button carries the Spanish catalogue's text for that meaning, and the requirement is met

#### Scenario: A new requirement
- **WHEN** a requirement is written after this change
- **THEN** it may name the reveal button as "the reveal button" and quote no language
