## ADDED Requirements

### Requirement: A review is in the language being read
A review session and its counts SHALL cover one studied language at a time, never several mixed. The review SHALL open in the language of the page or book it is opened beside, when the reader studies that language. Otherwise it SHALL open in the last language the reader chose in the review on this device, and otherwise in the reader's first studied language. A language the reader chooses in the review SHALL be remembered on the device as the last one chosen. The review's count of cards and of cards due SHALL be that language's, and the popup's count of cards to review SHALL follow the same language.

#### Scenario: Beside a Spanish page
- **WHEN** a reader of English and Spanish opens the review beside a page in Spanish
- **THEN** the review and its counts hold the Spanish cards only, and no choice of all languages is offered

#### Scenario: Away from a page
- **WHEN** the review opens beside no page in a studied language, and the reader last chose English in the review
- **THEN** it opens in English

#### Scenario: Choosing another language
- **WHEN** the reader chooses English in the review beside a Spanish page
- **THEN** the review and its counts switch to English, and English is remembered as the last language chosen

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no choice of language is shown, and the review and its counts are as before
