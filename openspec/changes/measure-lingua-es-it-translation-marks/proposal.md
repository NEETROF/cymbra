# measure-lingua-es-it-translation-marks — es-it's marks measured on the Spanish selections, through English, judged in Italian

## Why

Change 53 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4: Italian speakers
studying Spanish.

« Extended translation » marks in bold the words that render the reader's selection. It does so only for
a pair whose marks reach the first tier on the committed corpus: at least 90 % of shown marks correct, at
most 25 % of selections without a mark.

es-it goes through two models, es-en then en-it (change 2). Its marks are not es-en's or es-fr's: a pivot
loses marks, and can carry or add errors, as es-fr's (89 of 90, 10 % withheld) showed. The judged
language is Italian, which nobody on the team reads (I13). Change 49 measures fr-it the same way; this
change follows it, with no trial run beforehand, so that the criteria are drafted blind.

## What Changes

- **The es-it chain soaked** (design D1): the two models fed the Spanish selections in sequence, in one
  instance beside change 2's isolated soak, recorded for change 54 to read.
- **es-it measured** (D2): `measure_marks.mjs` over the 100 Spanish selections already committed (UD
  Spanish-PUD), through es-en then en-it, run twice to check the results are identical.
  `results-es-it.jsonl` is committed. Italian's function words come with change 38.
- **Judged in Italian by two agent judges**, as changes 38 and 49 judge (D3):
  - the Italian criteria are committed before the run, case classes anchored on es-en's and es-fr's
    committed verdicts and UD Italian-PUD's human Italian;
  - Judge B, change 33's Italian reviewer agent, works blind, without this change's directory or Judge
    A's verdicts, briefed with the PUD Italian; a line the judges disagree on is committed wrong;
  - doubtful lines are named, and the tier must hold with them counted wrong.
- **The list** (D4):
  - on the first tier, `MARKED_PAIRS` gains es-it, inert until change 54 lists the pair;
  - short of it, es-it is translated without a mark;
  - under 75 % correct or over 30 % withheld, the owner decides under M15 at change 54.
- **Tests and documents** (D5).
- **Nothing a reader sees moves** before change 54.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED *A pair's marks are measured before they are shown*, as change 49
  leaves it, every scenario kept, one added: *A Spanish selection, through English, judged in Italian*.

## Impact

- **Products.** Lingua only, all in `apps/lingua-extension`:
  - `tool/marks/{results-es-it.jsonl, judged-es-it.tsv, README.md}`;
  - `tool/measure_marks.mjs` and `tool/soak_engine.mjs` (their usage comments);
  - `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier);
  - `TRANSLATION.md`;
  - `test/translate-marks.spec.ts`, and `test/translate-relay.spec.ts` if on the tier.

  `docs/lingua/italian-programme.md` takes row 53's figures.
- **Prerequisites**, implemented: changes 2 (the route and en-it's pin), 19 (the structural test, UD
  Italian-PUD's pin), 38 (Italian's stop words, the two-judge protocol, change 33's reviewer agent) and
  49 (the latest holder). Changes 51 and 52 only fill the gloss columns.
- **Order**: before change 54. Archived after changes 49, 29, 38, 24, 19 and 2.
- **Effort, against 1–2 ideal days**: 1.1–1.65 (design, *Effort*).
