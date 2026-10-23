# Spec Delta

## ADDED Requirements

### Requirement: The apostrophe that stands for an accent reads as the accented word
The core SHALL read an Italian word — or what remains of one after its elided pieces are split off — ending in a vowel and an apostrophe with neither a letter nor a digit after it, no single quote opened before the word still open there, as the form the pack lists whose final vowel is that vowel accented, when the pack lists such a form and, where the pack carries its ranks, ranks that form's word within 23,000: one token spanning the rest and its apostrophe, its text the accented form, its written capital kept. Where the pack lists the vowel under both accents, a form the pack files under the other SHALL be read as the other's spelling and the other SHALL win; where neither is filed under the other, the one whose word the pack ranks higher SHALL win, and `è` SHALL be tried before `é` where the pack ranks neither. A word Italian's pre-pass lists with its apostrophe — a truncation, an elided form written alone — SHALL keep that reading, whatever the pack lists; the literary apocopes and truncations `ne'`, `de'`, `co'`, `pe'`, `a'`, `su'`, `que'`, `ca'`, `ve'`, `mi'`, `fra'`, `to'`, `i'`, `me'`, `vo'` and `fe'` SHALL never be read for an accent; and a word whose accented form the pack does not list, or ranks beyond that bound, SHALL be read as before, its apostrophe outside the word. In telling whether a quote is open, the apostrophe of a truncation or of a word read for an accent SHALL close none, and a quote mark after a space or the start of the text SHALL open one before `.`, `…` or a number but an elided decade (`‘80`). The rule SHALL bump Italian's analyser version and no other.

#### Scenario: An apostrophe for an accent
- **WHEN** « perche' », « piu' », « citta' », « e' » and « puo' » are tokenised as Italian with a pack that lists `perché`, `più`, `città`, `è` and `può`
- **THEN** the tokens are `perché`, `più`, `città`, `è` and `può`, each spanning the word and its apostrophe

#### Scenario: After an elided piece
- **WHEN** « non c'e' problema » and « dov'e' » are tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `non`, `ci`, `è`, `problema` and `dove`, `è`, each `è` spanning « e' »

#### Scenario: A capital
- **WHEN** « E' vero » is tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `È` and `vero`, `È` spanning « E' »

#### Scenario: Two accents listed
- **WHEN** « perche' » is tokenised with a pack listing `perché` under itself and `perchè` as a form of `perché`, and « caffe' » with a pack listing `caffè` under itself and `caffé` as a form of `caffè`
- **THEN** the tokens are `perché` and `caffè`

#### Scenario: A misspelling the pack ranks as a word of its own
- **WHEN** « E' » is tokenised with a pack listing `è` as a form of `essere` and `é` under itself
- **THEN** the token is `È`: neither is filed under the other, and `è` is tried first

#### Scenario: A stray apostrophe on a word the pack ranks low
- **WHEN** « vita' » and « governo' » are tokenised with a pack that lists `vità` under itself at rank 33,926 and `governò` as a form of `governare` at rank 3,722
- **THEN** the tokens are `vita`, its apostrophe outside the word, and `governò`

#### Scenario: The commoner of two accents
- **WHEN** « se' » is tokenised with a pack listing `sé` at rank 655 and `sè` at rank 2,911, neither filed under the other
- **THEN** the token is `sé`

#### Scenario: A quote an accent spelling does not close
- **WHEN** « Mi serve 'un po' di lavoro' », « Disse: 'c'e' lavoro' », « il cosiddetto '...lavoro' » and « '2 lavoro' » are tokenised with a pack that lists `lavorò` and `è`
- **THEN** each `lavoro` is read as written, the quote still open: `po'` and `e'` close none, and a quote mark before dots or a number opens one

#### Scenario: A listed word with its apostrophe
- **WHEN** « po' » and « da' » are tokenised as Italian with a pack that lists `pò` and `dà`
- **THEN** they are read as change 6's truncations, `poco` and `dai`, not as `pò` and `dà`

#### Scenario: An apocope
- **WHEN** « ne' », « su' » and « que' due » are tokenised as Italian with a pack that lists `né`, `sù` and `qué`
- **THEN** the tokens are `ne`, `su`, `que` and `due`, each apostrophe outside the word

#### Scenario: A word the tables do not accent
- **WHEN** « ciao' » is tokenised as Italian with a pack that lists no `ciaò`
- **THEN** the token is `ciao`, the apostrophe outside the word, as before

#### Scenario: A closing quote
- **WHEN** « ‘perche’ » is tokenised as Italian
- **THEN** the token is `perche`, the quotes outside the word: an apostrophe after a vowel is read for an accent only when no quote opened before the word is still open

#### Scenario: A pack built before is refused
- **WHEN** an Italian pack built for the analyser version before this rule is loaded
- **THEN** it is refused as built for another analyser generation

## MODIFIED Requirements

