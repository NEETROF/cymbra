## MODIFIED Requirements

### Requirement: Level-targeted deck feeding control
The extension SHALL offer a control to feed a deck from a chosen level ("Renforcer un
niveau"): the user picks a level, a word count (bounded by the seeding cap), and an order,
and the extension seeds that many words via the deck's level-targeted seeding, reporting how
many were added.
When a seeding adds no card, the control SHALL say so in one message, in the interface language,
that is true whichever reason left nothing to add: the level's remaining words are already tracked,
already in the deck, or have no translation in the reader's pack — in French « Aucune carte ajoutée —
ces mots sont déjà suivis, dans ton deck ou sans traduction. »

#### Scenario: Feed a level
- **WHEN** the user picks level B2, a count of 20, commonest-first, and confirms
- **THEN** up to 20 B2 words are added to the deck and the control reports the number added

#### Scenario: Nothing left to add
- **WHEN** a French-interface reader seeds a level whose remaining words are all in the deck or tracked, or have no translation in the reader's pack
- **THEN** the control reports « Aucune carte ajoutée — ces mots sont déjà suivis, dans ton deck ou sans traduction. », and an English- or Spanish-interface reader reads the same message in their language

#### Scenario: Only words without a translation are left
- **WHEN** a Spanish-native reader of French has every C2 word that fr-es glosses in the deck, and seeds C2 again
- **THEN** no card is added and the control reports the message in Spanish, which names words without a translation, while the ladder still counts C2's other words
