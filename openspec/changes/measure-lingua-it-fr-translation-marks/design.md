# Design

## Context

See proposal.md — Why. Where things stand:

- `apps/lingua-extension/tool/marks/`: `pud.mjs` pins UD English-, Spanish- and French-PUD (commit,
  file, sha256; fetched into the git-ignored `scripts/lingua-data/work/marks/`, never committed —
  CC BY-SA); `select_corpus.mjs` picks, every tenth PUD sentence, one word per language — NOUN, VERB,
  NOUN, ADJ in turn, not the sentence's first, letters only (three or more), not in a multiword
  token; else the first NOUN, VERB or ADJ; else the next sentence for every language — over the
  studied languages at once, English driving the sentence order (`LANGUAGES = ["en", "es", "fr"]`);
  `measure.mjs` runs the extension's marking through a pair's route on each selection, with a trap
  answered as the extension answers it — a two-model route through `translateViaPivoting`;
  `stop-words.mjs` holds the NATIVE language's function words for the gloss experiment (fr, en, es),
  read from the route's last model; `tier.mjs` the first tier (90 % correct, at most 25 % withheld).
  `src/translate/markup.ts`'s `MARKED_PAIRS` lists en-fr, es-fr, es-en, en-es, fr-en, fr-es.
- `test/translate-marks.spec.ts` holds the corpus (its languages, each step's, the letters-only
  rule), each listed pair's results and judgments, and the list as exactly the judged pairs on the
  tier; `test/translate-relay.spec.ts` asserts the list too.
- Change 2 pins it-en 2.0 and en-it 2.1, routes it-fr through it-en then en-fr, soaks en-it on three
  routes and hands the it-en soak to this change (its D4); `measure_marks.mjs` and `soak_engine.mjs`
  refuse a pair the catalogue does not route.
- Measured: the four-language selection over UD Italian-PUD at commit `71d932aa…` (proposal).

## Goals / Non-Goals

**Goals:**
- it-fr's marks measured on the committed corpus through its route and judged in French; listed if on
  the first tier.
- The it-en model soaked before any Italian pair ships, as change 2 requires.
- The corpus gains Italian without moving any other language's selection or result.

**Non-Goals:**
- it-en and it-es's marks (stage 2, their own measurements on the same Italian selections); fr-it,
  es-it and en-it (stage 3, an Italian stop-word set then, as the native language).
- Offering it-fr's translation (change 20), or deciding what happens below the tier (the owner under
  M15, at change 20).

## Decisions

### D1 — Italian in the corpus, by the same rule

`PUD.it` pins `UD_Italian-PUD` at `71d932aa096368331ba2fd9bc906ec8dd853da8e` (`master`'s head on
2026-10-22; `dev` is ahead), `it_pud-ud-test.conllu`, sha256
`ad5d302bdfd05194155d9ad43dc54c3cb915076d66b3eb6ef33eca9a11f1372a`. Change 9's `measure/it-ud.sh`,
implemented later, may take this pin; if its pin differs, the corpus keeps its own — moving it would
move selections the requirement holds — and `tool/marks/README.md` names both. `LANGUAGES` gains
`it`, after `fr`, so each step's Italian selection is written after its French one. A step takes a
sentence only where every language has a word: measured, Italian has one on every step, so no step
moves to a later sentence and the 300 committed selections stay byte for byte; a test holds it, as
French's D4 did.

### D2 — The soaks, by hand

