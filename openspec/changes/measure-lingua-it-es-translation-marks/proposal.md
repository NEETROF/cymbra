# measure-lingua-it-es-translation-marks — it-es's marks measured on the Italian selections, through English, judged in Spanish

## Why

Change 29 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: Spanish speakers
studying Italian. « Extended translation » marks in bold the words that render the reader's selection.
It does so only for a pair whose marks reach the first tier on the committed corpus: at least 90 % of
shown marks correct, at most 25 % of selections without a mark. it-es goes through two models, it-en then
en-es (change 2). Its marks are not it-en's or it-fr's: a pivot loses marks and can carry or add errors,
as es-fr's (89 of 90, 10 % withheld) and fr-es's (90 of 91, 9 % withheld) showed.

## What Changes

- **The it-es chain soaked** (design D1): the two models fed the Italian selections in sequence, isolated
  and in one instance, recorded for change 30 to read.
- **it-es measured** (D2): `measure_marks.mjs` over change 19's 100 Italian selections through it-en then
  en-es, run twice to check the results are identical. `results-it-es.jsonl` is committed. Spanish's
  function words already exist.
- **Judged in Spanish by one judge**, as every Spanish-judged precedent was (D3):
  - the Spanish criteria are committed before the run;
  - each line is checked against it-fr's and it-en's verdicts on the same selection, and against
    en-es's Spanish for the English sentence, as a reference on idiom;
  - a verdict departing from both precedents gives its reason;
  - doubtful lines are named, and committed wrong where they decide the tier;
  - the owner, who reads Spanish, may re-judge any line.
- **The list** (D4):
  - on the first tier, `MARKED_PAIRS` gains it-es, inert until change 30 lists the pair;
  - short of it, it-es is translated without a mark;
  - under 75 % correct or over 30 % withheld, the owner decides under M15 at change 30.
- **Tests and documents** (D5).
- **Nothing a reader sees moves** before change 30.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED *A pair's marks are measured before they are shown*, as change 38
  leaves it, every scenario kept, one added: *An Italian selection, through English, judged in Spanish*.

## Impact

- **Products.** Lingua only, all in `apps/lingua-extension`:
  - `tool/marks/{results-it-es.jsonl, judged-it-es.tsv, README.md}`;
  - `tool/measure_marks.mjs` and `tool/soak_engine.mjs` (their usage comments);
  - `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier);
  - `TRANSLATION.md`;
  - `test/translate-marks.spec.ts`, and `test/translate-relay.spec.ts` if on the tier.

  `docs/lingua/italian-programme.md` takes row 29's figures.
- **Prerequisites**: changes 2 (the route), 19 (the Italian selections, UD Italian-PUD's pin, the
  structural test, it-fr's verdicts) and 24 (it-en's results and verdicts, which the judgment reads)
  implemented. Change 27's tables, if committed, fill the gloss experiment.
- **Effort, against 1–2 ideal days**: 0.95–1.35 (design, *Effort*).
