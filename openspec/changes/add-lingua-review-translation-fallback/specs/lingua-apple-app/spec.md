## MODIFIED Requirements

### Requirement: The app reviews the deck the extension keeps
The Cymbra Lingua app SHALL open on the review of the deck the extension keeps on the same device: the extension's own Révision page — its language filter and due count, its sessions, answers, read-aloud, machine translations and end summary — laid out for the screen it is on. The app SHALL obtain the deck through the container it shares with the extension, never through a network request of its own. Until the extension has shared its state, the app SHALL say how to turn the extension on.
The only network requests the app SHALL make are the download of translation models the reader
accepted, from Cymbra's model host, and, for online translation, the reader's sign-in and the
sentence sent when the reader presses « Traduction en ligne »; none SHALL carry the deck.

#### Scenario: A review on the train
- **WHEN** a reader who captured words in Safari opens the app on their iPhone
- **THEN** the app shows the due count and starts the same review the Safari drawer would, its answers at the bottom of the screen

#### Scenario: On a Mac
- **WHEN** the reader opens the app on their Mac
- **THEN** the review is shown in the app's window, as a centred column

#### Scenario: Before the extension
- **WHEN** the reader opens the app before turning the extension on
- **THEN** the app explains how to turn it on, and shows no review

### Requirement: A review made in the app reaches the extension
What a review in the app changes — grades, cards marked known, words hidden, glosses the reader kept — SHALL reach the extension on the same device, which SHALL apply it as it applies changes from synchronisation, so that the more recent change of a card or a word wins on both sides and neither side's state replaces the other's.

#### Scenario: Back in Safari
- **WHEN** the reader reviews ten cards in the app, then opens Safari
- **THEN** the extension's due count no longer holds those cards

#### Scenario: Both changed
- **WHEN** the reader adds a word in Safari while a review runs in the app
- **THEN** both keep the new card and the review's grades

#### Scenario: A gloss kept in the app
- **WHEN** the reader keeps « se relâcher » for « slacken » in the app, then opens Safari
- **THEN** the extension's card shows « se relâcher », chosen by the reader, and pushes it at its next synchronisation

## ADDED Requirements

### Requirement: The app translates on the device with its own copy of the models
The app SHALL translate review cards on the device with the extension's engine, run off the page's painting thread, on its own copy of the models, downloaded only after the reader accepts it in the app and only while « Traduction étendue » is on in the extension of that device.
The app SHALL offer the download once per missing pair, quietly in its review, stating its size
(French UI copy: « Traduire aussi dans l'app ? Téléchargement de 26 Mo »), and SHALL NOT download
without that acceptance. The models SHALL come from Cymbra's model host and SHALL be checked against
the catalogue the app carries before use. When « Traduction étendue » is off in the extension, the
app SHALL delete its copy at its next opening. Without its models, the app's review SHALL be as it
was before the engine existed.

#### Scenario: Offered once
- **WHEN** a reader with « Traduction étendue » on in Safari opens the app's review without the app's models
- **THEN** the review offers the download with its size, and nothing is downloaded until the reader accepts

#### Scenario: Setting turned off
- **WHEN** the reader turns « Traduction étendue » off in Safari and opens the app again
- **THEN** the app's models are deleted and its review shows no machine translation from the engine

#### Scenario: A model that does not match
- **WHEN** a downloaded file's checksum differs from the catalogue's
- **THEN** it is discarded, and the review behaves as without the model
