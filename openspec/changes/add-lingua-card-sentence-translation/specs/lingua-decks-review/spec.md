## ADDED Requirements

### Requirement: A card keeps the translation of its sentence
A card SHALL keep the translation of its sentence that the word card or the selection card showed when the word was captured — the translated sentence, and where the word landed in it when the translation says so — and SHALL keep none when that card showed none. The translation SHALL be shown only with the card's answer, after the reveal, and SHALL travel with the card in backups and to the reader's other devices. An empty translation received from a device that predates it SHALL NOT erase a kept one.

#### Scenario: Captured with a translation
- **WHEN** the reader adds `countenance` to the deck from a word card that shows its sentence translated
- **THEN** the card keeps that translated sentence, with the part that translates `countenance` marked

#### Scenario: Captured without a translation
- **WHEN** the reader adds a word while the extended translation is off, or before the card's translation has arrived
- **THEN** the card keeps no translation, and its review shows the gloss alone

#### Scenario: In review
- **WHEN** a card that keeps a translation comes up in a review
- **THEN** the translated sentence appears under the gloss once the answer is revealed, with the word's translation marked, and never before the reveal

#### Scenario: On another device
- **WHEN** the reader's other device synchronises, even one without a translation model
- **THEN** the card there shows the same translation with its answer

#### Scenario: A device that predates the translation
- **WHEN** a device whose extension predates the translation grades that card and synchronises
- **THEN** the card keeps its translation on every device

#### Scenario: Backup round trip
- **WHEN** the reader backs up a deck holding translated cards and restores it onto a fresh install
- **THEN** each card comes back with its translation and its marks, and cards without one come back without
