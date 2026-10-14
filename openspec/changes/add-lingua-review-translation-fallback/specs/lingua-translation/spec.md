## RENAMED Requirements

- FROM: `### Requirement: A machine translation is never stored`
  TO: `### Requirement: A machine translation is stored only when the reader keeps it`

## MODIFIED Requirements

### Requirement: A machine translation is stored only when the reader keeps it
A translation produced by the engine or by the online translation service SHALL be computed for display and discarded, and SHALL NOT be stored on a card, used as a card's gloss, or sent anywhere, unless the reader keeps it by an explicit action in review.
Only dictionary data, or a translation the reader kept, SHALL be stored as a card's answer. This
holds whatever surface asked for the translation. When the reader keeps a translation, only the
translation of the card's word — the marked part of the translation, or the reader's correction of
it — SHALL be stored, as the card's gloss, marked as chosen by the reader, and SHALL travel with the
card wherever the card goes (backup, synchronisation, the relay between the app and the extension);
the translated sentence SHALL NOT be stored. The one exception to "sent anywhere" otherwise is the
sentence the reader sends to the online translation service by pressing its button; nothing else
SHALL be sent, and no translation SHALL be sent back to Cymbra once shown except as a kept gloss
inside its card.

#### Scenario: A card made from a translated selection
- **WHEN** the reader creates a card from a selection the engine translated
- **THEN** the card holds its source sentence and its dictionary data, and no machine translation

#### Scenario: A card made after an online translation
- **WHEN** the reader creates a card from a selection after pressing « Traduction en ligne »
- **THEN** the card holds its source sentence and its dictionary data, and not the online translation

#### Scenario: A translation shown in review and not kept
- **WHEN** a review shows a machine translation of a card and the reader answers without keeping it
- **THEN** nothing of the translation is in the card, the backup or what is synchronised

#### Scenario: A translation kept in review
- **WHEN** the reader keeps the translation of « slacken » shown as « Il sentit sa prise se **relâcher** »
- **THEN** the card's gloss becomes « se relâcher », marked as chosen by the reader, in the reader's native language, and the translated sentence is stored nowhere

### Requirement: The review translates a card the pack cannot answer
When a card without a gloss comes up in a review — an expression, or a single word the pack does not gloss — and this device has a model ready for the card's language, the review SHALL ask the engine for the card's sentence translated with the word's place marked, as soon as the card comes up, and SHALL show that translation with the answer once revealed, labelled as a machine translation (French UI copy: « Dans votre phrase — traduction automatique ») and rendered as text. This SHALL hold in every host of the review: the in-page drawer, the side panel, the EPUB reader and the Cymbra Lingua app. An expression whose sentence does not hold it SHALL be translated alone, under a label saying it is a machine translation of the expression (French UI copy: « Traduction automatique »); a single word SHALL never be translated alone. A card with a gloss SHALL show its gloss, and the engine SHALL NOT be asked for it unless the reader asks, once revealed, through a link (French UI copy: « Traduire la phrase ») shown only when a model is ready or online translation is offered for that card; the link SHALL be a small text control, not a block. The translation SHALL NOT show before the reveal, SHALL NOT hold the reveal back, and SHALL be discarded with the card unless the reader keeps it.

#### Scenario: An expression in review
- **WHEN** the card of « such as those made », which has no gloss, comes up and a model is ready for English
- **THEN** once the answer is revealed, it shows the card's sentence translated with the expression's place marked, under the machine-translation label, and nothing of it shows before the reveal

#### Scenario: Ready at the reveal
- **WHEN** the reader reveals the answer after the translation has arrived
- **THEN** the translation is part of the answer from that moment, with nothing added afterwards

#### Scenario: Still on its way
- **WHEN** the reader reveals the answer before the translation has arrived
- **THEN** the answer says that a translation is coming, the translation replaces that line when it lands, and if none comes the answer keeps saying the card has no translation

#### Scenario: A word the pack glosses
- **WHEN** a card with a gloss comes up and the reader reveals it without tapping « Traduire la phrase »
- **THEN** the engine is not asked, and the answer is the gloss, with the link beside it when a model is ready or online translation is offered

#### Scenario: A useless gloss
- **WHEN** the reader reveals « would », whose gloss is useless, and taps « Traduire la phrase » with a model ready for English
- **THEN** the card's sentence is translated with the word marked and shown under the gloss, labelled as a machine translation

#### Scenario: No model
- **WHEN** extended translation is off on this device, or the models of the card's language are not on it, and online translation is not offered
- **THEN** the review is as it was before the engine existed, with no line about a translation and no « Traduire la phrase » link

