# Design

## Context

See proposal.md — Why. What constrains the shape:

- The pipeline (matrix changes 6, 7, 38, 43): a pair's reducer (`reduce-<pair>.py`) turns the
  pinned raw sources into committed tables — the studied side in `tables/<studied>/` when the pair
  is its language's reference (`studied.json` names it), the native side in `tables/<pair>/` —
  with `reduce_common.py` for the rules every pair shares and `reduce_edition_<native>.py` for the
  Wiktionary edition the glosses come from; `pack_sources.py` registers each pair's dumps
  (`DUMPS`), its pinned treebanks (`PINNED`) and its releases; `build.sh` builds, reduces, updates
  and dry-runs a pair; `lingua-pack-update` dispatches one pair, the reference pair bringing its
  readers along, a group per studied language. French's reducer (`reduce-fr-en.py`) holds
  French's studied rules; `reduce_french_treebank.py` reads UD French-GSD's two sections for the
  counts that choose a form's lemma and the part of speech a gloss row opens on.
- The English Wiktionary's Italian section, as `pack_sources.derive` writes it from the English
  edition's dump of 2026-10-03 (`kaikki-Italian.jsonl`, 656,480,462 B, 623,702 entries) — measured
  here on kaikki.org's own extract of the section, 770,859,636 B on 2026-10-21, the same 623,702
  entries laid out differently (where `categories`, `derived`, `related` and `synonyms` sit) (the French section's extract is 583,677,104 B): 719,680
  senses, 589,335 distinct forms; verb paradigms listed form by form,
  participles with their agreement, nouns with their plurals, adjectives with their four forms.
- wordfreq 3.1.1 has an Italian large list (`it`), as it has for Spanish and French; its tokeniser
  splits a piece of one or two letters before a vowel (`l'uomo` → `l` + `uomo`) and strips a
  token's edge apostrophes (`po'` → `po`), so `l`, `d`, `un`, `po` carry the pieces' and
  truncations' frequency, while `dell'acqua`, `quest'anno`, `all'interno` stay whole tokens of
  their own.
- Treebanks (I11): UD Italian-ISDT, VIT, ParTUT and PoSTWITA are non-commercial (denied); UD
  Italian-PUD (1,000 sentences, news and Wikipedia, CC BY-SA 3.0) and MarkIT (≈ 1,300 sentences,
  students' essays of marked constructions, CC BY 4.0, lemmas automatic with corrections) are the
  clean ones. French read GSD's 16,000 sentences for its counts.
- Morph-it! (dual CC BY-SA 2.0 / LGPL; 504,906 entries, 34,968 lemmas at 0.4.7, with features), built
  from the la Repubblica corpus: its authors note gaps in everyday vocabulary. The archive its site
  serves holds 0.4.8 (2009, 505,074 lines) — measured at implementation, below.
- Change 6's pre-pass writes the elided pieces' words, keeps the `di` family whole, splits the other
  articulated prepositions, reads the truncations; change 7's enclitic rule reads a verb's
  clitics off an unlisted form and needs the verb's infinitive, gerund and imperatives listed;
  change 5's fixture holds two hundred forms; the Italian golden runs over it.
- Change 3 (the split) refuses a studied language whose `tags.tsv` is empty, and change 4 has the
  extension check build each pair's parts too (`build.sh --parts`) and `pin.json` record them;
  change 13 fills Italian's pool, replacing the placeholder. Until then `tags.tsv` holds the one canonical tag change 4 gave
  the French fixtures, `NOUN`: change 3 refuses an input with no `tags.tsv`, and an empty one is
  the case change 4 says no fixture exercises, so it-fr never puts it to the test.

## Goals / Non-Goals

**Goals:**

- Every form of the 60,000 commonest Italian lemmas, mapped to one lemma, with the rank that
  drives the counts, committed once and read by every pair studying Italian.
- The tables serve change 6's pre-pass and change 7's cascade exactly.
- Measured on held-out treebanks before they are committed, as Spanish's and French's were.

**Non-Goals:**

- Glosses (change 16), readings and the tag pool (13), levels (14), the dictionary words (16).
- The apostrophe-accent spellings (10); acronyms (11); expressions (12).
- A treebank the licence rule denies, whatever its size.

## Decisions

### D1 — it-fr is Italian's reference pair, and its reducer holds Italian's studied rules

`reduce-it-fr.py` writes `tables/it/` and `tables/it-fr/`; `tables/it/studied.json` names it-fr.
A later pair of Italian (it-en, it-es) reads `tables/it/` as committed and writes its own folder
alone; its reduction brings the reference's rules through `check_reducer`. The shared modules are
not edited.

### D2 — Sources: the English Wiktionary's Italian section, MarkIT's counts, wordfreq, Morph-it!

The Italian section as matrix change 38's catalogue derives it from the English edition's dump,
registered in `DUMPS["it-fr"]`; MarkIT at a commit, in `PINNED["it-fr"]`, its training and
development sections read for their counts and parts of speech (as French read GSD's two), its
test section held out; UD Italian-PUD is no pinned source — every `PINNED` entry is fetched into
the reduction's work folder and named by `pin.json`, which French's held-out test forbids for the
gate —, so `measure/it-ud.sh` pins its commit and sha256, as `es-pud.sh` and `fr-ud.sh` do, and
reads it alone (D8); wordfreq 3.1.1's `it`; Morph-it! 0.4.8 at its sha256, a `PINNED["it-fr"]` entry
whose URL is a one-off release the owner uploads (`lingua-pack-sources-morph-it-0.4.8`), since its
site offers no stable address and the snapshot release publishes the kaikki-derived files alone
(`pack_sources.assets`); it is read under its CC BY-SA 2.0 option, which the 4.0 stack of
`SOURCES.md` admits.

### D3 — What a form is

An Italian word, lowercased and in NFC, the typographic apostrophe read as `'`, matches
`[a-zàèéìíîòóùúâêôäëïöüçñ]+(?:['-][a-zàèéìíîòóùúâêôäëïöüçñ]+)*'?` — the letters the section's Italian
entries use, the loans' included (*würstel*, *crêpe*), words joined by hyphens or by an inner apostrophe,
and the elided pieces' and truncations' final apostrophe; an entry outside it is neither a form
nor a rank, and their count is measured. A form's candidate lemmas
come from the inflections a lemma's entry lists (less kaikki's bookkeeping and an inflection
tagged alternative, obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard, proscribed or
abbreviated, and a row naming a derived word — the `diminutive` (492), `augmentative` (180),
`pejorative` (140), `derogatory` (47) and `endearing` (12) rows of a noun (`casa` → `casétta`,
`casàccia`), the same rows of an adjective (≈ 80) and an adjective's `adverb` row (67: `vero` →
`veramente`), which name other words, read from their own entry where the section has one — 491 of the 1,006 rows
have one, the other 515 (`abatone`, `camerona`) are simply no form; *as implemented*, a noun's relational
adjective and a name's demonym too, and the own entry's « diminutive of » sense a meaning, not a link), from the `form_of` targets of the form's own entry, never from a gender or number
marker, and from a form of a form along one part of speech (`creduta` → `creduto` → *credere*; `parlata`,
the participle and a noun, *parlance*, goes by D5, whose own-entry rule keeps the noun; `vite`,
both the vine and *vita*'s plural, by the first choice of D5). Then **Morph-it!**, when the
owner keeps it among the pinned sources (I11, task 5.1): a form it lists under a lemma the
tables rank, which the Wiktionary lists under no lemma, joins the table under that lemma; a form
the Wiktionary reads is never moved; a form it lists under a lemma the tables do not rank is left
out. Measured: how many forms Morph-it! adds, of which lemmas, and the gates with and without
them; kept only if the gates do not fall.

**The tables' stress marks.** The section's conjugation tables write every form with its stress
marked — `dàre`, `darèbbe`, `dièdi`, `dàto`, `hà` — which no Italian text writes: measured on the
extract, 543,375 of the 615,286 distinct table forms carry a mark on a non-final vowel, and 37,978
a mark on the final vowel alone, where Italian's written accent sits (`darà`, `darò`) but which is
also the table's stress mark on a monosyllable the standard writes bare (`hà`, `dò`). A table form
is read with every non-final mark dropped (`dàre` → `dare`; 335,949 of the 543,375 then match a
headword of the section, the rest being cells no entry lists, read bare the same way), and its
final mark kept — the written accent — unless the entry that carries this verb's form-of link for
the cell is the bare spelling (`hà` → `ha`, `dò` → `do`: 7,627 of the 37,978 final-marked forms
have a bare headword; `darà` and `dà` keep theirs, `dà`'s own entry being accented and `da` the
preposition's, not a form of *dare*; a cell with no entry at all, ≈ 7,094, keeps its mark). The
helper is written in `reduce-it-fr.py` with the last-vowel guard: `reduce-fr-en.py`'s
`_marks_dropped` strips every mark and lives in a hyphen-named file no other reducer imports, and
kaikki's French tables carry no stress marks.

**A table row written with its pronoun.** 148,386 table forms hold a space: 136,225 are a reflexive
verb's rows carrying the clitic (`mi làvo`, `si làva`, `ci laviàmo`; `si` 51,295, `mi` 23,511, `ci`
21,092, `ti` 20,261, `vi` 20,066), and the 12,161 others are a multi-word lemma's cells (`stare
zitto` → `stò zitto`), a `canonical` row with its auxiliary in brackets (`esìstere [auxiliary
essere]`) and rows listing alternatives (`puote, pote`, tagged archaic) — all three give no form (an
expression is change 12's, the bracket is bookkeeping, the alternatives are marked); a row of a
verb carrying a clitic other than the reflexive one, alone or in a cluster (`la hò`, `le prèndo`,
`me ne vàdo`, `ce la fàccio`, ≈ 3,600), gives no form either — a page writes the cluster's words apart, and the plain verb's own rows carry the cell. For a
reflexive row the pronoun is dropped and the bare form is the candidate — this change's own
decision, not French's: change 45's readings pass reads a pronominal row bare (`without_pronoun`)
under the table's own entry, and change 43's forms pass drops spaced rows — filed under the plain
verb where the section has one (`lavo` → *lavare*: one lemma for one paradigm, the reflexive being
the same verb with a clitic, which change 7's enclitic rule reads off `lavarsi` as *lavare* only
while `lavarsi` is unlisted), and under the reflexive lemma where no plain verb exists (`pentirsi`,
`vergognarsi`: `pento` → *pentirsi*). So D6 ranks no reflexive lemma whose plain verb the section
has (`lavarsi` is no lemma of the tables; *lavare* carries its rank and its forms), and a
reflexive-only lemma's clitic-bearing rows — its infinitive, gerund and imperatives, `pentirsi`,
`pentendosi`, `pentitevi` — give their bare base under it (`pentire`, `pentendo`, `pentite`), its own
infinitive excepted from D4's « no form carrying clitics », so that change 7's rule has the bases
it needs. For the measurement: UD Italian-PUD lemmatises a reflexive without its clitic (« si
aggirerebbe » → *aggirare*, `riunirsi` → *riunire*), so forms filed under a reflexive-only lemma
count against D8's content gate — small, and named with the figures.

### D4 — The tables serve I7, as change 6 writes it

Every word the pre-pass writes is a form: the elided pieces' words (`lo`, `una`, `di`, `ci`, …,
by change 6's table, kept in the reducer as a reviewed closed table), the truncations' words
(`poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici`, `bene`), the pieces of the split articulated
prepositions (`a`, `da`, `in`, `su`, `con`, `il`, `lo`, `la`, `i`, `gli`, `le`). The `di` family —
`del`, `dello`, `della`, `dei`, `degli`, `delle` — are lemmas of their own, `dell'` a form of
`dello`; `al`, `nel`, `sul`, `dal`, `col` and their forms are neither forms nor ranks (the
pre-pass never hands them to the lookup) — `dai` excepted: a form of *dare* and no rank, since
change 6 reads the truncation `da'` as `dai` and hands it to the lookup, while a written `dai` is
split by the pre-pass and never reaches it, so listing it hides nothing (I7's cost, change 6). No plain word that begins with an
elided piece (`c'è`, `l'ho`, `dov'è`) is a form: the pre-pass splits them; a hyphenated run may be.
A verb form carrying clitics (`dirmelo`, `vattene`, `eccolo`) is left out: change 7's rule reads
it, and listing it would split nothing and hide the verb.

### D5 — One lemma per form (M8), and what it costs

When a form has several candidates, the first rule that decides wins — Spanish's and French's
order:
1. the override list (`OVERRIDES`: form, lemma, reason), empty of homographs;
2. MarkIT's training and development sections' counts of the form under each candidate;
3. the form's own entry, when it is a lemma;
4. the candidate's wordfreq frequency;
5. alphabetical order.

Before it, the names rule: a form that is a name and another word's keeps only the commoner
reading, by wordfreq — `roma` stays the city, `bianca` and `chiara` read as *bianco* and *chiaro*
against the names (measured; a name whose other reading is the same lemma string, `marco` the
name and the coin, needs no rule). The counts are thinner than French's (MarkIT's two sections,
about a thousand sentences, against GSD's 16,000): measured,
how many forms the counts decide, the own entry, the frequency; the cost M8 accepts — a
dictionary noun whose own form a verb takes (`porta`, `legge`, `sale`, `pesca`) — counted among
wordfreq's 1,000, 2,000 and 5,000 commonest words and compared with Spanish's 30 / 48 / 95 and
French's 30 / 55 / 135. *As implemented* (*Measured at implementation*): the names rule reads a form
whose own entries are names alone as the other word and leaves every other form to the rules above;
`OVERRIDES` holds the owner's `stato` and `stati` → *essere* and `bianca` → *bianco*.

### D6 — Lemmas and ranks: 60,000 by wordfreq, elision stems out, compounds by evidence

The ranks are wordfreq `it`'s order over its words that are not only inflected forms by the first
choice of D5, dense, the 60,000 first kept; a word whose own form reads as another is no rank
(the rank passes to the next); wordfreq's stems of the elided pieces of one or two letters (`l`,
`d`, `c`, `v`, `m`, `t`, `s`, `n`, `ch`, `gl`) and of the truncations (`po`, `mo`, `be`; its
tokeniser strips a trailing apostrophe) are no words, and so is a token it ranks whole that begins
with an elided piece (`dell'acqua`, `quest'anno`, `all'interno`; it splits only a one- or two-letter
piece before a vowel), as French drops them by `_ELIDED_START`; `un` (the article) and the
apocopated forms written without an apostrophe (`buon`, `nessun`, `qualcun`, `ciascun`, `alcun`;
`buon uomo`) are words, ranked or read as forms of *uno*, *buono*, … by the dictionary's links; a
hyphenated word is ranked when MarkIT's training or development section attests it, at the lower
of wordfreq's estimate and the treebank's frequency. `build.sh` keeps 60,000 lemmas for an
Italian pair, as for Spanish and French. *As implemented*: every piece's stem is no word, the longer
pieces' too (`quest`, `com`), wordfreq counting them, but `cent` and the apocopes written without an
apostrophe, which a reviewed table (`APOCOPES`) reads as their words (`bel` → *bello*, `san` →
*santo*); no verb form or infinitive carrying clitics is ranked.

### D7 — The cut, measured

The 60,000th lemma's Zipf frequency, the share of PUD's and MarkIT's tokens the lemmas cover,
and the forms count, against French's (124,040 forms, 2.26 MB) and Spanish's; the pack's size,
recorded, not gated: change 3's D8 holds a base without readings under 5 MiB by construction
(Spanish's is 1.6 MB with its readings, French's forms and ranks 1.24 MB), and Italian's forms,
listed form by form, are the largest of the four.

### D8 — The measurement: PUD gated, MarkIT's test section reported

`scripts/lingua-data/measure/it-ud.sh` after `fr-ud.sh`: the real analyser (change 7's cascade,
change 6's pre-pass) over UD Italian-PUD, held to Spanish's gates (98.5 % resolved, 93.5 %
content lemmas, 97 % AUX) — *as the owner decided on 2026-10-23, Italian's content gate is 90.5 %*
(*Measured at implementation*) —, and over MarkIT's test section, reported; both fetched at the pinned
commit, checked by sha256, never committed, never read by the reduction, which reads MarkIT's
training and development sections alone for its counts — PUD and MarkIT's test section are held
out.

### D9 — Committed tables and the pipeline

`tables/it/`: `forms.tsv`, `freq.tsv`, `studied.json`, `tags.tsv` holding `NOUN` alone until
change 13 fills the pool, a `lexical.tsv` holding nothing until change 11's acronym rows and change 16's dictionary words; `tables/it-fr/`: an empty `gloss.tsv`, `NOTICE` (the Italian
section, wordfreq, MarkIT, Morph-it! if kept — under its CC BY-SA 2.0 option —, credited as
their licences require; PUD is measured against, never shipped, as fr-en's NOTICE leaves it out),
`manifest.json` (studies `it`, glossed in `fr`, Italian's analyser version read from lingua-core,
`levels_estimated` absent until change 14), `pin.json`, `README.md`. Before the first update is dispatched, `tables/it/studied.json`
(`{"reference": "it-fr"}`) and `tables/it/tags.tsv` (`NOUN`) are committed by hand, as French's
were (change 43's D10): `record-build` raises when a studied language's `tags.tsv` is missing, a
kept input nothing writes. `lingua-pack-update` offers
it-fr in `pair` and an `it` group; the monthly check and the reduce job take it through `pairs`;
`lingua-extension-check` builds its pack, whole and in parts, from the committed tables against
its pin, as the six. The Italian baseline keeps its fixture and its golden: a pack of forms and
ranks alone would blind its gloss, expression and level probes, as French's forms change kept its
fixture (its D11); change 16 moves `PackSource` to `Tables` and re-blesses once, as change 5's D6
says and change 48 did for French. *As implemented*: no folder of it-fr's is committed before its
first update, which the pipeline reads from `tables/it/studied.json`, and the builder splits a pair
glossing nothing with an empty `lexical.tsv` (*Measured at implementation*).

### D10 — What moves, and what cannot

No golden moves: the Italian baseline keeps its fixture (D9); the six pairs' tables, pins and
goldens do not move; no package carries it-fr.

## Measured at implementation

Criteria written before any figure was read on UD Italian-PUD or MarkIT's test section: the gate is
PUD's three figures; no rule is added, removed or tuned on a figure of either; a departure from this
design is decided on the dictionary's own content, or by the owner, and recorded here; Morph-it! is
recommended to stay when it adds forms and none of PUD's figures falls with it.

**Material.** kaikki.org's own extract of the Italian section (2026-10-21, 770,859,636 B, the 623,702
entries the dump-derived file holds, laid out differently — the update reduces the derived file of
its day), MarkIT's training and development sections at `e210079d`, Morph-it!'s archive as its site
serves it, wordfreq 3.1.1; the pack built from the reduced tables with the builder `build.sh` runs
(`lingua-pack-build --studied`, and `--split` for the parts), measured by `measure/it-ud.sh` with
`LINGUA_IT_PACK`, Italian's analyser at `1.0.0` (change 7's cascade). Two runs of the reducer over the
same inputs write the same bytes.

**The owner's decisions (2026-10-23).** The first reduction (this design's rules, Morph-it! in) read
PUD 99.25 / 91.58 / 95.94 against Spanish's 98.5 / 93.5 / 97, MarkIT's test section 99.43 / 94.92 /
97.36: under the content and the auxiliary gates. Five variants, declared before any was measured,
were measured on the same inputs (a variant passes only if all three PUD figures meet the gate):

| Variant | PUD resolved / content / AUX | MarkIT test | M8 1k / 2k / 5k | Forms whose lemma changes |
|---|---|---|---|---|
| V0, as above | 99.25 / 91.58 / 95.94 | 99.43 / 94.92 / 97.36 | 28 / 43 / 66 | — |
| V1, `stato`, `stati` → *essere* (`OVERRIDES`) | 99.25 / 91.41 / 99.90 | 99.43 / 94.63 / 98.87 | 29 / 44 / 67 | 2 |
| V2, a form's own « past participle of X » reads as X ahead of the own entry | 99.09 / 92.73 / 95.94 | 99.41 / 95.38 / 97.36 | 33 / 57 / 116 | 2,386 (+1,715 / −1,720) |
| V3, the same ahead of MarkIT's counts | 99.07 / 91.74 / 99.90 | 99.39 / 94.29 / 98.87 | 46 / 75 / 144 | 2,513 (+1,780 / −1,768) |
| V4, V2 and V1 | 99.09 / 92.56 / 99.90 | 99.41 / 95.09 / 98.87 | 34 / 58 / 117 | 2,388 (+1,716 / −1,720) |

None passes. V2–V4's participle rule reads `presso` (the preposition) as *premere*, `contratto` as
*contrarre*, `concorso` as *concorrere*, `peccato` as *peccare*, `esercito` (the army; also *esercire*'s
participle) as *esercitare* — V3 also `fatto`, `posto`, `punto`, `subito`, `partito`, `passato`,
`risultato` as their verbs —, and its nouns read as an unranked verb leave the table
(`conflitto`, `campionato`). The owner chose **V1** — `stato` and `stati` read as *essere*, « lo Stato »
reading as *essere* its accepted cost — and set **Italian's content gate at 90.5 %**, a departure from
D8's « Spanish's gates », resolved staying at 98.5 % and auxiliaries at 97 %: MarkIT's two sections
(30,292 words) are about sixteen times thinner than UD French-GSD's (390,368), so the own entry and
the frequency decide where GSD's counts decided French's; UD reads a participle as its verb where the
dictionary keeps the noun or the adjective (`fatto`, `utilizzato`, `deciso`), and a `-mente` adverb
(77 of PUD's words) or a feminine noun (31) as another lemma by its conventions; and no measured
variant reaches 93.5. `measure/it-ud.sh` passes it to `lingua-pack-measure --min-content`. **Morph-it!
stays** (task 5.1's choice).

**The committed rules, measured** (the owner's decisions and the review's fixes below):

| | Words | Resolved (98.5) | Content (90.5) | AUX (97) |
|---|---|---|---|---|
| UD Italian-PUD (gate) | 19,155 | **99.26 %** | **91.39 %** of 8,909 | **99.90 %** of 985 |
| MarkIT test (reported) | 8,985 | 99.43 % | 94.65 % of 4,113 | 98.87 % of 530 |

**At the update (2026.10.23).** `lingua-pack-update` run 39974561742 reduced the section derived from
the English edition's dump of 2026-10-21 (624,122 entries, sha256 `ae7de484…`) under these rules:
128,429 forms (809 of them Morph-it!'s, under 638 lemmas), 60,000 lemmas, the last (`aiu`) at Zipf
2.06; of the 15,489 forms reaching several lemmas, 3 by an override, 1,268 by MarkIT's counts, 6,045 by
the own entry, 8,134 by frequency, 39 alphabetically; the names rule 1,710; M8's cost 32 / 47 / 69; the
pack 1,287,014 B (sha256 `2e03059f…`), its base 1,285,594 B and its layer 1,645 B. `it-ud.sh` on the
committed tables: **PUD 99.26 / 91.38 / 99.90** (19,155 / 8,909 / 985 words) against 98.5 / 90.5 / 97,
MarkIT's test section 99.43 / 94.68 / 98.87; 768 content words missed, 92.50 % without the 77 `-mente`
adverbs and the 31 feminine nouns; without Morph-it! 99.22 / 91.30 / 99.90. Reduced again on another
machine from the release's asset, the tables come out byte for byte. The figures below are the local
reduction's, on kaikki.org's per-language extract.

For information, not a gate: without the 77 `-mente` adverbs and the 31 feminine nouns, PUD's content
words read 92.51 % (8,142 of 8,801). Of the 15,471 forms of the tables that reach several lemmas, 3
are decided by an override, 1,267 by MarkIT's counts, 6,040 by the form's own entry, 8,122 by the
lemma's frequency, 39 alphabetically; the names rule reads 1,704 names alone as another word. M8's
cost, counted as French's README counts it — dictionary nouns whose own form reads as a verb and which
are no ranked lemma —: **32 / 47 / 69** among wordfreq's 1,000 / 2,000 / 5,000 commonest words (Spanish
30 / 48 / 95, French 30 / 55 / 135): `sono`, `era`, `stato`, `fa`, `detto`, `visto`, `dato`, `porta`,
`dati`, `vale`, `lega` among them. **D7**: 60,000 lemmas, the last (`zurighese`) at Zipf 2.07; 128,418
forms (French: 124,040), `forms.tsv` 2,390,841 B, `freq.tsv` 862,406 B; the pack 1,286,037 B (24.5 % of
the budget), its base 1,284,614 B and its layer 1,648 B; 98.22 % of PUD's measured words have a ranked
gold lemma (96.87 % of its content words), 97.83 % of MarkIT test's (97.93 %). **Morph-it!** adds 812
forms under 641 ranked lemmas — truncated infinitives (`aumentar`, `scavar`), truncated third persons
plural (`danzan`), past historic variants (`battè`), superlatives, `trent'` —, moving no reading and no
rank (on the first reduction it moved PUD by +0.03 / +0.08 / 0 and MarkIT test by +0.08 / +0.17 / 0).

**Departures from the design** — decided on the dictionary's content before PUD was read, or by the
review of 2026-10-23:
- *The names rule (D5).* Read literally — a form with a name's entry keeps itself when its wordfreq
  frequency beats every other word's lemma —, it kept 780 names alone against a word, 371 of them among
  wordfreq's 60,000 commonest: plural surnames and given names spelled as participles (`bambini`,
  `cittadini`, `soldati`, `parenti`, `firmato`, `annunciato`), wordfreq counting the string, which is
  the word's inflection too. The rule reads only a form whose own lemma entries are names alone, as
  French's and Spanish's `names` do: it reads as the other word (a page writes the name with its
  capital, which change 7's names rule sets aside), and every other form is D5's. The first
  implementation also let wordfreq decide a form with a common reading of its own beside a name; the
  review found it overrode D5 and MarkIT's counts on 109 of the 119 forms it decided (`ti` → *si*
  against MarkIT's 15 counts of `ti`, `sole` → *solo*, `santo` → *santa*, `conte` → *conto*, `messa` →
  *mettere*, `contessa` no form), and it is gone: those read as D5 reads them (`ti`, `sole`, `santo`,
  `conte`, `messa`, `contessa` themselves). `bianca` reads as *bianco* by an `OVERRIDES` row (its noun
  entry, the cue ball, and MarkIT's one count, a proper noun's, would keep the noun). A given name no
  ranked word reads is no form (`luca`, against the archaic *lucere*); `argentina` stays the country,
  ranked. A name's own rows are no forms either (21 rows).
- *The overrides (D5).* `OVERRIDES` holds three rows, each with its reason: `stato` and `stati` →
  *essere* (the owner's V1) and `bianca` → *bianco*.
- *Derived words (D3).* A noun's relational adjective (`mare` → `marittimo`, 36 rows), a name's demonym
  (`Bergamo` → `bergamasco`, 53) and a plural noun's `plural-of` row (4) name other words, as the
  diminutive rows do: no forms. A sense naming its entry a derived word (« diminutive of casa », 436
  nouns; augmentative 116) is a meaning, not a form link: `casetta`, `minestrone` are lemmas of their
  own, read from their own entry as D3 says.
- *The stems (D6).* wordfreq counts the longer pieces' stems too (`quest` 4.05, `com` 4.37, `cos` 3.91,
  `quell` 3.66, `bell`, `grand`, `all`, `dell`): every piece's stem is no word, as the spec's sentence
  and French's rule say, but the apocopes and `cent`, the euro cent, a word of its own.
- *The apocopes (D6).* A reviewed table (`APOCOPES`: form, lemma, reason) reads each apocope written
  without an apostrophe as its word alone, its lemma the form's one candidate and the form no lemma:
  `un` → *uno*, `buon` → *buono*, `nessun`, `alcun`, `ciascun`, `qualcun` (read so before by their
  `alt_of` links), and `bel` → *bello* (its entry a meaning: it was ranked 429th), `san` → *santo* (its
  own entry, the Greek letter, won: ranked 237th), `gran` → *grande*, `quel` → *quello*, `tal` → *tale*,
  `qual` → *quale*. `bei`, bello's plural before a consonant, is no apocope and reads as *bello*.
- *The pieces (D4).* Each piece and truncation is a form of its word's lemma: `un'` → *uno* (`una` is
  *uno*'s form), `va'` → *andare*, `da'` → *dare*.
- *Clitics (D4).* A clitic compound's entries (« compound of the infinitive dire with me and lo »,
  33,342 entries; and the two « combined form of » kaikki does not tag, `stammi`, `statemi`) are no
  entry of the tables, and a compound word's verb, adverb and interjection entries read the compound
  (`vattene` « go away! »); a word spelled like one with a noun or adjective entry of its own stands
  (`fallo`) — a name's entry does not (`Donati`, beside the compound `donati`) —, and a compound that
  is another word's inflection stays its form (`fatti`, `donati`). A form-of sense naming a verb form
  carrying clitics — its target an infinitive carrying them, the sense an infinitive's, a gerund's, an
  imperative's or a participle's, the form ending in a clitic (`pentirmi`, `avvalendomi`, `recatevi`,
  `figuriamoci`; 6,802 senses) — links nothing, and the form is no rank. A verb whose infinitive
  carries a clitic other than the reflexive one (`andarsene`, `esserci`, `farcela`) is no lemma and its
  rows give no form; a word is such an infinitive only through a verb entry holding a meaning or tagged
  reflexive or pronominal, never a finite form ending as one does (`importi`, `firmi`, `torci`,
  `supporti`). None of them is ranked. A verb's table row naming its auxiliary is bookkeeping.
- *Morph-it!'s version (D2).* The canonical archive holds 0.4.8, not 0.4.7: pinned as served (sha256
  `b137343d…`, which `PINNED` carries and every fetch checks, since a release asset can be uploaded
  again under its name), read without extraction, its lexicon in Latin-1.
- *An empty `lexical.tsv` (D9).* Change 3's split refused a studied language whose dictionary words
  are the pair's glossed lemmas, so that the base never takes a layer's glosses for them; with no gloss
  and no dictionary word, neither part names one, and the builder no longer refuses that case — the
  spec delta modifies split-lingua-pack-format's *One base per studied language* with the exception
  and its scenario, and this change archives after it.
- *The first update (D9).* No folder of it-fr's is committed before its update: `pack_sources.py
  pairs --after it-fr` names it from `tables/it/studied.json`, the update's Reduce step reports it
  against an empty folder, and the committed-tables check accepts a studied folder holding its record
  and its pool alone while no folder of its reference exists.
- *The update's version, reduced again (D9).* The update writes `pack_version` as its snapshot
  (`2026.10.23`); the reduce job of `lingua-extension-check` reduced the same pinned sources again and
  named the rules (`2026.10.23+06c137f`), moving the manifest, the pack and the layer — every earlier
  pair's update had been followed by a re-reduction committed by hand (fr-en's `2026.10.09` among
  them). A re-reduction of the tables an update committed, under the rules its pin records, now gives
  the update's version again (`pack_sources.reduces_an_update`): the same sources under the same
  rules make the same tables, the same dictionary (*Two releases, one dictionary*), and the reduce job
  reproduces it-fr's committed manifest and pin byte for byte. Under other rules a re-reduction still
  names them; no committed pin moves.
- *The gate's refusal (D8).* The measurement fails, naming the gate and the figure, as Spanish's and
  French's do; the update workflow does not run it, and it is run on the update's tables before they
  are merged.
- `perche` reads as nothing (the plural of *perca*, which is not ranked) and `citta` (Tuscan, *citto*'s
  feminine) is ranked as itself (3,416th): change 10's exception rows read them as *perché* and *città*.

## Risks / Trade-offs

- [Thin counts] → D5 measures what decides; the frequency rule and the own-entry rule carry more;
  the gates on PUD say whether it is enough.
- [Morph-it!'s gaps and newspaper bias] → read for additions only, never to move a reading; kept
  by the gates.
- [The Italian base's size] → D7 records it; change 3 holds a base without readings under the
  budget by construction.
- [kaikki's per-language extract] → matrix change 38's dump derivation is the source; the extract is a
  record, as French's.
- [MarkIT's lemmas automatic with corrections] → its test section is reported, not gated; PUD is
  the gate.

## Migration Plan

Nothing deploys and nothing is released: no package carries it-fr. The tables arrive by the
update workflow's pull request; rollback is a revert.
