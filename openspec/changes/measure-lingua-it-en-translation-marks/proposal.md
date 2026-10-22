# measure-lingua-it-en-translation-marks — it-en's marks measured on the Italian selections

## Why

Change 24 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. When
« Extended translation » translates a sentence, the extension marks in bold the words that render the
reader's selection — only for a pair whose marks were measured on the committed corpus and reached
the first tier: at least 90 % of shown marks correct, at most 25 % of selections without a mark
(matrix change 26). Change 19 put 100 Italian selections in the corpus (UD Italian-PUD), soaked the
it-en model over them and measured it-fr, which goes through it-en then en-fr. it-en, the it-en model
alone, has no measurement of its own: two models' alignment chained says nothing of one, as
es-fr's 89 / 90 said nothing of es-en's 94 / 96.

## What Changes

- **it-en measured** (design D1, D2): `measure.mjs` over change 19's Italian selections through the
  it-en model; `results-it-en.jsonl` and `judged-it-en.tsv` committed, judged in English against the
  English rendering by change 26's criteria, Italian examples in English added before the run; one
  judge, every judgment committed. The gloss experiment's columns read `tables/it-en/gloss.tsv` if
  change 22 has committed it by the run.
- **No corpus and no soak here**: change 19's selections and its it-en soak serve as they are.
- **The list** (D3): it-en on the first tier joins `MARKED_PAIRS`, inert until change 25 lists the
  pair; short of it, it stays out and is translated without a mark, and under 75 % correct or over
  30 % withheld the figures go to the owner under M15, who settles at change 25 whether it-en's
  translation is offered unmarked or not at all. The structural test already reads the list as the
  judged pairs on the tier (change 19).
- **Tests and documents** (D4): a test of the new scenario, whichever the outcome (and a relay case
  on the tier); `tool/marks/README.md`, `TRANSLATION.md` and the comment above `MARKED_PAIRS` give
  it-en's figures.
- **Nothing a reader sees moves**: no Italian pair is listed for English speakers before change 25;
  every other pair's results, judgments and marks stay byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED — *A pair's marks are measured before they are shown*, as change 19
  leaves it (open, so this change archives after it, and after change 2, whose route it measures):
  every scenario kept, one added — an Italian selection through the it-en model, judged in English.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site.

- `apps/lingua-extension/tool/marks/{results-it-en.jsonl, judged-it-en.tsv, README.md}`,
  `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier), `TRANSLATION.md`;
  `test/translate-marks.spec.ts` (the new scenario's test; the exact list if on the tier) and
  `test/translate-relay.spec.ts` (a case and the exact list if on the tier);
  `docs/lingua/italian-programme.md` (row 24's figures).
- **Precondition**: changes 2 and 19 implemented — it-en in the catalogue, the Italian selections
  committed, the it-en soak recorded — and the models assembled with `tool/assemble_model_site.mjs`
  or served after the owner's redeploy.
- Untouched: the corpus, the core, the packs, the models and routes, every other pair's results.
- **Effort, against 1–2 ideal days**: 0.75–1.25 (design, *Effort*).
