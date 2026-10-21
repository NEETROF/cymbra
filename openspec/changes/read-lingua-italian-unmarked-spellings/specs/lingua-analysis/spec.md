# Spec Delta

## ADDED Requirements

### Requirement: The apostrophe that stands for an accent reads as the accented word
The core SHALL read an Italian word ending in a vowel and an apostrophe, with no letter after it, as the form the tables list whose final vowel is that vowel accented, when the tables list that form and list the word as written under no word: one token, spanning the apostrophe, its text the accented form, its written capital kept. A word the tables list as written with its apostrophe SHALL keep its listed reading, and a word whose accented form the tables do not list SHALL be read as before, its apostrophe outside the word.

#### Scenario: An apostrophe for an accent
- **WHEN** « perche' », « piu' », « citta' », « e' » and « puo' » are tokenised as Italian with a pack that lists `perché`, `più`, `città`, `è` and `può`
- **THEN** the tokens are `perché`, `più`, `città`, `è` and `può`, each spanning the word and its apostrophe

#### Scenario: A capital
- **WHEN** « E' vero » is tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `È` and `vero`, `È` spanning « E' »

#### Scenario: A listed word with its apostrophe
- **WHEN** « po' » and « da' » are tokenised as Italian
- **THEN** they are read as change 6's truncations, `poco` and `dai`, not as `pò` and `dà`

#### Scenario: A word the tables do not accent
- **WHEN** « ciao' » is tokenised as Italian with a pack that lists no `ciaò`
- **THEN** the token is `ciao`, the apostrophe outside the word, as before

#### Scenario: A closing quote
- **WHEN** « ‘perche’ » is tokenised as Italian
- **THEN** the token is `perche`, the quotes outside the word: an apostrophe after a vowel is read for an accent only when no opening quote precedes the word

## MODIFIED Requirements

### Requirement: Italian tokenisation pre-pass
The core SHALL read Italian text through a pre-pass of its own, beside the rules that belong to no language. An elided word — `l'`, `un'`, `d'`, `c'`, `v'`, `m'`, `t'`, `s'`, `n'`, `ch'`, `gl'`, `dov'`, `com'`, `cos'`, `quand'`, `dev'`, `diss'`, `quest'`, `quell'`, `anch'`, `senz'`, `tutt'`, `bell'`, `buon'`, `grand'`, `pover'`, `sant'`, `mezz'`, `nient'`, `quant'`, `cent'`, `vent'`, `sott'`, `nessun'`, `qualcos'`, `qualcun'`, `ciascun'` or `alcun'`, written with the straight or the typographic apostrophe and followed by a letter — SHALL be split from the word it is joined to, each piece a token with its own source span, the elided piece's span holding its apostrophe, the elided piece read as the word it stands for (`lo`, `una`, `di`, `ci`, `vi`, `mi`, `ti`, `si`, `ne`, `che`, `gli`, `dove`, `come`, `cosa`, `quando`, `deve`, `disse` — `dissi` before `io` —, `questo`, `quello`, `anche`, `senza`, `tutto`, `bello`, `buona`, `grande`, `povero`, `santo`, `mezzo`, `niente`, `quanto`, `cento`, `venti`, `sotto`, `nessuna`, `qualcosa`, `qualcuna`, `ciascuna`, `alcuna`) with its written capital kept. The articulated prepositions of `di` — `del`, `dello`, `della`, `dei`, `degli`, `delle`, and `dell'` read as `dello` — SHALL stay whole; every other articulated preposition — `al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal`, `dallo`, `dalla`, `dai`, `dagli`, `dalle`, `nel`, `nello`, `nella`, `nei`, `negli`, `nelle`, `sul`, `sullo`, `sulla`, `sui`, `sugli`, `sulle`, `col`, `coi`, and the elided `all'`, `dall'`, `nell'`, `sull'` read with `lo` — SHALL be split into its preposition (`a`, `da`, `in`, `su`, `con`) and its article, the two tokens sharing the source span, the first keeping the written capital. A word ending in an apostrophe with no letter after it SHALL keep its apostrophe, one token whose span holds it, only when the word is on a list: an elided form of this requirement written alone — « l’ » as a word card hands it — read as the word it stands for, a truncation — `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'` and `be'` read as `poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici` and `bene` — whose read word is final and not re-read as an articulated preposition, or a word whose apostrophe stands for an accent (*The apostrophe that stands for an accent reads as the accented word*) — and never when an opening quote precedes the word; any other trailing apostrophe SHALL stay outside the word, as the rule of no language leaves it. Nothing SHALL be split inside a hyphenated compound beyond an elision on its first piece. Italian's analyser version SHALL be bumped with this pre-pass, and no other language's tokenisation SHALL move.

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
- **WHEN** « perche' », « e' », « ‘ciao’ » and « ‘due anni fa’ » are tokenised as Italian with a pack that lists `perché`, `è` and `e`
- **THEN** the tokens are `perché`, `è`, `ciao` and `due`, `anni`, `fa`, the first two spanning their apostrophe, the closing quote outside the third, and a quoted truncation's base not re-attached

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
