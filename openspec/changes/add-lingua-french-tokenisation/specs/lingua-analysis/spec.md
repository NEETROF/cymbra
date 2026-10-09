## ADDED Requirements

### Requirement: French tokenisation pre-pass
The core SHALL read French text through a pre-pass of its own, beside the rules that belong to no language. The narrow no-break space (U+202F) SHALL separate two words and belong to none. An elided word — `c'`, `ç'`, `d'`, `j'`, `l'`, `m'`, `n'`, `qu'`, `s'`, `t'`, `jusqu'`, `lorsqu'`, `puisqu'` or `quoiqu'`, written with the straight or the typographic apostrophe and followed by a letter — SHALL be split from the word it is joined to, each piece a token with its own source span, the elided piece's span holding its apostrophe; the elided piece SHALL be read as the word it stands for (`ce`, `ça`, `de`, `je`, `le`, `me`, `ne`, `que`, `se`, `te`, `jusque`, `lorsque`, `puisque`, `quoique`; `si` before `il` and `ils`; `moi` and `toi` right after a hyphen), its written first letter's capital kept, and the rule SHALL run again on what follows. An elided word followed by its apostrophe and by neither a letter nor a digit SHALL be read the same way, on its own. A word whose part before the apostrophe is not one of these forms SHALL stay whole. `au` and `aux` SHALL be split into `à` + `le` and `à` + `les`, the two tokens sharing the source span and the first taking the written capital, and `du` and `des` SHALL stay whole. Each piece SHALL then be lemmatised as any French word is. French's analyser version SHALL be `0.2.0`, and English and Spanish output SHALL not move.

#### Scenario: An elided article
- **WHEN** « L'homme » is tokenised as French
- **THEN** the tokens are `Le` and `homme`, `Le` spanning « L' » and `homme` spanning « homme »

#### Scenario: The typographic apostrophe
- **WHEN** « l’horizon » is tokenised as French
- **THEN** the tokens are `le` and `horizon`, `le` spanning « l’ », apostrophe included

#### Scenario: Elided words read as the words they stand for
- **WHEN** « s'il », « s'en », « qu'on », « n'est » and « lorsqu'elle » are tokenised as French
- **THEN** the tokens are `si` and `il`, `se` and `en`, `que` and `on`, `ne` and `est`, `lorsque` and `elle`

#### Scenario: A word whose elision is part of it
- **WHEN** « aujourd'hui », « presqu'île » and « quelqu'un » are tokenised as French
- **THEN** each is one token

#### Scenario: Contracted articles
- **WHEN** "au marché, aux halles, du pain, des pommes" is tokenised as French
- **THEN** the tokens are `à`, `le`, `marché`, `à`, `les`, `halles`, `du`, `pain`, `des`, `pommes`, each `à` sharing its span with the article after it

#### Scenario: A capitalised contraction
- **WHEN** « Au revoir » is tokenised as French
- **THEN** the tokens are `À`, `le` and `revoir`, the first two sharing the span of « Au »

#### Scenario: An elision, then a contraction
- **WHEN** « jusqu'au soir » is tokenised as French
- **THEN** the tokens are `jusque`, `à`, `le` and `soir`, `jusque` spanning « jusqu' » and `à` and `le` sharing the span of « au »

#### Scenario: French punctuation
- **WHEN** « C’est fini ! » is tokenised as French, set with a narrow no-break space after « and before ! and »
- **THEN** the tokens are `Ce`, `est` and `fini`, and no token or span holds a space

#### Scenario: An elided word on its own
- **WHEN** a word card asks the grammar of « l’ » for the dictionary form `le`
- **THEN** the written word reads as the one token `le`, spanning « l’ », with no pieces

#### Scenario: A word card on a contraction
- **WHEN** a word card asks the grammar of « au » for the dictionary form `à`
- **THEN** it answers the pieces `à` and `le`

#### Scenario: Other languages keep their words
- **WHEN** « l'homme », « au » and « dit-il » are tokenised as English and as Spanish, and the English, Spanish, es-en and en-es invariance baselines run
- **THEN** each word is tokenised as before, and every probe of the four baselines is byte for byte the output recorded before, without re-blessing