`tool/soak_engine.mjs` (M25's manual tool) over the 100 Italian selections, `--pair it-en` and
`--pair it-fr`, each `--isolate` (an engine per selection) and in one instance (one engine for all):
translated, trapped and timed out counted, recorded in `TRANSLATION.md` beside the other routes', as
change 50 soaked fr-en and fr-es. it-en alone tells an it-en trap from an en-fr one. The it-en runs
close change 2's D4: its « the it-en soak is row 19's » sentences in `TRANSLATION.md` and
`tool/marks/README.md`, and the tool's « no selection » for it-en, are replaced by the runs' figures.
The soak records each run's models; the figure a French-speaking reader is shown for the setting
(en-fr, es-en and it-en summed) is change 2's, displayed from change 20.

### D3 — it-fr measured and judged in French

`measure.mjs` over the 100 Italian selections through it-en then en-fr, the sentence and the fragment
each asked as the extension asks them, a trap answered as change 26 answers it; `results-it-fr.jsonl`
committed. One judge marks each shown mark correct or not against the French rendering of the
selected Italian word, by change 26's written criteria, with Italian examples taken from the corpus
added to them before the run:
- **a subject the route adds**: Italian drops its subject and its passato remoto is one word, so
  English adds a pronoun and French a compound past — step 37's « A Pyongyang vidi alcune persone »
  (also 65 « conobbe », 69 « rilasciò », 81 « accusò »): a mark on « ai vu » or « vu » is correct, on
  « J' » or « ai » alone it is not;
- **a word after a fused preposition**: step 6's « Nel corso della storia » (also 22, 82, 91): the
  mark renders `corso`, never the article the preposition carried;
- **a word after an elided piece**: step 86's « l'unico nel Governo », the only one of its kind (French
  had 8): the mark renders `unico` (« le seul »), never the article alone;
- **gender or number English lost**: a French rendering whose gender or number the route lost still
  counts as the word's rendering — step 79 « nemiche », step 71 « stereotipata ».

`judged-it-fr.tsv` committed. The gloss experiment's columns read `tables/it-fr/gloss.tsv` if change
16 has committed it by the run (`measure.mjs` reads no table that holds no gloss), and stay empty
otherwise; the tier reads the engine's column alone.

### D4 — The list

On the first tier (≥ 90 % of shown marks correct, ≤ 25 % withheld), `MARKED_PAIRS` gains `it-fr`,
inert until change 20 lists the pair in `packs.json`. Short of it, it-fr stays out and is translated
without a mark; under 75 % correct or over 30 % withheld, the figures go to the owner under M15, who
settles at change 20 whether it-fr's translation is offered unmarked or not at all. This change
decides nothing for the reader.

Either way `judged-it-fr.tsv` is committed, so `test/translate-marks.spec.ts`'s structural test —
today « the set of judged pairs equals the list » — reads the list as the judged pairs on the tier:
every listed pair judged, every judged pair listed exactly when it is on the tier, and the per-pair
checks of results and judgments run over every judged pair, listed or not. On the tier, the exact
lists of that test and of `test/translate-relay.spec.ts` gain it-fr.

### D5 — The documents

- `tool/marks/README.md`: the file table (`pud.mjs`'s four treebanks, `corpus.json`'s 400 selections
  in four languages), the run commands with it-en and it-fr, the Judging paragraph (judged per native
  language; it-fr across two alignments), the figures of task 2.3, both pins if they differ (D1).
- `pud.mjs`'s and `select_corpus.mjs`'s header comments and `select_corpus.mjs`'s closing count
  (« English, Spanish, French and Italian selections »); `measure_marks.mjs`'s and
  `soak_engine.mjs`'s usage lists name it-en and it-fr.
- `src/translate/markup.ts`'s comment above `MARKED_PAIRS`: « Every listed pair ships » becomes true
  of its time — it-fr, if listed, is inert until change 20 (`enable-lingua-italian`) — and its
  figures are added.
- `TRANSLATION.md`: the soaks (D2) and it-fr's figures.

## Risks / Trade-offs

- [Two models, more error in the marks] → measured, as fr-es's was (90 / 91 through English).
- [One judge] → change 26's written criteria, Italian examples fixed before the run; every judgment
  committed and re-checkable.
- [The rule never selects a word inside a multiword token] → it-fr's figures say nothing of a
  selection on an enclitic verb (`considerandolo`, `andarsene`, `vederlo`: 61 in UD Italian-PUD) or an
  articulated preposition, as Spanish's say nothing of `hacerlo`; the dogfood of change 20 reads them.
- [PUD's commit differs from change 9's pin] → the corpus keeps its own (D1); the committed
  selections are held by a test.

## Migration Plan

Nothing deploys: the list is inert until change 20. Rollback is a revert.

## Effort

1.25–2 ideal days, against the programme's 1–2: the corpus and its test 0.25; the two soaks
0.25–0.5; the run and the judging 0.5–1; the documents and the list 0.25.
