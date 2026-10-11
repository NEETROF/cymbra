## ADDED Requirements

### Requirement: Online translation is chosen in the settings, off by default
The settings SHALL offer « Traduction en ligne » only to a signed-in reader whose account has access to online translation, as a choice made on each device, unticked until the reader ticks it.
Access SHALL be granted to an account by an operator, one account at a time, and SHALL be read from
the server, never decided by the extension alone. Its note SHALL say that the selected sentence is
sent to an external translation service to be translated and is kept by no one, and SHALL link to
the privacy policy. It SHALL NOT name the service. A reader without access, signed in or not,
SHALL NOT be offered the setting, and a ticked setting SHALL have no effect while the reader has no
access. The setting SHALL be independent of « Traduction étendue »: either may be ticked without
the other.

#### Scenario: A reader who was given access
- **WHEN** a reader whose account was given access opens the settings for the first time on a device
- **THEN** « Traduction en ligne » is shown unticked, with a note saying the sentence leaves the device and is kept by no one

#### Scenario: A signed-in reader without access
- **WHEN** a signed-in reader whose account was not given access opens the settings
- **THEN** « Traduction en ligne » is not shown

#### Scenario: A signed-out reader
- **WHEN** a reader who is not signed in opens the settings
- **THEN** « Traduction en ligne » is not shown

#### Scenario: Another device
- **WHEN** the reader ticks the setting on one device and opens the settings on another
- **THEN** it is unticked on the other device

### Requirement: The online translation button appears only when chosen
A card that shows a selection of several words SHALL show a « Traduction en ligne » button if and only if the setting is ticked and the reader has access.
The button SHALL NOT depend on the on-device engine: it SHALL be shown whether or not a model is
present, and whether or not the engine has answered. A single word SHALL keep its dictionary card,
without the button. No text SHALL leave the device until the reader presses the button.

#### Scenario: Setting ticked, a phrase selected
- **WHEN** a reader with access who ticked the setting selects several words
- **THEN** the card shows the pack's answer, then the engine's translation if any, and a « Traduction en ligne » button

#### Scenario: Setting not ticked
- **WHEN** a reader who did not tick the setting selects several words
- **THEN** the card has no « Traduction en ligne » button, and no request leaves the device

#### Scenario: No model on the device
- **WHEN** a reader who ticked the setting but never downloaded the engine's model selects several words
- **THEN** the card shows the pack's answer and the button

#### Scenario: A single word
- **WHEN** the reader selects a single word
- **THEN** the card is the word's dictionary card, without the button

### Requirement: An online translation is shown in place of the engine's, as a machine translation
When the reader presses the button, the card SHALL ask once for the sentence holding the selection and SHALL show the answer, when it lands, in place of any translation it shows, under a label saying it is an online machine translation, with the selection's place marked.
While the answer is on its way the card SHALL say so and SHALL keep what it shows. The card SHALL
render every part of the answer as text. The label SHALL NOT name a provider. A card the reader has
since replaced, closed or acted on SHALL NOT be written over. Pressing the button again on the same
card SHALL NOT send the sentence again.

#### Scenario: The answer lands
- **WHEN** the reader presses the button and the answer arrives while the card is still open
- **THEN** the card shows the online translation under its label, with the selection marked, in place of the engine's translation

#### Scenario: The reader moved on
- **WHEN** the answer arrives after the reader closed the card or opened another
- **THEN** nothing is shown in its place

#### Scenario: Markup in the answer
- **WHEN** the online translation contains characters that would read as markup
- **THEN** the card shows them as text, and no element is created from them

### Requirement: The selection is marked in the online translation
The service SHALL return the translated sentence with marks on the words that render the reader's selection, and SHALL accept a rendering split into several segments.
The service SHALL place a mark only on text that occurs in the translated sentence, in order. When
the words cannot be found in the sentence, the answer SHALL carry the sentence without marks rather
than a misplaced mark.

#### Scenario: A phrasal verb split in the translation
- **WHEN** the reader selected "putting off" and the translation renders it as « remettait … à plus tard »
- **THEN** both « remettait » and « à plus tard » are marked

#### Scenario: Words not found
- **WHEN** the words the provider names do not occur in its translation
- **THEN** the card shows the translation without a mark

