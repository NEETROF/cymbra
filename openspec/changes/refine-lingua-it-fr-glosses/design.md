# Design — refine-lingua-it-fr-glosses

## Context

See proposal.md (Why). What exists, on `main` at `4eebb6dad` (change 16, #1054; its tables committed
by the owner's update of 2026.10.24, `lingua-pack-update` run 40130354122):

| Where | What |
|---|---|
| `reduce-it-fr.py` | Italian's reference reducer. Studied side (change 9, with 10, 11, 11a, 13, 14): `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, a form's one lemma chosen by `OVERRIDES`, MarkIT's counts, the form's own entry, the lemma's frequency, the alphabet; the apocopes by a reviewed table (`APOCOPES`). Native side (change 16): `section_as_read` (letters out, links read as glosses, name notes and acronyms kept off common words, the review's leaks) → `native_side` (the shared `reduce_gloss`, `reduce_expressions`, `fallback_glosses`) → `without_repeated_senses` → the split words → `acronym_glosses` → `french.dictionary_words` → `estimated_levels` |
| `reduce_common.py`, `reduce_edition_fr.py` | The shared rules (every pair's digest) and the French edition's (en-fr's, es-fr's and it-fr's): editing either re-pins en-fr and es-fr. Not edited here |
| `reduce_italian_treebank.py`, `reduce_french_treebank.py` | MarkIT's training and development sections (30,153 syntactic words, against GSD's 390,368): `markit_pos_counts`, written by change 9 for « the part of speech a gloss row opens on, which Italian's glosses (change 16) read », and read by no gloss rule yet. fr-en's treebank module holds the order rule (`commonest_first`, at least 10 and twice), which fr-es imports as written |
| `tables/it/`, `tables/it-fr/` | 60,000 ranked lemmas; 32,691 glossed rows (31,006 from the section, 279 from the Italian Wiktionary, 1,405 read backwards, `USA`); 18,296 expressions; coverage **89.2 / 81.8 / 73.3 %** against `FLOORS["it-fr"]` 87.2 / 79.8 / 71.3; 24,216 dictionary words; 8,302 levels (ranks 1–11,679); the pack 2,645,342 B |
| UD Italian-PUD at `71d932aa` | **99.26 / 91.31 / 99.90** (19,155 words, 8,909 content words, 985 auxiliaries) against Italian's gates 98.5 / 90.5 / 97; MarkIT's test section 99.43 / 94.65 / 98.87, reported |
| `it-fr.golden` | Over the pack the committed tables build, re-blessed once by change 16; its `nomi` page asserts six names (`italian_baseline.rs`). Change 18 (#1055) adds the word card's snapshot `word-card-it-fr.txt` over the same tables |
| The requirements this change touches | Open changes', none archived: change 16's *Italian is glossed in French…* and *it-fr is committed…*, change 9's *Italian's forms and frequencies* (re-copied by change 20, which archives after this change), change 11's *Italian's acronyms are a reviewed table…*, change 13's *Italian's word grammar* (read as written) |
| The lists for this change | Change 16's *Known data defects* (its design and README), change 13's readings list, the M8 rows change 9's review left (`dato`, `dati`, `lega`), change 11's *For change 17*, change 12's *Known gaps*; the owner's approval of change 16 (« `che`, `della`, `dirotto` « Écanguer », 458 unglossed lemmas… — are change 17's ») |

## Goals / Non-Goals

**Goals:**
- it-fr's glosses read as an Italian word's meanings, in the order Italian uses them, before change 20
  lists it-fr — by rules of it-fr's own reducer, each measured alone on the committed tables and kept
  only where it reads better (the criteria below).
- Italian's forms read the words change 9's review left to this change as their words.
- Every item of the lists above decided: a rule, a reviewed row, left with its reason, or put to the
  owner in plain words with examples (*For the owner*).

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en, en-es, fr-en or fr-es: no rule of `reduce_common.py`,
  `reduce_edition_fr.py` or `reduce_french_treebank.py` is edited, though some of these rules would
  suit es-fr (D13).
- Italian's analysis (lingua-core): a capital after a title's abbreviation, the press's « la Nato »,
  « dai retta » — recorded in the programme's backlog (D13).
- A new source or fetch: it-fr keeps its pin and its releases.
- The readings' own defects (`braccio`'s plurals, `bello`'s rows, `uova`'s head): the section's, to
  report upstream (change 13's list), but `fate` (D12) and the forms rules D11 shares with them.

## How this was measured

**Criteria, written on 2026-10-25 before any figure was read**: a class is counted in ranked lemmas by
band (1–1,000, 1,001–5,000, 5,001–20,000, past 20,000); a rule is kept when every row it changes
among the 5,000 commonest reads right or no worse and the rows it changes among 5,001–20,000 read
right in a majority, else narrowed or dropped, the narrowing recorded; a rule never removes a lemma
from the pack — scoped at review to the gloss rules, since a forms rule takes a lemma out by design and
is therefore listed line by line (D14) — and no rule edits a shared module; a reviewed row is written only for a clear error no rule
reaches, with its reason, and a matter of taste goes to the owner with examples; coverage stays at or
above the floor; a forms rule is judged on the dictionary's entries and MarkIT's training and
development counts, never on PUD, whose figures are read after the rule, for its direction only.
MarkIT's test section is never read.

**The data.** The committed tables, read as they are. For what the source says: local copies of
it-fr's pinned sources — the French Wiktionary's Italian section as kaikki extracts it, the two
translation files, the English section of `lingua-pack-sources-it-fr-2026.10.23`, MarkIT at
`e210079d` — which change 16's review reduced to the committed tables byte for byte, read as data
by scripts outside the repository. A prototype runs it-fr's real section rules (`section_as_read`'s
output, then `reduce_common.reduce_gloss` with `WORD_GLOSS` and `without_repeated_senses`) over an
extract of the section holding every ranked lemma, every form and every word a pointer names
(87,810 entries); unchanged, it gives the 31,006 rows the section glosses byte for byte. Each gloss
rule is that extract rewritten as the rule would, reduced, and compared row by row with **the
committed rows** — a lemma counted gained only when `gloss.tsv` does not gloss it, from any source.
The forms rules' figures are their candidates in the English section, counted with the reducer's own
helpers (`_dialect_rows`, `without_stress`), and re-counted by the real reduction at implementation
(task 3.3). Every figure below is re-measured there (task 5.2).

## Measured

**The classes, on the committed tables**, and what this design does:

| # | Class | Ranked rows, by band (≤1k / ≤5k / ≤20k / beyond) | Examples | Decision |
|---|---|---|---|---|
| 1 | The wrong part of speech first, by the entries' order | 1,580 rows hold two parts of speech or more (191 / 324 / 606 / 459) | `fra` « Frère; Entre », `spesso` « Gros, épais; Beaucoup… », `gente` « Gentil, fin; Gens… », `fino` « Fin; Jusque », `pure` « Pure; …; Aussi », `loro` opening on the courtesy `Loro` | D2: 14 rows |
| 2 | A common word read as a name, its sense repeated by its name | 12 rows (0 / 3 / 5 / 4) | `bibbia`, `rinascimento`, `corano` « Bible », « Renaissance », « Coran » as proper nouns alone, so no dictionary words and no levels | D3 |
| 3 | A name under a pronoun or an adverb | 5 rows (5 / 0 / 0 / 0) | `io` « …; Io, fille d'Inachus; Io, satellite… », `qui` « Ici; Riri », `qua` « Ici; Loulou », `oltre` « …; Preko, commune de Croatie », `anzi` « Au contraire; Anzi » | D4 |
| 4 | A pointer the shared rules cannot follow | 56 unglossed lemmas gain a gloss (0 / 7 / 13 / 36), 27 rows read the section instead of a table | `lettere` « Pluriel de lettera », `paro` « Variante de paio », `pochissimo` « Superlatif de poco : « très peu » », `ceneri` « Pluriel de cenere (« cendre ») » | D5 |
| 5 | A noun's pointer kept from lending by a name or an old sense | 3 rows (0 / 1 / 2 / 0) | `gemelli` « Gémeaux (constellation…) » alone, `spezie` « Espèce » (archaic) | D6. `rose` and `panni`, in change 16's list, already read `rosa`'s and `panno`'s glosses since its commune rule |
| 6 | A verb form glossed by its verb's infinitive | 809 participle rows (4 / 62 / 285 / 458), 294 finite-form rows (3 / 20 / 99 / 172) | `assicurato` « Assurer, certifier, garantir », `avvolto`, `usato`, `utilizzo` « Utiliser » | D7: as now, the card names the verb (the owner, 2026-10-25) |
| 7 | The `di` family's links; the articles' usage notes | 10 rows | `della` « De » (read backwards), `degli` « Article contracté souvent traduit par des. Note d'usage : … », `del` « Du. Note d'usage : il s'utilise devant… » | D8 |
| 8 | A commune noted otherwise | 31 common rows hold a proper noun's sense naming a commune; 14 change (2 / 4 / 5 / 3) | `vittoria` « …; Vittoria, commune et ville… », `ro` « Rhô; Ro (commune italienne… », `don`, `lana` | D9 (the owner, 2026-10-25) |
| 9 | An example and its translation in one sense | 1 row, 3 senses | `io` « io sono in Francia. — moi, je suis en France » | D10, a rule |
| 10 | A source's slip | 4 rows | `che` « Quoi; Quel », with no « Que »; `volere`'s example in three senses; `dirotto` « Écanguer »; `dati`, read as a noun of its own, borrowing the singular « Donnée » | D10, `CORRECTIONS` (the owner, 2026-10-25) |
| 11 | A split word that is no Italian | 3 of 75 | `c'hwerty`, `l'wha`, `d'n'b` | D10 |
| 12 | Unglossed lemmas that lost their level | 456 (13 / 144 / 299 / 0) | `entro`, `supporto`, `verifica`, `web`, `ok` | 12 glossed (D5), `signor` and `od` read as their words (D11); 442 left (D13) |
| 13 | Forms read as another word, or as no word | `sto` (136), `vado` (451), `sapete` lemmas of their own; `viti` → *vita*, `crochi` → *croce*, `signorini` → *signorina*, `avi`, `ava` → *avere*, `usa` → *uso*; `dato`, `dati` → *dare*, `lega` → *legare*, `prova` → *provare* | D11 (three rows, the owner, 2026-10-25) |
| 14 | A reading the tables give a form wrongly | 1 | `fate` read as *fare*'s present subjunctive (the form is *facciate*) | D12 |
| 15 | Names read as words | — | `Rossi` *rosso*, `Bianchi` *bianco*, `Conti` *conto*, `Bari` *barare*; `Esposito`, `Ferrari` after « Dott. », « Prof. » | Left (D13) |
| 16 | The `di` retry's leftovers | 7 partitives, 30 literal phrases of 3,000 retries; `pur`/`pure`, `prova`/`provare`, `meriti`/`merito`; « dai retta » | — | Left, or the owner's `prova` row (D13) |
| 17 | Acronyms written otherwise | « la Nato », « gli Usa » (5 and 1 in change 11's corpora); « tra USA e NATO » | — | Left: the analysis's (D13) |

**Each gloss rule alone**, against the committed rows:

| Rule | Rows changed (bands) | First sense | Gained | Examples |
|---|---|---|---|---|
| D2, MarkIT for function words (fr-en's rule, ≥ 10 and ≥ 2×) and `FIRST_RUN` | 14 (14 / 0 / 0 / 0) | 13 | — | `fra` « Entre, durant; Frère », `spesso`, `proprio` « Vraiment; … », `attraverso`, `molto`, `solo`, `dopo`, `ne`, `uno`, `più` (an adverb's run); `gente`, `fino`, `pure`, `loro` by the table; `come` held |
| D3, a name's sense its common entry repeats | 12 (0 / 3 / 5 / 4) | 2 | — | `bibbia` « Bible », `rinascimento`, `corano` read as nouns; `viola`, `fedele`, `candido` lose a name's run |
| D4, no name under a pronoun or an adverb | 5 (5 / 0 / 0 / 0) | 0 | — | `io`, `qui`, `qua`, `oltre`, `anzi` |
| D5, pointers named by their text, an `alt_of` not marked obsolete, archaic or dated, a superlative's or a plural's quoted meaning | 27 (0 / 1 / 7 / 19) | 26 | 56 (0 / 7 / 13 / 36) | `lettere`, `terzi` « Tiers », `nodi` « Nœud », `cosí`, `pochissimo` « Très peu », `altissimo`, `ceneri` « Cendre », `baffi`, `paro` « Paire »; `can` « Khan » → « Khan, prince mongol » |
| D6, a noun's or an adjective's pointer beside a name or an old sense | 3 (0 / 1 / 2 / 0) | 3 | — | `gemelli` « Jumeau; Gémeaux…; Bouton de manchette », `spezie` « Épice…; Espèce », `arme` |
| D8, the `di` family and the articles' notes | 10 (6 / 1 / 2 / 1) | 8 | 1 (past 20,000) | `della` « De la », `degli` and `delle` « Des », `cogli` « Avec les », `pegli`; `del` « Du », `der`, `pel`; `dei` and `dello` gain their contraction; `pella` « Pour la, par la » |
| D9, a commune's note (provincial capitals, chefs-lieux, capitals and the owner's two cities spared) | 14 (2 / 4 / 5 / 3) | 0 | 2 lost | `vittoria` « Victoire », `ro` « Rhô », `oltre`, `don` « Dom; Don (fleuve russe) », `lana` « Laine »; `trani`, `melano` unglossed |
| D10, the Italian example | 1 (1 / 0 / 0 / 0) | 0 | — | `io` loses « io sono in Francia. — moi, je suis en France » and two more |

**Together**: **88 rows change** (26 / 10 / 21 / 31), 56 of them in their first sense, and **57 lemmas
gain a gloss** (0 / 7 / 13 / 37); `trani` and `melano` lose theirs, which no table holds; the four
corrections (D10) and three split words come on top. Coverage **89.3 / 82.0 / 73.4 %** (+7 / +14 / +19
lemmas) against the floor 87.2 / 79.8 / 71.3, before D11's forms move the ranks (re-measured, task
5.2). Of the 456 levelled lemmas change 16 left unglossed, 12 are glossed again (`lettere`, `terzi`,
`scomparso`, `pochissimo`, `obbiettivo`, `nodi`, `altissimo`, `ceneri`, `fortissimo`, `baffi`,
`parallele`, `coppe`) and `signor` and `od` read as their words (D11). With D3's and D6's new
dictionary words (`bibbia`, `rinascimento`, `corano`, `gemelli`) about twenty lemmas join the levels
and as many leave at C2's end, English's sizes kept. **PUD**: the gloss rules move no lemma; D14
states what the forms rules move.

## Decisions

### D1 — Every rule is it-fr's, in its reducer; the treebank's order is fr-en's, imported

The gloss rules extend `section_as_read`, it-fr's pre-pass, and its post-passes after `native_side`;
the forms rules extend change 9's studied side. MarkIT's order calls fr-en's
`reduce_french_treebank.commonest_first` over `reduce_italian_treebank.markit_pos_counts`, which
counts as GSD's counts do — as fr-es imports it, no copy: `reduce_french_treebank.py` joins it-fr's
rule digest, and editing it would re-pin fr-en, fr-es and it-fr together. `reduce_common.py`,
`reduce_edition_fr.py` and the treebank module are not edited, so en-fr, es-fr, fr-en and fr-es keep
their digests, tables and packs; no other pair studies Italian yet, so only it-fr re-pins.

### D2 — A function word's row opens on the part of speech MarkIT reads it as; a reviewed table for the rest

The shared round-robin takes each entry's first sense in the page's order: `fra` opens on « Frère »
(a friar, `frate` truncated) before « Entre », `spesso` on « Gros, épais » before « souvent ».
fr-en's rule reads the treebank: a function word's row, or one the page opens on a name, opens on the
part of speech the treebank reads the word as at least ten times and at least twice as often as the
page's first, never a proper noun; a noun, a verb or an adjective stays as the page writes it.

Over MarkIT it moves **11 rows, all of the 1,000 commonest**: `fra` (13 prepositions against none),
`spesso` (27 adverbs against 1), `proprio` (132 against 8), `attraverso`, `molto`, `solo`, `dopo`
(« Après, derrière » before « Après, ensuite », no worse), `ne`, `uno`, `più` (whose « Plus » becomes an
adverb's run) — and `come`, which would open on « Dès que; Que dans une comparaison » before « Comme »
(39 conjunctions against 16 adverbs): worse. **fr-en's twice is kept, and `come` is a row of the
reviewed table.** A ratio of three would have spared `come` and `dopo`, but it was chosen after
reading them — fitted to two words —, while fr-en's constant was measured on another treebank and a
row naming `come` says what it does and can be read.

**The reviewed table, `FIRST_RUN`**, names, for a row the rule does not move or moves wrongly, the
headword's case and the part of speech whose entries come first — each row with its reason, the
settled by the owner on 2026-10-25 as proposed: `come` (the adverb, held against the treebank), `gente` (the noun « Gens »:
MarkIT reads it 12 times as a noun and never as an adjective, but the rule moves no content word, the
boundary fr-en's owner set), `fino` (the preposition « Jusque »: MarkIT reads it 8 times, as an
adverb, a part of speech the row has not), `pure` (the adverb « Aussi »: 3 readings), `loro` (the
lowercase pronoun « Ils, elles, eux »: both entries are pronouns, the courtesy `Loro` first, so a part
of speech alone cannot order them). The four rows move 4 first senses; a row moves only entries, never
adds or removes a sense. **Content words** keep the page's order otherwise, as French's do (with no
boundary MarkIT would also move `parlare`, `potere`, `amico`, `diritto`, better, and `piacere`,
worse, « Plaisir; Plaire » for the lemma of « mi piace »).

### D3 — A name's sense its common entry repeats is the common entry's

`Bibbia` (a proper noun, « Bible ») comes before `bibbia` (a noun, « Bible ») in the section, and the
shared join keeps a repeated sense once, as the first entry's: the row is a proper noun's alone, so
`bibbia` is no dictionary word and loses its level (change 16's list: `bibbia`, `rinascimento`,
`corano`). The pre-pass drops a name's sense that a lowercase entry of a common part of speech gives
word for word (up to case and its closing period), so the common entry's run carries it. **12 rows**:
`viola`, `fedele`, `bibbia` (1,001–5,000), `rinascimento`, `corano`, `candido`, `borgogna`,
`arlecchino`, `fabiano`, `anfitrione`, `settebello`, `matusalemme`; only `arlecchino` and
`settebello` change their first sense. `bibbia`, `rinascimento` and `corano` become dictionary words
and take a level again.

*Rejected — a capitalised headword's entries after the lowercase one's.* It fixes `loro`, but of its
79 rows `cristo` would open on « Hère, bougre », `corea` on « Chorée », `scozia` on « Scotie »;
narrowed to capitalised entries of a common part of speech, 14 rows, `svizzera` opening on
« Habitante… de la Suisse » and six acronyms' nouns (`pc` « PC, ordi personnel ») after a province.

### D4 — A pronoun's or an adverb's row holds no name

A proper noun's entry of a capitalised headword, no acronym, is left out when every lowercase entry
of the word is a preposition's, a conjunction's, a pronoun's, a determiner's, an article's, a
particle's or an adverb's. fr-en's set of parts of speech (its rule for `le` and `on`) moves one row,
`io`; **adverbs were added after reading that figure**, on `qui` « Riri », `qua` « Loulou » (Donald's
nephews), `oltre` « Preko, commune de Croatie » and `anzi` — a widening, decided on the figures,
recorded as such. **5 rows, all of the 1,000 commonest, all better.** A word with a common sense
beside a name keeps the name's (D9 for communes).

### D5 — A pointer that names its word, or carries its meaning, is read

Three wordings the shared rules cannot follow, each read by it-fr's pre-pass, which writes the
pointer the shared rules read (a `form_of` naming the word) or the meaning:
- **an untagged pointer** whose text the French edition reads as an inflection and that names no word
  in kaikki's fields (« Pluriel de lettera. »): the word after its last « de », « du » or « d' », when
  it is the whole remainder;
- **an `alt_of`** (« Variante de paio. »), the word it names — **unless the sense is marked obsolete,
  archaic or dated**, as D6 reads an old sense: `maio` (« Di Maio » would read as the word), `fora`
  and `podesta` are not followed;
- **a superlative's or a plural's pointer carrying its meaning** in « » (« Superlatif de poco :
  « très peu ». », « Pluriel de cenere (« cendre »). »): the quoted text, capitalised, read as the
  sense — kaikki names the quoted French word as the target (`form_of` « peu »), which lends
  nothing. Only these two wordings: a clitic compound's « (« le ») » names a pronoun (`vallo`).

The shared rules then lend as they lend to any pointer — the base's senses in the same part of
speech. `signor` and `od`, in the earlier count, are read as their words by the apocope table instead
(D11). Measured against the committed rows: **56 lemmas gain a gloss** (7 of 1,001–5,000: `lettere`,
`terzi`, `scomparso`, `pochissimo`, `cosí` « Ainsi; Comme ceci; Tellement », `obbiettivo`, `nodi`;
13 of 5,001–20,000: `altissimo`, `napolitano`, `ceneri`, `nicolò`, `fortissimo`, `baffi`, `parallele`,
`coppe`, `alaska`, `bahamas`, `paro`, `vicinissimo`, `satirica`), and **27 rows** glossed by a table
today read the section instead — `can` « Khan » → « Khan, prince mongol » (1,001–5,000); among
5,001–20,000, `gravissimo` « Très grave, gravissime », `santissimo`, `gentilissimo`, `molino`,
`piscia` « Pisse » better, `obbedire` no worse, `consolle` « Consul; Console » worse. Left as the
section writes them: `cripto` and `crypto`, an untagged « Variante de kripton (l'élément chimique Kr) »
(past 33,000); `stupidissimo` « Très léger », the section's slip (past 50,000).

### D6 — A noun's or an adjective's pointer lends beside a name or an outdated sense

The shared rules lend only to a word with no glossed sense, so a constellation or an archaic sense
blocks the plural's: `gemelli` reads « Gémeaux (constellation et signe zodiacal) » alone (a name, so no
dictionary word), `spezie` the archaic « Espèce » and never « Épice ». The pre-pass lends a lowercase
entry's pointers their base's senses — the base's own lowercase entries of that part of speech, none
archaic, obsolete or dated — when the word's every other sense is a capitalised proper noun's or tagged
archaic, obsolete or dated, the lent entries first. **3 rows**: `gemelli` « Jumeau; Gémeaux…; Bouton
de manchette », `spezie` « Épice…; Espèce », `arme`. *Narrowed*: with a verb's pointers too, 12 rows,
5 worse (`nitro` « Nitrer », `puffo` « Schtroumpfer », `regolo` « Régler », `veglio` « Veiller »,
`musico` « Musiquer »): a verb form's pointer lends nothing beside a name.

### D7 — A verb form's borrowed senses: as now, the card names the verb

A lemma whose section senses are only a verb form's pointer borrows the verb's senses: `assicurato`
reads « Assurer, certifier, garantir », `utilizzo` (the noun « use ») « Utiliser » — 809 participles and
294 finite forms (280 once D11 reads 14 of them as their verb's forms: `sto`, `vado`, `sapete`…). Change 18's card
(#1055) already names the verb for such a lemma, from change 13's `other` marks:

```
assicurato
peut aussi être le participe passé de assicurare
[verbe] Assurer, certifier, garantir
```

Labelling the borrowed sense with the section's own pointer — « (participe passé de assicurare)
Assurer, certifier, garantir », 32 characters more; « (première personne du singulier du présent de
utilizzare) Utiliser », 58 — says it twice on the card and lengthens 1,089 rows; it would help only the
page's one-line gloss, which has no grammar line. The translation tables' word first (218
participles, 102 finite forms) gives `dovuto` « Dû », `impegnato` « Engagé », `utilizzo` « Utilisation »,
but `girato` « Tourne », `superato` « Révolu, éculé », `coinvolto` « Empêtré ». **This design leaves
the rows as they are**, the card explaining the form. **Settled by the owner on 2026-10-25**: (c), as
they are; the label, (a), and the tables first, (b), are rejected, and no rule or requirement of this
change labels a verb form's borrowed senses.

### D8 — The `di` family's links read as glosses; an article's usage note goes

Change 16 reads a link as a gloss only where no Italian entry heads its target, so `della`'s
« Article contracté souvent traduit par De la. » (a link to the Italian `la`) stays a link and
`della` reads « De », read backwards. The `di` family are lemmas of their own by Italian's
tokenisation (`DI_FAMILY`): their links read as glosses whatever their target; and a sense « Article
contracté souvent traduit par X » reads « X » where the word has no other glossed sense — `della`
« De la », `degli` and `delle` « Des », `cogli` « Avec les », `pegli`, `pella`. `dei` gains its
contraction, `dello` « Du ». *Narrowed*: without the condition the wording would add « Pour les »,
« Avec les », « Avec la » to `pelle`, `collo`, `colle` and `colla`, the rows change 16 kept clean.

20 rows hold a « Note d'usage ». Six are an article's or an articulated preposition's, about the next
word's first letters (`del` « Du. Note d'usage : il s'utilise devant un mot commençant par une
consonne… »), which Italian's tokenisation and the card's readings already carry: **the note goes from
a determiner's or a preposition's sense** (`del`, `degli`, `der`, `cogli`, `pel`, `pegli`). The 14
others teach usage a reader needs (`sebbene` « Quoique. Note d'usage : suivi d'un verbe au
subjonctif », `granché`, `mai`, `quesito`, `retaggio`) and stay — decided by the rule as written.
**10 rows** (6 of the 1,000), 8 first senses, `pella` gained.

### D9 — A sense naming its place a commune is a commune's note

Change 16 reads « commune d'/de » after the headword. 31 common rows hold a proper noun's sense naming
a commune otherwise. The pre-pass reads as a name's note, set aside from a word with a lowercase
entry as change 16's notes are, a proper noun's sense that **designates its place as a commune right
after its name** — « Vittoria, commune et ville… », « Ro (commune italienne… », « Lana, commune du Pays
basque… » —, the word « commune » alone deciding, and spares:
- a sense that also names its place a **chef-lieu or a capital** (`bordeaux` « commune, ville et
  chef-lieu », `valenza`'s Valence and Valencia), and one naming it something else first (`colonia`
  « Cologne, ville, commune… »);
- a headword one of whose proper nouns' senses is, or ends on, « province autour de cette ville »,
  whatever its case — a provincial capital: `brindisi`, `foggia`, `cuneo`, `potenza`, `matera` (« Province
  autour de cette ville »), `prato`;
- a commune a reviewed list keeps: `piacenza` (Plaisance) and `cesena`, the owner's (2026-10-25).

**14 rows** (`oltre`, `vittoria`; `don`, `stella`, `quinto`, `lana`; `ro`, `belvedere`, `rho`,
`pigna`, `bellino`; `brusio`, `pirano`, `lumino`), none in its first sense. Without the reviewed list
`piacenza` (3,630) and `cesena` (5,360), provincial capitals whose province the section does not give,
would lose their city (`cesena` reading « Grive litorne »): the owner keeps both (2026-10-25, option
(a)). The remaining cost: `trani` (8,813) and `melano`, names alone, lose their only gloss, which no
table holds. *Rejected*: the rule without the list, (b); also sparing « commune et ville », (c), 13
rows, `vittoria`, `cesena`, `rho` and `trani` keeping their town. *Measured and left*: every other place note (a town, a province,
a river, an island): 79 rows, `porto`, `sale`, `nave`, `canna` cleaned, but `brindisi` reading « Toast »,
`foggia` « Mode, style », `cuneo` « Coin » with no city. 294 rows mix a name with common senses and 65
open on it; fr-en's owner accepted those until a card reads the token's capital.

### D10 — The Italian example, corrections by name, and split words that are no Italian

**An example and its translation in one sense** — Italian, a dash, French (« io sono in Francia. —
moi, je suis en France ») — goes: a sense whose text before « — » ends a sentence (« . », « ? »,
« ! »), holds a form of the headword, is written in Italian forms of the tables (at least 60 % of its
words) and holds none of a closed list of French-only words (« pour », « avec », « est », « les »,
« je »…). Over every row and expression, the senses holding « — » are 33; the rule takes **three, all
`io`'s** (« io sono in Francia. — moi, je suis en France », « Pago io o paghi tu ? — je paie ou tu
paies ? », « io pago. — c'est moi qui paie »); `mai`'s « Jamais, en nul temps. — Note d'usage… » and
`toccata`'s « Toccata, pièce musicale… — orgue, clavecin… » are not taken (no form of the word, French
words). The rule is clean, so `io` needs no correction.

**`CORRECTIONS`**, fr-es's reviewed list for it-fr: each entry the source, the headword, its part of
speech, the source's text as the pinned file writes it, the corrected text or none, the reason and
the page to report upstream; an update finding a correction firing on nothing names it. A correction
that adds an entry is read before the section's own entries of the word, so D2's order and the
round-robin keep it in front unless the treebank or the table says otherwise. Four rows:
- `che` (rank 3): the section gives only « Quoi » (the interrogative) and « Quel »: no conjunction, no
  relative, though MarkIT reads `che` 406 times as a conjunction and 700 as a pronoun, and the French
  tables list `che` under « que ». Proposed: the relative pronoun « Qui, que » and the conjunction
  « Que », read first; MarkIT's commonest part of speech is the pronoun, so the row reads « Qui, que;
  Quoi; Que; Quel » — settled by the owner on 2026-10-25, as proposed.
- `volere` (2,041): three senses are an example and its translations with no marker and no dash
  (« Il faut dix heures pour aller à Brindisi », « Ci vuole un uovo per la torta », « Il faut un œuf
  pour le gâteau »): out — a mechanical correction.
- `dirotto` (19,945): « Écanguer » is `dirompere`'s, lent through the participle; the section has no
  adjective. « À verse (dans a dirotto) », settled by the owner on 2026-10-25. Both are reported to
  the French Wiktionary.
- `dati`, the plural noun read as a lemma of its own (D11, the owner, 2026-10-25): the section's only
  noun sense is « Pluriel de dato. », which lends `dato`'s singular « Donnée ». The lemma stands for
  « i dati », the data, which French says in the plural, « les données »; the singular « donnée » is
  one given fact or datum — the card would read « a datum » for « the data ». No source writes the
  plural: the French tables list `dato`, never `dati`, under « donnée ». The correction, written by
  the owner's decision, glosses `dati` « Données » (a noun's run), as the section glosses its other
  plural nouns of their own in the plural (`forbici` « Ciseaux », `occhiali` « Lunettes », `nozze`
  « Noces, mariage »), and is reported upstream with the others.

**A split word that is no Italian** (change 16 offers the words the pre-pass splits): one whose
letters after the elided piece hold `j`, `k`, `w`, `x` or `y`, or a second apostrophe, is not offered:
`c'hwerty`, `l'wha`, `d'n'b` — 3 of 75; `d'antan` (French, harmless) stays.

### D11 — Italian's forms: an own entry by its meaning, no chain through a plural, the reviewed rows

Change 9's studied side, re-reduced with four rules and five reviewed rows, judged on the dictionary's
entries and MarkIT's training and development counts.

**1. A form's own entry counts only by a meaning, for a verb's finite form.** `sto` is a lemma of
its own through the phrase « ellipsis of sto bene », `vado` through « alternative form of guado »
(obsolete, poetic), `sapete` through « (you) know » (kaikki's untagged « of sapere »): MarkIT meets
none of them, so the own entry wins. The rule: when a verb's table gives a form as a person of a mood,
the form's own entry counts, among the rules choosing its lemma, only by a meaning — that verb's
inflection written without its tag, an ellipsis, or a pointer of another part of speech the section
marks obsolete, archaic, literary, poetic, dated or colloquial is none. **The census** — the 92 ranked
lemmas the English section gives no meaning of their own beside a verb's inflection:

| Kind | Rows (bands) | Examples | The rule |
|---|---|---|---|
| The verb's inflection alone | 19 (0 / 3 / 6 / 10) | `sapete`, `speriamo`, `segui`; `lavoriamo`, `finisci`, `capiva`, `capivo`, `tari`, `saliamo` | applies (but `tari`) |
| Beside a pointer marked doubtful | 17 (2 / 3 / 9 / 3) | `sto`, `vado`; `torno`, `guardo`, `incontra`; `desidero`, `nego`, `rota`, `scola`, `loda`, `consolo`, `sanguina`, `ei`, `contra` | applies (but `ei`, `contra`, `isso`) |
| Beside an alternative form not marked | 20 (0 / 2 / 3 / 15) | `ascolta`, `compra`, `annunzio`, `molino`, `situa` | does not apply |
| No finite inflection | 27 (0 / 2 / 11 / 14) | `cagare`, `obbedire` (alternative infinitives), `elisa`, `annunziata`, `sovrintendente`, `dormiente` (participles), `sn`, `asp` (abbreviations) | does not apply |
| Another word's inflection beside | 6 (0 / 0 / 4 / 2) | `dee` (the plural of `dea`), `este`, `mei`, `paro` | does not apply |
| A contraction beside | 3 (0 / 0 / 2 / 1) | `cogli` (D8's « Avec les »), `trai`, `sommi` | does not apply |

Narrowed so: never a participle or an alternative infinitive, a form another word's entry names as
its inflection, a contraction, a function word's homograph (`contra`, `ei`, `isso`), nor a form the
section also heads as an acronym (`tari`, the tax). **32 candidates**: among the 5,000 commonest, `sto`
→ *stare*, `vado` → *andare*, `torno` → *tornare* (today « Torno, commune… »), `guardo` → *guardare*
(today unglossed), `incontra`, `sapete`, `speriamo`, `segui`: 8 rows, all better; among 5,001–20,000,
12 rows — `desidero`, `nego`, `loda` and `consolo` (today « Nom de famille »), `sanguina`, `lavoriamo`,
`finisci`, `capiva`, `capivo` better, `rota`, `scola`, `saliamo` no worse; 12 past 20,000. The
regressions the wider rule made are out: `dee` « Déesse » → *dovere*, `elisa` → *elidere*,
`sovrintendente` → *sovrintendere*, `annunzio`, `este` → *essere*, `contra`, `cogli` → *cogliere*. Each
leaving lemma gives its rank to the next (`freq.tsv` renumbered).

**2. No form of a form through a plural.** A form reaches its candidate's word along one part of
speech, unless the candidate is itself a noun's plural: **3 forms** — `viti` (the plural of `vite`,
itself `vita`'s plural) and `arpi` (through `arpe`, `arpa`'s plural) leave the tables, `vite` and `arpe`
being no lemmas; `crochi` (kaikki's « croco crocuses, croci ») reads as *croco* through `croco`'s own
table. A chain through a feminine stands (`maestre` → `maestra` → *maestro*), and so does a form whose
own entry names its word too (`armi`, `fuse`, `pomidori`).

**3. A feminine noun's masculine row gives no form** the form's own entry does not name, as it gives
no reading: **5 forms** — `signorini` reads as *signorino*; `gattaro`, `lavandaio`, `pecoro` and
`puttano`, no ranked lemmas, leave the tables. `echi` (*eco*'s masculine plural, named by its own
entry) and `cenerentolo` stay.

**4. The tables the readings set aside give no form another word claims.** The Classical Roman
Dialect's tables and `essere`'s untagged usage notes (change 13's exclusion) give their verb **95 forms**
nothing else gives it. Setting them all aside would lose Italian's truncations and literary forms only
those tables list (`son`, `siam`, `eran`, `avea`, `dovea`): rejected. Narrowed: their row does not
outrank the form's own entry naming another ranked word — **11 forms** read as it (`avi`, `ava` → *avo*,
`abbino` → *abbinare*, `pozzi` → *pozzo*, `favi` → *favo*, `facete` → *faceto*, `parei` → *pareo*,
`parino` → *parare*, `fuor` → *fuori*, `sien` → *sieno*, `fian` → *fiano*, the grape, a cost) — and a row
whose form is one of the table's own tags gives none (« archaic », « imperative », « infinitive »,
« subjunctive »: 4 forms leave). `ate` stays *avere*'s, its own entry naming it.

**The reviewed rows.** `APOCOPES` gains `signor` → *signore* (an apocope: « Variante de signore »,
`Sig.`) and `od` → *o*, the euphonic « od » before a vowel, which the section glosses only as « Variante
de o », too short a word to lend (`ed` and `ad`, glossed as words by the section, « et », « À, dans sa
forme euphonique », stay lemmas); both leave the ranks. `OVERRIDES` gains `usa` → *usare* (the third
person « usa », against `uso`'s literary feminine; MarkIT's one count lemmatises it `usa`): a clear
error. **The owner's rows**, settled on 2026-10-25: `lega` → *lega* (the league, `leghe` coming back
as its plural; MarkIT's one count is the verb's), `prova` → *prova* (the noun; `a prova di` then keys
on it; MarkIT 3 verbs, 1 noun), and « i dati » read as the noun (MarkIT reads `dati` 3 times as
*dato*). Each reads every verb form it shares as the noun. **`dato` stays *dare*'s**: « mi ha dato »
is the very common participle of *dare* (MarkIT 9 against 3), and reading it everywhere as the noun
« donnée » would be wrong in conversation — the owner's reason.

**One consequence, settled by the owner on 2026-10-25 (task 0.6).** A row cannot read `dati` as *dato* while `dato`
reads as *dare*: a lemma is keyed by its own form (M8, *Every rank on its own lemma*), so *dato* is no
lemma of the tables unless `dato` reads as it, and an override row may only choose among ranked
lemmas. The nearest row that can be written is `dati` → *dati*, the plural noun as a lemma of its own
— the English section heads it so (« data (plural of dato) »), and the French section's « Pluriel de
dato » lends it `dato`'s noun, « Donnée ». « i dati » then reads as the noun and « ha dato » stays
*dare*; the participle `dati` (« sono stati dati ») reads as the noun. **The owner's row**: `dati` →
*dati*, glossed « Données », in the plural, by a correction (D10); PUD reads one word lower for it
(D14), accepted: the reader's card shows « Données » instead of « Donner ».

### D12 — `fate` reads no subjunctive

`grammar.tsv` reads `fate` as *fare*'s present indicative, imperative and present subjunctive, second
person plural; the subjunctive is *facciate*. *fare*'s second and third tables in the English section
list `fàte` beside `facciàte` in the subjunctive's cell, untagged, so change 18's card says
« indicatif ou subjonctif ». A rule « a later table's other form of a cell reads nothing » would take
383 cells of 33 verbs, among them the dialect's tables already set aside and literary forms a reader
meets (`era` for « io era », `sii`): rejected. A reviewed list sets the reading aside instead — one
row, `fate` *fare* `VERB|Mood=Sub|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin`, with its reason —, the
reduction failing on a row whose reading it does not find. PUD has no `fate`.

### D13 — Measured and left

- **The 442 unglossed levelled lemmas the rules cannot reach.** 423 are the sources' gaps: 410 have
  no entry in the French section, 4 more only a proper noun's in a table (`iphone`, `apollo`), 9 the
  section's placeholder « Définition manquante » (`intuire`, `rintracciare`); by the English section,
  399 of the 423 are Italian words (`entro`, `supporto`, `verifica`, `avvenuto`, `incluso`, `pressi`, and
  loans: `web`, `post`, `club`, `leader`) and 24 have no lowercase entry (`ok`, `iphone`, `usb`). 19
  more hold a pointer whose word lends nothing (`convocato`, `renato`, `preavviso`). They stay unglossed
  and unlevelled (no gloss written by a machine); the README and the pull request list the 25 commonest
  Italian words among them, to contribute upstream.
- **Names spelled as a common word's form** (`Rossi` *rosso*, `Bianchi` *bianco*, `Conti` *conto*,
  `Bari` *barare*): the lemma is a dictionary word, so the names rule keeps the word, as French's
  `Lot`, `Orange`, `Nice`. **`Esposito`, `Ferrari`** are no dictionary words, but on the `nomi` page
  they follow « Dott. » and « Prof. », whose period the names rule reads as a sentence's end.
- **The press's acronyms** « la Nato », « gli Usa » and « tra USA e NATO », and **« dai retta »**,
  « da' retta », which change 11a's context rule reads as `da` + `i`: Italian's analysis, not the
  tables. These four go to the programme's backlog (`docs/lingua/italian-programme.md`, *Backlog*).
- **The `di` retry's partitives** (`fuori di`, `da parte di`, `a prova di`: 7 of 3,000 retries): a
  partitive is no table's to tell. **Lemma clashes**: `pur` → *pure* is right (an apocope); `meriti` →
  *merito* stays; `prova` → *prova* is the owner's row (D11).
- **The 30 literal phrases** (`in via di` 8 of 8 retries, `alle spalle di` 4 of 4, `alla luce di` 5
  of 13): each is a real expression, right when written with `di` (« in via di estinzione »); the retry
  over the `di` family reads the literal phrase (« in via del Corso »). They stay offered, as they are.
- **Species in common rows**: 18 rows of the 20,000 hold a sense naming a species by its Latin name
  beside another sense (a pattern that also matches « (Au singulier) »); some are a rare sense of a
  common word — `papa` « Sarcoramphe, vautour pape », `caro` « Carvi », `nonna` « Nom populaire du héron
  cendré », `limone` (fishes), `adorno` « Bondrée apivore », `galleria` « Fausse teigne de la cire » —,
  others the plant or the animal the word names (`tasso` « Blaireau », `orzo`, `zafferano`). A rule
  cannot tell them apart; left as the section writes them.
- **Names opening a common row** (65) and **other place notes** (D9), **`marco` « Marc »** (a first
  name, no lowercase entry), **the readings' defects** (change 13's list): left.
