# Spec Delta

## ADDED Requirements

### Requirement: Italian's estimated levels
Italian's level table SHALL be estimated from word frequency, as Spanish's and French's are, and written by the reduction of Italian's reference pair, it-fr, into `tables/it/level.tsv`: in rank order, the commonest Italian lemmas a CEFR list would hold SHALL take the sizes of English's CEFR levels — 1,020 at A1, 1,158 at A2, 2,015 at B1, 2,347 at B2, 886 at C1 and 876 at C2 — kept as constants equal to those es-fr's and fr-en's reductions use. Which lemmas a CEFR list would hold SHALL be read from the English Wiktionary's Italian section, the source it-fr's forms come from, and, until Italian's dictionary words are written, never from a pair's glosses; once they are, a level SHALL go to a dictionary word alone, as French's do. A ranked lemma SHALL take no level when the section gives it no sense that is not a form of another word, or only a name's; when it is a single character whose every sense in the section is a letter's, a symbol's, a name's or an abbreviation; when every sense the section gives it only spells another word; when its own form reads as another lemma in Italian's forms table; or when it is a row of Italian's acronym table. Every pack studying Italian that is built from the committed tables SHALL say in its metadata that its levels are estimated. No list whose licence the pack builder refuses SHALL be read by the reduction or committed.

#### Scenario: The commonest words are A1
- **WHEN** it-fr is reduced
- **THEN** `di`, `e`, `il`, `essere`, `avere`, `che` and `a` are A1, and `al` and `nel`, which are no words of Italian's tables, have no level

#### Scenario: Six levels of English's sizes
- **WHEN** it-fr is reduced
- **THEN** `tables/it/level.tsv` holds 8,302 lemmas, as many at each level as English's CEFR lists, each a ranked lemma of `tables/it/freq.tsv`

#### Scenario: What a CEFR list leaves out
- **WHEN** it-fr is reduced, the section writes `Roma` only as a place with a capital and does not list `the`, gives `k` only as a letter's name, gives `perchè` only as a misspelling of *perché*, and the acronym table holds `NATO`
- **THEN** `roma`, `the`, `k`, `perchè` and `NATO` have no level, while `e` and `o`, conjunctions, are A1

#### Scenario: A lemma whose own form reads as another
- **WHEN** it-fr is reduced, `porta` is a ranked lemma, and the forms table maps the form `porta` to *portare*
- **THEN** `porta` has no level, and *portare* has the level its own rank gives it

#### Scenario: The table does not wait for the glosses
- **WHEN** it-fr's glosses are committed later, with no change to the section, the ranks or the forms, and before Italian's dictionary words narrow the levels
- **THEN** `tables/it/level.tsv` is byte for byte as before

#### Scenario: Every Italian pack says so
- **WHEN** a pack studying Italian is built from the committed tables, glossed in French, English or Spanish
- **THEN** its metadata says its levels are estimated, the engine reports them as estimated, and the ladder shows English's typical vocabularies and says they are English's

#### Scenario: Nothing else moves
- **WHEN** Italian's level table is committed while the Italian invariance baseline still runs over its fixture pack
- **THEN** the six other pairs' tables, pins and packs are byte for byte as before, their invariance baselines pass without re-blessing, and the Italian invariance baseline does not move
