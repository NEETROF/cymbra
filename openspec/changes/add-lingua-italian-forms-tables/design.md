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
- The English Wiktionary's Italian section, as kaikki derives it from the English edition's dump
  (`kaikki-Italian.jsonl`, 770,859,636 B on 2026-10-21, regenerated 2026-10-03; the French section's
  is 583,677,104 B): 719,680 senses, 589,335 distinct forms; verb paradigms listed form by form,
  participles with their agreement, nouns with their plurals, adjectives with their four forms.
- wordfreq 3.1.1 has an Italian large list (`it`), as it has for Spanish and French; its tokeniser
  splits at the apostrophe, so `l`, `d`, `un`, `dell`, `nell`, `all`, `quest`, `anch` carry the
  elided pieces' frequency.
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
- Change 3 (the split) needs a non-empty `tags.tsv` per studied language for the parts; change 13
  fills Italian's with its pinned pool. Until then the file is written empty, as French's was, and
  the whole pack is what the baseline and the checks build.

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

### D2 — Sources: the English Wiktionary's Italian section, PUD's and MarkIT's counts, wordfreq, Morph-it!

The Italian section as change 38's catalogue derives it from the English edition's dump,
registered in `DUMPS["it-fr"]`; UD Italian-PUD and MarkIT's training section at a commit, in
`PINNED["it-fr"]`, read for their counts and parts of speech; wordfreq 3.1.1's `it`; Morph-it!
0.4.7 at its sha256, as a release asset of it-fr's since its site offers no stable address.

### D3 — What a form is

An Italian word, lowercased and in NFC, the typographic apostrophe read as `'`, matches
`[a-zàèéìíîòóùú]+(?:['-][a-zàèéìíîòóùú]+)*'?` — letters, words joined by hyphens or by an inner
apostrophe, and the elided pieces' and truncations' final apostrophe. A form's candidate lemmas
come from the inflections a lemma's entry lists (less kaikki's bookkeeping and an inflection
tagged alternative, obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard, proscribed or
abbreviated), from the `form_of` targets of the form's own entry, never from a gender or number
marker, and from a form of a form along one part of speech (`parlata` → `parlato` → *parlare*;
`vite`, both the vine and *vita*'s plural, goes by the first choice of D5). Then **Morph-it!**: a form it lists under a lemma the
tables rank, which the Wiktionary lists under no lemma, joins the table under that lemma; a form
the Wiktionary reads is never moved; a form it lists under a lemma the tables do not rank is left
out. Measured: how many forms Morph-it! adds, of which lemmas, and the gates with and without
them; kept only if the gates do not fall.

### D4 — The tables serve I7, as change 6 writes it

Every word the pre-pass writes is a form: the elided pieces' words (`lo`, `una`, `di`, `ci`, …,
by change 6's table, kept in the reducer as a reviewed closed table), the truncations' words
(`poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici`, `bene`), the pieces of the split articulated
prepositions (`a`, `da`, `in`, `su`, `con`, `il`, `lo`, `la`, `i`, `gli`, `le`). The `di` family —
`del`, `dello`, `della`, `dei`, `degli`, `delle` — are lemmas of their own, `dell'` a form of
`dello`; `al`, `nel`, `sul`, `dal`, `col` and their forms are neither forms nor ranks (the
pre-pass never hands them to the lookup); `dai` is no form either, the verb's second person being
reached through `dare`'s other forms (I7's cost, change 6). No plain word that begins with an
elided piece (`c'è`, `l'ho`, `dov'è`) is a form: the pre-pass splits them; a hyphenated run may be.
A verb form carrying clitics (`dirmelo`, `vattene`, `eccolo`) is left out: change 7's rule reads
it, and listing it would split nothing and hide the verb.

### D5 — One lemma per form (M8), and what it costs

When a form has several candidates, the first rule that decides wins — Spanish's and French's
order:
1. the override list (`OVERRIDES`: form, lemma, reason), empty of homographs;
2. PUD's and MarkIT's training section's counts of the form under each candidate;
3. the form's own entry, when it is a lemma;
4. the candidate's wordfreq frequency;
5. alphabetical order.