### Requirement: Italian tokenisation pre-pass
The core SHALL read Italian text through a pre-pass of its own, beside the rules that belong to no language. An elided word — `l'`, `un'`, `d'`, `c'`, `v'`, `m'`, `t'`, `s'`, `n'`, `ch'`, `gl'`, `dov'`, `com'`, `cos'`, `quand'`, `dev'`, `diss'`, `quest'`, `quell'`, `anch'`, `senz'`, `tutt'`, `bell'`, `buon'`, `grand'`, `pover'`, `sant'`, `mezz'`, `nient'`, `quant'`, `cent'`, `vent'`, `sott'`, `nessun'`, `qualcos'`, `qualcun'`, `ciascun'` or `alcun'`, written with the straight or the typographic apostrophe and followed by a letter — SHALL be split from the word it is joined to, each piece a token with its own source span, the elided piece's span holding its apostrophe, the elided piece read as the word it stands for (`lo`, `una`, `di`, `ci`, `vi`, `mi`, `ti`, `si`, `ne`, `che`, `gli`, `dove`, `come`, `cosa`, `quando`, `deve`, `disse` — `dissi` before `io` —, `questo`, `quello`, `anche`, `senza`, `tutto`, `bello`, `buona`, `grande`, `povero`, `santo`, `mezzo`, `niente`, `quanto`, `cento`, `venti`, `sotto`, `nessuna`, `qualcosa`, `qualcuna`, `ciascuna`, `alcuna`) with its written capital kept. The articulated prepositions of `di` — `del`, `dello`, `della`, `dei`, `degli`, `delle`, and `dell'` read as `dello` — SHALL stay whole; every other articulated preposition — `al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal`, `dallo`, `dalla`, `dai`, `dagli`, `dalle`, `nel`, `nello`, `nella`, `nei`, `negli`, `nelle`, `sul`, `sullo`, `sulla`, `sui`, `sugli`, `sulle`, `col`, `coi`, and the elided `all'`, `dall'`, `nell'`, `sull'` read with `lo` — SHALL be split into its preposition (`a`, `da`, `in`, `su`, `con`) and its article, the two tokens sharing the source span, the first keeping the written capital. A word ending in an apostrophe SHALL keep its apostrophe, its span holding it, only when the word is on a list: an elided form of this requirement with no letter after its apostrophe — written alone, « l’ » as a word card hands it, or before a number, « l'11 », « nell'800 » — read as the word it stands for, or a truncation with neither a letter nor a digit after its apostrophe — `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'` and `be'` read as `poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici` and `bene` — whose read word is final and not re-read as an articulated preposition, or a word, or the rest of one after its elided pieces, ending in a vowel and an apostrophe with neither a letter nor a digit after it that stands for an accent (*The apostrophe that stands for an accent reads as the accented word*), read as the accented form — and, but before a number, never when a single quote opened before the word is still open there, the apostrophe then closing it; any other trailing apostrophe SHALL stay outside the word, as the rule of no language leaves it. Nothing SHALL be split inside a hyphenated compound beyond an elision on its first piece. Italian's analyser version SHALL be bumped with this pre-pass, and no other language's tokenisation SHALL move.

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
- **WHEN** « perche' », « c'e' », « ‘ciao’ » and « ‘due anni fa’ » are tokenised as Italian with a pack that lists `perché`, `è` and `e`
- **THEN** the tokens are `perché`, `ci`, `è`, `ciao` and `due`, `anni`, `fa`, `perché` and `è` spanning their apostrophe, the closing quote outside `ciao`, and a quoted truncation's base not re-attached

#### Scenario: A hyphenated compound
- **WHEN** « l'anglo-americano » and « week-end » are tokenised as Italian with a pack listing neither
- **THEN** the tokens are `lo` and `anglo-americano`, and `week-end`

#### Scenario: An elided word on its own
- **WHEN** a word card asks the grammar of « l’ » for the dictionary form `lo`
- **THEN** the written word reads as the one token `lo`, spanning « l’ », with no pieces

#### Scenario: An elided word before a number
- **WHEN** « l'11 settembre », « nell'800 » and « dell'85 » are tokenised as Italian
- **THEN** the tokens are `lo` spanning « l' » and `settembre`, `in` and `lo` sharing the span of « nell' », and `dello` spanning « dell' », the digits dropped as any are — never a bare `l`, `nell` or `dell`

#### Scenario: A quote closed before a truncation
- **WHEN** « ‘Vieni!’ disse, un po' » and « anni ‘80, un po' » are tokenised as Italian
- **THEN** each ends with `poco` spanning « po' »: a quote closed after punctuation is closed, and a `‘` before a digit opens none

#### Scenario: A word card on a contraction
- **WHEN** a word card asks the grammar of « al » for the dictionary form `a`
- **THEN** it answers the pieces `a` and `il`

#### Scenario: Other languages keep their words
- **WHEN** « l'uomo », « nel » and « po' » are tokenised as English, as Spanish and as French, and the six invariance baselines run
- **THEN** each word is tokenised as before, and every probe of the six baselines is byte for byte the output recorded before, without re-blessing
