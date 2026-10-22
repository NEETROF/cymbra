# measure-lingua-it-fr-translation-marks — Italian selections in the marks corpus, it-fr measured

## Why

Change 19 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. When
« Traduction étendue » translates a sentence, the extension marks in bold the words that render the
reader's selection — only for a pair whose marks were measured on the committed corpus and reached
the first tier: at least 90 % of shown marks correct, at most 25 % of selections without a mark
(matrix change 26; French's change 50 measured fr-en 95 / 96 and fr-es 90 / 91). `MARKED_PAIRS`
lists the six measured pairs; a pair outside it is translated without a mark.

it-fr has a route since change 2 — it-en 2.0 then en-fr, through English as es-fr and fr-es — and no
measurement: the corpus (`tool/marks/corpus.json`) holds English, Spanish and French selections
taken from the same 100 sentences of the parallel treebank UD PUD, and no Italian one.

Measured before proposing: UD Italian-PUD (1,000 sentences, the parallel set; commit
`71d932aa096368331ba2fd9bc906ec8dd853da8e`, `it_pud-ud-test.conllu`, 1,697,113 B, sha256
`ad5d302b…`), read by the corpus's own rule over four languages at once, gives an Italian word on
**every one of the 100 steps** — no sentence skipped — and every English, Spanish and French
selection is byte for byte the committed one (`parte`, `insegnare`, `volta`, `società`, `esperti`,
`cresciuti`… the first Italian selections).

## What Changes

- **An Italian corpus for the marks** (design D1): `tool/marks/pud.mjs` pins UD Italian-PUD at the
  commit and sha256 change 9's `measure/it-ud.sh` pins — the one file, as French's pin is change
  43's —; `select_corpus.mjs`'s rule runs over English, Spanish, French and Italian at once;
  `corpus.json` gains 100 Italian selections, and no other moves (measured).
- **The soak, by hand** (M25's recommendation, D2): it-fr over the 100 Italian selections, isolated
  and in one instance — translated, trapped, timed out —, recorded in `TRANSLATION.md`.
- **it-fr measured** (D3): `results-it-fr.jsonl`, `judged-it-fr.tsv`, judged in French against the
  French rendering by the criteria of change 26, Italian examples added to the criteria before the
  run, one judge, every judgment committed. The gloss experiment fills in from it-fr's committed
  glosses (change 16), never from an empty `gloss.tsv`.
- **The list** (D4): it-fr on the first tier joins `MARKED_PAIRS`, inert until change 20 lists it-fr;
  short of it, it stays out and the figures go to the owner under M15, who settles at change 20
  whether it-fr's translation is offered unmarked or not at all.
- **The documents**: `TRANSLATION.md`, `tool/marks/README.md`, the harness's and the soak's usage
  comments name Italian and it-fr's route.
- **Nothing a reader sees moves**: no Italian pair is listed before change 20; the six measured pairs'
  results, judgments and marks stay byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: MODIFIED — *A pair's marks are measured before they are shown*, as French's
  change 50 leaves it (open, so this change archives after it, after change 26 which holds it first,
  after change 2 whose route it measures, and after change 16 whose glosses fill the experiment):
  every scenario kept, two added — an Italian selection through English judged in French, and Italian
  added to the corpus moving nothing.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension/tool/marks/{pud.mjs, select_corpus.mjs, corpus.json, results-it-fr.jsonl,
  judged-it-fr.tsv, README.md}`, `tool/measure_marks.mjs` where it names the pairs,
  `src/translate/markup.ts` (`MARKED_PAIRS`, if on the tier), `TRANSLATION.md`; the tests that hold
  the corpus and the list (`test/translate-measure.spec.ts`).
- Untouched: the core, the packs, the models and routes (change 2's), the six pairs' results.
- **Effort, against 1–2 ideal days**: 1–1.75 (design, *Effort*).
