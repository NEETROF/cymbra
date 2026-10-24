# Spec Delta

## ADDED Requirements

### Requirement: An Italian voice from Italy by default
For Italian, the automatic voice choice SHALL prefer a voice of Italy to the voices of other regions within the same quality, on every browser, Firefox for Android's three-letter regions included, the other regions keeping the browser's order. Apple's Italian Eloquence voices SHALL be treated as the other Eloquence voices are. The extension SHALL offer no accent setting: the reader SHALL choose another voice in Réglages, and that choice SHALL be kept for Italian. English's, Spanish's and French's automatic choices and the order of their voices SHALL NOT change.

#### Scenario: Alice before the Eloquence voices
- **WHEN** Chrome on macOS lists `Alice` (it-IT), the eight Eloquence voices `Eddy (Italian (Italy))` … `Shelley (Italian (Italy))` and `Google italiano`, Italian is read and online voices are not allowed
- **THEN** the automatic voice is `Alice`, the Eloquence voices are offered under « Autres voix » ("Other voices", « Otras voces »), and `Google italiano`, not on the device, is not offered

#### Scenario: A Swiss voice listed first
- **WHEN** a device lists an `it-CH` voice before an `it-IT` voice of the same quality
- **THEN** the automatic voice is the `it-IT` one

#### Scenario: No Italian voice on the device
- **WHEN** Chrome on Windows lists only `Google italiano`, online voices are not allowed, and Italian is read
- **THEN** no voice speaks, and Réglages says no Italian voice is installed and names the Windows language to install

#### Scenario: The other languages do not move
- **WHEN** the voice rankings of English, Spanish and French are computed over every captured list and every setting, pinned before this change
- **THEN** each is what it was before this requirement

### Requirement: Italian voices are named in the reader's interface language
Firefox for Android's three-letter region of Italy SHALL be read as Italy, so that a voice written `ita-ITA-…` is placed in Italy and its region named in the reader's interface language — « Italie » in French, "Italy" in English, « Italia » in Spanish — never by its code.

#### Scenario: Firefox for Android
- **WHEN** Firefox for Android lists `ita-ITA-default`, the reader has allowed Android's voices and reads Réglages in Spanish
- **THEN** the voice is a voice of Italy, named « italien (ITA,DEFAULT) — Italia », never « — ITA »

### Requirement: Réglages speaks of Italian voices in the reader's interface language
The catalogue SHALL name Italian in each interface language — its name, the forms its sentences need, the Windows and macOS language an Italian voice is installed with, and a preview sentence written in Italian whatever the interface — and the languages the catalogue names SHALL include Italian, so that the voice block says "No Italian voice is installed on this device.", « Aucune voix italienne n'est installée sur cet appareil. », « No hay ninguna voz italiana instalada en este dispositivo. » and offers an Italian preview through the speaker's language, without widening the studied languages the extension offers.

#### Scenario: No voice, in French
- **WHEN** the reader, with the interface in French, reads Réglages for Italian on a device with no Italian voice
- **THEN** Réglages says « Aucune voix italienne n'est installée sur cet appareil. » and names « Italien (Italie) » as the Windows language to install

#### Scenario: The preview
- **WHEN** the reader asks for a preview of an Italian voice, whatever the interface language
- **THEN** the voice reads an Italian sentence

### Requirement: An Italian piece is heard as the page writes it
When the card's speaker reads Italian, the word button of a card opened on an elided piece — a selection the page writes glued to the next word, as `l'` in « l'uomo » — SHALL read the piece together with the word it leans on as the page writes them, as a French elided piece is read, the word it leans on taken as the analysis reads it when that word is a spelling read for an accent (« c'e' » heard « c'è »), and a piece the page writes before a number SHALL read glued to that number (« l'8 » of « l'8 marzo »); a card opened on a piece of an articulated preposition the analysis split SHALL read the word as the page writes it (« al » for its `a`); a card opened on a truncation SHALL read it as the page writes it (« po' », not `poco`); any other card SHALL read as a French card reads, so that a word read for an accent is heard as its accented text (« perché » for « perche' »). Whatever text is chosen, a word of one vowel, accented or written as a capital, its trailing apostrophe aside (« A' » of « A' piedi »), SHALL be read followed by a full stop, so that a voice reads the vowel and not the letter's name — except « é » and its capital « É », which a full stop silences. The button SHALL be labelled with that text where the card labels the form seen, without the full stop added for the voice. The dictionary form's button and the sentence button SHALL read as before, and a card read in English, Spanish or French SHALL read exactly as before.

#### Scenario: An elided article
- **WHEN** the reader, with the interface in French, opens the card of `l'` in « L'uomo è arrivato. » on an Italian page
- **THEN** the card offers « ▶ L'uomo », which speaks « L'uomo », then the dictionary form's button, then « ▶ Phrase »

#### Scenario: An articulated preposition
- **WHEN** the reader opens the card of `a` in « Vado al mare. », whose word the analysis split into `a` and `il`
- **THEN** the word button speaks « al »

#### Scenario: A truncation
- **WHEN** the reader opens the card of « po' » in « un po' di pane », read as `poco`
- **THEN** the word button speaks « po' »

#### Scenario: The apostrophe for an accent
- **WHEN** the reader opens the card of « perche' », read as `perché`, of « E' » in « E' vero », read as `È`, and of `c'` in « non c'e' problema »
- **THEN** the first speaks « perché », the second « È. », and the third « c'è »

#### Scenario: A vowel alone
- **WHEN** the reader opens the card of « è » in « Roma è grande. », of « E » in « E poi è partito. », of `a` in « A' piedi. » and of « É » in « É vero. »
- **THEN** the first card's form-seen button is labelled « è » and speaks « è. », the second card's word button speaks « E. », the third's « A'. », and the fourth's speaks « É », with no full stop

#### Scenario: An elided piece before a number
- **WHEN** the reader opens the card of `l'` in « È nato l'8 marzo. » or of `in` in « Nato nell'800. », whose digits the analysis drops
- **THEN** the word button speaks « l'8 » and « nell'800 », never `lo` or `in` alone

#### Scenario: English, Spanish and French
- **WHEN** the reader opens a card on an English, Spanish or French page, a split word's included
- **THEN** its buttons are and speak what they were before this requirement
