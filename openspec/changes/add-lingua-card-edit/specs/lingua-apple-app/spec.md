## MODIFIED Requirements

### Requirement: A review made in the app reaches the extension
What a review in the app changes — grades, cards marked known, words hidden, cards edited, renamed or merged, and the statuses an edit moves — SHALL reach the extension on the same device, which SHALL apply it as it applies changes from synchronisation, so that the more recent change of a card or a word wins on both sides and neither side's state replaces the other's. A card an edit in the app removed SHALL be removed in the extension too, unless the extension changed it after the edit.

#### Scenario: Back in Safari
- **WHEN** the reader reviews ten cards in the app, then opens Safari
- **THEN** the extension's due count no longer holds those cards

#### Scenario: Both changed
- **WHEN** the reader adds a word in Safari while a review runs in the app
- **THEN** both keep the new card and the review's grades

#### Scenario: A card corrected in the app
- **WHEN** the reader edits « colloquially referred to as » to « referred to as » and writes its translation in the app, then opens Safari
- **THEN** the extension holds « referred to as » with the reader's translation, marked as theirs, and no longer holds « colloquially referred to as »
