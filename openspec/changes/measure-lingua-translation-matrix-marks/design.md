# Design — measure-lingua-translation-matrix-marks

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `tool/measure_marks.mjs` | `--pair`, optional `--models <dir>`; throws without `catalogue.routes[pair]`; per selection: the sentence translated with `markSelection`, read back with `readMarked`, reconciled against the fragment translated alone (`reconcileMarks`); a French `STOP` set; the result key `french`; a gloss experiment reading `tables/<studied>/forms.tsv` and `tables/<pair>/gloss.tsv` unconditionally; no trap handling (a `RuntimeError` exits 1) |
| `tool/marks/` | `corpus.json` (100 en and 100 es selections from PUD, the rule in `select_corpus.mjs`), `pud.mjs` (pinned by commit and sha256), `engine.mjs` (loads the pinned engine and the models, every file checked against the catalogue), `tier.mjs` (`firstTier`: correct/shown ≥ 0.9 and withheld/total ≤ 0.25; `judgedCounts` reads the `engine` column), `results-{en-fr,es-fr}.jsonl`, `judged-{en-fr,es-fr}.tsv` (`k id word upos engine_marks engine engine_reason gloss_marks gloss gloss_reason`), `README.md` (criteria: correct = the marks cover the French rendering of the selected word; figures en-fr 96/97, 3 % withheld; es-fr 89/90, 10 %) |
| `src/translate/markup.ts` | `MARKED_PAIRS = ["en-fr", "es-fr"]`; `relay.ts` sends a pair outside it untagged |
| `test/translate-marks.spec.ts` | the harness names `--pair` and `results-${pair}.jsonl`; each marked pair's results and judged totals equal its corpus; `MARKED_PAIRS` equals en-fr and es-fr, each first tier; es-en and en-es have no judged file |
| Change 9 (merged #765) | the channel respawns after a trap and asks once more; a second trap answers `TRAPPED_TWICE` (unavailable) |
| Change 25 | routes `es-en` (es-en 2.0 alone) and `en-es` (en-es 2.1 alone); the en-es soak recorded in `TRANSLATION.md` |
| Spanish programme D2 | ≥ 90 % correct and ≤ 25 % withheld: marked; 75–90 %: translated unmarked; under 75 % or over 30 % withheld: withdrawn |

## Goals / Non-Goals

**Goals:**
- es-en's and en-es's marks measured as en-fr's and es-fr's were, every judgment committed.
- The harness generalised to any native language, en-fr's and es-fr's measurements unchanged.
- A trap counted, not fatal.

**Non-Goals:**
- Shipping the pairs or offering translation to their readers (34, 35, M15).
- Re-measuring en-fr or es-fr: their figures stand; only their results' key is renamed.
- A second judge: one judge, as for the shipped pairs; the owner may re-judge any line in the pull
  request.

## Decisions

### D1 — The native language's stop words and a neutral key

The harness reads the pair's native language from the route's last model and picks its stop
words: the French set as today, an English set and a Spanish set of function words (articles,
prepositions, conjunctions, pronouns, auxiliaries), each a constant in `tool/marks/stop-words.mjs`
with a test that it holds no content word of its own language from the committed tables. The translated sentence's
key becomes `translation`; en-fr's and es-fr's committed results are rewritten with the key
renamed, every value byte for byte — checked once in the pull request: `git diff --word-diff` of
each file shows only `french` → `translation`, the key in its place.

### D2 — A trap answered as the extension answers it, per request

The channel of change 9 retries per request, and a selection costs two: the sentence with its
selection tagged, then the fragment alone (`relay.ts`). Each request that traps (`isTrap`) is asked
once more on a fresh engine — `engine()` called again: `loadBergamot` keeps its state per call, so
a new instance is clean. A sentence that traps twice is recorded `trapped: true`, with no
translation and no mark; a fragment that traps twice leaves the sentence's tagged marks
unreconciled, as `relay.ts` shows them. A trapped selection's row in `judged-<pair>.tsv` reads
`withheld` in the `engine` column and `trapped twice` in `engine_reason`, so `judgedCounts` counts
it as withheld, unchanged. The per-selection loop moves into `tool/marks/measure.mjs`, a function
that takes the engine factory, tested with a fake engine that traps; `measure_marks.mjs` keeps
`--pair`, `catalogue.routes[pair]` and `results-${pair}.jsonl`, which `test/translate-marks.spec.ts`
reads. The README lists the trapped ids, and `TRANSLATION.md`, where change 25 recorded the soak,
points to them.

### D3 — The gloss experiment fills in when the pair's table exists

The experiment's columns (`gloss_marks`, `gloss`, `gloss_reason`) are filled when
`tables/<pair>/gloss.tsv` exists and left empty otherwise; a later run, after the pair's tables
are committed, may fill them without touching the engine's columns. The tier reads the engine's
column only, as today.

### D4 — Judging, fixed before the run

The README's criteria are kept word for word — the article, preposition or auxiliary may be
included, marks may be split, an expression's rendering counts, only a function word or only part
of a compound is wrong, and a mark on a word the translation omitted is wrong — « the French
rendering » becoming « the rendering in the pair's native language » (English for es-en, Spanish
for en-es), with an English and a Spanish example beside the French ones; they are committed
before any result is read. One judge reads both pairs, as the shipped pairs were judged; a
doubtful line is marked as such in `engine_reason`. The owner may re-judge any line in the pull
request; the figures are recomputed from the committed judgments by `judgedCounts`.

### D5 — The list

A pair that reaches the first tier joins `MARKED_PAIRS`; it is inert until the pair ships (no
reader has it), and `test/translate-relay.spec.ts`'s es-en and en-es cases follow the list. A pair
short of the first tier but at or above 75 % correct with at most 30 % withheld stays out and
translates without a mark; under 75 % correct or over 30 % withheld, the figures are reported to
the owner, who settles under M15 whether changes 34 or 35 offer the pair's translation unmarked or
not at all — this change decides nothing for it.

## Risks / Trade-offs

- **One judge** → the criteria fixed before the run, every judgment committed and re-judgeable.
- **en-es traps on a selection of the corpus** → counted as withheld (D2); the rate shows it.
- **The French results rewritten** → only the key moves; a test compares every value.

## Migration Plan

No release: a measurement and an inert list entry.
