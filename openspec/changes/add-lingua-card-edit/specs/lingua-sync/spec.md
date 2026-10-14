## ADDED Requirements

### Requirement: A card removed by an edit stays removed on every device
A device SHALL record each card an edit removes — its studied language, its form and the removal time — SHALL send it as a deleted card operation, and SHALL apply a deleted card operation it receives under last-write-wins: the local card SHALL be removed unless it changed after the removal, and the removal SHALL be recorded even when the device holds no such card. A card operation for a removed card SHALL be refused when it is not later than the removal, and SHALL bring the card back, the removal forgotten, when it is later. Adding the same form again on the device — a capture, a seeding, an edit onto it — SHALL bring the card back. The server SHALL keep storing and returning removals as it does today.

#### Scenario: A rename reaches another device
- **WHEN** the reader renames « four-poster bed, » to « four-poster bed » on their iPhone, then their Mac synchronises
- **THEN** the Mac holds « four-poster bed » and no longer holds « four-poster bed, »

#### Scenario: A stale copy arrives later
- **WHEN** a device that removed a card by an edit receives an older copy of that card
- **THEN** the copy is refused and the card stays removed

#### Scenario: Captured again
- **WHEN** the reader later captures « four-poster bed, » again from a page
- **THEN** the card exists again, on this device and, after synchronisation, on the others

#### Scenario: Changed elsewhere after the removal
- **WHEN** a device receives the removal of a card it graded after the removal time
- **THEN** it keeps the card, and its next push brings the card back on the server

## MODIFIED Requirements

### Requirement: Complete card synchronisation, media excluded
`DeckService` SHALL synchronise complete cards — lemma, surface form, source sentence, gloss, whether the reader wrote that gloss, FSRS state — identified by a stable client id, last-write-wins per card (deletions included). The page a card was captured from stays on the device (`add-lingua-privacy-controls`): the card's `source` field is deprecated, sent empty and never stored. Whether the reader wrote the gloss SHALL be an optional field of the card operation, absent meaning not, stored and returned as received, so that clients built before it keep working. The contents of the `media` field SHALL NOT be synchronised in this change (the schema slot stays local).

#### Scenario: Card created on mobile, reviewed on desktop
- **WHEN** a card created while reading in Safari on iOS is synced and the user then opens the side panel on their Mac
- **THEN** the card appears there with its source sentence and FSRS state, and reviewing it on the Mac propagates back

#### Scenario: Card with an image
- **WHEN** a local card carries media and is synchronised
- **THEN** all of its fields go up except the media content, which stays on the originating device

#### Scenario: A translation the reader wrote
- **WHEN** the reader writes a card's translation on one device and another device synchronises
- **THEN** the other device holds that translation marked as the reader's, and its review shows it as written

#### Scenario: A client that predates the mark
- **WHEN** a client built before the mark existed pushes a card
- **THEN** the server stores the card as not written by the reader, and accepts it as before