- **es-fr and en-fr** keep the leaks it-fr reads out, and would gain from D4, D5 and D8's wordings:
  a refinement of the French edition re-pins both, a change of its own.

### D14 — it-fr re-reduced at its pin; every moved line listed

One owner dispatch (`lingua-pack-update`, mode `reduce`, it-fr) on this change's branch, from the
pinned releases (`…-2026.10.23` for the English section, `…-2026.10.24` for the French and Italian
files), Morph-it! and MarkIT, nothing fetched beyond them. The pin keeps its snapshot and sources byte
for byte; its rule digest moves (`reduce-it-fr.py`, and `reduce_french_treebank.py` joining it), with
`pack_version`, the pack, base and layer. `gloss_coverage.py --pair it-fr` passes the floor.

**What may move, and nothing else**: `tables/it/forms.tsv` by D11's rules and rows, every moved line
listed by its rule; `freq.tsv` by renumbering alone (the 34 lemmas D11 takes out — its rule 1's 32,
`signor`, `od` —, and `lega`, `prova` and `dati` joining by the owner's rows, the ranks closing up); `grammar.tsv` in its own and `other`
marks where a form changes lemma, and `fate`'s reading (D12); `lexical.tsv` and `level.tsv` (D3, D5,
D6, the ranks); `tables/it-fr/gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json`, `pin.json`,
`README.md`; `SOURCES.md`; `it-fr.golden` and change 18's `word-card-it-fr.txt`. **PUD**, read after
the rules: 99.26 % of its words and 99.90 % of its auxiliaries, as now; **91.32 %** of its content
words with the four rows: `signor` +1 (read as *signore*), `lega` +1, `prova` +2 and `dati` −1 (read
as *dati*: PUD lemmatises its ten nouns « dato », which stay misses, and its one participle *dare*,
which `dati` → *dati* loses) — **91.35 %** (8,138 of 8,909, against 8,135 today). The `dati` share of
the first count's +5 was +10 −1 with the `dato` row, and only with it. No other form named here is a PUD token: `dee`,
`este` and `mei` occur, as proper nouns outside the content gate, and the rule leaves them.

