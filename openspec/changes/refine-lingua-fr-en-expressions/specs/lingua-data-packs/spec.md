## ADDED Requirements

### Requirement: fr-en's own expressions are read where they are met
fr-en SHALL offer an expression of the English Wiktionary's French section only where its gloss says what the words mean where a reader meets them: an expression a person read wrong in most of the contexts it is met in SHALL be left out by name with its reason, a sense that only notes a form or a usage SHALL be no meaning, and « de l' » SHALL read « of the » first.
The names left out are reviewed on French text, each with its gloss, how it is met and an example: a
sense the section labels with a register or a region met as plain words, a headword met through
another word's form, common words met as they are where the headword names a set phrase, a field's
sense or a cry, and a colloquial spelling met as the standard one. A sense, or a piece of it between
semicolons, that opens on a form's or a usage's note — a compound's pieces, an interrogative or
inflected form of a verb in a person, a pointer to the non-idiomatic uses, an equivalent written in
French, the one construction it is used in — goes, and an expression left with no sense is none and
counts as only pointing. « de l' » opens on « de la »'s first sense before its own. A name left out
and an expression that only points are not read from the French Wiktionary's table either. These
rules read the section's expressions alone: fr-en's word glosses, sense runs and dictionary words,
French's tables and every other pair do not move with them.

#### Scenario: A labelled sense met as plain words
- **WHEN** the section glosses « pas de » "without", a Quebec sense, and « sa mère » "(so) fucking", slang, both left out by name with their reasons
- **THEN** « Il n'a pas d'enfants. » meets no « pas de », and « Sa mère est venue. » no « sa mère »

#### Scenario: A colloquial spelling met as the standard one
- **WHEN** the section glosses « m'a » "I'm going", left out by name
- **THEN** « Il m'a sauvé la vie. » meets no « m'a »

#### Scenario: A headword met through another word's form
- **WHEN** the section glosses « en fait » "actually" and « en fait de » "with respect to", both left out by name
- **THEN** « Il est arrivé en faisant du bruit. » meets « faire du bruit », not « en fait » nor « en fait de »

#### Scenario: Words met as they are
- **WHEN** the section glosses « première fois » "first time (the first instance of sexual intercourse)", left out by name
- **THEN** « Pour la première fois, elle a souri. » meets no « première fois »

#### Scenario: A sense that only notes a form
- **WHEN** the section glosses « puis-je » "an interrogative form of je peux" alone and « j'ai » "I've, I have; compound of je and ai"
- **THEN** fr-en has no « puis-je », and glosses « j'ai » "I've, I have"

#### Scenario: « de l' » reads "of the" first
- **WHEN** the section glosses « de l' » "some; the singular prevocalic partitive article" and « de la » "of the; some; the feminine partitive article"
- **THEN** « Il est à la tête de l'équipe. » and « Je bois de l'eau. » meet « de l' », its gloss opening "of the; some"

#### Scenario: The table does not bring them back
- **WHEN** the French Wiktionary's table glosses « en fait » "in fact" and « de rien » "you're welcome", both left out by name in fr-en
- **THEN** fr-en's `mwe.tsv` has neither « en fait » nor « de rien »

#### Scenario: Nothing else moves
- **WHEN** fr-en is re-reduced at its pin with these rules
- **THEN** `tables/fr/`, fr-en's `gloss.tsv` and `senses.tsv`, and the tables, pins and packs of fr-es, en-fr, es-fr, es-en and en-es are byte for byte what they were
