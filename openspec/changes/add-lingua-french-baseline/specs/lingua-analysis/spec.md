## ADDED Requirements

### Requirement: French is a studied language served by the baseline analysis
The core SHALL analyse French as a studied language, tagged `fr` and detected as French, and SHALL serve it with the baseline analysis until its own rules are written: segmentation, the pre-pass rules that belong to no language, and the pack's form→lemma lookup, the lowercased form when the pack does not list it; no contraction or elision split, no Unicode (NFC) normalisation, no exception table, no morphological rule, no function-word table and no names rule. French SHALL carry its own analyser version, `0.1.0` while it is the baseline, reported by every page analysed as French and compared with a French pack's. Adding French SHALL leave English and Spanish output byte for byte unchanged, and the en-fr, es-fr and es-en packs loadable and byte-identical.

#### Scenario: An elided word is one token
- **WHEN** « l'homme », « qu'il » and « aujourd'hui » are tokenised as French with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself, lowercased

#### Scenario: Contracted articles are whole
- **WHEN** « au », « aux », « du » and « des » are tokenised as French
- **THEN** each is one token, lemmatised as the pack lists it or as itself

#### Scenario: No French word is a function word yet
- **WHEN** a selection holding « ne », « pas », « le » and « de » is glossed word by word as French
- **THEN** no token is flagged as a function word, and each is counted like any other word

#### Scenario: A French block for a learner of French only
- **WHEN** a French paragraph is analysed by a learner of French, and the same paragraph by a learner of English or of Spanish
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish and one as French
- **THEN** they report `1.1.0`, Spanish's own version and `0.1.0`

#### Scenario: No rule of another language runs on French
- **WHEN** « don't » is tokenised as French, and `as`, `are`, `ate` and `has` are lemmatised as French with a pack that does not list them
- **THEN** « don't » is one token, and each word comes back as itself, never as an English lemma, and a form with an acute accent is not retried without it

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after the core gains French
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged

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
- **THEN** « l'homme » is one token with the lemma « l'homme », « au » is whole, no token is a function word, and the analysis reports `0.1.0`

#### Scenario: A French rule lands
- **WHEN** a later change gives French a tokenisation or lemmatisation rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps French's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the French tables and the fr-en pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are French
- **THEN** the backup has schema version 3