**What does not**: the six other pairs' tables, pins, packs, goldens and snapshots;
`reduce_common.py`, `reduce_edition_fr.py`, `reduce_french_treebank.py`; the extension's source;
Italian's analyser version.

### D15 — The golden and the card, read probe by probe

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline` once, after the dispatch; the owner
approves the re-bless on the listed probes (task 4.1). Expected to move: the `gloss` and
`phrase-gloss` probes of `che`, `del`, `della`, `degli`, `io`, `qui`, `gente`, `fra` and every page
token they gloss; `word-grammar` where a run's part of speech moves (`più`); the vocabulary
estimate's universe and the ladder's bands. The `nomi` page's six names stay as asserted (D13).
Change 18's `word-card-it-fr.txt` moves with the same rows and is re-blessed once, every moved line
listed; the six other baselines and `cross_native` pass unblessed.

### D16 — Order

| Change | Relation |
|---|---|
| 16 `add-lingua-pack-it-fr` | before: its two requirements are MODIFIED here, so this change archives after it |
| 9, 11, 13, 14 | before: change 9's and 11's requirements MODIFIED here; change 13's readings and 14's levels read as written |
| The cycle 11 → 16 → 14 → 11 | found by change 20's implementation: change 11 listed change 16 in its `archiveAfter` for no mechanical need (all three only ADD the requirements involved; 16's text names 11's and 14's, never the reverse), so none of the three, nor 17 and 20, could archive. This pull request drops that edge and records why in change 11's design (*The archive order*); the order is then 9 < 11 < 14 < 16 < 13 < 17 < 20, 20 after 17 once its `archiveAfter` names this change (task 5.5) |
| 18 `add-lingua-italian-word-card` | either side: whichever lands second re-blesses `word-card-it-fr.txt` on these tables |
| 20 `enable-lingua-italian` | after: it-fr is listed only once this change has merged (its D2); 20 MODIFIES *Italian's forms and frequencies* too, so its block is re-copied from this change's and its `archiveAfter` gains this change (task 5.5) |
| 23, 28 | the it-en and it-es refinements read D11's forms as committed (`tables/it/`), and may reuse D2–D10 as rules of their own reducers |

## Settled by the owner (2026-10-25)

The five questions of tasks group 0, answered by Guillaume on 2026-10-25:
- **Q1, the first meaning of a few common words** (D2): the five `FIRST_RUN` rows as proposed —
  `gente`, `fino`, `pure`, `loro`, `come`.
- **Q2, verb forms read as words of their own** (D7): (c), left as they are; the label is rejected.
- **Q3, communes and cities** (D9): (a), the commune rule, `piacenza` and `cesena` kept by a reviewed
  list.
- **Q4, two corrections** (D10): both as proposed, `che` « Qui, que; Quoi; Que; Quel » and `dirotto`
  « À verse (dans a dirotto) », each reported upstream.
- **Q5, nouns read as verbs** (D11): three rows — `dati` read as the noun, `lega` → *lega* with
  `leghe` back, `prova` → *prova* —, not `dato`, whose participle « mi ha dato » the noun would
  misread in conversation.
- **Task 0.6, `dati` without `dato`** (D11): `dati` → *dati*, the plural noun as a lemma of its own,
  glossed « Données » (a correction, D10); « ha dato » stays *dare*. PUD reads one word lower for it,
  accepted: the card shows « Données » instead of « Donner ».

**PUD, expected with the four rows** (`signor`, `lega`, `prova`, `dati`): 99.26 % of its words and
99.90 % of its auxiliaries, as now, and **91.35 %** of its content words (8,138 of 8,909).

Decided here, so not asked: the usage notes go from articles alone, by the rule as written (D8); the
four analysis items go to the programme's backlog (D13); the 30 literal phrases stay offered (D13);
the source gaps' 25 commonest Italian words are listed in the README and the pull request (D13); the
golden's re-bless is approved on its listed probes after the implementation (task 4.1).

## Risks / Trade-offs

- **[MarkIT is thin]** → it moves 10 rows at fr-en's ratio, all read; what it cannot reach or reads
  wrongly is a reviewed row, the owner's.
- **[Forms rules move ranks and levels]** → judged on the dictionary and MarkIT; every moved line of
  `forms.tsv`, `freq.tsv` and `grammar.tsv` listed (task 3.3); PUD's expected figures stated before the
  run (task 3.2).
- **[A reviewed list grows]** → `FIRST_RUN`, `CORRECTIONS`, the readings' list, `APOCOPES` and
  `OVERRIDES` each hold a reason per row; an update finding a correction or a reading's row firing on
  nothing names it.
- **[Three requirements of open changes are modified]** → copied whole, the clauses this change edits
  named in the pull request; this change archives after them, and change 20 re-copies its block.
- **[es-fr and en-fr keep defects it-fr loses]** → named (D13); they do not move here.
- **[A golden that moves]** → re-blessed once, each probe with its rule, approved on the list.

## Migration Plan

No reader holds an Italian pack before change 20: nothing to migrate. The tables arrive by the
owner's dispatch on this change's branch; rollback is a revert of the tables, the pin and the golden.

## Effort

3–4.5 ideal days, against the programme's 2–3.5 (48b): the gloss rules D2–D10 and their tests
1–1.5; the forms rules, the reviewed rows and `fate` with their tests 0.75–1.25; the dispatch, the
pin, the moved-lines listing, the README, `SOURCES.md` and the measurements 0.75–1; the golden, the
card and the checks 0.25–0.5; the owner's reading 0.25. The forms rules, absent from 48b, account for
the difference; leaving the verb forms' gloss as it is (D7) saves what the label would have cost.

## Implementation (2026-10-25)

Implemented on `main` after change 18; **the tables are not in the pull request**: they arrive with the
owner's dispatch (task 3.1). Every figure below is predicted — reduced locally from it-fr's pinned
release assets (`lingua-pack-sources-it-fr-2026.10.23`, `…-2026.10.24`, `…-morph-it-0.4.8`, MarkIT at
`e210079d`; Python 3.12 and `requirements-reduce.txt`), `main`'s rules first giving `main`'s committed
tables byte for byte — at `pack_version` `2026.10.24+d85215a`: the pack 2,644,651 B (`26c51cc8…`), its
base 1,551,202 B (`a2af1872…`), its layer 1,093,689 B (`f9befdeb…`).

**The gloss rules (D2–D10) reproduce the proposal's prototype row for row**, but where the full section
holds what its extract lacked. Each alone, against the committed rows: D2 14 (14 / 0 / 0 / 0), 13 first
senses; D3 12 (0 / 3 / 5 / 4); D4 5; D6 3; D8 10, 1 gained; D9 14, `trani` and `melano` lost; D10's
example 1 — as measured. **D5**: 28 rows (0 / 1 / 7 / 20) and 60 lemmas gained (0 / 8 / 13 / 39), against
27 and 56: `signor` (the proposal's prototype skipped it, read as *signore* by D11's apocope row, which
takes it out of the ranks anyway), and `intimissimi`, `pressapochismo`, `aeronavale` gained and `uzbeko`
changed past 20,000 — their pointers' words were not in the extract. Together, with the corrections:
92 rows change (27 / 11 / 22 / 32), 59 in their first sense, 61 gain a gloss; coverage 89.4 / 82.0 /
73.4 on the committed ranks. D4's function words count kaikki's `contraction` (an articulated
preposition) as the measured prototype did. A correction that adds an entry is keyed by the texts the
section writes for the word's entries of that part of speech (`()`: none), so a page corrected upstream
leaves it firing on nothing.

**The forms rules (D11), as the requirement words them, move other lines than the census counted:**
- **Rule 1 takes 27 ranked lemmas, not 32.** As written — the form's own entries holding only the verb's
  inflection written without its tag, an ellipsis, or another part of speech's pointer marked obsolete,
  archaic, literary, poetic, dated or colloquial, where a verb's table gives the form as a person —, it
  leaves `sanguina` (its noun is marked « rare », which the list does not name), `rota` (*rotare*'s table
  writes `ruota`, no `rota`), `deviate` (also *deviata*'s plural: another word's inflection), `enno` and
  `risto` (no table gives them as a person), `accorre` (its poetic sense names *accogliere*, a meaning)
  and `carcano` (a surname's lemma, the names rule's); and it takes `semo` and `sieno`, *essere*'s
  inflections written without their tag (« alternative form of siamo: inflection of essere »), which
  *essere*'s dialect table gives as persons. Adding « rare » to the list would take `sanguina` too: the
  owner's call.
- **Rule 2 is a plural of a plural**, as the census counted it (`viti`: `vite`'s plural, `vite` *vita*'s):
  read literally — any candidate that is a noun's plural —, it would have taken out of the tables
  `faccie`, `guancie`, `premii`, `principii` and six more misspellings of a plural. The requirement's
  wording is amended so.
- **Rule 4 reads the dialect's row where its verb is the lemma chosen**, and a verb the form's own entry
  names is no row of the dialect's alone (`cresi`, *credere*'s obsolete past historic by its own entry
  too, stays *credere*'s); otherwise a common word the dialect's table also lists would have moved (`amo`
  → *amare*, `dono` → *donare*, `foro` → *forare*). It moves 21 forms, not 11: besides the design's list,
  `pareranno`, `parerebbe`, `parerà` → *parare*, `poteranno`, `poterebbe`, `poterà` → *potare*, `volse`,
  `volsero`, `volsi` → *volgere*, `faci` → *face* and `fusti` → *fusto* — each its own entry's word, the
  Italian table marking those cells doubtful; `sien` stays *essere*'s, `sieno` being no ranked lemma
  any more (rule 1). The headings rule takes `archaic`, `infinitive` and `subjunctive` out; `imperative`
  reads as *imperativo*, its own entry.
- **The moved lines** (task 3.3): `forms.tsv` 58 forms change lemma, 10 leave, 29 join; `freq.tsv` by
  renumbering alone — 29 lemmas leave (rule 1's 27, `signor`, `od`), `dati`, `prova`, `lega` and 26 at the
  cut join —; `grammar.tsv` in 62 forms, their own and `other` marks and `fate`'s subjunctive.

**D14's figures**: coverage 89.4 / 82.0 / 73.4; 24,269 dictionary words (24,267 after the review's round); 8,302 levels, 19 joining and 19
leaving, 25 changing band; M8's cost 30 / 44 / 66. **UD Italian-PUD 99.27 / 91.37 / 99.90**, not 99.26 /
91.35: the owner's `prova` row brings back its plural `prove` (2 words, unresolved before), as `lega`'s
brings `leghe` back — +2 content words and +2 resolved beyond the design's count (`signor` +1, `lega` +1,
`prova` +2, `prove` +2, `dati` −1). MarkIT's test section 99.43 / 94.65 / 98.87, as before.

**The 456**: by the reduction's own rule — a ranked lemma of the levelled span the dictionary words do not
list and no source glosses —, `main` holds 511; 12 are glossed again (the design's twelve), 5 leave the
ranks (`signor`, `od`; `guardo`, `segui`, `nego` by rule 1), 494 are left.

**Dispatched** (`lingua-pack-update` run 40177647583, mode `reduce`, commit `a7ada5414`): the tables,
the pin, the golden and the card equal the prediction byte for byte.

**Fixed at review (2026-10-25)**, a second reduction:
- **`dati`'s own reading is a plural.** The English section heads « data (plural of dato) » with no
  number, so its own entry gave `dati` a masculine *singular* noun's reading. D12's list now lets a row
  carry a replacement tag — the reading read as that tag, its mark kept —, and holds `dati` *dati*
  `NOUN|Gender=Masc|Number=Sing` → `NOUN|Gender=Masc|Number=Plur`, a tag of the pinned pool, which stays
  byte for byte.
- **Two more corrections** (D10): `crypto` and `cripto`, whose only sense is « Variante de kripton
  (l’élément chimique Kr) » — readers meet them as the cryptocurrency, and no sense of the section says
  « crypto-monnaie » —, lose it rather than lend « Krypton »; `stupidissimo` « Superlatif de stupido :
  « très léger » », the section's slip, reads « Très stupide ». Each is reported upstream.
- **`vadi`**, which leaves the tables with rule 1, is a real word: the colloquial subjunctive of
  *andare* (« che tu vadi »), which `andare`'s table does not list and the tables read before as the
  plural of the obsolete noun `vado`, the ford — as `viti` leaves by rule 2.

Predicted for the round (the owner's second dispatch), reduced locally as before at `pack_version`
`2026.10.24+1cb6715`: the pack 2,645,721 B (`7c73fd30…`), its base 1,551,196 B (`b1301e06…`), its layer
1,094,765 B (`274908f7…`); `tables/it/grammar.tsv` one line (`dati`'s plural), `lexical.tsv` −`cripto`
−`crypto` (24,267), `tables/it-fr/gloss.tsv` and `senses.tsv` −`cripto` −`crypto`, `stupidissimo` « Très
stupide » (32,739 glosses); `forms.tsv`, `freq.tsv`, `level.tsv`, `mwe.tsv`, `tags.tsv` and the NOTICE
byte for byte; coverage 89.4 / 82.0 / 73.4; UD Italian-PUD 99.27 / 91.37 / 99.90, as before. The golden
moves in its pack line and the two vocabulary estimates' universe (24,267 → 24,265, 24,266 → 24,264);
the card does not move.
