## ADDED Requirements

### Requirement: A selected word resolves to its dictionary form
A selection holding no whitespace that no page token covers SHALL be read by the analyser, and SHALL open the word card of the first token the analyser finds in it, as a page token would have.
That card SHALL show the dictionary form as its headword, the form as seen when it differs,
the pack gloss and the actions matching the token's real status, and its actions SHALL act
on the dictionary form: the card and the status are keyed by it, not by the form as written.
A hyphen SHALL NOT set a selection apart: a hyphenated selection opens the word card like
any other selection holding no whitespace, from its page token when there is one. When the
analyser classes that first token as a proper noun outside the lexicon, or finds no token,
the selection SHALL open the same card as before this change, headed and keyed by the text
as written. The card a selection opens SHALL still be open once the pointer gesture that
made the selection has ended, and a click that lands on an untreated word inside a link
SHALL still not follow the link.

#### Scenario: An inflected word in a block the page analysis skipped
- **WHEN** the reader selects `endeavors` in a block the page analysis did not cover and presses "+ Deck"
- **THEN** the card shows the headword `endeavor` with `endeavors` as the form seen, the created card and the "learning" status are keyed by `endeavor`, and `endeavors` is highlighted as "learning" wherever the page is analysed

#### Scenario: A contraction in a block the page analysis skipped
- **WHEN** the reader selects `don't` in a block the page analysis did not cover
- **THEN** the word card opens for `do`, as it does where the page analysis covers the block

#### Scenario: A compound the pack lists
- **WHEN** the reader selects `well-known` on an analysed page and the pack glosses `well-known`
- **THEN** the word card opens with that gloss and the actions matching the word's status, "Je connais" included

#### Scenario: A name
- **WHEN** the reader selects `Jenkins` and the analyser classes it as a proper noun outside the lexicon
- **THEN** the card is headed and keyed by `Jenkins` as written, with the actions it offered before this change

#### Scenario: Selecting with the mouse
- **WHEN** the reader double-clicks or drags across a single word in a block the page analysis did not cover
- **THEN** the card the selection opened is still open after the click that ends the gesture

#### Scenario: Double-clicking an unknown word inside a link
- **WHEN** the reader double-clicks a highlighted unknown word that sits inside a link
- **THEN** its card opens and neither click follows the link

### Requirement: Word-by-word gloss is a labelled last resort
A card SHALL show a word-by-word gloss only when it has no better answer for what the reader selected, and SHALL label it as not being a translation of the expression.
It applies to the card of a selection of several words and to the word card of a compound
the lexicon does not list. The candidates are the words of the selection; a compound the
lexicon does not list is replaced by its parts, alone or inside a phrase, unless the reader
has marked the compound itself known or ignored, in which case it gives no row. The rows
SHALL be limited to candidates the reader does not already know (unknown or learning),
SHALL leave out function words and proper nouns, SHALL list each dictionary form once with
its pack gloss, and SHALL be bounded in number. When no row qualifies, the card SHALL state
that the pack has no translation for the expression, and SHALL show no row. The rows are a
reading aid: they SHALL NOT be stored on a card, used as a card's gloss, or sent anywhere.

#### Scenario: A free combination of words
- **WHEN** the reader selects `a compelling argument`, knows `argument`, and the pack glosses `compelling`
- **THEN** the card shows, under the word-by-word label, one row for `compelling` with its gloss, and no row for `a` or `argument`

#### Scenario: Nothing worth showing
- **WHEN** the reader selects `put up with` and already knows `put`
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

### Requirement: A known word shows its gloss
Opening the card of a known or ignored word SHALL show its pack gloss, as it does for a word the reader does not know.

#### Scenario: Opening a known word
- **WHEN** the reader opens the card of a word whose status is "known" and the pack glosses it
- **THEN** the card shows that gloss rather than the absence of a translation

### Requirement: A card that waits for the engine
A card that needs an answer from the engine SHALL open at once in a pending state that offers no action, and SHALL become complete exactly once — with the engine's answer, or with a fallback when the engine fails or stays silent past a bounded wait — so that nothing the reader can press moves or changes meaning afterwards.
The fallback SHALL state that the pack has no translation, and SHALL offer actions only when
their key is known without the answer: the text of a selection of several words, or the
dictionary form of a page token. A word whose dictionary form never arrived SHALL offer no
action. An answer SHALL be applied only to the pending card that asked for it: a card
replaced or closed in the meantime discards it, and so does a card already completed by its
fallback.

#### Scenario: An engine that takes time to wake
- **WHEN** the reader selects three words while the engine's host is suspended
- **THEN** the card is visible at once in its pending state, and its answer and its actions appear together when the engine replies

#### Scenario: An answer that arrives too late
- **WHEN** the reader extends the selection before the answer to the first selection has arrived
- **THEN** the card shows only the answer to the latest selection

#### Scenario: Another card opened in the meantime
- **WHEN** the reader opens a highlighted word's card while an answer for an earlier selection is pending, and that answer then arrives
- **THEN** the highlighted word's card is left as it was

#### Scenario: A pending card closed by the reader
- **WHEN** the reader closes a pending card with its close button and the answer then arrives
- **THEN** no card reopens

#### Scenario: An engine that does not answer, for a selection of several words
- **WHEN** the engine fails or stays silent past the bounded wait
- **THEN** the card states that the pack has no translation and offers "+ Deck" and "Ignorer", keyed by the selection's text

#### Scenario: An engine that does not answer, for a known word
- **WHEN** the reader opens a known word's card and the engine fails or stays silent past the bounded wait
- **THEN** the card states that the pack has no translation and offers the actions of the word's status, keyed by its dictionary form

#### Scenario: An engine that does not answer, for a word outside the page analysis
- **WHEN** the reader selects `endeavors` in a block the page analysis did not cover and the engine fails or stays silent past the bounded wait
- **THEN** the card states that the pack has no translation and offers no action, so that nothing is keyed by `endeavors`

#### Scenario: An answer that arrives after the fallback
- **WHEN** the bounded wait has passed, the card has been completed by its fallback, and the engine's answer then arrives
- **THEN** the card is left as the fallback made it
