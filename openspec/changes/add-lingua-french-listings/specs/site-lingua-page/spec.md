## ADDED Requirements

### Requirement: The Lingua page says what each studied language's card and levels are
The site's Lingua page SHALL say, in each site language, which studied languages of the shipped pairs have a word card that names the tense and the gender, and which have levels estimated from word frequency, naming them together in one sentence each, the page's readers' pairs first; it SHALL name the level scale as the extension's interface in that language names it: CEFR in French and in English, MCER in Spanish. What each studied language's card and levels are SHALL be read from one table, and the build SHALL fail on a shipped pair whose studied language the table does not describe.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr
- **THEN** the French page says that in Spanish the card also names the tense and the gender, and that Spanish's levels are estimated for want of a freely licensed CEFR list, the scale named « CEFR »

#### Scenario: French studied
- **WHEN** fr-en ships beside es-en
- **THEN** the English page says that in Spanish and French the card also names the tense and the gender, and that Spanish's and French's levels are estimated

#### Scenario: English is never said to be estimated
- **WHEN** en-fr and en-es ship
- **THEN** no page says that English's levels are estimated or that its card names a gender

#### Scenario: A studied language the table does not describe
- **WHEN** a shipped pair studies a language the table has no entry for
- **THEN** the site's build fails, naming that language

### Requirement: The coverage table reads at a phone's width
While every shipped pair is glossed in one language, the Lingua page's coverage table SHALL keep one column per pair. Once pairs of two or more native languages are listed, it SHALL have one row per pair, named "<studied> → <native>", the page's readers' pairs first, with one column per size of the commonest-words list; and the page SHALL NOT scroll sideways at a phone's width, the table scrolling within its own box where it is wider than the text.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr
- **THEN** the table has one column per pair, as before

#### Scenario: Six pairs on a phone
- **WHEN** the six pairs ship and the English page is shown 375 pixels wide
- **THEN** the table has six rows, Spanish → English and French → English first, and the page does not scroll sideways