#### Scenario: A sentence that does not hold the expression
- **WHEN** an expression's card holds a sentence the expression is not in
- **THEN** the expression is translated alone, under the label of a machine translation of the expression

#### Scenario: Another card
- **WHEN** a translation lands after the reader has answered its card
- **THEN** it is shown nowhere

#### Scenario: Never stored
- **WHEN** the reader backs the deck up, or the deck synchronises, after a review where no translation was kept
- **THEN** no machine translation is in the backup or in what is sent

#### Scenario: In the app
- **WHEN** the card of « oppressive », which has no gloss, comes up in the Cymbra Lingua app on an iPhone whose app holds the English models
- **THEN** once revealed, the answer shows the sentence translated on the device with the word marked, as the Safari drawer would

## ADDED Requirements

### Requirement: The review offers online translation under the selection card's rule
A revealed review card SHALL show a « Traduction en ligne » button if and only if the « Traduction en ligne » setting is ticked on this device and the reader's account has access, and the card has no gloss or the reader tapped « Traduire la phrase ».
Unlike the selection card, a card of a single word SHALL be offered the button. Pressing it SHALL
send the card's sentence with the word's place as the selection, the card's language and the
reader's native language — never the word alone; an expression its sentence does not hold SHALL be
sent alone as the whole selection, and a single word its sentence does not hold SHALL NOT be offered
the button. The answer SHALL be shown in place of the engine's, labelled as an online machine
translation, with the word marked, rendered as text. A second press on the same card in the same
review SHALL NOT send again; an answer landing after the card was answered SHALL be shown nowhere;
an unavailable service or a reached daily limit SHALL read as on the selection card. No text SHALL
leave the device until the reader presses the button. This SHALL hold in every host of the review,
the Cymbra Lingua app included.

#### Scenario: A word the pack does not know
- **WHEN** a reader with access who ticked the setting reveals « slacken », which has no gloss
- **THEN** the button is shown, and pressing it sends the card's sentence with « slacken » marked by position

#### Scenario: Setting not ticked
- **WHEN** a reader who did not tick the setting reveals a card without a gloss
- **THEN** no button is shown and no request leaves the device

#### Scenario: A glossed card
- **WHEN** a reader with access who ticked the setting reveals a card with a gloss
- **THEN** no button is shown until the reader taps « Traduire la phrase »

#### Scenario: Pressed twice
- **WHEN** the reader presses the button again on the same card
- **THEN** the sentence is not sent a second time

#### Scenario: The daily limit is reached
- **WHEN** the reader presses the button after using up the day's online translations
- **THEN** the answer keeps what it shows and says the daily limit is reached and when it resets

### Requirement: The reader can keep a machine translation as the card's gloss
Under a machine translation shown in review, engine or online, the review SHALL offer an action that keeps the translation of the card's word as the card's gloss, and SHALL show what it keeps before it is kept.
The candidate SHALL be the marked text of the translation — several marks joined in order by « … » —
or, for an expression translated alone, the whole translation. A candidate that is non-empty, in
range, at most 80 characters and drawn from a single mark — or an expression translated alone —
SHALL be kept in one tap from an action naming it (French UI copy:
« Garder « se relâcher » »), with a neighbouring « Modifier » opening a one-line field prefilled with
it. Otherwise the action (French UI copy: « Garder une traduction… ») SHALL open that field
prefilled with the candidate, or empty without one, and SHALL NOT save while the trimmed field is
empty or longer than 80 characters. Keeping SHALL replace the card's gloss, glossed or not, with the
kept text in the reader's native language, marked as chosen by the reader, SHALL date the card's
change, and SHALL show the new gloss in the answer at once. Every control of it SHALL be a small text
control, with no transition.

#### Scenario: One tap
- **WHEN** the translation marks « se relâcher » for « slacken » and the reader taps « Garder « se relâcher » »
- **THEN** the card's gloss is « se relâcher », the answer shows it, and the action is gone

#### Scenario: No mark
- **WHEN** the translation carries no mark and the reader taps « Garder une traduction… »
- **THEN** an empty field opens, « Enregistrer » stays disabled until the reader types a translation, and the typed text becomes the gloss

#### Scenario: A split mark
- **WHEN** the translation marks « remettait » and « à plus tard » for « putting off »
- **THEN** no action keeps it in one tap: « Garder une traduction… » opens the field prefilled with « remettait … à plus tard », and the reader can save « remettre à plus tard » instead

#### Scenario: Replacing a useless gloss
- **WHEN** the reader keeps « aurait » under the translation of a « would » card whose gloss was the pack's
- **THEN** the card's gloss is « aurait », and the next review of that card shows « aurait » without asking any engine
