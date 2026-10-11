## ADDED Requirements

### Requirement: An expression's gloss keeps its senses whole
Every pair's reduction SHALL write an expression's gloss with each sense it picks whole — never cut at a fixed number of characters, never mid-word —, SHALL NOT cut the joined gloss, and SHALL bound a sense only past 300 characters, the bound of a word's sense, cutting it there after a whole word and ending it with « … ».
The senses are picked as before: up to three, one per entry in turn, a sense equal to one already
picked left out — compared whole. The rule is the same for every source of an expression's senses: a
Wiktionary section's multi-word entries, the words French's tokenisation splits that fr-en glosses as
expressions, and a sense one expression lends another (fr-en's « de l' »). A label fr-es opens a sense
on lies outside the bound. The translation tables' fallback glosses keep their count of words. It is a
rule every pair shares: it re-pins every pair, and moves no gloss of a word, no sense run, no studied
table and no coverage.

#### Scenario: A sense longer than the old cut
- **WHEN** es-en's section defines `sin embargo` "however, notwithstanding, nevertheless, in spite of, nonetheless"
- **THEN** its gloss reads "however, notwithstanding, nevertheless, in spite of, nonetheless", not "however, notwithstanding, nevertheless, in"

#### Scenario: Senses longer than 80 characters together
- **WHEN** an expression's three senses, picked one per entry, hold 150 characters together
- **THEN** its gloss holds all three whole, separated by « ; »

#### Scenario: A sense past the bound
- **WHEN** a sense of an expression runs 1,132 characters, as es-fr's `sistema cardiovascular`'s second does
- **THEN** the gloss holds it cut after the last whole word within 300 characters, « … » included, and its other senses whole

#### Scenario: Senses the cut made alike
- **WHEN** two senses of fr-en's `saint-cyrien` begin with the same 42 characters and differ after them
- **THEN** both are picked, whole

#### Scenario: A lent sense
- **WHEN** fr-en lends « de la »'s first sense to « de l' », whose own senses read "some; the singular prevocalic partitive article"
- **THEN** the gloss of « de l' » reads "of the; some; the singular prevocalic partitive article", uncut

#### Scenario: A word's gloss does not move
- **WHEN** a pair is reduced again with this rule
- **THEN** its `gloss.tsv`, `senses.tsv` and its studied language's tables are byte for byte what they were, and only `mwe.tsv`, `manifest.json` and `pin.json` move

## MODIFIED Requirements

### Requirement: A gloss groups its senses by part of speech
A pack's gloss SHALL list its senses grouped by part of speech, each group in the order its part of speech first appears among the senses picked, and SHALL NOT hold the sense separator inside a sense.
The senses SHALL be picked as the pack picked them before this change, one per entry in turn, up to
eight for a word, but for the acronyms and the Wiktionary's notes the next requirements leave out.
They are then reordered so that the senses of one part of speech are adjacent, and a separator
inside a sense becomes a comma. A word's gloss SHALL hold each sense within 300 characters and all
of them within 800: the card pages it, so its length is no longer what one card can show. A cut SHALL
fall on a word boundary and end with an ellipsis, never mid-word, and a sense that would keep fewer
than 20 characters SHALL be left out rather than cut. An expression's gloss is bounded only as *An expression's gloss keeps its senses whole* says. The part-of-speech table SHALL account
for exactly the senses the final gloss holds, and the build SHALL fail when the two disagree.

#### Scenario: Senses picked across parts of speech
- **WHEN** the senses picked for a word are a preposition sense, a particle sense and a second preposition sense, in that order
- **THEN** its gloss holds the two preposition senses, then the particle sense, and the table records two preposition senses followed by one particle sense

#### Scenario: A separator inside a sense
- **WHEN** a picked sense reads « Lettre; caractère »
- **THEN** the gloss holds it as « Lettre, caractère », one sense

#### Scenario: A word with many senses
- **WHEN** the dictionary gives a word six senses of a few words each
- **THEN** its gloss holds all six

#### Scenario: A long sense ends on a whole word
- **WHEN** a word's sense is longer than the room its gloss has left for it
- **THEN** the gloss holds it cut after a whole word and ended with « … », never mid-word, or leaves it out when fewer than 20 characters of room remain

#### Scenario: A table that disagrees with its gloss
- **WHEN** the part-of-speech table records a different number of senses than a word's gloss holds
- **THEN** the build fails and names the word

### Requirement: fr-es's glosses show a sense's register, age and place in the Spanish Wiktionary's words
The fr-es reduction SHALL open a sense of a word or an expression with the labels the Spanish Wiktionary's French section gives it of three kinds — its register, its age and its place —, in parentheses, in the edition's own Spanish words as its categories file the sense, register first, then age, then place; SHALL NOT move a sense in its row because it is labelled; and SHALL NOT let an expression's label shorten its meaning.
The words come from closed tables: register « coloquial », « malsonante », « jergal », « despectivo »,
« literario », « vulgar », « eufemístico », « formal », « infantil », « irónico », « jocoso »; age
« anticuado », « obsoleto »; place the regions and countries the section names (« Quebec »,
« Canadá », « Francia », « Bélgica », « Suiza »…), a place left out when a place inside it is shown. A
category whose template named another language than French is read alike. A figurative or a rare
sense shows no label, and neither does a sense that only points at another word. Each labelled sense
carries its own labels. The senses the edition marks outdated or obsolete stay after the current ones
of their entry, as the Spanish edition's pre-pass writes them. An expression's senses are labelled
after the rules every pair shares have read them, so its meaning keeps every character those rules
keep and the label lies outside the bound an expression's sense keeps. These are rules of fr-es's reducer alone: they
re-pin fr-es and no other pair, and no lemma gains or loses a gloss through them.

#### Scenario: A vulgar sense and an outdated one
- **WHEN** the section defines `baiser` as a verb whose first sense « Coger (sexualmente). » is filed under « Términos malsonantes », and whose sense « Besar. » is filed under « Términos anticuados », « Canadá » and « Bélgica », and as a noun « Beso, besuqueo u ósculo. »
- **THEN** the gloss of `baiser` reads « (malsonante) Coger (sexualmente) » first, « (anticuado, Canadá, Bélgica) Besar » after the verb's other senses, and « Beso, besuqueo u ósculo » unlabelled

#### Scenario: A place inside another
- **WHEN** a sense of `oui` is filed under « América » and « Quebec »
- **THEN** it opens on « (Quebec) »

#### Scenario: A label the English tags miss
- **WHEN** a sense of `avec` is filed under « Quebec » and « Bélgica » and kaikki tags it `Canada` and `Quebec` alone
- **THEN** it opens on « (Quebec, Bélgica) »

#### Scenario: No reordering for a label
- **WHEN** the section's `cul` opens its noun on « Culo. », filed under « Términos malsonantes », before « Fundo (de un objeto). »
- **THEN** the gloss of `cul` still opens on « (malsonante) Culo »

#### Scenario: An expression's label outside its cut
- **WHEN** the section defines `mal aux cheveux` « Resaca, caña, chaqui, chuchaqui, cruda, goma, guayabo, hachazo, hangover, perseguidora, ratón. », filed under « Términos anticuados »
- **THEN** its gloss reads « (anticuado) Resaca, caña, chaqui, chuchaqui, cruda, goma, guayabo, hachazo, hangover, perseguidora, ratón », the whole sense with its label, the same meaning as without the label

#### Scenario: A figurative sense
- **WHEN** a sense of a word is tagged figurative and filed under « Términos en sentido figurado » alone
- **THEN** it shows no label
