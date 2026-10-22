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
  answered as the extension answers it; `stop-words.mjs` holds the NATIVE language's function words
  for the gloss experiment (fr, en, es); `tier.mjs` the first tier (90 % correct, at most 25 %
  withheld). `src/translate/markup.ts`'s `MARKED_PAIRS` lists en-fr, es-fr, es-en, en-es, fr-en,
  fr-es.
- Change 2 pins it-en 2.0 and en-it 2.1 and routes it-fr through it-en then en-fr; change 9's
  `measure/it-ud.sh` pins UD Italian-PUD for the forms gates; change 16 commits it-fr's glosses.
- Measured: the four-language selection over UD Italian-PUD at commit `71d932aa…` (proposal).

## Goals / Non-Goals

**Goals:**
- it-fr's marks measured on the committed corpus through its route and judged in French; listed if on
  the first tier.
- The corpus gains Italian without moving any other language's selection or result.

**Non-Goals:**
- it-en and it-es (stage 2, their own measurements on the same Italian selections); fr-it, es-it and
  en-it (stage 3, an Italian stop-word set then, as the native language).
- Offering it-fr's translation (change 20), or deciding what happens below the tier (the owner under
  M15, at change 20).

## Decisions

### D1 — Italian in the corpus, by the same rule

`PUD.it` pins `UD_Italian-PUD` at the commit and sha256 `measure/it-ud.sh` pins (change 9; measured
here at `71d932aa096368331ba2fd9bc906ec8dd853da8e`, sha256
`ad5d302bdfd05194155d9ad43dc54c3cb915076d66b3eb6ef33eca9a11f1372a` — the implementation takes
change 9's pin if it differs, and re-runs the check below). `LANGUAGES` gains `it`, after `fr`, so
each step's Italian selection is written after the others'. A step takes a sentence only where every
language has a word: measured, Italian has one on every step, so no step moves to a later sentence and
the 300 committed selections stay byte for byte; a test holds it, as French's D4 did.

### D2 — The soak, by hand

`tool/soak_engine.mjs` (M25's manual tool) over the 100 Italian selections through it-fr's route,
isolated (an engine per selection) and in one instance (one engine for all): translated, trapped and
timed out counted, recorded in `TRANSLATION.md` beside the other routes'. Two models in one engine,
as fr-es and es-fr; the route's download — it-en 2.0, measured at change 2's task 1.1, and en-fr's —
recorded as the reader will read it.

### D3 — it-fr measured and judged in French

`measure.mjs` over the 100 Italian selections through it-en then en-fr, the sentence and the fragment
each asked as the extension asks them, a trap answered as change 26 answers it; `results-it-fr.jsonl`
committed. One judge marks each shown mark correct or not against the French rendering of the
selected Italian word, by change 26's written criteria, Italian examples added to them before the run
(an article fused into a preposition, `al mare` → « à la mer »; an elided piece, `l'uomo` →
« l'homme »; a clitic on a verb, `dirmelo` → « me le dire »); `judged-it-fr.tsv` committed. The
gloss experiment's columns fill in from `tables/it-fr/gloss.tsv` once change 16 commits it, empty
before; the tier reads the engine's column only.

### D4 — The list

On the first tier (≥ 90 % of shown marks correct, ≤ 25 % withheld), `MARKED_PAIRS` gains `it-fr`,
inert until change 20 lists the pair in `packs.json`. Short of it, it-fr stays out and is translated
without a mark; under 75 % correct or over 30 % withheld, the figures go to the owner under M15, who
settles at change 20 whether it-fr's translation is offered unmarked or not at all. This change
decides nothing for the reader.

## Risks / Trade-offs

- [Two models, more error in the marks] → measured, as fr-es's was (90 / 91 through English).
- [One judge] → change 26's written criteria, Italian examples fixed before the run; every judgment
  committed and re-checkable.
- [PUD's commit differs from change 9's pin] → the implementation takes change 9's and re-runs D1's
  check; the committed selections are held by a test.

## Migration Plan

Nothing deploys: the list is inert until change 20. Rollback is a revert.

## Effort

1–1.75 ideal days, against the programme's 1–2: the corpus and its test 0.25; the soak 0.25; the
run and the judging 0.5–1; the documents and the list 0.25.
