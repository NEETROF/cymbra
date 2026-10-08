# The marks measurement

How often the translated sentence marks the right words for the reader's selection, per pair: en-fr
and es-fr (release-lingua-spanish-translation). es-fr goes through English, so its mark crosses two
alignments. A measurement is a pair's — es-fr's says nothing of es-en, whose one model is measured
when it ships (generalise-lingua-translation-routes-by-pair D6). Decision D2 of the Spanish
programme fixed what the result decides before anything was measured:

- **≥ 90 % correct and ≤ 25 % withheld:** marked;
- **75–90 %:** unmarked;
- **below that:** withdrawn.

The corpus is per **studied language** — a selection is of the text it was made in, and es-en is
measured on the same Spanish selections as es-fr — while the results and the judgments are filed
per **pair**:

| File                   | What it holds                                                                                                                                                             |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `select_corpus.mjs`    | The selection rule: every tenth PUD sentence, the same in both languages, one word each (noun, verb, noun, adjective in turn)                                             |
| `pud.mjs`              | PUD English and Spanish, fetched at pinned commits and checked by sha256 (CC BY-SA, never committed)                                                                      |
| `corpus.json`          | The 200 selections, 100 per studied language (`lang`): sentence id, token, word, offsets                                                                                  |
| `engine.mjs`           | The pinned engine in Node with a route's models built as `engine-worker.ts` builds them: one markup string in, its translation out — shared by the harness and the soak   |
| `../measure_marks.mjs` | The harness: the pinned engine and the catalogue's models, each selection of a pair's studied language marked exactly as `relay.ts` marks it, through that pair's route   |
| `../soak_engine.mjs`   | The soak (harden-lingua-translation-engine D4): a pair's route over the same corpus, reporting what trapped, the time per sentence and the memory high-water mark         |
| `results-<pair>.jsonl` | Per selection: the translated sentence with its marks bracketed, the fragment's own translation, and the experiment's mark — `results-en-fr.jsonl`, `results-es-fr.jsonl` |
| `judged-<pair>.tsv`    | Every mark judged correct, wrong or withheld, with a reason for each wrong one — `judged-en-fr.tsv`, `judged-es-fr.tsv`                                                   |
| `tier.mjs`             | D2's first tier and the count of a judged file, as read above; `test/translate-marks.spec.ts` holds `MARKED_PAIRS` to it                                                  |

## Running it again

One pair per run, named as the catalogue routes it:

```bash
node tool/assemble_model_site.mjs /tmp/models                                                  # or read models.cymbra.app: omit --models
node --experimental-strip-types tool/measure_marks.mjs --pair en-fr --models /tmp/models      # ~15 s, writes results-en-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-fr --models /tmp/models      # writes results-es-fr.jsonl
node tool/marks/select_corpus.mjs                                                             # only to rebuild corpus.json; deterministic
```

## Soaking a route

Beside the measurement, by hand and never in CI (the programme's M25): the real engine through a
pair's route over the same corpus, to find the inputs that trap it — measured through the en-es
model in the study (2026-10-06), a trap poisons the instance for every model built after it — and
what a run costs in time and memory. It is how en-es is tried before it ships (change 35), now
that change 25 pins the en-es route in the catalogue — its run is recorded in `TRANSLATION.md`; a
pair the catalogue does not route is refused.

```bash
node --experimental-strip-types tool/soak_engine.mjs --pair en-fr --models /tmp/models            # ~15 s
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models            # two models, through English
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models --limit 10 # the first ten selections
node --experimental-strip-types tool/soak_engine.mjs --pair en-es --models /tmp/models --isolate  # each sentence in a child process, ~45 s
```

Each selection goes through the engine as `relay.ts` sends it: the sentence with the selection
tagged — timed, since it is what the card waits for — then the fragment alone. The report says how
many were translated, which trapped (by corpus id, never by text), the median, mean and maximum
time per sentence, and the process's resident high-water mark (`process.resourceUsage().maxRSS`).
In Node a trap throws a `WebAssembly.RuntimeError` the tool catches, but the instance is poisoned
from then on, so without `--isolate` the run **stops at the first trap and says so**; with it, each
sentence runs in a child process of its own and the run goes on to the end — slower, since each
child loads the engine and the route again. A child that ends without its report (a trap while the
route is built, a crash, a signal) counts as trapped for its id; one past 120 s is counted apart,
as timed out. The exit status is 1 when anything trapped or timed out.

What the memory figure is, and is not. It is Node's RSS — the parent's, or the highest child's
under `--isolate`. The ≈ 322 MiB of es-fr's two models (≈ 195 MiB for one) was measured in the
browser's worker, and Node's figure is not like for like: read it for a run's order of magnitude
and for growth across the corpus, not against the worker's. The soak does not check the two-model
bound of `engine-worker.ts` either: a run loads one route and deletes nothing — and a deletion
would not show in RSS anyway, since a wasm instance's linear memory never shrinks; the bound caps
growth, with the freed blocks reused by the next model built. The catalogue lists three models
(en-fr, es-en, en-es), but today's shipped pairs need two at most, so no route makes a third and the
eviction never runs in production: it is for the matrix's readers, once changes 34 and 35 ship es-en
and en-es.

## Judging

The criteria were written before the run (design D3).

- **Correct:**
  - the marks cover the French rendering of the selected word in that sentence;
  - they may include the article, preposition or auxiliary it carries, and may be split;
  - an expression's rendering counts, as « sin embargo » → « Cependant ».
- **Wrong:** the marks cover another word, only a function word, or only part of a compound.
- **Withheld:** no mark.

The rates read D2 as the study did:

- correct is the share of the shown marks;
- withheld is the share of all 100 selections.

## Results (2026-10-05)

Engine pinned by `engine-pin.json`; models `en-fr` and `es-en` base-memory 2.0. The pairs listed in
`MARKED_PAIRS` (`src/translate/markup.ts`) are the ones on the first tier here, and
`test/translate-marks.spec.ts` holds the list to the judged files.

|                                    | Correct (of shown marks) | Withheld | D2                                   |
| ---------------------------------- | ------------------------ | -------- | ------------------------------------ |
| **es-fr, engine (pivot es→en→fr)** | **89 / 90 — 98.9 %**     | **10 %** | **First tier: marked**               |
| en-fr, engine                      | 96 / 97 — 99.0 %         | 3 %      | (reference; the study found 83 / 87) |
| es-fr, gloss-located (experiment)  | 76 / 78 — 97.4 %         | 22 %     | —                                    |
| en-fr, gloss-located (experiment)  | 76 / 79 — 96.2 %         | 21 %     | —                                    |

What these numbers say:

- **es-fr is on D2's first tier.** One wrong mark in 90 (« dos facciones enemigas » marked « deux »
  instead of « ennemies »).
- **It holds under a stricter reading:** if the five expression marks counted as wrong, es-fr would
  still score 84 / 90, 93 %.
- **Sampling error:** about ±6 % at 100 selections.

**The experiment** marks the first word of the sentence that matches the pack's French gloss of the
selected word.

- It is about as precise as the engine.
- It withholds about one selection in five: a translation often picks a synonym the gloss does not
  list.
- Where the engine withholds, it would have found 7 of Spanish's 10 missing marks, bringing Spanish
  to 3 % withheld.

A later change may try it as a fallback; nothing here ships it.