### Requirement: An unavailable online translation costs the reader nothing
When the service answers unavailable, refuses the request, or does not answer within its bound, the card SHALL keep what it shows and SHALL say, in the reader's language, that online translation is unavailable for now.
The message SHALL NOT expose a technical error. A refused request because the daily limit is
reached SHALL say so and SHALL say when it resets.

#### Scenario: No provider answers
- **WHEN** the reader presses the button and every provider fails
- **THEN** the card keeps the engine's translation or the pack's answer, and says online translation is unavailable for now

#### Scenario: The daily limit is reached
- **WHEN** the reader presses the button after using up the day's online translations
- **THEN** the card says the daily limit is reached and when it resets, and no provider is asked

### Requirement: Online translation is limited per account
The service SHALL serve only a signed-in reader whose account has access, identified by their access token, and SHALL count each answered request against a per-account limit over a rolling day.
The limit SHALL be tunable at runtime without a redeploy. An operator SHALL be able to grant access
to an account, withdraw it from an account, and withdraw it from every account at once, all without
a release; access is withdrawn from everyone until granted. A request from an account without
access SHALL be answered as unavailable without asking any provider. The service SHALL refuse a
sentence longer than a fixed bound, and the extension SHALL NOT send one.

#### Scenario: A request without an account
- **WHEN** a request reaches the service without a valid access token
- **THEN** it is refused as unauthenticated and no provider is asked

#### Scenario: Access withdrawn from everyone
- **WHEN** the operator withdraws access from every account and a reader who had it presses the button on a card already open
- **THEN** the card says online translation is unavailable for now, no provider is asked, and the setting is no longer shown once the extension next reads its access

#### Scenario: Access granted on request
- **WHEN** the operator grants access to one reader's account
- **THEN** that reader is offered the setting once the extension next reads its access, and no other reader is

### Requirement: Online translation goes straight from the document's language to the reader's
The card SHALL ask for a translation from the language of the selection's document into the reader's native language, and the service SHALL translate directly between them, without a pivot language.
The service SHALL accept a pair whose two languages are among Lingua's languages and differ, and
SHALL refuse any other pair. The button SHALL be offered on every pair the reader reads, including
a pair the on-device engine translates through English or does not mark.

#### Scenario: A French reader of a Spanish book
- **WHEN** a reader whose native language is French presses the button on a Spanish sentence
- **THEN** the sentence is translated from Spanish into French directly, with the selection marked

#### Scenario: An English reader of a French book
- **WHEN** a reader whose native language is English presses the button on a French sentence
- **THEN** the sentence is translated from French into English, and the card's copy is in English

### Requirement: The online translation service tries a second provider before giving up
The service SHALL ask a primary provider and, when it fails, times out or answers something it cannot read, a secondary provider, and SHALL answer unavailable only when both fail.
The reader SHALL see the same card whichever provider answered.

#### Scenario: The primary provider is down
- **WHEN** the primary provider returns an error and the secondary answers
- **THEN** the reader sees the secondary's translation, under the same label

### Requirement: The online translation service keeps nothing it is given
The service SHALL NOT write a sentence it receives, or a translation it returns, to a log, a store, a metric, a trace or an error report.
It MAY record, per request, the account, the time, the pair, the provider that answered, the
latency, and the token counts. A provider SHALL be used only under terms that exclude training on
the request.

#### Scenario: A provider error
- **WHEN** a provider fails and the service logs the failure
- **THEN** the log holds the provider, the status and the latency, and no sentence

## MODIFIED Requirements

### Requirement: A machine translation is never stored
A translation produced by the engine or by the online translation service SHALL be computed for display and discarded, and SHALL NOT be stored on a card, used as a card's gloss, or sent anywhere.
Only dictionary data SHALL be stored as a card's answer. This holds whatever surface asked for
the translation. The one exception to "sent anywhere" is the sentence the reader sends to the
online translation service by pressing its button; nothing else SHALL be sent, and no translation
SHALL be sent back to Cymbra once shown.

#### Scenario: A card made from a translated selection
- **WHEN** the reader creates a card from a selection the engine translated
- **THEN** the card holds its source sentence and its dictionary data, and no machine translation

#### Scenario: A card made after an online translation
- **WHEN** the reader creates a card from a selection after pressing « Traduction en ligne »
- **THEN** the card holds its source sentence and its dictionary data, and not the online translation
