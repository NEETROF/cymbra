## ADDED Requirements

### Requirement: A French word standing alone in capitals is read as the acronym its pack lists
Before French's lemmatisation cascade, the core SHALL read a French token as an acronym of its pack when the token is one word written wholly in capitals — two letters or more and no lowercase letter, read without its soft hyphens —, when neither the nearest word of two letters or more written before it nor the one written after it, in the text being read, is written wholly in capitals, and when the pack's forms list the token as written, in NFC and not lowercased; the token's dictionary form SHALL then be that acronym, its capitals kept, and the cascade SHALL not run on it.
A word that shares the token's written span (the pieces of « au »), a single letter, a number and
punctuation SHALL not count as a neighbour, and a token with no neighbour stands alone. Every other
French token SHALL be read by French's cascade as before, lowercased — a word in capitals in a line
written in capitals, in a run of words in capitals, or with a lowercase letter among its own, and a
word in capitals the pack does not list as written. The page analysis SHALL read the neighbours in
its block, the phrase gloss in the reader's selection, the word grammar in the written word, and the
reading of an expression's headword in the headword, through one rule, so that the four read a word
alike. Asked about a dictionary form the pack holds as a lemma in capitals, the word grammar SHALL
answer for it as asked, and for every other dictionary form lowercased, as before.
The rule SHALL bump French's analyser version and no other, and no part of it SHALL run on text read
as English or Spanish, whose output SHALL not move.

#### Scenario: The acronym in a line of lowercase words
- **WHEN** « Le CA a voté le budget. » is glossed as French with a pack whose acronym table holds `CA`
- **THEN** `CA`'s dictionary form is `CA`, it carries the pack's gloss of `CA`, and it is not flagged as a function word

#### Scenario: A headline in capitals
- **WHEN** « LE CA A VOTÉ LE BUDGET » is glossed as French with that pack
- **THEN** `CA` is read by the cascade — as `ça` with French's tables —, since the words beside it, `LE` and `VOTÉ` (the single letter `A` passed over), are written in capitals

#### Scenario: A shout in a line of lowercase words
- **WHEN** « Il a crié : CA SUFFIT ! » is glossed as French with that pack
- **THEN** `CA` is read as `ça`, since `SUFFIT` beside it is written in capitals, and an expression `ça suffit` the pack lists is met

#### Scenario: A capital initial
- **WHEN** « Ca fait du bien. » is glossed as French with that pack
- **THEN** `Ca` is read as `ça`

#### Scenario: A word in capitals the table does not hold
- **WHEN** « L'ONU a voté » is analysed as French with a pack that lists `onu` and no form `ONU`
- **THEN** `ONU`'s dictionary form is `onu`, as before

#### Scenario: The acronym on a page
- **WHEN** a page reads « Le CA de l'association se réunira demain avec les membres du bureau. », and `CA` is one of the pack's dictionary words
- **THEN** `CA` is read as `CA`, counted, glossed with the pack's gloss of `CA`, and not set aside as a name

#### Scenario: The acronym's card
- **WHEN** the core is asked about the written word `CA` for the dictionary form `CA`
- **THEN** it answers the pack's gloss of `CA`, grouped by its part of speech, and not the gloss of `ça`

#### Scenario: French's version moves alone
- **WHEN** a page is analysed as French after this rule is added, and a French pack stamped with French's previous analyser version is loaded
- **THEN** the page reports French's new version, the pack is refused as built for another analyser version, and English's and Spanish's versions are unchanged

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after this rule is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
