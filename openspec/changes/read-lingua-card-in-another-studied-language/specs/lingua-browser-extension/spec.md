## MODIFIED Requirements

### Requirement: Each document is read in its own language
When the reader accepts several languages, the reading session SHALL choose each document's language among them, with the document's declared language as a hint, and SHALL ask every language-bound question about that document in it: its analysis, glosses, cards, statuses and exposures. A word card whose word the document's language does not know SHALL be the exception the requirement *A word the document's language does not know is read in another studied language* defines: that card, and what the reader records from it, SHALL be in the language it was read in. With one accepted language, it SHALL read every document in that language without asking for a detection.

#### Scenario: A Spanish page
- **WHEN** a reader who accepts English and Spanish opens a page in Spanish
- **THEN** the page is analysed and highlighted in Spanish, and a word marked there is recorded in Spanish

#### Scenario: An English page for the same reader
- **WHEN** the same reader opens a page in English
- **THEN** the page is read in English

#### Scenario: Every reader today
- **WHEN** a reader who accepts English alone opens any page
- **THEN** no detection is asked for, and every request names English, as before

#### Scenario: A Spanish word on an English page
- **WHEN** a reader who accepts English and Spanish opens an English page quoting `hablaba`, which the English pack does not know
- **THEN** the page is still analysed, highlighted and counted in English, and only the card of `hablaba` is read in Spanish

## ADDED Requirements

### Requirement: A word the document's language does not know is read in another studied language
A word card SHALL be read in another language the reader accepts when, read in the document's language, its word is unknown, has no gloss, no reading, no other dictionary form and no frequency rank. This SHALL apply to the card of a highlighted word the reader clicks, of a one-word selection over a page word, and of a one-word selection no page word covers. The other languages SHALL be asked in the order of the reader's studied languages, and the first whose pack reads the word as one word, not a name, with a gloss SHALL answer; when none does, the card SHALL be the one it was before this requirement. The card SHALL then be that language's card of the word: its dictionary form, the form seen, its gloss, its readings named as that language's forms, its frequency band from that language's pack, and one line naming the language, the name taken from the labels module. Its listen buttons for the word and its dictionary form SHALL use that language's voice, and its sentence button the document's. No translation SHALL be asked for it. A word the reader marked in the document's language, a name, a word the document's pack ranks or reads, and a selection of several words SHALL keep their card. The re-read SHALL be asked only once the document's answer is known, and only for that card; past one more grammar bound, the card SHALL be shown as before, and the re-read SHALL replace it when it arrives if that card is still shown. A reader who accepts one language SHALL see no change.

#### Scenario: A Spanish word on a French page
- **WHEN** a reader whose native language is English, who studies Spanish and French, opens the card of `hablaba`, highlighted on a French page, which the French pack does not know
- **THEN** the card is headed `hablar`, shows `hablaba` as the form seen, glosses it "to talk, to speak", says it is the first- and third-person singular imperfect indicative of `hablar`, says "Common — among the 1,000 most frequent words." and says "Spanish word"

#### Scenario: A French reader's English page about Spanish
- **WHEN** a reader whose native language is French, who studies English and Spanish, clicks `hablaba` on an English page
- **THEN** the card is the Spanish card of `hablar`, glossed in French, and says « Mot espagnol »

#### Scenario: A selection on a page in the reader's own language
- **WHEN** a reader whose native language is French, who studies English and Spanish, selects `hablaba` on a French page, which is not analysed for them
- **THEN** the card is the Spanish card of `hablar`

#### Scenario: A word the document's pack knows without a gloss
- **WHEN** the reader opens the card of a French word the French pack ranks but does not gloss
- **THEN** the card is the French card it was, with its frequency line, and no other language is asked

#### Scenario: A word the reader marked in the document's language
- **WHEN** the reader opens the card of a word they marked ignored in French, which another language they study glosses
- **THEN** the card is the French card it was

#### Scenario: No other language knows the word
- **WHEN** the reader opens the card of a French typo that no pack they study glosses
- **THEN** the card is the one it was before this requirement, with « Rare — au-delà des 20 000 mots les plus fréquents. » in a French interface

#### Scenario: Several words
- **WHEN** the reader selects « yo hablaba » on a French page
- **THEN** the whole-selection card is read in French, as before

#### Scenario: One studied language
- **WHEN** a reader who studies Spanish alone opens the card of a word the Spanish pack does not know
- **THEN** no other language is asked, and the card is the one it was

#### Scenario: The Spanish pack not loaded yet
- **WHEN** the re-read waits for the Spanish pack longer than the grammar bound
- **THEN** the card is first shown as before, and becomes the Spanish card when the answer arrives, unless the reader has opened another card or closed it

#### Scenario: The word's voice
- **WHEN** the reader presses the word's listen button on the Spanish card of `hablaba`, met on a French page
- **THEN** `hablaba` is spoken with the reader's Spanish voice, and the sentence button reads the sentence with the French voice

### Requirement: A word read in another language is recorded and painted in that language
Every status, card and learnt-word count the reader records from a card read in another language SHALL be recorded under the language the card was read in, the card storing the gloss that language's pack gives, and nothing SHALL be recorded for the word under the document's language. They SHALL be synchronised with that language as every record is. For the open document, the session SHALL remember each word a card read in another language, and SHALL paint that word, at each paint, by the reader's status in that language: unpainted when known or ignored, as being learnt when learning, as unknown otherwise. The document's analysis, its counts and its exposures SHALL NOT change, and the remembered words SHALL be forgotten when another document, or none, is read.

#### Scenario: Known in Spanish
- **WHEN** on a French page the reader presses « Je connais » on the Spanish card of `hablaba`
- **THEN** `hablar` is recorded known in Spanish, no French status is recorded for `hablaba`, and `hablaba` is no longer highlighted on that page

#### Scenario: Added to the deck
- **WHEN** the reader presses « + Deck » on that card
- **THEN** a Spanish card of `hablar` is created with the gloss the Spanish pack gives, and `hablaba` is painted as being learnt on that page

#### Scenario: Another device
- **WHEN** the reader syncs a device that accepts Spanish, and one that does not
- **THEN** the first records `hablar` known in Spanish, and the second applies nothing, as for any Spanish status

#### Scenario: The page's figures
- **WHEN** the reader marks `hablaba` known from its Spanish card on a French page
- **THEN** the page's percentage of known words and its exposures are the ones the French analysis gives, as before

#### Scenario: The next document
- **WHEN** the reader leaves that page and opens another French page that quotes `hablaba`
- **THEN** `hablaba` is painted by the French analysis, as before, until its card is opened again

### Requirement: No word is read in a language whose guard keeps the document's language out
A word card SHALL NOT be read in Spanish when the document declares Catalan, Galician or Occitan, and SHALL NOT be read in French when the document declares Catalan, Occitan or Romanian, the languages the detection guards keep out of those languages, in their two- or three-letter codes. Such a card SHALL be the one it was before the requirement *A word the document's language does not know is read in another studied language*.

#### Scenario: A Catalan page
- **WHEN** a reader who studies Spanish and French selects `arribar` on a page that declares `ca`
- **THEN** the card is not read in Spanish, and is the one it was before

#### Scenario: A Galician page declared with three letters
- **WHEN** a reader who studies English and Spanish selects a word on a page that declares `glg`
- **THEN** the card is not read in Spanish

#### Scenario: A French page
- **WHEN** a reader who studies Spanish and French opens the card of `hablaba` on a page that declares `fr`
- **THEN** the card is read in Spanish
