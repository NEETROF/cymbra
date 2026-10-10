## ADDED Requirements

### Requirement: fr-es leaves out, by name, the expressions read wrong where French text meets them
The fr-es reduction SHALL leave out of `mwe.tsv` every expression a reviewed list names, each with its reason, whatever source glosses it, and SHALL name on its summary line a listed expression no source glosses any more.
The list names an expression the engine meets on French text and reads wrong in at least 60 % of its
matches — a partitive read where the page means « of the », a shorter idiom swallowing a compound noun,
words met as they are where the source names one use or a set phrase, a form of another word met as
the headword, a wrong translation no other source corrects, a name. Where another source glosses the
expression right, the wrong text is corrected by name instead (*fr-es corrects, by name, the slips its
sources write*), so that source glosses it. The words of a listed expression keep their glosses. No
other pair's expressions, and no gloss of a word, move with the list.

#### Scenario: A partitive met as « of the »
- **WHEN** the section glosses « de la » « Algo de, un poco de, una cierta cantidad de », and the list names it
- **THEN** « le maire de la ville » meets no expression, and `de` and `la` keep their glosses

#### Scenario: A shorter idiom before a compound noun
- **WHEN** the direct table glosses « sur le champ » « En el acto » and « champ de bataille » « Campo de batalla », and the list names « sur le champ »
- **THEN** « sur le champ de bataille » meets « champ de bataille »

#### Scenario: A form of another word
- **WHEN** the direct table glosses « de fait » « De hecho », which the builder keys as « de faire » is read, and the list names it
- **THEN** « Il a décidé de faire le voyage » meets no « de fait »

#### Scenario: Every headword of one key
- **WHEN** the direct table glosses « laisser faire » « Dejarlo » and the inverted table « laissez faire » « Laissez faire », one key, and the list names both
- **THEN** « On laisse faire » meets no expression

#### Scenario: A wrong sense another source glosses right
- **WHEN** the section glosses « se lever » « Estar de pie uno », the direct table « Levantarse », and a correction sets the section's text aside
- **THEN** « Elle se leva » meets « se lever » « Levantarse »

#### Scenario: A listed expression no source glosses
- **WHEN** an update finds no source glossing an expression the list names
- **THEN** the summary line names it, to be removed from the list

#### Scenario: Only fr-es's expressions move
- **WHEN** fr-es is reduced again from its pinned sources with the list
- **THEN** its `gloss.tsv`, `senses.tsv` and coverage are byte for byte, its pin keeps its snapshot, its studied record and its sources, and en-fr's, es-fr's, es-en's, en-es's and fr-en's tables and pins, and every table of `tables/fr/`, are byte for byte unchanged

### Requirement: fr-es writes its expressions as French pages write them
The fr-es reduction SHALL write in its traditional spelling an expression its tables list only in a post-1990 spelling — one word of it whose circumflex restored on an i or a u, or whose « ognon » written « oignon », is a form of the same lemma in French's forms table that wordfreq rates commoner — and SHALL write an elided piece against the word after it.
A headword the tables list beside its traditional spelling is kept as written. The key the builder
gives a rewritten headword is its spelling's key; the name a reader sees moves.

#### Scenario: A rectified spelling alone
- **WHEN** the direct table lists « crème brulée » and no « crème brûlée », and French's forms table reads `brulée` and `brûlée` as forms of `brûler`, `brûlée` the commoner
- **THEN** fr-es's expression is « crème brûlée », and « une crème brûlée » meets it under that name

#### Scenario: Both spellings listed
- **WHEN** the direct table lists « boîte aux lettres » and « boite aux lettres »
- **THEN** both are fr-es expressions, as written

#### Scenario: An elided piece written apart
- **WHEN** the inverted table lists « huile d' olive » and the direct table « huile d'olive » « Aceite de oliva »
- **THEN** fr-es has one expression « huile d'olive », the direct table's, and « de l'huile d'olive » meets it under that name
