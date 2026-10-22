# measure-lingua-fr-it-translation-marks — fr-it's marks measured on the French selections, through English, judged in Italian

## Why

Change 49 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4: Italian speakers
studying French.

« Extended translation » marks in bold the words that render the reader's selection. It does so only for
a pair whose marks reach the first tier on the committed corpus: at least 90 % of shown marks correct, at
most 25 % of selections without a mark.

fr-it goes through two models, fr-en then en-it (change 2). Its marks are not fr-en's or fr-es's: a
pivot loses marks, and can carry or add errors, as es-fr's (89 of 90, 10 % withheld) and fr-es's (90 of
91, 9 % withheld) showed. The judged language is Italian, which nobody on the team reads (I13).

## What Changes

- **The fr-it chain soaked** (design D1): the two models fed the French selections in sequence, in one
  instance beside change 2's isolated soak, recorded for change 50 to read.
- **fr-it measured** (D2): `measure_marks.mjs` over the 100 French selections already committed (UD
  French-PUD, matrix change 50), through fr-en then en-it, run twice to check the results are identical.
  `results-fr-it.jsonl` is committed. Italian's function words come with change 38.
- **Judged in Italian by two agent judges**, as change 38 judges en-it (D3):
  - the Italian criteria are committed before the run;
  - each line is checked against fr-en's and fr-es's committed verdicts on the same selection, and
    against UD Italian-PUD's human Italian;
  - Judge B, change 33's Italian reviewer agent, works blind; a line the judges disagree on is committed
    wrong;
  - doubtful lines are named, and the tier must hold with them counted wrong.
- **The list** (D4):
  - on the first tier, `MARKED_PAIRS` gains fr-it, inert until change 50 lists the pair;
  - short of it, fr-it is translated without a mark;
  - under 75 % correct or over 30 % withheld, the owner decides under M15 at change 50.
- **Tests and documents** (D5).
- **Nothing a reader sees moves** before change 50.

A trial outside the repository, on the same 100 selections, found 0 traps, 7 withheld and 90 of 93 marks
correct by one agent's reading (85 of 93 with the doubtful lines counted wrong): on the tier either way.
It is indicative only; the committed run decides (*Context*).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED *A pair's marks are measured before they are shown*, as change 29
  leaves it, every scenario kept, one added: *A French selection, through English, judged in Italian*.

## Impact

- **Products.** Lingua only, all in `apps/lingua-extension`:
  - `tool/marks/{results-fr-it.jsonl, judged-fr-it.tsv, README.md}`;
  - `tool/measure_marks.mjs` and `tool/soak_engine.mjs` (their usage comments);
  - `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier);
  - `TRANSLATION.md`;
  - `test/translate-marks.spec.ts`, and `test/translate-relay.spec.ts` if on the tier.

  `docs/lingua/italian-programme.md` takes row 49's figures.
- **Prerequisites**, implemented: changes 2 (the route and en-it's pin), 19 (the structural test, UD
  Italian-PUD's pin) and 38 (Italian's stop words and the two-judge protocol, with change 33's reviewer
  agent). Changes 47 and 48 only fill the gloss columns: if their tables are not committed, the columns
  stay empty.
- **Order**: before change 50. Archived after changes 29, 38, 24, 19 and 2.
- **Effort, against 1–2 ideal days**: 1.15–1.75 (design, *Effort*).
