## ADDED Requirements

### Requirement: French text is read in NFC
The core SHALL read French text in Unicode normalisation form C: its tokenisation pre-pass SHALL compose every word before comparing it with anything — the elided forms, `au` and `aux`, a hyphenated run the pack lists whole — and every French token's text SHALL be composed, while each token's span SHALL still point into the source text, combining marks included. English text SHALL still be read as it came, and English and Spanish output SHALL not move.

#### Scenario: A decomposed accent
- **WHEN** « mémoire » written with `e` and a combining acute accent is analysed as French with a pack that lists `mémoire`
- **THEN** its token is the composed `mémoire`, with the pack's lemma and gloss, and its span covers the decomposed letters

#### Scenario: A decomposed elision
- **WHEN** « ç'a » written with `c` and a combining cedilla is tokenised as French
- **THEN** the tokens are `ça` and `a`, `ça` spanning the `c`, its cedilla and the apostrophe

#### Scenario: A decomposed run the pack lists
- **WHEN** « peut-être » written with a combining circumflex is tokenised as French with a pack that lists `peut-être`
- **THEN** it is one token, `peut-être`

#### Scenario: English keeps its text as it came
- **WHEN** a word holding a combining mark is tokenised as English
- **THEN** it is not composed, as before

### Requirement: French lemmatisation cascade
For each French token, the core SHALL produce a lowercase lemma in NFC through French's cascade: the pack's forms → for a form written in lowercase that the pack does not list, and whose singular it does not list either, that singular (`-aux` read as `-al`, `-eaux` as `-eau`, a final `-s` after a letter other than `s` dropped; a form of four letters or fewer, a form ending in `-us`, `-is`, `-ès` or `-os`, a hyphenated or elided form and a form ending in `-âmes`, `-îmes`, `-ûmes`, `-âtes`, `-îtes` or `-ûtes` left as they are) → the form itself. The pack's forms SHALL decide every lemma they hold, one per form, and a form the pack does not list SHALL never be read as a word the pack lists. These are French's lemmatisation rules: French SHALL no longer be served by the baseline's lemmatisation, and its analyser version SHALL no longer be a `0.x` version — `1.0.0` or a later one —, reported by every page analysed as French and compared with a French pack's. No rule of French's SHALL run on English or Spanish text, and their output SHALL not move.

#### Scenario: An unlisted plural
- **WHEN** `mégalithes` is lemmatised as French and the pack lists neither `mégalithes` nor `mégalithe`
- **THEN** the lemma is `mégalithe`, so the plural and the singular count as one word

#### Scenario: The tables decide
- **WHEN** `porte` is lemmatised as French with a pack that lists it as a form of `porter`
- **THEN** the lemma is `porter`

#### Scenario: A plural whose singular is another word's form
- **WHEN** `étés` is lemmatised as French with a pack that lists `été` as a form of `être` and does not list `étés`
- **THEN** the lemma is `étés`, never `être`

#### Scenario: A singular in -s and a verb form
- **WHEN** `campus` and `dormîmes`, neither of them listed, are lemmatised as French
- **THEN** each lemma is the form itself

#### Scenario: A capitalised form outside the pack
- **WHEN** `Belfons`, not listed, is lemmatised as French
- **THEN** the lemma is `belfons`

#### Scenario: French leaves the baseline's version
- **WHEN** a page is analysed as French after this change, and a French pack stamped `0.2.0` is loaded
- **THEN** the page reports French's own version, `1.0.0` or later and never a `0.x` one, and the pack is refused as built for another analyser version

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline runs after this change
- **THEN** the technical page's NFD block reads `mémoire` composed and glossed, the phrase glosses flag French's closed-class words, the `noms` page sets aside `Paris`, `Lot`, `Aube`, `Jean-Pierre` and `Saint-Étienne`, and every page reports French's own version

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged

### Requirement: French closed classes
The core SHALL flag as function words, in a word-by-word gloss in French, the French articles and other determiners, pronouns, prepositions, conjunctions, the auxiliaries and modals `être`, `avoir`, `pouvoir` and `devoir`, and the negation `ne`, `pas` and `non`, by the lemma the cascade produced, as English's and Spanish's tables do for their languages; `pas` SHALL be flagged although its form is also the noun « step ». The tables SHALL hold the inflected forms a pack may keep as lemmas of their own (`cet`, `ma`, `mes`, `ton`, `ta`, `tes`), and SHALL NOT hold `personne`, `point`, `or`, `certain`, `plus`, `jamais`, `guère`, `falloir`, `vouloir`, `aller`, `faire`, `voici` or `voilà`, whose readings carry meaning of their own. The flag SHALL change nothing in a page analysis.

#### Scenario: A French phrase
- **WHEN** « la maison de mon père » is glossed word by word as French
- **THEN** `la`, `de` and `mon` are flagged as function words, and `maison` and `père` are not

#### Scenario: The negation
- **WHEN** « Il ne fait pas un pas » is glossed word by word as French
- **THEN** `Il`, `ne`, both `pas` and `un` are flagged, and `fait` is not

#### Scenario: Words that look closed but carry meaning
- **WHEN** « personne », « point », « or » and « jamais » are glossed word by word as French
- **THEN** none of them is flagged

#### Scenario: The pieces of a contraction and of an elision
- **WHEN** « au marché » and « l'homme qu'il attendait » are glossed word by word as French
- **THEN** `à`, `le`, `que` and `il` are flagged, and `marché`, `homme` and `attendait` are not

#### Scenario: Each language keeps its own tables
- **WHEN** `de` and `the` are checked as French, as Spanish and as English
- **THEN** `de` is a function word in French and in Spanish and not in English, and `the` only in English

### Requirement: A French document's names are set aside
The page analysis of a French document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form — a word, or a hyphenated run the pack does not list whole — that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form is not one of the pack's dictionary words (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*); a capital right after an elided piece (`l'`, `d'`, `qu'`…) SHALL count as one in mid-sentence. A dictionary word, a form also written in lowercase, a form capitalised only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their classification. What is set aside SHALL NOT depend on the pack's glosses, and Spanish's names rule and English's analysis SHALL NOT change.

#### Scenario: A city
- **WHEN** a document reads « Nous partons de Paris. Paris est loin. », and `paris` is a lemma of the pack but not one of its dictionary words
- **THEN** both occurrences of `Paris` are set aside, are not underlined, and do not count in the percentage

#### Scenario: After an elided piece
- **WHEN** a document's only capitalised `Aube` follows `l'` (« l'Aube rejoint la Seine »), and `aube` is a lemma of the pack but not one of its dictionary words
- **THEN** `Aube` is set aside

#### Scenario: A hyphenated name
- **WHEN** a document reads « ont retrouvé Jean-Pierre à Saint-Étienne », and the pack lists `pierre` and `saint` but neither run
- **THEN** `Jean-Pierre` and `Saint-Étienne` are set aside

#### Scenario: A dictionary word
- **WHEN** a document reads « entre Orange et Vienne », `orange` is a dictionary word of the pack, and the pack lists `vienne` as a form of the dictionary word `venir`
- **THEN** `Orange` and `Vienne` stay words, with their cards

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes `Mme` only at the head of its blocks
- **THEN** `Mme` stays a word

#### Scenario: The same form as a word
- **WHEN** a document writes « le Lot » in mid-sentence and also « un lot de livres »
- **THEN** `Lot` stays a word

#### Scenario: Spanish's rule does not change
- **WHEN** the Spanish invariance baseline runs after this change
- **THEN** every probe is byte for byte the output recorded before, a Spanish capital after an apostrophe still giving no evidence and a Spanish hyphenated run still judged by its parts
