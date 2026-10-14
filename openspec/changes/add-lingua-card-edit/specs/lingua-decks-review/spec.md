## ADDED Requirements

### Requirement: The reader can edit the card under review
The review card SHALL offer an edit action (French UI copy: « Modifier ») beside marking the word known and hiding it from review, before and after the reveal, in every review host — the side panel, the in-page drawer on every browser, and the Cymbra Lingua app. Choosing it SHALL reveal the answer when it is hidden, as the reveal action does, and SHALL open, in place of the answer and the answers, a form with two labelled fields: the card's word or expression (French UI copy: « Mot ou expression »), filled with its dictionary form, and its translation (French UI copy: « Traduction »), filled with the gloss the review shows for the card and empty when it shows none — never with a machine translation. The form SHALL offer to save (French UI copy: « Enregistrer ») and to cancel (French UI copy: « Annuler »); Enter in a field SHALL save, Escape SHALL cancel, and the review's own keys SHALL NOT act while the focus is in the form. Opening and closing the form SHALL each be one repaint, without transition. A save that changes nothing, and a cancel, SHALL close the form and write nothing. After a save, the card SHALL stay the current card, revealed, showing its new form and gloss; its front SHALL mark the new form in the card's sentence when the sentence holds it, and SHALL otherwise show the form above the sentence; the sentence SHALL NOT change. The edit SHALL NOT count as an answer or a review, and SHALL NOT move the card's schedule. Every label, button and error of the form SHALL exist in each interface language.

#### Scenario: A free selection trimmed
- **WHEN** the card « colloquially referred to as » is revealed and the reader edits its form to « referred to as » and saves
- **THEN** the card shows « referred to as », marked in the same sentence, still revealed, and the reader answers it as usual

#### Scenario: Editing before the reveal
- **WHEN** the reader chooses « Modifier » on a card whose answer is hidden
- **THEN** the answer is revealed and the form opens in the same repaint

#### Scenario: Writing a missing translation
- **WHEN** a card without a gloss is revealed and the reader types a translation and saves
- **THEN** the card shows that translation as its gloss, and shows it again at its next review

#### Scenario: Cancelling
- **WHEN** the reader opens the form, changes both fields, then cancels
- **THEN** the card is as it was and nothing is saved

#### Scenario: Typing in the form
- **WHEN** the reader presses Space or the arrow keys while typing in a field
- **THEN** the characters go into the field, and the card is neither revealed nor answered

#### Scenario: A form the sentence does not hold
- **WHEN** the reader edits a card's form to words its sentence does not contain
- **THEN** the front shows the new form above the unchanged sentence

#### Scenario: In every host
- **WHEN** the reader edits a card in the side panel, in the in-page drawer, or in the Cymbra Lingua app
- **THEN** the form, its copy and its result are the same, in the reader's interface language

### Requirement: An edited form is checked before it is saved
An edited form SHALL be trimmed, its inner runs of whitespace collapsed to one space, and its leading and trailing punctuation stripped by the same rule capture applies to a selection; it SHALL then be refused when it is empty or longer than 120 characters, with a message in the interface language tied to the field and no technical wording (French UI copy: « Écrivez un mot ou une expression. », « 120 caractères au plus. »). An edited translation SHALL be trimmed and collapsed, and refused when longer than 200 characters (French UI copy: « 200 caractères au plus. »); an empty translation SHALL be accepted as the reader having none. Letter case SHALL be kept as typed. The card SHALL stay in its studied language, and the form SHALL NOT be refused for being absent from the dictionary. A refused save SHALL change nothing.

#### Scenario: Trailing punctuation
- **WHEN** the reader saves the form « four-poster bed, »
- **THEN** the card's form becomes « four-poster bed »

#### Scenario: An empty form
- **WHEN** the reader saves a form made only of spaces or punctuation
- **THEN** the save is refused, the field says a word or an expression is needed, and the card is unchanged

#### Scenario: Too long
- **WHEN** the reader saves a form of 121 characters
- **THEN** the save is refused with the length limit stated, and the card is unchanged

#### Scenario: A word the dictionary does not know
- **WHEN** the reader saves a form the pack has no entry for
- **THEN** the form is saved

### Requirement: An edit settles the card's gloss, the reader's own first
On a save, the card's gloss SHALL be settled in this order: a translation the reader changed to some text SHALL become the gloss and SHALL be marked as written by the reader; a translation the reader emptied SHALL give the pack's gloss for the card's form when the pack of the card's language has one, and no gloss otherwise, unmarked; a translation left as it was filled, with the form changed, SHALL keep a gloss the reader wrote, and otherwise SHALL give the pack's gloss for the new form when the pack has one, and otherwise SHALL keep the card's previous text. A gloss taken from the pack, or written by the reader, SHALL be labelled with the engine's native language. The mark of a reader-written gloss SHALL be kept when the card is backed up, synchronised or relayed by the Cymbra Lingua app, and SHALL stay on a card whose incoming copy carries no mark but the same gloss text.

#### Scenario: The pack knows the new form
- **WHEN** the reader edits « colloquially referred to as » to « referred to as », leaves the translation as it was, and the pack glosses « referred to as »
- **THEN** the card's gloss is the pack's gloss for « referred to as »

#### Scenario: The pack does not know the new form
- **WHEN** the reader edits the form, leaves the translation as it was, and the pack has no gloss for the new form
- **THEN** the card keeps the translation the reader saw in the field

#### Scenario: The reader's translation wins
- **WHEN** the reader edits the form and types a translation, and the pack glosses the new form otherwise
- **THEN** the card's gloss is the reader's translation, marked as theirs

