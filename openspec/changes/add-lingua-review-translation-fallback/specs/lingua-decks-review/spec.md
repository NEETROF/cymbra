## MODIFIED Requirements

### Requirement: Review shows a gloss the reader can read
Review SHALL show, for every card, the first of: a gloss the reader chose for the card — a machine translation they kept, or a translation they wrote — as written, whatever its language; the gloss of the current pack held for the card's studied language — a word's gloss, or an expression's from the pack's expression table; the card's own stored text.
This precedence SHALL apply whatever language the card's gloss is written in, so that a pack fixed
after a card was captured reaches that card at its next review, and a gloss the reader chose is
never replaced by the pack's. The precedence SHALL be display only: the card's stored gloss, its
label and its mark SHALL NOT be rewritten by review, and the substitution SHALL NOT add or rename a
key of the view. A card without a gloss SHALL be read through the same pack lookup
(`fix-lingua-selection-card-punctuation`).

#### Scenario: The English baseline
- **WHEN** the English baseline runs
- **THEN** every review view is byte for byte as pinned, the views of cards whose stored gloss differs from the current pack's re-pinned to the pack's gloss

#### Scenario: A word glossed in another native language
- **WHEN** a French-native engine reviews a Spanish word card whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for that lemma, and the card keeps its English text and label

#### Scenario: An expression glossed in another native language
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key

#### Scenario: A gloss the pack has not
- **WHEN** the card's lemma is not glossed by the current pack
- **THEN** the view shows the card's own text, in its language

#### Scenario: A pack fixed after the capture
- **WHEN** a « would » card captured with the old, useless gloss is reviewed after the pack's gloss for « would » was fixed
- **THEN** the view's gloss is the fixed pack gloss, and the card still stores the old text

#### Scenario: A gloss the reader chose
- **WHEN** a card whose gloss the reader chose is reviewed and the pack glosses its lemma
- **THEN** the view's gloss is the reader's

#### Scenario: A gloss the reader chose in another language
- **WHEN** a French-native engine reviews a Spanish card whose gloss the reader chose, labelled English, and es-fr glosses that lemma
- **THEN** the view's gloss is the reader's text

### Requirement: Card schema with provenance
A card SHALL carry at minimum: the lemma, the encountered form, the originating context
sentence, the source of the encounter (URL or agent session identifier, timestamp), the
gloss, whether the reader chose that gloss, and an optional media slot (`media`, with
`source: capture|stock|generated` and `sync_policy`) — not populated in this change but present in
the schema. Multi-word expressions SHALL be cards in their own right.

#### Scenario: Card created from the popup
- **WHEN** the user adds `conundrum` to the deck from a web page
- **THEN** the card holds the lemma, the complete originating sentence and the page URL

#### Scenario: Expression card
- **WHEN** the user captures the selection "compelling starting point" with the keyboard shortcut
- **THEN** an expression card is created with the originating sentence

## ADDED Requirements

### Requirement: A gloss the reader kept is the card's gloss and wins over the pack
A card SHALL carry whether the reader chose its gloss — by keeping a machine translation in review, or by writing it — absent meaning not, which is every card before this requirement and every card added from the pack or a surface.
Keeping a machine translation SHALL set the mark. A gloss the reader chose SHALL NOT be replaced by
the pack's: adding the same word to the deck again SHALL keep that gloss, its language and its mark
while refreshing the rest of the card, and any rewrite of stored glosses from a newer pack SHALL
skip it. In review it SHALL come first, as « Review shows a gloss the reader can read » orders it,
and no engine SHALL be asked for it unless the reader asks. The backup SHALL carry the mark only
when it is set, so that every backup without a chosen gloss keeps its bytes; the backup's schema
version SHALL NOT change for it, and a build released before this requirement SHALL read a backup
that carries it. The local export of card operations SHALL emit the mark only when it is set, and
the apply SHALL read it. There SHALL be one such mark, shared by every way the reader chooses a
gloss.

#### Scenario: Kept, then captured again
- **WHEN** the reader kept « se relâcher » for « slacken », then adds « slacken » to the deck again from another page
- **THEN** the card takes the new sentence and keeps « se relâcher », marked as the reader's

#### Scenario: A pack that knows the word later
- **WHEN** a newer pack glosses « slacken » after the reader kept « se relâcher »
- **THEN** the review still shows « se relâcher »

#### Scenario: Every backup today
- **WHEN** a deck with no chosen gloss is backed up
- **THEN** the backup is byte for byte what the previous build wrote

#### Scenario: An older build reads the backup
- **WHEN** a build released before this requirement restores a backup holding a kept gloss
- **THEN** it restores the card with its gloss, without the mark, and refuses nothing
