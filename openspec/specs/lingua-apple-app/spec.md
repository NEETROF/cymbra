# lingua-apple-app Specification

## Purpose
TBD - created by archiving change add-lingua-release-pipeline. Update Purpose after archive.
## Requirements
### Requirement: Versioned App Store releases

The host application's shipped version SHALL come from the release tooling — one file the tooling maintains, stamped onto the build at archive time — and SHALL NOT be a value written by hand in the Xcode project, which CI already rewrites for distribution signing.

Pushing an application release tag SHALL build that tag, stamp its version, and deliver both platforms to App Store Connect. A tag whose version disagrees with the maintained file SHALL stop the run. Building from a branch SHALL remain possible and SHALL keep its explicit opt-in before anything is delivered, so a dispatch cannot deliver branch code by surprise.

Its version SHALL be independent of the extension's: the two are released separately and are not read as a pair.

#### Scenario: A tag ships a versioned build
- **WHEN** an application release tag is pushed
- **THEN** that tag is built, both packages carry its version, and both are delivered to App Store Connect

#### Scenario: A tag that disagrees with the maintained version
- **WHEN** the tag's version and the maintained file differ
- **THEN** the run stops before building, rather than shipping a version the changelog does not describe

#### Scenario: A dispatch from a branch
- **WHEN** the lane is dispatched without a tag
- **THEN** it builds and signs from that branch and delivers nothing unless delivery was asked for explicitly

### Requirement: The app reviews the deck the extension keeps
The Cymbra Lingua app SHALL open on the review of the deck the extension keeps on the same device: the extension's own Révision page — its language filter and due count, its sessions, answers, read-aloud and end summary — laid out for the screen it is on. The app SHALL obtain the deck through the container it shares with the extension, never through a network request of its own. Until the extension has shared its state, the app SHALL say how to turn the extension on.

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

