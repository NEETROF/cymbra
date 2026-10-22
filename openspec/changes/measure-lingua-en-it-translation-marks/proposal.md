# measure-lingua-en-it-translation-marks — en-it's marks measured on the English selections, judged in Italian

## Why

Change 38 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3. When « Extended
translation » translates a sentence, the extension marks in bold the words that render the reader's
selection. It does so only for a pair whose marks were measured on the committed corpus and reached the
first tier: at least 90 % of shown marks correct, at most 25 % of selections without a mark (matrix
change 26).

en-it, English studied for Italian speakers, has a model of its own (en-it 2.1, change 2) that no one has
measured. The precedents for English selections are:
- en-fr: 96 of 97 shown marks correct, 3 % withheld;
- en-es: 96 of 96 correct, 4 % withheld.

## What Changes

- **Italian's function words** (design D1): `tool/marks/stop-words.mjs` gains its Italian set. The
  harness refuses any native language it lacks, so `--pair en-it` cannot run without it. The set also
  serves fr-it and es-it (changes 49, 53).
- **en-it measured** (D2): `measure_marks.mjs` over the 100 committed English selections through the
  en-it model, run twice to check the results are identical. `results-en-it.jsonl` is committed. No
  corpus work is needed.
- **Judged in Italian by two agent judges** (D3):
  - no one on the team reads Italian (I2), so the implementing agent and change 33's Italian reviewer
    agent judge, the second blind to the first;
  - the written Italian criteria and examples are committed before the run;
  - each line is checked against UD Italian-PUD's human Italian and the en-fr and en-es verdicts on the
    same step;
  - disagreements are recorded as doubtful, and the tier must hold with them counted wrong;
  - `judged-en-it.tsv` is committed.
- **The list** (D4):
  - on the first tier, `MARKED_PAIRS` gains en-it, inert until change 45 lists the pair;
  - short of it, en-it is translated without a mark;
  - under 75 % correct or over 30 % withheld, the figures go to the owner, who settles under M15 at
    change 45 whether en-it's translation is offered unmarked or not at all.
- **Tests and documents** (D5): the new scenario's test, the README, `TRANSLATION.md`, the comment above
  `MARKED_PAIRS`.
- **Nothing a reader sees moves**: no pair glossed in Italian is listed before change 45, and every other
  pair's results and judgments stay byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED *A pair's marks are measured before they are shown*, as change 24
  leaves it, every scenario kept, one added: *An English selection, judged in Italian*.

## Impact

- **Products.** Lingua only, all in `apps/lingua-extension`:
  - `tool/marks/{stop-words.mjs, stop-words.d.mts, results-en-it.jsonl, judged-en-it.tsv, README.md}`;
  - `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier);
  - `TRANSLATION.md`;
  - `test/translate-marks.spec.ts`, `test/translate-measure.spec.ts` (Italian's function words), and
    `test/translate-relay.spec.ts` if on the tier.

  `docs/lingua/italian-programme.md` takes row 38's figures.
- **Prerequisites**:
  - change 2 implemented, its soak of en-it included;
  - changes 9 and 16 (the Italian tables the function-word test reads);
  - change 19 (UD Italian-PUD's pin, and the structural test that lists only the judged pairs on the
    tier);
  - change 33 (the reviewer agent);
  - change 36's tables, optional, for the gloss experiment.
- **Effort, against 1–2 ideal days**: 1.35–1.85 (design, *Effort*).
