## ADDED Requirements

### Requirement: An expression is the card's answer
When the pack recognises an expression covering the whole selection, the card SHALL show that expression's gloss as its answer, ahead of any word-by-word gloss, and SHALL key the card and its status by the expression's dictionary form.
Being dictionary data, that gloss SHALL be stored on a card the reader creates, as a word's
gloss is. The card SHALL offer the actions of a word, "Je connais" included: a keyed
expression is a lexical unit the reader can settle. It SHALL show the expression as written
when it differs from the dictionary form, as a word card does. An expression covering only
part of a longer selection SHALL take the place, in the same list and within the same bound,
of the rows of the words it covers, and SHALL NOT be stored; an expression the reader has
marked known or ignored SHALL give no row there, as a settled compound gives none.

#### Scenario: A phrasal verb whose words the reader knows
- **WHEN** the reader selects `put up with` and the pack holds it
- **THEN** the card shows its gloss as the answer instead of stating that the pack has no translation

#### Scenario: An inflected expression makes one card
- **WHEN** the reader selects `gave up`, presses "+ Deck", then later selects `give up` and presses "+ Deck"
- **THEN** both act on one card, keyed `give up`, and its stored gloss is the expression's

#### Scenario: The form as seen
- **WHEN** the reader selects `gave up`
- **THEN** the card is headed `give up` and shows `gave up` as the form seen

#### Scenario: An expression inside a phrase
- **WHEN** the reader selects `a compelling starting point`, the pack holds `start point`, and the reader knows neither it nor `compelling`
- **THEN** the card shows a row for `start point` and a row for `compelling`, and no separate row for `start` or `point`

#### Scenario: An expression the reader has settled
- **WHEN** the reader has marked `start point` known and selects `a compelling starting point`
- **THEN** the card shows a row for `compelling` and none for the expression

#### Scenario: An idiom is not taken apart
- **WHEN** the reader selects `raining cats and dogs` and the pack holds the expression
- **THEN** the card shows its gloss, and no row for `cat` or `dog`

#### Scenario: No expression
- **WHEN** no expression matches the selection
- **THEN** the card behaves as it did: word-by-word rows, or the line stating the pack has no translation for the expression

## MODIFIED Requirements

### Requirement: Word-by-word gloss is a labelled last resort
A card SHALL show a word-by-word gloss only when it has no better answer for what the reader selected, and SHALL label it as not being a translation of the whole selection.
It applies to the card of a selection of several words and to the word card of a compound
the lexicon does not list. The candidates are the words of the selection; a compound the
lexicon does not list is replaced by its parts, alone or inside a phrase, unless the reader
has marked the compound itself known or ignored, in which case it gives no row. The rows
SHALL be limited to candidates the reader does not already know (unknown or learning),
SHALL leave out function words and proper nouns, SHALL list each dictionary form once with
the FIRST sense of its pack gloss — a gloss carries up to three, and a row is scanned, not
read — skipping any sense that says the definition is missing, and SHALL be bounded in
number. A word whose gloss holds no sense worth showing SHALL give no row. When no row
qualifies, the card SHALL state that the pack has no translation for the expression, and
SHALL show no row. The rows are a reading aid: they SHALL NOT be stored on a card, used as
a card's gloss, or sent anywhere.

#### Scenario: A free combination of words
- **WHEN** the reader selects `a compelling argument`, knows `argument`, and the pack glosses `compelling`
- **THEN** the card shows, under the word-by-word label, one row for `compelling` with its gloss, and no row for `a` or `argument`

#### Scenario: A gloss carrying several senses
- **WHEN** a row's word is glossed `Commencement, début, inauguration; Commencer, débuter, initier, entamer; Procédu`
- **THEN** the row shows `Commencement, début, inauguration`, so it does not end mid-word

#### Scenario: Nothing worth showing
- **WHEN** the reader selects `put up with`, already knows `put`, and the pack holds no expression covering the selection
- **THEN** the card shows no row and states that the pack has no translation for the expression

#### Scenario: A compound the lexicon does not list, on an analysed page
- **WHEN** the reader selects `error-prone` on a page the analysis covered, the lexicon does not list the compound, the reader knows `error`, and the pack glosses `prone`
- **THEN** the word card headed `error-prone` opens from its page token and shows, under the word-by-word label, one row for `prone` and none for `error`

#### Scenario: A compound inside a phrase
- **WHEN** the reader selects `an error-prone approach`, under the same conditions, and knows `approach`
- **THEN** the card shows one row, for `prone`

#### Scenario: A compound the reader has marked known
- **WHEN** the reader has marked `error-prone` known and selects `an error-prone approach`
- **THEN** the card shows no row for `prone`

#### Scenario: The rows never reach a card
- **WHEN** the reader presses "+ Deck" on a card showing word-by-word rows for a selection of several words
- **THEN** the phrase card is created with its source sentence and no gloss

#### Scenario: A long selection
- **WHEN** the reader selects a phrase holding more unknown content words than the bound
- **THEN** the card shows no more rows than the bound