#### Scenario: A reader's translation outlives a later form edit
- **WHEN** a card whose gloss the reader wrote has its form edited, the translation left as it was
- **THEN** the reader's translation stays, even where the pack glosses the new form

#### Scenario: Withdrawing one's translation
- **WHEN** the reader empties the translation of a card whose gloss they wrote, and saves
- **THEN** the card shows the pack's gloss for its form, or says it has no translation when the pack has none

#### Scenario: An older copy echoed back
- **WHEN** a card whose gloss the reader wrote comes back from synchronisation without the mark but with the same gloss text
- **THEN** the card keeps the mark

### Requirement: An edited form already in the deck merges the two cards
When an edited form equals, letter case aside, the form of another card of the same studied language, the two cards SHALL become one card under that other card's form as the deck spells it, and the edited card SHALL be removed. The merged card SHALL keep the review state of the card with more reviews — on a tie, the one reviewed most recently; on a further tie, the other card's; SHALL keep the reader-written gloss of the edited card, else that of the other card, else the gloss the edit settled, else the other card's gloss; SHALL keep the edited card's sentence, source and encountered form, and the earlier of the two capture times. In the session, the merged card SHALL stay the current card, revealed; the other card SHALL NOT come up again in that session; and when the other card was already answered in that session, the merged card's next answer SHALL NOT move its schedule. The card SHALL say once, after the save, that it was merged with the card already in the deck (French UI copy: « Réunie avec la carte « referred to as » déjà dans vos révisions. »). A form that differs from the edited card's own only by letter case SHALL rename the card, not merge it.

#### Scenario: Two cards for one expression
- **WHEN** the deck holds « referred to as », reviewed six times, and the reader edits « colloquially referred to as », reviewed twice, to « referred to as »
- **THEN** one card « referred to as » remains, with the six-review history, the sentence of the edited card, and the earlier capture time, and the card says it was merged

#### Scenario: The edited card has the longer history
- **WHEN** the edited card has more reviews than the card already in the deck
- **THEN** the merged card keeps the edited card's review state

#### Scenario: The reader's translation survives the merge
- **WHEN** the card already in the deck has a gloss the reader wrote, and the edit leaves the translation as it was
- **THEN** the merged card shows the reader's gloss

#### Scenario: The same word later in the session
- **WHEN** the card merged into was due later in the same session
- **THEN** it does not come up again in that session

#### Scenario: Only the letter case
- **WHEN** the reader edits « Referred to as » to « referred to as » and no other card holds that form
- **THEN** the card is renamed to « referred to as », with its history

### Requirement: A renamed card moves its word's status
When an edit changes a card's form, the old form's status SHALL be cleared when it is `learning`, and left otherwise; the new form SHALL become `learning` when it holds no explicit status, and SHALL keep any status it holds. Each change SHALL be stamped with the edit's time, and synchronise as any status change does. When the new form is `known` or `ignored`, the card SHALL be retired and SHALL leave the session, as when the reader marks a word known in review.

#### Scenario: A rename
- **WHEN** the reader edits « four-poster bed, », which is `learning`, to « four-poster bed », which holds no status
- **THEN** « four-poster bed, » no longer has a status and « four-poster bed » is `learning`, and both changes reach the reader's other devices

#### Scenario: A form already known
- **WHEN** the reader edits a card to a form they marked known earlier
- **THEN** that form stays known, the card is retired, and the review moves to the next card

### Requirement: The backup keeps a card's edits
The backup SHALL hold whether a card's gloss was written by the reader, and the cards removed from the deck by an edit with their removal times, so that a restore reproduces both. Both SHALL be left out when absent, so that a backup of a deck without an edit keeps its bytes; neither SHALL raise the backup's schema version, and a build released before this requirement SHALL read a backup that carries them.

#### Scenario: A deck without an edit
- **WHEN** a deck in which no card was ever edited is backed up
- **THEN** the backup is byte for byte what the previous build wrote

#### Scenario: Round trip
- **WHEN** a deck with a renamed card and a reader-written gloss is backed up and restored
- **THEN** the restored deck holds the renamed card with its marked gloss, and refuses an older copy of the removed card as the original deck does

#### Scenario: A previous build
- **WHEN** a build released before this requirement restores a backup that carries edits
- **THEN** the restore succeeds and the cards are there

## MODIFIED Requirements

### Requirement: Review shows a gloss the reader can read
When the gloss of the card under review is in a language other than the engine's native language, review SHALL show the gloss of the pack held for the card's studied language — a word's gloss, or an expression's from the pack's expression table — and SHALL show the card's own text when that pack has none or none is held. A card whose gloss is in the engine's native language SHALL be shown as before. A gloss the reader wrote SHALL always be shown as written, whatever its language, and SHALL never be replaced by the pack's. The card's stored gloss and label SHALL NOT be rewritten by review, and the substitution SHALL NOT add or rename a key of the view.

#### Scenario: The English baseline
- **WHEN** the English baseline runs
- **THEN** every review view is byte for byte as pinned

#### Scenario: A word glossed in another native language
- **WHEN** a French-native engine reviews a Spanish word card whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for that lemma, and the card keeps its English text and label

#### Scenario: An expression glossed in another native language
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key

#### Scenario: A gloss the pack has not
- **WHEN** the card's lemma is not glossed by the current pack
- **THEN** the view shows the card's own text, in its language

#### Scenario: A gloss the reader wrote in another language
- **WHEN** a French-native engine reviews a Spanish card whose gloss the reader wrote, labelled English, and es-fr glosses that lemma
- **THEN** the view's gloss is the reader's text
