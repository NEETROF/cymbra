# Design — add-lingua-fr-en-translated-expressions

## Context

See proposal.md (Why). What exists, on `main` at `9c88442a`:

| Where | What |
|---|---|
| `tables/fr-en/mwe.tsv` | 17,635 expressions, the English Wiktionary's French section alone (change 48 D3, D11; 48b D4, D7): its headwords with a space and the single words French's tokenisation splits that are no form (`d'abord`, `allez-y`), less the senses that only point and the 17 names of `LEFT_OUT` (`à la`, `et des`, `sur ce`, `à le`…). The builder keys 14,493 of them |
| `reduce-fr-en.py` `native_side` | the English edition's pre-passes, fr-en's `read_as_french`, `expression_senses` (pointer-only senses and `LEFT_OUT` out), `common.native_tables(…, fallbacks=[])` — « No fallback: no translation table glosses a French word or expression (D3) » —, `split_words`, `traditional_spellings`, `with_etc_period` |
| `reduce_common` | `read_translations(path, inverted, studied)` (a proper noun's entry left out), `fallback_expressions(expressions, sources)` (a multi-word headword the section does not gloss, up to `FALLBACK_WORDS` = 3 translations in the source's order, one sense) — what es-en reads its direct table with |
| The catalogue | `pack_sources.py EDITIONS["fr"]`: `kaikki-Anglais.jsonl`, `kaikki-fr-Espagnol.jsonl`, `kaikki-fr-traductions.jsonl` (fr→es). Change 48 removed `kaikki-fr-traductions-en.jsonl` (fr→en, « fr-en's direct fallback ») — no pair read it. `DUMPS["fr-en"] = {"en": ("kaikki-French.jsonl",)}` |
| fr-en's pin | snapshot `2026.10.09`; `kaikki-en` (the English dump regenerated 2026-10-03 08:24, no longer served: kaikki regenerates it daily), `kaikki-French.jsonl` under `lingua-pack-sources-fr-en-2026.10.09`; GSD's two sections; wordfreq 3.1.1. fr-es's pin (`2026.10.10`) records the French dump regenerated 2026-10-02 00:10, decompressed sha256 `968f7df0…` |
| The update | `fetch-live` reads today's dump of every edition the pair reads and records the snapshot; `fetch-pinned` follows each record's own `release`, so a pin whose records name different releases already reduces; `assets --release TAG` lists the files of the records naming that release alone |
| The builder | keys a French headword through the core's reading (change 44), two to seven tokens; two headwords of one key: the one that is the key, then the one writing more pieces as the key does, then the alphabetical first |
| The matcher | `match_expressions`: from each token, the longest run that is a key; the next run starts after it |
| The goldens | `fr-en.golden` (217 probes) over the committed tables; `fr-es.golden` shares its phrase probes (`support/french.rs` `PHRASES`); the extension's `word-card-fr-en.txt` and `word-card-fr-es.txt` render every phrase probe of those goldens |
| The owner's decision (2026-10-10) | Change 48's open question 1: the French Wiktionary's English translations of French expressions go into fr-en, in a change of its own |

## Goals / Non-Goals

**Goals:**
- fr-en meets the French expressions the French Wiktionary translates into English and the English
  Wiktionary does not gloss, where they read right.
- The section's expressions keep every key they reach, and what fr-en's rules leave out stays out.
- Each choice measured where a reader meets it: matches on French text, read in context.
- `tables/fr/`, fr-en's glosses, fr-es, en-fr, es-fr, es-en and en-es byte for byte.

**Non-Goals:**
- Words: the table glosses no French word (change 48 D3 stands, measured there).
- The section's own expressions, which read wrong as often (*Measured*, *Found while measuring*):
  a refinement of their own (Open Question 3).
- fr-es, which reads the French Wiktionary's Spanish translations already and shows two of the
  defects D2 avoids (Open Question 4).
- A new snapshot of French: the English edition's bytes are kept (D6).
- The matcher or the builder: no engine change.

## Measured

A prototype in the scratchpad (never committed). Its inputs: the French Wiktionary's dump regenerated
2026-10-02 00:10:16 (736,590,407 B as served, 6,865,136,428 B decompressed, sha256 `968f7df0…`),
the dump fr-es's pin records — `pack_sources.derive` read it in 34 s and wrote, beside
`kaikki-fr-traductions-en.jsonl`, fr-es's `kaikki-fr-traductions.jsonl` with the sha256 fr-es's pin
records (`bd14a5c4…`); fr-en's pinned `kaikki-French.jsonl` (`2d7bbe5f…`) and GSD's two sections;
wordfreq 3.1.1; `reduce-fr-en.py` of `main`, which reproduced every committed table of `tables/fr/`
and `tables/fr-en/` byte for byte. A harness (a scratch example of `lingua-pack`) builds a pack from
tables with `build_pack`, reports each headword's key and the headword that won it, and runs the
engine's `gloss_phrase` on every line of five corpora — each line a selection, its `expressions` the
matches a reader's phrase card shows: UD French-GSD's training and development sentences (323,269
words), Gutenberg (210,338), French Wikipedia (379,182), a Tatoeba sample (147,113) and the
baseline's 13 pages (1,002) — 1,060,904 words in 51,790 lines. On `main`'s tables it builds the pinned
pack (2,537,386 B) and meets 60,359 expressions.

**The source.** `kaikki-fr-traductions-en.jsonl`: 129,366 French entries of the French Wiktionary
that list English translations (14,573,988 B, sha256 `6f54aeda…`; 1,914,447 B with zstd at level
19, the release asset). 37,485 of them are headwords with a space, 5,303 of those names.

**As es-en reads its direct table** (`read_translations`, `fallback_expressions`, the section's
glosses winning a headword they share): 24,112 expressions with a space (21,991 without the
apostrophe rule: 2,121 headwords are written with `’`), the pack 2,997,458 B. On the corpora they
are met 11,440 times (1,607 distinct), 1,067 of the section's matches displaced; their commonest
« à la » "a la, à la" (2,173: a name `LEFT_OUT` holds for the section), « les autres » "other
people" (310), « les deux » "the two, both" (305), « autour de » "around" (257), « plus tard »
"later" (219), « xixe siècle » "nineteenth century" (208), « le grand » "the Great" (173),
« il existe » "there exists" (138). Change 48's 3,248 occurrences counted the table's keys in UD's
425,111 syntactic words through the forms table; this harness runs the engine on raw text.

**The rules, in order** (D1–D3, D5), each counted on what the previous ones leave:

| Rule | With a space | Split words | Example |
|---|---|---|---|
| Candidates: a headword of the table that is no word | 31,546 | 4,187 | — |
| The section glosses the headword (the shared fallback's rule) | 7,434 | 1,151 | « par exemple »: the section's « for example, for instance; on the other hand, by the way » |
| `LEFT_OUT`, as committed | 4 | 0 | « à la » "a la, à la", « sur ce » "on that note", « et si », « point d'inflexion » |
| The section's rules leave it out as only pointing | 2 | 3 | « boite à gants », « crème brulée »; `appuie-tête` |
| A key the section reaches (D2) | 133 | 0 | « fondu enchainé » "cross fading" (`enchainé` read as *enchaîner*, as `enchaîné`); « feu de croisement »; « agente secrète » |
| Only repeats the headword (D3) | 783 | 265 | « à deux » "à deux", « in situ », « panna cotta » |
| Reviewed: wrong where met (D5) | 49 | 2 | « les autres » "other people", « d'argent » "argent" |
| **Kept** | **23,141** | **2,766** | « il existe » "there exists", « s'asseoir » "sit, sit down, sit up" |

211 kept headwords lose a translation that repeated them and keep the others. The builder keys
17,014 of the 23,141 and 117 of the 2,766 (the rest hold a word the lexicon does not, run past seven
tokens, or are one hyphenated token, inert as change 48's split words are); 141 keys are reached by
two table rows, settled by the builder's order.

**The section's key.** Without D2's key rule, 126 keys are reached by a section headword and a table
headword, and the builder's order gives 36 to the table's: a singular over the section's plural
(« feu de croisement » over « feux de croisement », harmless), but also « crème brulée » "creme
brulee, crème brulée, crème brûlée" over the section's « crème brûlée », « fondu enchainé » "cross
fading" over « fondu enchaîné » "lap dissolve" (« par un fondu enchaîné » met as "cross fading"),
« tête brulée », « couteau à huitres », « à coup de » "through" over « à coups de ». With it — each word
read through `tables/fr/forms.tsv`, an elided piece through `ELISIONS` — the harness finds no key
both reach.

**Filters, measured on a reviewed sample.** 200 matches drawn at random (seed 48) from the table's
9,063 after the merge rules (D1, D2), each read in its sentence by one judge (the prototype's
author): 149 right, 13 partly (« en plein » "slap-bang, right smack, chock" for « en plein hiver »),
38 wrong (« les autres » "other people" in « les autres États », « le grand » "the Great" in « le
grand pont de bois », « au vent » "windward, upwind" in « flottait au vent », « première ligne »
"front line" in « la première ligne ferroviaire », « en conserve » "canned" met by « en conservant »).
Each filter, alone:

| Filter (left out) | Matches kept of 200 | Right among them | Right kept | Wrong removed | Table rows kept |
|---|---|---|---|---|---|
| None | 200 | 74 % | 100 % | 0 of 38 | 23,973 |
| Compositional: every word of the first translation in its words' fr-en glosses | 90 | 58 % | 35 % | 8 | 17,813 |
| Compositional, every translation | 110 | 65 % | 48 % | 8 | 18,931 |
| A determiner and one word (« les autres », « le même ») | 176 | 78 % | 92 % | 10 | 23,941 |
| Interjections, pronouns, adjectives (the table's parts of speech) | 154 | 81 % | 84 % | 20 | 23,477 |
| Two words both among the 1,000 commonest | 119 | 82 % | 65 % | 22 | 23,391 |
| No word outside the 2,000 commonest | 76 | 82 % | 42 % | 28 | 20,846 |
| Anything but nouns | 74 | 85 % | 42 % | 30 | 21,801 |
| Fewer than three words | 38 | 84 % | 21 % | 34 | 11,276 |
| A translation repeating the headword (D3) | 198 | 74 % | 99 % | 0 | 23,190 |

A compositional expression is mostly a right one (« autour de » "around", « essayer de » "try",
« le même » "the same"): the reader would have read it word by word, but the expression says it
whole. The wrong ones are common words met as they are where the table names a set phrase, a
field's sense or an address — a reading no shape of the headword tells. A second sample, 100
matches of the expressions outside the 130 most met (seed 4848), reads 77 right, 7 partly, 16 wrong:
the tail is no cleaner than the head.

**The reviewed list.** The 307 expressions with a space met at least five times (78 % of their
matches; the 130 most met are 64 %) and the 55 split words met as often were each read in three or
four of their contexts; 51 of them read wrong in most (D5). On the two samples, the list and D3
remove 33 of the 38 wrong matches and 10 of the 16, for 3 of the 149 right ones and none of the 77
(in sample: the list was read from the same corpora). A fresh sample drawn after them (100 matches,
seed 4849) reads **85 right, 6 partly, 9 wrong** (« premières sorties de l'eau » "roll-out", «
prendre pour garantir » "take for", « oreilles d'âne » "bunny ears", « quantité de matières
premières » "amount of substance"). The section's own expressions, 100 matches sampled the same way
(seed 4800), read **73 right, 1 partly, 26 wrong** (*Found while measuring*).

**The result** (all rules), on the corpora:

| Corpus | Words | Expressions met per 1,000 words, `main` → this change | Lines meeting one |
|---|---|---|---|
| UD French-GSD (train, dev) | 323,269 | 54.3 → 61.6 | 62 → 66 % |
| Gutenberg | 210,338 | 58.9 → 67.9 | 72 → 74 % |
| Wikipedia | 379,182 | 51.6 → 58.8 | 51 → 54 % |
| Tatoeba | 147,113 | 73.1 → 80.5 | 41 → 44 % |
| The baseline's pages | 1,002 | 54.9 → 57.9 | 56 → 58 % |
| **All** | **1,060,904** | **56.9 → 64.5** (60,359 → 68,382) | **52.2 → 55.7 %** |

8,898 matches are the table's (1,598 distinct; 7,661 with a space, 1,237 split words, 960 of them a
reflexive verb before a vowel: « s'étend », « s'installer », « s'assit »); 882 of the section's
(1.5 %) are displaced by a longer or an earlier table expression. Of 50 displaced, sampled (seed
4850), 37 read better (« la Seconde [Guerre mondiale] » → « seconde guerre mondiale » "World War II"
for « guerre mondiale » "world war"; « un certain nombre de »; « aussi bien que » no longer meeting
« bien que » "although"), 11 the same, 2 worse (« Tom et Marie sont comme chien et chat » meets
« être comme » "be like" before « comme chien et chat »; « avait au minimum » meets « avoir à »).
`mwe.tsv` grows from 17,635 rows (833,953 B) to 43,542 (1,909,492 B); the pack from 2,537,386 to
2,989,253 B (+451,867 B, the new notice paragraph about 250 B more), 57 % of the 5 MiB budget and the
largest of the six (es-en 2,608,413 B). `gloss.tsv`, `senses.tsv` and every table of `tables/fr/` are
byte for byte: coverage stays 93.6 / 86.9 / 76.3 %.

**The golden.** `fr-en.golden` re-blessed over the prototype's tables: 2 of its 217 probes move —
`pack` (2,537,386 → 2,989,253 B) and « Il est prêt à le faire », which meets « prêt à » "ready, about
to" —, and the implementation's notice and manifest move `notice` and `licences` too. Five phrase
probes are added (D8): « Il existe plusieurs solutions. » meets « il existe » "there exists"; « Les
autres enfants jouent dehors. » meets nothing; « une crème brûlée » meets the section's « crème
brûlée »; « au Moyen Âge » meets « moyen âge » "Middle Ages"; « sur le champ de bataille » meets the
section's « champ de bataille », not « sur le champ » "on the spot". `fr-es.golden` gains the same
five probes (fr-es's own answers) and nothing else of it moves. Every other test of `lingua-wasm`
passes as committed, the English, Spanish, es-en, en-es and cross-native baselines among them.

**Found while measuring** (not this change's; Open Questions 3 and 4). The section's expressions
read wrong as often as the raw table: « de la » and « de l' » glossed as a partitive where they mean
"of the" (« la tête de l'équipe »), « m'a » "“I'm going”" on « il m'a sauvé », « en être » on « qui
en sont membres », « dire que » "to think that" on « ont dit qu'ils », « en fait » "actually" met by
« en faisant » (33 of its 175 matches), « en tenue » by « s'en tenir », « sa mère » "(so) fucking"
on « sa mère favorise », « espèce de » "you" on « une espèce d'amphibien », « grand pont »
"kick/pass around a player's legs" on « le grand pont de bois », « au fait » glossed "au fait".
fr-es, which reads the French Wiktionary's Spanish translations as its direct fallback, meets « sur
le champ » "En el acto" in « sur le champ de bataille » and gives the key of « crème brûlée » to its
table's « crème brulée ».

## Decisions

### D1 — The table completes the expressions, as es-en reads its direct table, and glosses no word

`reduce-fr-en.py` reads `kaikki-fr-traductions-en.jsonl` from its work folder, a typographic
apostrophe in a headword read as `'` first (as `native_fields` reads the section and fr-es its
tables: 2,121 headwords), through `common.read_translations(…, inverted=False, studied=FR)` — a
proper noun's entry left out —, and offers, after the section's expressions are computed, every
headword of the table in change 48's two shapes: a headword with a space, and a single word that
`split_word` accepts (no form of `tables/fr/forms.tsv`, an apostrophe or a hyphen, a French token).
Its gloss is `common.translation_gloss`'s: up to three English translations, in the table's order,
as one sense, their case kept; « etc » takes its period as the section's (`with_etc_period`). 55 of
them run over the section's 80 characters, as es-en's direct table's do (Open Question 5). The table
is read for expressions alone: no word of `gloss.tsv` comes from it, so French's dictionary words,
the levels and the coverage cannot move with it.

`common.fallback_expressions` reads a headword with a space alone; the split words go through the
same steps in `reduce-fr-en.py` (« s'asseoir », « s'installer »: without them the table would gloss
« se lever » and not « s'asseoir »). No shared module is edited: the rules are fr-en's.

*Rejected — the table's words* (change 48 D3, measured there: names, English words and
misspellings made French dictionary words). *Rejected — expressions with a space alone*: 1,237
matches lost, 960 of them a reflexive verb before a vowel; 2 of the 55 split words met five times
read wrong (D5).

### D2 — The section's expression wins: its headword, its key, and what its rules leave out

1. **Its headword**: a headword the section glosses is not read from the table (the shared
   fallback's rule; 8,585).
2. **Its key**: a table headword whose words, each read through `tables/fr/forms.tsv` and an elided
   piece through `ELISIONS` (`d'achats` → `de achat`), are a section expression's is left out (133).
   The builder's order would otherwise give 36 keys to the table's spelling (*Measured*). The key is
   approximated — the core's reading is the builder's alone (change 44 D1 rejects a copy of it) —
   and the pull request runs the prototype's harness to show that no key is reached by both (0 in
   the prototype).
3. **What its rules leave out**: `LEFT_OUT`'s headwords and the section's headwords whose every
   sense only points (change 48 D11) are not read from the table: « à la » would otherwise be met
   2,173 times as "a la, à la", « boite à gants » would come back beside « boîte à gants ».

A section entry that leaves no gloss for another reason — a proper noun's entry (« Moyen Âge », «
Seconde Guerre mondiale »), a pointer the dump tags (« tout d'abord ») — lets the table gloss it: «
au Moyen Âge » meets « moyen âge » "Middle Ages" (88 matches), « tout d'abord » "first of all,
mainly". *Rejected — every headword the section has an entry for*: 117 more rows out, 368 matches
with them, nearly all right (« moyen âge » 88, « seconde guerre mondiale » 66, « tout d'abord » 17).
*Rejected — a source rank in the builder*: a new column of `mwe.tsv` and a rule every pair's builder
reads, for what the reducer settles alone.

### D3 — A translation that repeats the headword is no gloss

A translation equal to its headword (case aside, `’` read as `'`) is the studied language, not the
reader's (M5; *A gloss is in the reader's language, written by a person*), as fr-es's D5 reads its
own: it is dropped, and a headword left with no translation is no expression — 1,048 headwords
(« à deux » "à deux", 64 matches, mostly « à deux heures »; « in situ »; « panna cotta »), 211 others
keeping their other translations. English writes some of them alike (« in situ », « a cappella »): no
list of those is kept (Open Question 5).

### D4 — No structural filter

Each filter of *Measured* trades right matches for wrong ones at a worse rate than reading them:
compositionality removes 8 of the 38 wrong for 65 % of the right, because a compositional expression
is mostly right; the parts of speech 20 for 16 %; the nouns alone 30 for 58 %; a determiner and one
word 10 for 8 %, and only what the reviewed list removes anyway (« les autres », « le grand », « mon
cœur »). None is taken. *Rejected — attestation in a corpus*: an expression met nowhere costs no
reader anything and its bytes are counted (D7); an expression met often is the one to read, which D5
does.

### D5 — A reviewed list: 51 expressions left out by name

`LEFT_OUT` gains the table expressions read wrong in most of the contexts they are met in, each with
its reason, as change 48b's are: a person's decision, as an override row is. The review read every
expression met at least five times on the corpora in three or four of its sentences: 307 with a
space (78 % of their matches) and 55 split words. The 51, by reason (matches on the corpora in
parentheses):

- **Words met as they are, where the table names a set phrase or a field's sense** (33): « les
  autres » "other people" (310), « le grand » "the Great" (173), « d'argent » "argent" (79), « d'azur »
  "azure" (75), « plus haut » "above" (65), « de haut » "haughtily" (51), « travailler à » "work
  towards, strive to" (45), « un temps » "once" (39), « une étoile » "one stars, one-star" (25), « en
  premier » "first" (25), « se mêler » "get involved in" (21), « il se trouve » "it happens" (19), « la
  peau » "like hell" (18), « sortir avec » "date" (15), « au vent » "windward, upwind" (15), « bien
  du » "many, a lot of" (14), « première ligne » "front line, front row" (12), « ombre de » "in umbra"
  (11), « quatre heures » "afternoon snack" (10), « grand homme » "big man" (9), « la mettre » "trick"
  (8), « première partie » "opening act" (7), « bon vieux » "trusty" (7), « c'est une chose » "is one
  thing" (7), « du gâteau » "piece of cake" (7), « la barbe » "bother" (7), « au temps » "as you
  were" (7), « petite bête » "bug" (6), « à la ligne » "new line" (6), « à ses yeux » "in my view" (6),
  « sur le champ » "on the spot" (6), « avoir l'œil » "keep an eye on" (5), « graine de » "in the
  making" (5);
- **An address or an interjection met as plain words** (4): « à vous » "over" (64), « mon cœur »
  "honey, darling" (53), « et bien » "well, so" (41), « tu vois » "you know" (31);
- **A name met as itself** (4): « côte d'azur » "blue band" (38), « louis xiv » "Louis XIV-style"
  (23), « henri iv » "Henri IV style" (10), « de gaulle » "winged corkscrew" (7);
- **A number met as itself** (1): « à neuf » "anew" (9: « à neuf heures »);
- **A verb's form met as a noun's headword** (5; M8 reads the noun as the verb): « être cher » "loved
  one" (12: « tout est cher »), « groupe a » "blood type A" (8: « le groupe a recruté »), « en laisse »
  "on a leash" (7: « en laissant »), « en joue » "aim" (6: « en jouant »), « en conserve » "canned" (5:
  « en conservant »);
- **A wrong translation** (4): « se mettre d'accord » "pull together", « voie navigable »
  "thoroughfare", « se heurter » "blunder", « se serrer » "clench" (5 each).

Together 1,449 matches. Left in, read partly right: « en plein » "slap-bang, right smack, chock",
« avoir à » "have to", « on dit », « aussi bien », « mon père » "father", « pour tout » "for all".
The expressions met fewer than five times (1,285 distinct, 22 % of the table's matches) are not
reviewed: the owner's sample reads them (task 6.2; Open Question 1).

*Rejected — a list of what reads right* (the 311 others): it would freeze the table at this
snapshot; a later update brings new expressions, which this list leaves in and a later review reads.

### D6 — fr-en's pin gains the French edition, its English edition kept

`EDITIONS["fr"]` registers `kaikki-fr-traductions-en.jsonl` again (`("translations", "fr", "en")`,
« fr-en's expressions »), and `DUMPS["fr-en"]` reads it: `{"en": ("kaikki-French.jsonl",), "fr":
("kaikki-fr-traductions-en.jsonl",)}`. An update of fr-en would read today's English dump too — a
new French dictionary, `tables/fr/` and fr-es moving in the same pull request as the expressions —
and the English dump of 2026-10-03 is no longer served. So an update can keep an edition:

- `pack_sources.py fetch-live --keep-edition E` (repeatable): for each edition the pair reads that is
  named, its record is carried byte for byte (release, dump identity, files) and its files fetched
  from the release it names, checked by sha256, as `fetch-pinned` does; the other editions are read
  from today's dumps under the new snapshot, whose release is the pair's own; PINNED sources and
  wordfreq as in any update. A named edition the pin has no derived-files record for, or one the pair
  does not read, fails before anything is fetched. `assets --release` and `dumps --release` list the
  new release's files alone, as they already do.
- `build.sh`'s update mode passes `LINGUA_KEEP_EDITIONS` (comma-separated) to `fetch-live`;
  `lingua-pack-update` gains a `keep` input (empty by default; accepted in mode `update` alone).

The owner dispatches it [manual] on this change's branch: mode `update`, pair `fr-en`, keep `en`. It
reads the French dump alone (≈ 0.74 GB served), publishes `lingua-pack-sources-fr-en-<snapshot>`
holding `kaikki-fr-traductions-en.jsonl.zst` (≈ 1.9 MB), and pushes `lingua-pack/fr-en/<snapshot>`,
bringing fr-es along, reduced from its own pin. The branch's pin is taken; `build.sh --reduce fr-en`
from it writes the committed tables — `pack_version` `<snapshot>+<digest[:7]>`, as changes 48 and 48b
committed — and reproduces the branch's `tables/fr/` and fr-es byte for byte. fr-en's pin then names
two releases: `kaikki-en` the 2026.10.09 one, `kaikki-fr` the new one; `fetch-pinned` and the reduce
job read both. If kaikki still serves the dump of 2026-10-02, fr-en's and fr-es's pins record one
regeneration of the French dump; if a newer one, the figures are re-measured on it (task 3.2).

`NOTICE` gains the French Wiktionary (« the English translations its French entries list, for the
expressions the English Wiktionary's French section does not gloss »), the manifest's licences read
« kaikki / enwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL) ». The next plain update of fr-en reads
both dumps and publishes both files.

*Rejected — a plain update*: a dictionary update, the expressions and every French golden moving
in one pull request (change 48 D5's reason). *Rejected — the asset published by hand*: the workflow
is the only place upstream is read (*A dictionary update is a reviewed decision*). *Rejected —
re-publishing `kaikki-French.jsonl` under the new release*: 510 MB uploaded again for bytes another
release holds, a new code path to copy assets between releases.

### D7 — The pack, measured

2,989,253 B in the prototype (+451,867 B; the notice's paragraph about 250 B more), under the 5 MiB
budget, built and checked against its pin wherever the committed pairs' packs are. No package
carries it before change 52; once listed, the extension package grows by about as much.
`row-gloss-tables.spec.ts` checks fr-en's rows as before: none of the table's 25,907 is empty or
ends on an opening mark.

### D8 — What moves, what cannot

- **Moves**: `tables/fr-en/mwe.tsv`, `NOTICE`, `manifest.json` (`pack_version`, `licences`),
  `pin.json` (`snapshot`, `sources.kaikki-fr`, `reducer`, `pack`), `README.md`; `SOURCES.md`;
  `fr-en.golden` (`pack`, `notice`, `licences`, « Il est prêt à le faire », and the five probes of
  *Measured* added to `support/french.rs` `PHRASES`); `fr-es.golden` (the five probes added, its 217
  byte for byte); `word-card-fr-en.txt` (« Il est prêt à le faire »'s row « prêt à — ready » for
  « prêt — ready », and the five probes) and `word-card-fr-es.txt` (the five probes).
- **Cannot move**: `tables/fr/` (the studied side reads `kaikki-French.jsonl`, GSD and wordfreq,
  byte for byte, and fr-en's glosses do not move, so neither do its dictionary words and levels);
  `gloss.tsv`, `senses.tsv`; fr-es's tables, pin and pack (its pin's studied record reads `tables/fr/`
  as committed); en-fr, es-fr, es-en and en-es — no file of their rule digests is edited
  (`reduce_common.py`, the three `reduce_edition_*.py` and their reducers), `pack_sources.py`,
  `build.sh` and the workflow are in no digest, and a catalogue file more in the French edition is
  derived in a run that reads that dump without entering any other pair's pin. Their tables, pins,
  packs, goldens and `selection-rows-fr.txt` pass as committed.

### D9 — Order and archive

| Change | Relation |
|---|---|
| 48 fr-en, 48b refinement | before (on `main`): the section's expressions and `LEFT_OUT`, which this change extends |
| 43b, 43c (`ca`, `CA`) | either side: they re-reduce fr-en from the same pin and move other tables of it; whichever lands second re-reduces on the other |
| 49 fr-es | none: fr-es does not move |
| 52 enable | the owner's (Open Question 2): before it, fr-en ships with these expressions |

`archiveAfter`: 38 (the catalogue), 43 (French's tables and the update of fr-en), 44 (the keys),
48 (fr-en's glosses and pin), 48b (`LEFT_OUT`). Sentences of open changes this change makes true of
their time, to amend when each is archived (nothing to decide): 48's « reduced from the English
Wiktionary's French section alone » and « no translation table SHALL gloss a French word or
expression in fr-en » → « … French word; its expressions are completed by the French Wiktionary's
English translations »; 48's « fetching no source the pin does not record » and *The same snapshot*
(« its pin keeps its snapshot and its sources »), true of 48's pull request; 43's and 52's scenario
*An update of fr-en* (« it fetches the English edition's dump alone ») → « the English and French
editions' dumps »; 38's *A pair of stage 3 registers what it reads* (« three files ») → « the
English Wiktionary's French section and the French Wiktionary's English translations ».

## Risks / Trade-offs

- **[One judge]** → every figure of *Measured* is one reader's, ± 7 % on 100; the samples and the
  list are given to the owner (task 6.2), the list with its contexts.
- **[The list is read on the corpora it is measured on]** → the confirmation sample was drawn after
  it; the expressions met fewer than five times are not read one by one (outside the 130 most met,
  16 wrong of 100 before the list, 6 of 90 after it).
- **[The approximate key]** → it can miss a key the core reads otherwise; the pull request runs the
  harness, and a residual table row winning a section key is left out by name.
- **[A section expression displaced]** → 882 of 60,359, of which 2 in 50 read worse; a worse one met
  often goes on the list.
- **[The kept edition goes stale]** → the next plain update of fr-en reads both dumps; keeping an
  edition is for adding one, not for avoiding updates.
- **[A second release in fr-en's pin]** → `fetch-pinned` already follows each record's release, and
  the cache keys an asset by its sha256.
- **[A bigger package]** → 0.45 MB once fr-en ships, inside the budget.
- **[kaikki regenerates the French dump before the dispatch]** → the figures and the list are
  re-measured on it (task 3.2); fr-en's and fr-es's pins then record two regenerations, as es-fr's
  and es-en's do.

## Migration Plan

Nothing to migrate: no reader holds a French pack. Rollback is a revert of the tables, the pin and
the rules; the published release stays, as every snapshot's does.

## Effort

2.5–4.25 ideal days: the reducer's table, merge rules, `LEFT_OUT` rows and tests 1–1.5; the kept
edition (`fetch-live`, `build.sh`, the workflow input) and its tests 0.75–1.25; the dispatch, the
re-reduction, pin, README, `SOURCES.md` and notice 0.25–0.5; the goldens, snapshots and probes
0.25–0.5; the harness re-run, the comparison and the owner's sample 0.25–0.5.

## Open Questions

**Settled by the owner on 2026-10-10 (in session):** the reviewed list of 51 is enough, no finer review
(1); the expressions are added before change 52 ships French (2); the section's own noisy expressions
and fr-es's two defects are each a change of their own (3, 4); the two small defaults as recommended —
every self-translation dropped, glosses kept whole (5).

For the owner, none blocking but the first:
1. **The list of 51** (D5). Do these read wrong to you, and should any stay — « travailler à » (met
   right in « il travaille à un projet », wrong in « il travaille à Genève »), « sortir avec » "date"
   (« il sortit avec elle »), « d'argent » "argent" (the heraldic silver, met as money)? And is
   reading down to five matches enough, or should the review go further? The 1,285 expressions met
   fewer than five times are not read one by one; by the samples, about one match in ten of the
   result still reads wrong.
2. **Before or after French ships** (change 52). Before: English speakers meet « il existe », « plus
   tard », « au Moyen Âge » from the first release, and 52 waits for this change as it waits for 48b.
   After: a later update of fr-en, released on its own. Recommended: before.
3. **The section's own expressions** read wrong as often (26 of 100 sampled: « de l'équipe » met as a
   partitive "some", « il m'a sauvé » as "I'm going", « en faisant » as « en fait » "actually"): a
   refinement of their own, before or after 52?
4. **fr-es** meets « sur le champ » "En el acto" in « sur le champ de bataille » and gives the key of
   « crème brûlée » to « crème brulée »: the same merge rules for fr-es, in a fr-es change of its own?
5. **Two small defaults**: a translation identical to the French (« in situ », « a cappella »,
   « panna cotta ») is dropped even where English uses it — keep a list of those English writes
   alike, as fr-es keeps Spanish's? And 55 table glosses run past 80 characters (the longest, « même
   une horloge arrêtée donne l'heure juste deux fois par jour », 142): cut them at 80 like the
   section's, or keep them whole like es-en's? Recommended: no list, kept whole.
