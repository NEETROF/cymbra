## ADDED Requirements

### Requirement: French's hyphenated words without their marks and its wrong accents read as the words a reviewed list names
fr-en's reducer SHALL read a hyphenated word the web writes without its accents, cedilla or diaeresis, and a word written with marks that are not its own, as another French word only when one of the owner's two reviewed lists names that spelling, row by row, beside change 43e's list; no rule SHALL read a spelling by its marks alone. A hyphenated row SHALL name the whole run (`peut-etre` → *peut-être*), so that the pack lists it and French's pre-pass reads it as one token, as it reads a run the pack lists; a misaccented row SHALL name the word of the form whose marks the spelling keeps in the most places, then the commoner (`agé` → *âgé*, `elève` → *élever*). A row of either list SHALL name a spelling to which the English Wiktionary's French section gives no meaning of its own and which it does not give only as another word's spelling, and the reduction SHALL fail otherwise, as it does for change 43e's list. The lists SHALL hold the spellings the owner approved — among them `peut-etre`, `c'est-a-dire`, `lui-meme`, `grand-mere`, `la-bas`, `apres-midi`, `celle-la` → *celui-là*; `trés` → *très*, `déja` → *déjà*, `aprés` → *après*, `pére` → *père*, `oú` → *où*, `arreté` → *arrêter* — and SHALL never hold a spelling another language writes more often than French (`á`, `lá`, `accés`, `procés`), `çe` or `abbès`. A spelling a list names SHALL be a form of the named word alone: no ranked lemma, French dictionary word, levelled word or word a pair glosses. It SHALL carry the readings of its word's forms written with the same letters — for a spelling with marks, of those whose marks it keeps in the most places, when it keeps any —, each kept as its own tag. fr-es SHALL read the spellings through the `tables/fr/` fr-en writes, French's levels SHALL not move, and en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: A hyphenated word without its marks is one word
- **WHEN** a reader studying French through fr-en selects « peut-etre qu'il viendra »
- **THEN** « peut-etre » is one token read as *peut-être* « maybe, perhaps », not a compound whose pieces read as *pouvoir* and `etre`

#### Scenario: An elision and an inversion do not split a listed run
- **WHEN** a French page writes « c'est-a-dire demain » and « celle-la »
- **THEN** « c'est-a-dire » is one token read as *c'est-à-dire*, not `ce` and a compound, and « celle-la » one token read as *celui-là*, not `celle` and `la`

#### Scenario: A wrong accent reads as its word
- **WHEN** fr-en is reduced from its pinned sources with the reviewed lists
- **THEN** `tables/fr/forms.tsv` maps `trés` to *très* and `déja` to *déjà*, neither is in `freq.tsv`, `lexical.tsv`, `level.tsv` or fr-en's `gloss.tsv`, and *très* and *déjà* keep their ranks and their levels

#### Scenario: The marks kept in place choose the word and its readings
- **WHEN** the card asks the grammar of `arreté` as *arrêter*, and fr-en is reduced with `agé` in the list
- **THEN** `arreté` answers the masculine singular past participle alone, and `agé` maps to *âgé*, not to *âge*

#### Scenario: The dictionary names it, so no row does
- **WHEN** a row names a spelling the French section gives only as another word's spelling — `problême`, « alternative form of problème »
- **THEN** the reduction fails, naming the row

#### Scenario: Another language's word is no row
- **WHEN** fr-en is reduced
- **THEN** `á` and `lá` are no forms of the tables, and `élève` and `élevé` read as they did before this change

#### Scenario: A spelling whose word is no form of the tables
- **WHEN** a French page writes « elle l'a fait elle-meme »
- **THEN** « elle-meme » reads as it did before this change, since `elle-même` is no form of the tables and no row names it

#### Scenario: The levels do not move
- **WHEN** the change's tables are committed
- **THEN** `tables/fr/level.tsv` is byte for byte as before

#### Scenario: fr-es follows French's tables
- **WHEN** fr-es is reduced again on the `tables/fr/` fr-en writes
- **THEN** no spelling of the lists is a lemma fr-es glosses, a lemma entering the cut may gain its gloss, every other gloss is byte for byte as before, and its pin records the new studied tables and its pack

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing
