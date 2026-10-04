## ADDED Requirements

### Requirement: The library groups its books by language
The library SHALL show its books in sections when they fall in more than one: one section per language the reader studies and the extension reads, in the reader's order, headed with the language's name, then a last section, « Autres langues », holding the books in any other language or declaring none. A book's language SHALL be the first language its package declares, reduced to its primary subtag. Each section SHALL hold its books in the library's order, and a section with no book SHALL NOT be shown. When every book falls in one section, the library SHALL show no heading.

#### Scenario: A reader of Spanish and English
- **WHEN** a reader who studies Spanish, then English, has books declaring Spanish, English and French
- **THEN** the library shows « Espagnol » with the Spanish books, then « Anglais » with the English ones, then « Autres langues » with the French one

#### Scenario: A book that declares no language
- **WHEN** a book's package declares no language, and another book is in a language the reader studies
- **THEN** the first book is shown under « Autres langues »

#### Scenario: A regional or three-letter code
- **WHEN** a book declares `es-MX` or `spa`, for a reader who studies Spanish and English
- **THEN** it is shown under « Espagnol »

#### Scenario: Every book in one language
- **WHEN** every book of the library declares English
- **THEN** the library shows them with no heading, as before
