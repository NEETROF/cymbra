# Spec Delta

## MODIFIED Requirements

### Requirement: Italian tokenisation pre-pass
The core SHALL read Italian text through a pre-pass of its own, beside the rules that belong to no language. An elided word — `l'`, `un'`, `d'`, `c'`, `v'`, `m'`, `t'`, `s'`, `n'`, `ch'`, `gl'`, `dov'`, `com'`, `cos'`, `quand'`, `dev'`, `diss'`, `quest'`, `quell'`, `anch'`, `senz'`, `tutt'`, `bell'`, `buon'`, `grand'`, `pover'`, `sant'`, `mezz'`, `nient'`, `quant'`, `cent'`, `vent'`, `sott'`, `nessun'`, `qualcos'`, `qualcun'`, `ciascun'`, `alcun'`, `cinquant'`, `diciott'`, `fin'`, `gliel'`, `neanch'`, `novant'`, `null'`, `ogn'`, `ottant'`, `prim'`, `qual'`, `quarant'`, `sessant'`, `settant'`, `tant'`, `trent'`, `ultim'` or `ventiquattr'`, written with the straight or the typographic apostrophe and followed by a letter — SHALL be split from the word it is joined to, each piece a token with its own source span, the elided piece's span holding its apostrophe, the elided piece read as the word it stands for (`lo`, `una`, `di`, `ci`, `vi`, `mi`, `ti`, `si`, `ne`, `che`, `gli`, `dove`, `come`, `cosa`, `quando`, `deve`, `disse` — `dissi` before `io` —, `questo`, `quello`, `anche`, `senza`, `tutto`, `bello`, `buona`, `grande`, `povero`, `santo`, `mezzo`, `niente`, `quanto`, `cento`, `venti`, `sotto`, `nessuna`, `qualcosa`, `qualcuna`, `ciascuna`, `alcuna`, `cinquanta`, `diciotto`, `fino`, `glielo`, `neanche`, `novanta`, `nulla`, `ogni`, `ottanta`, `primo`, `quale`, `quaranta`, `sessanta`, `settanta`, `tanto`, `trenta`, `ultima`, `ventiquattro`) with its written capital kept. The articulated prepositions of `di` — `del`, `dello`, `della`, `dei`, `degli`, `delle`, `dell'` read as `dello`, `degl'` read as `degli`, and the truncated `de'` read as `dei` — SHALL stay whole; every other articulated preposition — `al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal`, `dallo`, `dalla`, `dai`, `dagli`, `dalle`, `nel`, `nello`, `nella`, `nei`, `negli`, `nelle`, `sul`, `sullo`, `sulla`, `sui`, `sugli`, `sulle`, `col`, `coi`, the elided `all'`, `dall'`, `nell'`, `sull'` and `coll'` read with `lo`, and the truncated `da'`, `a'`, `co'`, `ne'` and `su'` read with `i`, the last four only when a word follows them with nothing but a space between — SHALL be split into its preposition (`a`, `da`, `in`, `su`, `con`) and its article, the two tokens sharing the source span, the first keeping the written capital; `a'`, `co'`, `ne'` and `su'` with no word after them SHALL be read as before, their apostrophe outside the word. `dai` and `da'` SHALL instead be read as the verb, one token `dai` spanning the written word, its written capital kept, when the word just before it, with nothing but a space between them, is a clitic pronoun — `mi`, `ti`, `ci`, `gli`, `le`, `lo`, `la`, `ne`, `ce`, `glielo`, `gliela`, `glieli`, `gliele` or `gliene` —, or when what follows it, right after the written word, is `,`, `!`, `?`, `.`, `…` or a closing quotation mark, or nothing but spaces to the end of the text: a clitic is never followed by a preposition, nor the preposition by punctuation; after `non`, `me` or `te`, as anywhere else, they SHALL be read as the preposition. A word ending in an apostrophe SHALL keep its apostrophe, its span holding it, only when the word is on a list: an elided form of this requirement with no letter after its apostrophe — written alone, « l’ » as a word card hands it, or before a number, « l'11 », « nell'800 » — read as the word it stands for; a truncated articulated preposition of this requirement, read as above — `da'` with neither a letter nor a digit after its apostrophe, `a'`, `co'`, `ne'` and `su'` with a word after them —; a truncation with neither a letter nor a digit after its apostrophe — `po'`, `fa'`, `va'`, `sta'`, `di'`, `be'`, `ca'`, `de'` and `que'` read as `poco`, `fai`, `vai`, `stai`, `dici`, `bene`, `casa`, `dei` and `quei`, and `mo'` read as `modo` when the next word, as read, is `di` and as `mò` otherwise —, whose read word is final and not re-read as an articulated preposition; or a word, or the rest of one after its elided pieces, ending in a vowel and an apostrophe with neither a letter nor a digit after it that stands for an accent (*The apostrophe that stands for an accent reads as the accented word*), read as the accented form — and, but before a number, never when a single quote opened before the word is still open there, the apostrophe then closing it; any other trailing apostrophe SHALL stay outside the word, as the rule of no language leaves it. A written word alone, as a word card hands it, that this requirement reads two ways by its context — `dai`, `da'`, `mo'` — SHALL be read the way that gives the card's dictionary form, and, where neither way gives it, as the word alone reads — `dai` and `da'` the verb, `mo'` as `mò`. Nothing SHALL be split inside a hyphenated compound beyond an elision on its first piece. Italian's analyser version SHALL be bumped with this pre-pass, and no other language's tokenisation SHALL move.

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

#### Scenario: Elided numbers and quantities
- **WHEN** « trent'anni », « cinquant'anni », « ventiquattr'ore », « tant'è », « null'altro » and « ogn'uno » are tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `trenta` and `anni`, `cinquanta` and `anni`, `ventiquattro` and `ore`, `tanto` and `è`, `nulla` and `altro`, `ogni` and `uno`, each elided piece spanning its letters and its apostrophe

#### Scenario: More elided words
- **WHEN** « Qual'è », « gliel'ho », « neanch'io », « fin'ora », « prim'ordine » and « l'ultim'ora » are tokenised as Italian with a pack that lists `è`
- **THEN** the tokens are `Quale` and `è`, `glielo` and `ho`, `neanche` and `io`, `fino` and `ora`, `primo` and `ordine`, and `lo`, `ultima` and `ora`

#### Scenario: Two more elided prepositions
- **WHEN** « coll'aiuto » and « degl'Italiani » are tokenised as Italian
- **THEN** the tokens are `con`, `lo` and `aiuto`, `con` and `lo` sharing the span of « coll' », and `degli` and `Italiani`, `degli` spanning « degl' »

#### Scenario: Truncations a book writes
- **WHEN** « Ca' Foscari », « Lorenzo de' Medici » and « que' due » are tokenised as Italian
- **THEN** the tokens are `Casa` and `Foscari`, `Lorenzo`, `dei` and `Medici`, and `quei` and `due`, `Casa` spanning « Ca' », `dei` « de' » and `quei` « que' »

#### Scenario: A truncated articulated preposition
- **WHEN** « a' piedi », « co' suoi », « ne' campi » and « su' colli » are tokenised as Italian
- **THEN** the tokens are `a`, `i`, `piedi`; `con`, `i`, `suoi`; `in`, `i`, `campi`; `su`, `i`, `colli` — each preposition sharing with its article the span of the written word, apostrophe included

#### Scenario: A truncated preposition with no word after it
- **WHEN** « ne' » and « Vieni su'. » are tokenised as Italian, and « 'Vieni su', disse, e un po' » with a single quote opened before `Vieni`
- **THEN** « ne' » is `ne` and « su' » `su`, each apostrophe outside the word, as before; and in the quoted phrase the apostrophe after `su` closes the quote, so that `po'` after it is read as `poco`