### Requirement: French hyphenated inversions are read as words
The core SHALL read a French hyphenated run that the pack does not list whole, and whose pieces after the first are all pronouns written in lowercase — `je`, `tu`, `il`, `elle`, `on`, `nous`, `vous`, `ils`, `elles`, `ce`, `le`, `la`, `les`, `lui`, `leur`, `moi`, `toi`, `y`, `en`, or an elided `m'`, `t'` or `l'` before `en` or `y` — the euphonic `t` allowed right before `il`, `elle`, `on`, `ils` or `elles`, as separate words: each piece a token with its own source span, the euphonic `t` and the hyphens in no token. An elision on a run's first piece SHALL be split off before the run is read again. A run the pack lists whole SHALL stay one token, and any other run SHALL follow the compound rule.

#### Scenario: A subject pronoun after its verb
- **WHEN** « dit-il » is tokenised as French
- **THEN** the tokens are `dit` and `il`, each spanning its own letters

#### Scenario: The euphonic t
- **WHEN** « a-t-il » and « pense-t-elle » are tokenised as French
- **THEN** the tokens are `a` and `il`, `pense` and `elle`, and no token is `t`

#### Scenario: A question with an elision
- **WHEN** « Qu’est-ce » is tokenised as French
- **THEN** the tokens are `Que`, `est` and `ce`

#### Scenario: An imperative with its pronouns
- **WHEN** « Donne-m'en », « allez-vous-en » and « coupez-les » are tokenised as French
- **THEN** the tokens are `Donne`, `moi` and `en`; `allez`, `vous` and `en`; `coupez` and `les`

#### Scenario: A compound the pack lists
- **WHEN** « rendez-vous » and « peut-être » are tokenised as French with a pack that lists both
- **THEN** each is one token

#### Scenario: An elision before a compound
- **WHEN** « l'arc-en-ciel » is tokenised as French with a pack that lists « arc-en-ciel »
- **THEN** the tokens are `le` and `arc-en-ciel`

#### Scenario: Runs that are no inversion
- **WHEN** « celui-ci », « moi-même », « Jean-Pierre » and « Saint-Y » are tokenised as French with a pack that lists none of them
- **THEN** each is one token whose parts are its pieces, as the compound rule gives

## MODIFIED Requirements

### Requirement: French is a studied language served by the baseline analysis
The core SHALL analyse French as a studied language, tagged `fr` and detected as French, and SHALL serve it with the baseline analysis until its own rules are written: segmentation, the pre-pass rules that belong to no language, French's own tokenisation pre-pass (*French tokenisation pre-pass*, *French hyphenated inversions are read as words*), and the pack's form→lemma lookup, the lowercased form when the pack does not list it; no Unicode (NFC) normalisation, no exception table, no morphological rule, no function-word table and no names rule. French SHALL carry its own analyser version, `0.2.0` while its lemmatisation is the baseline's, reported by every page analysed as French and compared with a French pack's. Adding French, or writing its rules, SHALL leave English and Spanish output byte for byte unchanged, and every committed pack (en-fr, es-fr, es-en and en-es) loadable and byte-identical.

#### Scenario: An elided word is one token
- **WHEN** « aujourd'hui », « presqu'île » and « quelqu'un » are tokenised as French with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself, lowercased

#### Scenario: Contracted articles are whole
- **WHEN** « du » and « des » are tokenised as French
- **THEN** each is one token, lemmatised as the pack lists it or as itself

#### Scenario: No French word is a function word yet
- **WHEN** a selection holding « ne », « pas », « le » and « de » is glossed word by word as French
- **THEN** no token is flagged as a function word, and each is counted like any other word

#### Scenario: A French block for a learner of French only
- **WHEN** a French paragraph is analysed by a learner of French, and the same paragraph by a learner of English or of Spanish
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish and one as French
- **THEN** they report `1.1.0`, Spanish's own version and `0.2.0`

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
- **THEN** « L'homme » is two tokens, `Le` and `homme`, each with its own span, « au » is `à` and `le` sharing its span, « du » is whole, no token is a function word, and the analysis reports `0.2.0`

#### Scenario: A French rule lands
- **WHEN** a later change gives French a tokenisation or lemmatisation rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps French's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the French tables and the fr-en pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are French
- **THEN** the backup has schema version 3
