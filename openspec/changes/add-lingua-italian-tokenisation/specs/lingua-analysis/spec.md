# Spec Delta

## ADDED Requirements

### Requirement: Italian tokenisation pre-pass
The core SHALL read Italian text through a pre-pass of its own, beside the rules that belong to no language. An elided word — `l'`, `un'`, `d'`, `c'`, `v'`, `m'`, `t'`, `s'`, `n'`, `ch'`, `gl'`, `dov'`, `com'`, `cos'`, `quand'`, `dev'`, `diss'`, `quest'`, `quell'`, `anch'`, `senz'`, `tutt'`, `bell'`, `buon'`, `grand'`, `pover'`, `sant'`, `mezz'`, `nient'`, `quant'`, `cent'`, `vent'`, `sott'`, `nessun'`, `qualcos'`, `qualcun'`, `ciascun'` or `alcun'`, written with the straight or the typographic apostrophe and followed by a letter — SHALL be split from the word it is joined to, each piece a token with its own source span, the elided piece's span holding its apostrophe, the elided piece read as the word it stands for (`lo`, `una`, `di`, `ci`, `vi`, `mi`, `ti`, `si`, `ne`, `che`, `gli`, `dove`, `come`, `cosa`, `quando`, `deve`, `disse` — `dissi` before `io` —, `questo`, `quello`, `anche`, `senza`, `tutto`, `bello`, `buona`, `grande`, `povero`, `santo`, `mezzo`, `niente`, `quanto`, `cento`, `venti`, `sotto`, `nessuna`, `qualcosa`, `qualcuna`, `ciascuna`, `alcuna`) with its written capital kept. The articulated prepositions of `di` — `del`, `dello`, `della`, `dei`, `degli`, `delle`, and `dell'` read as `dello` — SHALL stay whole; every other articulated preposition — `al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal`, `dallo`, `dalla`, `dai`, `dagli`, `dalle`, `nel`, `nello`, `nella`, `nei`, `negli`, `nelle`, `sul`, `sullo`, `sulla`, `sui`, `sugli`, `sulle`, `col`, `coi`, and the elided `all'`, `dall'`, `nell'`, `sull'` read with `lo` — SHALL be split into its preposition (`a`, `da`, `in`, `su`, `con`) and its article, the two tokens sharing the source span, the first keeping the written capital. A word ending in an apostrophe with no letter after it SHALL keep its apostrophe, one token whose span holds it, only when the word is on a list: an elided form of this requirement written alone — « l’ » as a word card hands it — read as the word it stands for, or a truncation — `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'` and `be'` read as `poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici` and `bene` — whose read word is final and not re-read as an articulated preposition — and never when an opening quote precedes the word; any other trailing apostrophe SHALL stay outside the word, as the rule of no language leaves it, until the apostrophe that stands for an accent is read by a reviewed list. Nothing SHALL be split inside a hyphenated compound beyond an elision on its first piece. Italian's analyser version SHALL be `0.2.0` with this pre-pass, and no other language's tokenisation SHALL move.

#### Scenario: An elided article
- **WHEN** « L'uomo » is tokenised as Italian
- **THEN** the tokens are `Lo` and `uomo`, `Lo` spanning « L' » and `uomo` spanning « uomo »

#### Scenario: The typographic apostrophe
- **WHEN** « l’amica » is tokenised as Italian
- **THEN** the tokens are `lo` and `amica`, `lo` spanning « l’ », apostrophe included

#### Scenario: Elided words read as the words they stand for
- **WHEN** « c'è », « un'amica », « dov'è », « quest'anno », « anch'io » and « senz'altro » are tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `ci` and `è`, `una` and `amica`, `dove` and `è`, `questo` and `anno`, `anche` and `io`, `senza` and `altro`

#### Scenario: More elided forms
- **WHEN** « cos'è », « quand'ero », « dev'essere », « cent'anni » and « nessun'altra » are tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `cosa` and `è`, `quando` and `ero`, `deve` and `essere`, `cento` and `anni`, `nessuna` and `altra`

#### Scenario: The di family stays whole
- **WHEN** « del pane, della casa, dei libri, dell'acqua » is tokenised as Italian
- **THEN** the tokens are `del`, `pane`, `della`, `casa`, `dei`, `libri`, `dello`, `acqua`, `dello` spanning « dell' »

#### Scenario: The other articulated prepositions split
- **WHEN** « al mare, nella casa, sul tavolo, dagli amici, col treno » is tokenised as Italian
- **THEN** the tokens are `a`, `il`, `mare`, `in`, `la`, `casa`, `su`, `il`, `tavolo`, `da`, `gli`, `amici`, `con`, `il`, `treno`, each preposition sharing its span with the article after it

#### Scenario: An elided articulated preposition
- **WHEN** « all'alba, nell'acqua » is tokenised as Italian
- **THEN** the tokens are `a`, `lo`, `alba`, `in`, `lo`, `acqua`, `a` and `lo` sharing the span of « all' », `in` and `lo` the span of « nell' »

#### Scenario: A capitalised contraction
- **WHEN** « Nel mezzo » is tokenised as Italian
- **THEN** the tokens are `In`, `il` and `mezzo`, the first two sharing the span of « Nel »

#### Scenario: Truncations
- **WHEN** « un po' di pane, va' via » is tokenised as Italian
- **THEN** the tokens are `un`, `poco`, `di`, `pane`, `vai`, `via`, `poco` spanning « po' » and `vai` spanning « va' »

#### Scenario: An apostrophe that stands for an accent, and a closing quote
- **WHEN** « perche' », « e' », « ‘ciao’ » and « ‘due anni fa’ » are tokenised as Italian with a pack that lists `e`
- **THEN** the tokens are `perche`, `e`, `ciao` and `due`, `anni`, `fa`, the trailing apostrophe or closing quote outside the word as in every language — a quoted truncation's base is not re-attached —, until a reviewed list reads the first two as `perché` and `è`

#### Scenario: A hyphenated compound
- **WHEN** « l'anglo-americano » and « week-end » are tokenised as Italian with a pack listing neither
- **THEN** the tokens are `lo` and `anglo-americano`, and `week-end`

#### Scenario: An elided word on its own
- **WHEN** a word card asks the grammar of « l’ » for the dictionary form `lo`
- **THEN** the written word reads as the one token `lo`, spanning « l’ », with no pieces

#### Scenario: A word card on a contraction
- **WHEN** a word card asks the grammar of « al » for the dictionary form `a`
- **THEN** it answers the pieces `a` and `il`

#### Scenario: Other languages keep their words
- **WHEN** « l'uomo », « nel » and « po' » are tokenised as English, as Spanish and as French, and the six invariance baselines run
- **THEN** each word is tokenised as before, and every probe of the six baselines is byte for byte the output recorded before, without re-blessing

## MODIFIED Requirements

### Requirement: Italian is a studied language served by the baseline analysis
The core SHALL analyse Italian as a studied language, tagged `it` and detected as Italian, and SHALL serve it with the baseline analysis until its own rules are written, its tokenisation pre-pass excepted (*Italian tokenisation pre-pass*): segmentation, the pre-pass rules that belong to no language, and the pack's form→lemma lookup, the lowercased form when the pack does not list it; no enclitic split, no Unicode (NFC) normalisation, no exception table, no function-word table, no names rule and no detection guard. Italian SHALL carry an analyser version of its own, below `1.0.0` while the baseline serves it, which a pack studying Italian is checked against and no other language's pack is. A rule written for English, Spanish or French SHALL NOT run on Italian, and adding Italian SHALL leave the English, Spanish and French analyses byte for byte what they were.

#### Scenario: An elided word is one token
- **WHEN** « l'uomo », « un'amica », « dell'acqua » and « c'è » are tokenised as Italian with a pack that lists `è` and none of the whole words
- **THEN** « l'uomo » is `lo` and `uomo`, « un'amica » `una` and `amica`, « dell'acqua » `dello` and `acqua`, « c'è » `ci` and `è`, each piece lemmatised as itself, lowercased

#### Scenario: Articulated prepositions are whole
- **WHEN** « del », « nel », « al », « dal », « sul » and « col » are tokenised as Italian
- **THEN** « del » is one token, lemmatised as the pack lists it or as itself, and the five others are a preposition and an article sharing the word's span

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
- **THEN** each reports its own language's version, and Italian's is `0.2.0`

#### Scenario: No rule of another language runs on Italian
- **WHEN** « don't » and « l'homme » are tokenised as Italian, and `as`, `has`, `dió` and `été` are lemmatised as Italian with a pack that does not list them
- **THEN** « don't » is one token, « l'homme » is `lo` and `homme` by Italian's own rule, and each word comes back as itself, never as another language's lemma, a form with an acute accent not retried without it

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
- **THEN** « l'uomo » is `lo` and `uomo`, « del » is whole, « al » is `a` and `il`, « po' » is `poco`, « dirmelo » is one token, no token is a function word, the Catalan and Galician blocks are analysed as Italian, and the analysis reports `0.2.0`

#### Scenario: An Italian rule lands
- **WHEN** a later change gives Italian a tokenisation, lemmatisation or detection rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps Italian's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the Italian tables and the it-fr pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are Italian
- **THEN** the backup has schema version 4