#### Scenario: mo' by the word after it
- **WHEN** « a mo' di esempio », « a mo' d'esempio » and « mo' arrivo » are tokenised as Italian
- **THEN** the tokens are `a`, `modo`, `di`, `esempio` twice, `di` spanning « d' » the second time, and `mò` and `arrivo`, each read word spanning « mo' »

#### Scenario: dai, the verb
- **WHEN** « Dai, andiamo! », « mi dai una mano? », « Me lo dai? », « Ma dai! » and « “Ma dai” disse » are tokenised as Italian
- **THEN** each `dai` is one token, `Dai` or `dai`, spanning the written word, and no `da` or `i` is written

#### Scenario: dai, the preposition
- **WHEN** « Torno dai nonni », « dai molti suoi cocuzzoli », « non dai retta » and « vieni con me dai nonni? » are tokenised as Italian
- **THEN** each `dai` is `da` and `i`, sharing the written word's span: neither `non` nor `me` is a sign of the verb

#### Scenario: da' by its context
- **WHEN** « da' primi suoi anni » and « mi da' una mano? » are tokenised as Italian
- **THEN** the first « da' » is `da` and `i` sharing its span, apostrophe included, and the second is the one token `dai` spanning « da' »

#### Scenario: A word card on a word read by its context
- **WHEN** a word card asks the grammar of « dai » for the dictionary form `da`, then for `dare`, and of « mo' » for `modo`, then for `mezzo`, which neither reading gives
- **THEN** the first answers the pieces `da` and `i`, the second the one token `dai` with no pieces, the third the one token `modo`, and the fourth the one token `mò`, as the word alone reads

