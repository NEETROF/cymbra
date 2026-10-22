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
- Morph-it! 0.4.7 (dual CC BY-SA 2.0 / LGPL; 504,906 entries, 34,968 lemmas, with features), built
  from the la Repubblica corpus: its authors note gaps in everyday vocabulary.
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
reads it alone (D8); wordfreq 3.1.1's `it`; Morph-it! 0.4.7 at its sha256, a `PINNED["it-fr"]` entry
whose URL is a one-off release the owner uploads (`lingua-pack-sources-morph-it-0.4.7`), since its
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
have one, the other 515 (`abatone`, `camerona`) are simply no form), from the `form_of` targets of the form's own entry, never from a gender or number
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
French's 30 / 55 / 135.

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
Italian pair, as for Spanish and French.

### D7 — The cut, measured

The 60,000th lemma's Zipf frequency, the share of PUD's and MarkIT's tokens the lemmas cover,
and the forms count, against French's (124,040 forms, 2.26 MB) and Spanish's; the pack's size,
recorded, not gated: change 3's D8 holds a base without readings under 5 MiB by construction
(Spanish's is 1.6 MB with its readings, French's forms and ranks 1.24 MB), and Italian's forms,
listed form by form, are the largest of the four.

### D8 — The measurement: PUD gated, MarkIT's test section reported

`scripts/lingua-data/measure/it-ud.sh` after `fr-ud.sh`: the real analyser (change 7's cascade,
change 6's pre-pass) over UD Italian-PUD, held to Spanish's gates (98.5 % resolved, 93.5 %
content lemmas, 97 % AUX), and over MarkIT's test section, reported; both fetched at the pinned
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
says and change 48 did for French.

### D10 — What moves, and what cannot

No golden moves: the Italian baseline keeps its fixture (D9); the six pairs' tables, pins and
goldens do not move; no package carries it-fr.

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
