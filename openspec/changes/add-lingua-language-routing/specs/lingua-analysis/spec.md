## ADDED Requirements

### Requirement: A document's language, chosen among the reader's
The core SHALL choose the language of a document among candidate studied languages. Each block long enough to be detected SHALL vote, weighted by its length, for the language detected in it when that language is a candidate. The language with the most weight SHALL win. The document's declared language SHALL break a tie between candidates and SHALL decide when no block votes, and the first candidate SHALL win otherwise. A single candidate SHALL be chosen without detection. The choice SHALL NOT change the page analysis, which is then made in the chosen language.

#### Scenario: A Spanish page for a reader of English and Spanish
- **WHEN** a page whose blocks are Spanish is offered with English and Spanish as candidates
- **THEN** Spanish is chosen

#### Scenario: A page mostly in one language
- **WHEN** a page has long English paragraphs and a short Spanish quotation
- **THEN** English is chosen, and the quotation is excluded from the English analysis

#### Scenario: Nothing to detect
- **WHEN** no block of a page is long enough to be detected, and the page declares Spanish
- **THEN** Spanish is chosen; without a declared candidate language, the first candidate is

#### Scenario: One candidate
- **WHEN** English is the only candidate
- **THEN** English is chosen without detecting anything