#### Scenario: Other languages keep their words
- **WHEN** « l'uomo », « nel » and « po' » are tokenised as English, as Spanish and as French, and the six invariance baselines run
- **THEN** each word is tokenised as before, and every probe of the six baselines is byte for byte the output recorded before, without re-blessing

### Requirement: The apostrophe that stands for an accent reads as the accented word
The core SHALL read an Italian word — or what remains of one after its elided pieces are split off — ending in a vowel and an apostrophe with neither a letter nor a digit after it, no single quote opened before the word still open there, as the form the pack lists whose final vowel is that vowel accented, when the pack lists such a form and, where the pack carries its ranks, ranks that form's word within 23,000: one token spanning the rest and its apostrophe, its text the accented form, its written capital kept. Where the pack lists the vowel under both accents, a form the pack files under the other SHALL be read as the other's spelling and the other SHALL win; where neither is filed under the other, the one whose word the pack ranks higher SHALL win, and `è` SHALL be tried before `é` where the pack ranks neither. A word Italian's pre-pass lists with its apostrophe — a truncation, a truncated articulated preposition, an elided form written alone — SHALL keep that reading, whatever the pack lists; the literary apocopes and truncations the pre-pass does not read — `pe'`, `ve'`, `mi'`, `fra'`, `to'`, `i'`, `me'`, `vo'` and `fe'`, and `a'`, `co'`, `ne'` and `su'` with no word after them — SHALL never be read for an accent; and a word whose accented form the pack does not list, or ranks beyond that bound, SHALL be read as before, its apostrophe outside the word. In telling whether a quote is open, the apostrophe of a truncation or of a truncated articulated preposition the pre-pass reads with its apostrophe, or of a word read for an accent, SHALL close none, and a quote mark after a space or the start of the text SHALL open one before `.`, `…` or a number but an elided decade (`‘80`). The rule SHALL bump Italian's analyser version and no other.

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
- **WHEN** « un po' », « da' primi anni » and « mi da' una mano » are tokenised as Italian with a pack that lists `pò` and `dà`
- **THEN** they are read as Italian's pre-pass reads them — `poco`; `da` and `i` sharing the span of « da' »; the verb `dai` spanning « da' » — never as `pò` or `dà`

#### Scenario: An apocope
- **WHEN** « to' », « ve' », « mi' » and « ne' » are tokenised as Italian with a pack that lists `tò`, `vè`, `mì` and `né`
- **THEN** the tokens are `to`, `ve`, `mi` and `ne`, each apostrophe outside the word

#### Scenario: A book's truncations are not accents
- **WHEN** « ne' campi », « su' colli » and « que' due » are tokenised as Italian with a pack that lists `né`, `sù` and `qué`
- **THEN** « ne' » is `in` and `i`, « su' » `su` and `i`, and « que' » `quei`, each spanning its apostrophe — never `né`, `sù` or `qué`

#### Scenario: A word the tables do not accent
- **WHEN** « ciao' » is tokenised as Italian with a pack that lists no `ciaò`
- **THEN** the token is `ciao`, the apostrophe outside the word, as before

#### Scenario: A closing quote
- **WHEN** « ‘perche’ » is tokenised as Italian
- **THEN** the token is `perche`, the quotes outside the word: an apostrophe after a vowel is read for an accent only when no quote opened before the word is still open

#### Scenario: A pack built before is refused
- **WHEN** an Italian pack built for the analyser version before this rule is loaded
- **THEN** it is refused as built for another analyser generation

### Requirement: Italian closed classes
The core SHALL flag as function words, in a word-by-word gloss in Italian, the Italian articles and other determiners, pronouns, prepositions, conjunctions, the quantifiers `molto`, `tanto`, `poco`, `tutto` and `troppo` among the determiners, the auxiliaries and modals `essere`, `avere`, `potere` and `dovere`, and the negation `non`, `no` and `né`, by the lemma the cascade produced, as English's, Spanish's and French's tables do for their languages; among them SHALL be the prepositions `davanti` and `dietro`, the determiners `ambedue`, `entrambi` and `parecchio` with their inflected forms, and the conjunctions `anziché`, `bensì`, `nonché`, `ossia`, `pertanto` and `qualora`. The pieces of a split articulated preposition and of an elision SHALL be flagged by their own lemmas. `mai`, `più`, `anche`, `neanche`, `nemmeno`, `neppure`, `volere`, `secondo`, `lungo`, `prima`, `salvo` and `stare` SHALL NOT be flagged, and neither SHALL a combined clitic that is a noun standing alone (`mela`, `vela`, `tela`, `velo`, `cene`).

#### Scenario: An Italian phrase
- **WHEN** « Non ho visto il cane di Marco » is glossed word by word as Italian with a pack filing `ho` under `avere`
- **THEN** `non`, `avere` (of « ho ») and `di` and `il` are function words, and `visto`, `cane` and `Marco` are not

#### Scenario: The negation
- **WHEN** « non lo so » and « né oggi né mai » are glossed word by word as Italian
- **THEN** `non`, `lo` and `né` are function words, and `mai` is not

#### Scenario: Words that look closed but carry meaning
- **WHEN** « più », « mai », « anche », « sto bene », « sto parlando », « stare », « voglio », « il secondo giorno », « prima di tutto », « una mela » and « molto tempo » are glossed word by word as Italian with a pack filing `sto` under `stare` and `voglio` under `volere`
- **THEN** none of `più`, `mai`, `anche`, `stare`, `volere`, `secondo`, `prima` and `mela` is a function word, and `molto` in « molto tempo » is one

#### Scenario: Closed words the treebanks read as closed
- **WHEN** « fino a dove », « nonostante tutto » and « salvo qualunque cosa » are glossed word by word as Italian
- **THEN** `fino`, `a`, `dove`, `nonostante`, `tutto` and `qualunque` are function words, and `salvo` and `cosa` are not

#### Scenario: The pieces of a contraction and of an elision
- **WHEN** « al mare » and « l'uomo » are glossed word by word as Italian
- **THEN** `a`, `il` and `lo` are function words, and `mare` and `uomo` are not

#### Scenario: Closed words an Italian reader's review added
- **WHEN** « davanti alla porta », « dietro la casa », « entrambe le mani », « ambedue i figli », « parecchie volte », « anziché partire », « non oggi, bensì domani », « il pane nonché il vino », « ossia », « pertanto » and « qualora venisse » are glossed word by word as Italian with a pack filing `entrambe` under `entrambi` and `parecchie` under `parecchio`
- **THEN** `davanti`, `dietro`, `entrambi`, `ambedue`, `parecchio`, `anziché`, `bensì`, `nonché`, `ossia`, `pertanto` and `qualora` are function words, and `porta`, `casa`, `mani`, `figli`, `volte`, `partire`, `domani`, `pane` and `vino` are not

#### Scenario: Each language keeps its own tables
- **WHEN** `the` is glossed as Italian, and `di` as English, Spanish or French
- **THEN** neither is a function word
