## ADDED Requirements

### Requirement: Expression lookup in a phrase gloss
The phrase gloss SHALL report, beside its tokens, the expressions the pack recognises in the text: for each, the tokens it covers and its gloss.
A match SHALL be the longest run of consecutive tokens whose dictionary forms, joined in
order, are a key of the pack's expression table; runs SHALL be tried longest first, over a
bounded number of tokens, and a token SHALL belong to at most one match. Matching SHALL use
the dictionary forms the analysis already produced, so the words as written need not be the
dictionary spelling. A pack with no expression table SHALL report no match, and the token
list SHALL be the same whether the table is there or not.

#### Scenario: An inflected expression
- **WHEN** the text `gave up` is glossed and the pack holds `give up`
- **THEN** one match is reported, covering both tokens, with the gloss of `give up`

#### Scenario: The longest run wins
- **WHEN** the pack holds both `look forward` and `look forward to`, and the text is `look forward to it`
- **THEN** the match covers `look forward to` and not the shorter run inside it

#### Scenario: An expression inside a longer selection
- **WHEN** the text `a compelling starting point` is glossed and the pack holds `start point`
- **THEN** a match covers the last two tokens, and `compelling` is reported as an ordinary token

#### Scenario: Nothing matches
- **WHEN** no run of the text's dictionary forms is a key of the table
- **THEN** no match is reported and every token is returned as before

#### Scenario: A pack without the table
- **WHEN** the loaded pack carries no expression table
- **THEN** the phrase gloss reports no match, and its tokens are byte-for-byte what it returned before
