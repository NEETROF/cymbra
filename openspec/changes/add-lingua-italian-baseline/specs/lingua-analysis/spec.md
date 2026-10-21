# Spec Delta

## ADDED Requirements

### Requirement: Italian is a studied language served by the baseline analysis
The core SHALL analyse Italian as a studied language, tagged `it` and detected as Italian, and SHALL serve it with the baseline analysis until its own rules are written: segmentation, the pre-pass rules that belong to no language, and the pack's form→lemma lookup, the lowercased form when the pack does not list it; no elision, contraction or enclitic split, no Unicode (NFC) normalisation, no exception table, no function-word table, no names rule and no detection guard. Italian SHALL carry an analyser version of its own, below `1.0.0` while the baseline serves it, which a pack studying Italian is checked against and no other language's pack is. A rule written for English, Spanish or French SHALL NOT run on Italian, and adding Italian SHALL leave the English, Spanish and French analyses byte for byte what they were.

#### Scenario: An elided word is one token
- **WHEN** « l'uomo », « un'amica », « dell'acqua » and « c'è » are tokenised as Italian with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself, lowercased

#### Scenario: Articulated prepositions are whole
- **WHEN** « del », « nel », « al », « dal », « sul » and « col » are tokenised as Italian
- **THEN** each is one token, lemmatised as the pack lists it or as itself

#### Scenario: An enclitic form is one token
- **WHEN** « dirmelo », « vattene » and « eccolo » are tokenised as Italian with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself

#### Scenario: No Italian word is a function word yet
- **WHEN** a selection holding « non », « di », « il » and « che » is glossed word by word as Italian
- **THEN** no token is flagged as a function word, and each is counted like any other word

#### Scenario: An Italian block for a learner of Italian only
- **WHEN** an Italian paragraph is analysed by a learner of Italian, and the same paragraph by a learner of English, of Spanish or of French
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish, one as French and one as Italian
- **THEN** each reports its own language's version, and Italian's is `0.1.0`

#### Scenario: No rule of another language runs on Italian
- **WHEN** « don't » and « l'homme » are tokenised as Italian, and `as`, `has`, `dió` and `été` are lemmatised as Italian with a pack that does not list them
- **THEN** each is one token, and each word comes back as itself, never as another language's lemma, a form with an acute accent not retried without it

#### Scenario: English, Spanish and French output do not move
- **WHEN** the six invariance baselines run after the core gains Italian
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged

### Requirement: An Italian invariance baseline runs beside the English, Spanish and French ones
The core's tests SHALL hold an Italian invariance baseline: the engine's output over a fixed thirteen-page raw-text Italian corpus — pages authored around one phenomenon each (news, fiction, homographs, elisions, the articulated prepositions in both readings, enclitic pronouns, names and titles, informal Italian with the apostrophe for an accent, a recipe, technical prose with one block in NFD, a mixed page with English, Spanish, French, Catalan, Galician and Latin blocks, verb forms) and two public-domain excerpts, set in real Italian punctuation — through the same probes the other baselines use, over a fixture pack studying Italian glossed in French until the Italian tables are committed, with the engine started on the real en-fr pack. The baseline SHALL run wherever the others run; a change that moves its golden SHALL re-bless it and say why, and a change of an Italian rule SHALL bump Italian's analyser version.

#### Scenario: The corpus
- **WHEN** the baseline reads its corpus
- **THEN** it holds the thirteen pages in order, each with at least one block, the reader's pages are among them, and the technical page's NFD block is still decomposed

#### Scenario: Nothing of Italian changed
- **WHEN** the baseline runs on a pull request that changes no Italian rule, no fixture table and no beside pack
- **THEN** every probe is byte for byte the golden

#### Scenario: What the baseline shows today
- **WHEN** the baseline analyses its `elisioni`, `preposizioni`, `clitici` and `misto` pages
- **THEN** « l'uomo » is one token with the lemma « l'uomo », « del » is whole, « dirmelo » is one token, no token is a function word, the Catalan and Galician blocks are analysed as Italian, and the analysis reports `0.1.0`

#### Scenario: An Italian rule lands
- **WHEN** a later change gives Italian a tokenisation, lemmatisation or detection rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps Italian's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the Italian tables and the it-fr pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are Italian
- **THEN** the backup has schema version 4
