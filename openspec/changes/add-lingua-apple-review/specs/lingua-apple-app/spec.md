## ADDED Requirements

### Requirement: The app reviews the deck the extension keeps
The Cymbra Lingua app SHALL open on the review of the deck the extension keeps on the same device: each studied language's due count, a way to start a review, and the extension's own Révision page — its sessions, answers, read-aloud and end summary — laid out for the screen it is on. The app SHALL obtain the deck through the container it shares with the extension, never through a network request of its own. Until the extension has shared its state, the app SHALL say how to turn the extension on.

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
What a review in the app changes — grades, cards marked known, words hidden — SHALL reach the extension on the same device, which SHALL apply it as it applies changes from synchronisation, so that the more recent change of a card or a word wins on both sides and neither side's state replaces the other's.

#### Scenario: Back in Safari
- **WHEN** the reader reviews ten cards in the app, then opens Safari
- **THEN** the extension's due count no longer holds those cards

#### Scenario: Both changed
- **WHEN** the reader adds a word in Safari while a review runs in the app
- **THEN** both keep the new card and the review's grades
