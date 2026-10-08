# measure-lingua-translation-matrix-marks — es-en's and en-es's marks, measured before they are shown

## Why

Change 26 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, under M15's recommendation (open: marks per pair, under the Spanish programme's D2). When « Traduction
étendue » translates a sentence, the extension marks in bold the words that render the reader's
selection — only for a pair whose marks were measured on a committed corpus and reached the first
tier: at least 90 % of shown marks correct, at most 25 % of selections without a mark. en-fr (96/97)
and es-fr (89/90) were measured that way; `MARKED_PAIRS` lists them, and a pair outside it is
translated without a mark.

The two new audiences' pairs are not measured. The study's figures — es-en 94/96, en-es 96/96, one
judge — are indicative "until each pair's committed measurement". Change 25 gives both pairs a
route; this change runs the harness through them, judges the marks, commits every judgment, and
lists a pair that reaches the first tier. Nothing is shown to anyone yet: the pairs ship with
changes 34 and 35.

The harness was written for French natives: its stop words are French, its result key is
`french`, and its judging rule speaks of "the French rendering". It also has no answer for a trap,
and en-es is the model measured to trap.

## What Changes

- **The harness speaks the pair's native language**: the stop words are the native language's (fr,
  en, es), the translated sentence's key is `translation` (en-fr's and es-fr's committed results
  rewritten with the key renamed, their values byte for byte), the judging rule says "the
  rendering, in the native language, of the selected word".
- **A trap is answered as the extension answers it** (change 9, per request): each of a selection's
  two requests — the sentence and the fragment alone — that traps is asked once more on a fresh
  engine; a sentence that traps twice is recorded `trapped: true` and counted as withheld (the reader
  would get no translation); a fragment that traps twice leaves the sentence's own marks unreconciled,
  as the extension shows them.
- **The gloss experiment is optional**: its columns are filled when `tables/<pair>/gloss.tsv`
  exists (es-en after change 21, en-es after change 22), left empty otherwise; the tier reads the
  engine's column only.
- **es-en and en-es measured**: `results-es-en.jsonl`, `results-en-es.jsonl` (100 selections each:
  the Spanish selections es-fr used, the English ones en-fr used), `judged-es-en.tsv`,
  `judged-en-es.tsv`, one judge, the criteria fixed before the run; figures in the harness's
  README and the programme's table.
- **`MARKED_PAIRS`** gains each pair that reaches the first tier; a pair short of it stays out and
  translates without a mark. Under 75 % correct or over 30 % withheld, the figures go to the owner,
  who settles under M15 whether changes 34 or 35 offer the pair's translation unmarked or not at all —
  this change decides nothing for it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED *A pair's marks are measured before they are shown* — the
  measurement speaks the pair's native language, a selection the engine traps on twice counts as
  withheld, and the scenario *A pair measured in another native language* now names the pairs
  measured; every scenario kept, two added. Held by no open change.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`tool/measure_marks.mjs`,
  `tool/marks/`, `src/translate/markup.ts` `MARKED_PAIRS`, `test/translate-marks.spec.ts`,
  `test/translate-relay.spec.ts`, `TRANSLATION.md`). Nothing a reader of today sees moves: en-fr's and es-fr's marks are as before.
  ID, Music, Live, the back office and the site are untouched.
- **Order.** After change 25 (the routes it measures; archived after it). Independent of changes
  21 and 22: the gloss experiment fills in when their tables exist.
- **Not here.** Shipping the pairs (34, 35); the copy that says a translation is a machine's in
  English or Spanish (14).