Before it, the names rule: a form that is a name and another word's keeps only the commoner
reading, by wordfreq — `roma` stays the city, `marco` the name against the coin (measured),
`sale` the salt. The counts are thinner than French's (2,300 sentences against 16,000): measured,
how many forms the counts decide, the own entry, the frequency; the cost M8 accepts — a
dictionary noun whose own form a verb takes (`porta`, `legge`, `sale`, `pesca`) — counted among
wordfreq's 1,000, 2,000 and 5,000 commonest words and compared with Spanish's 30 / 48 / 95 and
French's 30 / 55 / 135.

### D6 — Lemmas and ranks: 60,000 by wordfreq, elision stems out, compounds by evidence

The ranks are wordfreq `it`'s order over its words that are not only inflected forms by the first
choice of D5, dense, the 60,000 first kept; a word whose own form reads as another is no rank
(the rank passes to the next); wordfreq's elision stems (`l`, `d`, `c`, `s`, `m`, `t`, `v`, `n`,
`un`, `dell`, `nell`, `all`, `dall`, `sull`, `quest`, `quell`, `anch`, `senz`, `tutt`, `cos`,
`quand`, `dev`, …) are no words; a hyphenated word is ranked when PUD or MarkIT attests it, at the
lower of wordfreq's estimate and the treebank's frequency. `build.sh` keeps 60,000 lemmas for an
Italian pair, as for Spanish and French.

### D7 — The cut, measured

The 60,000th lemma's Zipf frequency, the share of PUD's and MarkIT's tokens the lemmas cover,
and the forms count, against French's (124,040 forms, 2.26 MB) and Spanish's; the pack's size
against the budget (change 3 holds a base under 5 MiB: Italian's forms, listed form by form, are
the largest of the four — measured, and the cut lowered to 50,000 if the base would not fit).

### D8 — The measurement: PUD gated, MarkIT's test section reported

`scripts/lingua-data/measure/it-ud.sh` after `fr-ud.sh`: the real analyser (change 7's cascade,
change 6's pre-pass) over UD Italian-PUD, held to Spanish's gates (98.5 % resolved, 93.5 %
content lemmas, 97 % AUX), and over MarkIT's test section, reported; both fetched at the pinned
commit, checked by sha256, never committed, never read by the reduction, which reads MarkIT's
training section alone for its counts — the test section is held out.

### D9 — Committed tables and the pipeline

`tables/it/`: `forms.tsv`, `freq.tsv`, `studied.json`, an empty `tags.tsv` and `lexical.tsv`
(filled by changes 13 and 16); `tables/it-fr/`: an empty `gloss.tsv`, `NOTICE` (the Italian
section, wordfreq, PUD, MarkIT, Morph-it! if kept, credited as their licences require),
`manifest.json` (studies `it`, glossed in `fr`, Italian's analyser version read from lingua-core,
`levels_estimated` absent until change 14), `pin.json`, `README.md`. `lingua-pack-update` offers
it-fr in `pair` and an `it` group; the monthly check and the reduce job take it through `pairs`;
`lingua-extension-check` builds its pack from the committed tables against its pin and runs the
Italian baseline over it: the baseline's `PackSource` moves from `Testdata` to `Tables`, the golden
re-blessed once (change 5's scenario), the fixture kept for the tests that build it, as French's.

### D10 — What moves, and what cannot

The Italian golden moves once (the pack); the six pairs' tables, pins and goldens do not; no
package carries it-fr.

## Risks / Trade-offs

- [Thin counts] → D5 measures what decides; the frequency rule and the own-entry rule carry more;
  the gates on PUD say whether it is enough.
- [Morph-it!'s gaps and newspaper bias] → read for additions only, never to move a reading; kept
  by the gates.
- [The Italian base over 5 MiB] → D7 measures the forms count; the cut lowers to 50,000 before
  anything else is dropped.
- [kaikki's per-language extract] → change 38's dump derivation is the source; the extract is a
  record, as French's.
- [MarkIT's lemmas automatic with corrections] → its test section is reported, not gated; PUD is
  the gate.

## Migration Plan

Nothing deploys and nothing is released: no package carries it-fr. The tables arrive by the
update workflow's pull request; rollback is a revert.
