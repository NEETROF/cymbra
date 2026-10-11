## ADDED Requirements

### Requirement: French is a studied language served by its own analysis
The core SHALL analyse French as a studied language, tagged `fr` and detected as French, and SHALL serve it with its own analysis — its tokenisation pre-pass (*French tokenisation pre-pass*, *French hyphenated inversions are read as words*), its normalisation (*French text is read in NFC*), its lemmatisation cascade (*French lemmatisation cascade*), its closed classes (*French closed classes*) and its names rule (*A French document's names are set aside*) — beside segmentation and the pre-pass rules that belong to no language, and SHALL no longer serve it with the baseline analysis. French SHALL carry its own analyser version, no longer a `0.x` one, reported by every page analysed as French and compared with a French pack's. Adding French, or writing its rules, SHALL leave English and Spanish output byte for byte unchanged, and every committed pack (en-fr, es-fr, es-en and en-es) loadable and byte-identical.

#### Scenario: An elided word is one token
- **WHEN** « aujourd'hui », « presqu'île » and « quelqu'un » are tokenised as French with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself, lowercased

#### Scenario: Contracted articles are whole
- **WHEN** « du » and « des » are tokenised as French
- **THEN** each is one token, lemmatised as the pack lists it or as itself

#### Scenario: A French block for a learner of French only
- **WHEN** a French paragraph is analysed by a learner of French, and the same paragraph by a learner of English or of Spanish
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish and one as French
- **THEN** they report `1.1.0`, Spanish's own version and French's own version, which is no `0.x` one

#### Scenario: No rule of another language runs on French
- **WHEN** « don't » is tokenised as French, and `as`, `are`, `ate` and `has` are lemmatised as French with a pack that does not list them
- **THEN** « don't » is one token, and each word comes back as itself, never as an English lemma, and a form with an acute accent is not retried without it

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after the core gains French
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged

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
For each French token, the core SHALL produce a lowercase lemma in NFC through French's cascade: the pack's forms → for a form written in lowercase that the pack does not list, and whose singular it does not list either, that singular (`-eaux` read as `-eau`, any other `-aux` as `-al`, a final `-s` after a letter other than `s` dropped; a form of four letters or fewer, a form ending in `-us`, `-is`, `-ès` or `-os`, a hyphenated or elided form and a form ending in `-âmes`, `-îmes`, `-ûmes`, `-âtes`, `-îtes` or `-ûtes` left as they are) → the form itself. The pack's forms SHALL decide every lemma they hold, one per form, and a form the pack does not list SHALL never be read as a word the pack lists. These are French's lemmatisation rules: French SHALL no longer be served by the baseline's lemmatisation, and its analyser version SHALL no longer be a `0.x` version — `1.0.0` or a later one —, reported by every page analysed as French and compared with a French pack's. No rule of French's SHALL run on English or Spanish text, and their output SHALL not move.

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
- **WHEN** the French invariance baseline runs over its fixture pack after this change
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

## MODIFIED Requirements

### Requirement: A French invariance baseline runs beside the English and Spanish ones
The core's tests SHALL hold a French invariance baseline: the engine's output over a fixed thirteen-page raw-text French corpus — pages authored around one phenomenon each (news, fiction, homographs, elisions and the forms before a vowel, the contracted articles, hyphenated inversions and compounds, names, informal French, a recipe, technical prose, a mixed page with English, Spanish, Catalan, Occitan and Italian blocks, verb forms) and two public-domain excerpts, the opening of Proust's *Du côté de chez Swann* (1913) and the first sentence of Hugo's *Les Misérables* (1862), the fiction and Proust pages set with the typographic apostrophe and the narrow no-break space of French punctuation, and one block of the technical page committed in NFD — answered through the harness the English and Spanish baselines share, over a fixture fr-en pack built from hand-written tables until the French tables are committed, the engine started on es-en as an English-native reader's is. The baseline SHALL run wherever the English and Spanish baselines run, and a pull request that moves its golden SHALL re-bless it and say why: a French rule that bumps French's analyser version, the fixture replaced by the committed tables, or the beside pack's update.

#### Scenario: The corpus
- **WHEN** the baseline reads its corpus
- **THEN** it holds the thirteen pages in order, each with at least one block, the reader's pages are among them, and the technical page's NFD block is still decomposed

#### Scenario: Nothing of French changed
- **WHEN** the baseline runs on a pull request that changes no French rule, no fixture table and no beside pack
- **THEN** every probe is byte for byte the golden

#### Scenario: What the baseline shows today
- **WHEN** the baseline analyses its `elisions` and `contractions` pages
- **THEN** « L'homme » is two tokens, `Le` and `homme`, each with its own span, « au » is `à` and `le` sharing its span, « du » is whole, and the analysis reports French's own version, which is no `0.x` one

#### Scenario: A French rule lands
- **WHEN** a later change gives French a tokenisation or lemmatisation rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps French's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the French tables and the fr-en pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are French
- **THEN** the backup has schema version 3

## REMOVED Requirements

### Requirement: French is a studied language served by the baseline analysis
**Reason**: Its name, its « until its lemmatisation rules are written » and its scenarios *No French word is a function word yet* and *Each language reports its own version* (`0.2.0`) describe the baseline this change ends; OpenSpec refuses a MODIFIED block that drops a scenario. Replaced by *French is a studied language served by its own analysis*.

**Migration**: None for a reader. Its other scenarios are carried over verbatim into the new requirement, *Each language reports its own version* naming French's own version instead of `0.2.0`; the function words are *French closed classes*'.
