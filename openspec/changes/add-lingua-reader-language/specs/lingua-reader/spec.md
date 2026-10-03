## ADDED Requirements

### Requirement: A book is read in its own language
The reader page SHALL give each section, as its declared language, the language its document declares or, when it declares none, the language the book's package declares (`dc:language`). The reading session SHALL then choose the section's language as it chooses a page's. A section's document SHALL be taken to declare its language through `lang` or `xml:lang`, on its root element or its body.

#### Scenario: A short section of a Spanish book
- **WHEN** a reader of English and Spanish opens a book that declares Spanish, at a section that declares nothing and holds too little text to detect
- **THEN** the section is read in Spanish

#### Scenario: An English preface in a Spanish book
- **WHEN** a section of the same book is in English, with enough text to detect
- **THEN** it is read in English

#### Scenario: An XHTML section
- **WHEN** a section declares its language with `xml:lang` only
- **THEN** that language is its declared language

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** every section is read in English without detecting anything, as before
