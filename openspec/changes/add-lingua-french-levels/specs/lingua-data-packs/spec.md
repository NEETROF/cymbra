## ADDED Requirements

### Requirement: French's estimated levels
French's level table SHALL be estimated from word frequency, as Spanish's is, and written by the reduction of French's reference pair, fr-en, into `tables/fr/level.tsv`: in rank order, the commonest French lemmas a CEFR list would hold SHALL take the sizes of English's CEFR levels — 1,020 at A1, 1,158 at A2, 2,015 at B1, 2,347 at B2, 886 at C1 and 876 at C2 — kept as constants equal to those es-fr's reduction uses. Which lemmas a CEFR list would hold SHALL be read from the English Wiktionary's French section, the source fr-en's forms come from, and never from a pair's glosses. A ranked lemma SHALL take no level when the section gives it no sense that is not a form of another word, or only a name's; when it is a single character whose every sense in the section is a letter's, a symbol's, a name's or an abbreviation; when every sense the section gives it only spells another word; or when its own form reads as another lemma in French's forms table. Every pack studying French that is built from the committed tables SHALL say in its metadata that its levels are estimated. No FLELex data, nor any other list whose licence the pack builder refuses, SHALL be read by the reduction or committed.

#### Scenario: The commonest words are A1
- **WHEN** fr-en is reduced
- **THEN** `de`, `le`, `et`, `à`, `être`, `avoir`, `du` and `des` are A1, and `au` and `aux`, which are no words of French's tables, have no level

#### Scenario: Six levels of English's sizes
- **WHEN** fr-en is reduced
- **THEN** `tables/fr/level.tsv` holds 8,302 lemmas, as many at each level as English's CEFR lists, each a ranked lemma of `tables/fr/freq.tsv`

#### Scenario: What a CEFR list leaves out
- **WHEN** fr-en is reduced, the section names `paris` only as a place and does not list `the`, gives `b` only as a letter's name, and gives `etre` only as an obsolete spelling of `être`
- **THEN** `paris`, `the`, `b` and `etre` have no level, while `y`, a pronoun, is A1

#### Scenario: A lemma whose own form reads as another
- **WHEN** fr-en is reduced, `donnée` is a ranked lemma, and the forms table maps the form `donnée` to *donner*
- **THEN** `donnée` has no level, and *donner* has the level its own rank gives it

#### Scenario: The table does not wait for the glosses
- **WHEN** fr-en's glosses are committed later, with no change to the section, the ranks or the forms
- **THEN** `tables/fr/level.tsv` is byte for byte as before

#### Scenario: Every French pack says so
- **WHEN** a pack studying French is built from the committed tables, glossed in English or in Spanish
- **THEN** its metadata says its levels are estimated, the engine reports them as estimated, and the ladder shows English's typical vocabularies and says they are English's

#### Scenario: Nothing else moves
- **WHEN** French's level table is committed while the French invariance baseline still runs over its fixture pack
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, their invariance baselines pass without re-blessing, and the French invariance baseline does not move

### Requirement: A level reaches the lemma it is written for
The pack built from a pair's committed tables SHALL give every lemma of its studied language's level table the level that table gives it, and no other lemma a level the table does not give it. The checks SHALL build every committed pair's pack and fail, naming the pair and the lemma, when a level lands on another lemma than the one it is written for.

#### Scenario: The committed pairs
- **WHEN** the checks build en-fr, es-fr, es-en, en-es and fr-en from their committed tables
- **THEN** each pack gives every lemma of its studied language's `level.tsv` exactly that level

#### Scenario: A level written for a lemma whose form reads as another
- **WHEN** a level table gives `donnée` B1 and `donner` A1, and the forms table maps the form `donnée` to *donner*
- **THEN** the checks fail, naming the pair and `donner`, which the pack gives the level written for `donnée`
