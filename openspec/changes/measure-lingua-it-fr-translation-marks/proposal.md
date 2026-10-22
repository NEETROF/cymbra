# measure-lingua-it-fr-translation-marks — Italian selections in the marks corpus, it-fr measured

## Why

Change 19 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. When
« Traduction étendue » translates a sentence, the extension marks in bold the words that render the
reader's selection — only for a pair whose marks were measured on the committed corpus and reached
the first tier: at least 90 % of shown marks correct, at most 25 % of selections without a mark
(matrix change 26; French's change 50 measured fr-en 95 / 96 and fr-es 90 / 91). `MARKED_PAIRS`
lists the six measured pairs; a pair outside it is translated without a mark.

Change 2 routes it-fr through it-en 2.0 then en-fr, through English as es-fr and fr-es are, and
hands this change the it-en model's soak: no Italian corpus exists to run it over. The corpus
(`tool/marks/corpus.json`) holds English, Spanish and French selections taken from the same 100
sentences of the parallel treebank UD PUD, and no Italian one; it-fr has no measurement.

Measured before proposing: UD Italian-PUD (1,000 sentences, the parallel set; commit
`71d932aa096368331ba2fd9bc906ec8dd853da8e`, `master`'s head on 2026-10-22, `it_pud-ud-test.conllu`,
1,697,113 B, sha256 `ad5d302b…`), read by the corpus's own rule over four languages at once, gives
an Italian word on **every one of the 100 steps** — no sentence skipped: 57 nouns, 24 verbs and 19
adjectives, 8 of them by the fallback (steps 3, 11, 23, 31, 41, 49, 75, 83). Against the committed
`corpus.json`, the diff is 1,006 lines added and none removed: `source.it` and each Italian
selection after its step's French one (`parte`, `insegnare`, `volta`, `società`, `esperti`,
`cresciuti`… the first). A three-language rebuild reproduces the committed file byte for byte.

## What Changes

- **An Italian corpus for the marks** (design D1): `tool/marks/pud.mjs` pins UD Italian-PUD at
  `71d932aa…` and its sha256; `select_corpus.mjs`'s rule runs over English, Spanish, French and
  Italian at once; `corpus.json` gains 100 Italian selections, the 300 others byte for byte.
- **The soaks, by hand** (M25, D2): `--pair it-en` and `--pair it-fr` over the 100 Italian
  selections, each isolated and in one instance — translated, trapped, timed out —, recorded in
  `TRANSLATION.md`. The it-en run closes change 2's D4, whose « row 19's » sentences it replaces.
- **it-fr measured** (D3): `results-it-fr.jsonl`, `judged-it-fr.tsv`, judged in French against the
  French rendering by the criteria of change 26, Italian examples added to them before the run, one
  judge, every judgment committed. The gloss experiment's columns read `tables/it-fr/gloss.tsv` if
  change 16 has committed it by the run, and stay empty otherwise; the tier reads the engine's
  column alone.
- **The list** (D4): it-fr on the first tier joins `MARKED_PAIRS`, inert until change 20 lists it-fr
  in `packs.json`. Short of the tier it stays out and is translated without a mark; under 75 %
  correct or over 30 % withheld, the figures go to the owner under M15, who settles at change 20
  whether it-fr's translation is offered unmarked or not at all. The tests that hold the list read
  it as the judged pairs on the tier, so either outcome passes them.
- **The documents** (D5): `TRANSLATION.md`, `tool/marks/README.md`, the usage and header comments of
  `pud.mjs`, `select_corpus.mjs`, `measure_marks.mjs` and `soak_engine.mjs`, and the comment above
  `MARKED_PAIRS` name Italian and it-fr's route.
- **Nothing a reader sees moves**: no Italian pair is listed before change 20; the six measured pairs'
  results, judgments and marks stay byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED — *A pair's marks are measured before they are shown*, as French's
  change 50 leaves it (open, so this change archives after it, after change 26 which holds it first,
  and after change 2, whose requirement routing the Italian pairs the new scenario relies on): every
  scenario kept, two added — an Italian selection through English judged in French, and Italian
  added to the corpus moving nothing.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension/tool/marks/{pud.mjs, select_corpus.mjs, corpus.json, results-it-fr.jsonl,
  judged-it-fr.tsv, README.md}`, `tool/measure_marks.mjs` and `tool/soak_engine.mjs` (their usage),
  `src/translate/markup.ts` (the comment, and `MARKED_PAIRS` if on the tier), `TRANSLATION.md`; the
  tests that hold the corpus and the list, `test/translate-marks.spec.ts` and
  `test/translate-relay.spec.ts`.
- **Precondition**: change 2 implemented — it-en in the catalogue and it-fr routed — and the models
  assembled with `tool/assemble_model_site.mjs` or served after the owner's redeploy;
  `measure_marks.mjs` and `soak_engine.mjs` refuse a pair the catalogue does not route.
- Untouched: the core, the packs, the models and routes (change 2's), the six pairs' results.
- **Effort, against 1–2 ideal days**: 1.25–2 (design, *Effort*).
