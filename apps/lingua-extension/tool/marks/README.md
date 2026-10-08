# The marks measurement

How often the translated sentence marks the right words for the reader's selection, per pair: en-fr
and es-fr (release-lingua-spanish-translation), es-en and en-es
(measure-lingua-translation-matrix-marks). es-fr goes through English, so its mark crosses two
alignments. A measurement is a pair's — es-fr's says nothing of es-en, measured on its own route and
judged in English (generalise-lingua-translation-routes-by-pair D6). Decision D2 of the Spanish
programme fixed what the result decides before anything was measured:

- **≥ 90 % correct and ≤ 25 % withheld:** marked;
- **75–90 %:** unmarked;
- **below that:** withdrawn.

The corpus is per **studied language** — a selection is of the text it was made in, and es-en is
measured on the same Spanish selections as es-fr, en-es on the same English ones as en-fr — while
the results and the judgments are filed per **pair**, and the judge reads the pair's **native
language**, the one its route translates into:

| File                   | What it holds                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `select_corpus.mjs`    | The selection rule: every tenth PUD sentence, the same in both languages, one word each (noun, verb, noun, adjective in turn)                                                                                                                                                                                                                                                                                                                     |
| `pud.mjs`              | PUD English and Spanish, fetched at pinned commits and checked by sha256 (CC BY-SA, never committed)                                                                                                                                                                                                                                                                                                                                              |
| `corpus.json`          | The 200 selections, 100 per studied language (`lang`): sentence id, token, word, offsets                                                                                                                                                                                                                                                                                                                                                          |
| `engine.mjs`           | The pinned engine in Node with a route's models built as `engine-worker.ts` builds them: one markup string in, its translation out — shared by the harness and the soak                                                                                                                                                                                                                                                                           |
| `measure.mjs`          | The loop: each selection marked exactly as `relay.ts` marks it — tagged sentence, fragment alone, reconciled — and the experiment's mark; a trap answered per request, as the extension answers it                                                                                                                                                                                                                                                |
| `stop-words.mjs`       | The experiment's stop words per native language (fr, en, es): the function words skipped when the gloss is looked for in the sentence; the harness picks the route's native language's                                                                                                                                                                                                                                                            |
| `../measure_marks.mjs` | The harness: the pinned engine and the catalogue's models, the corpus of a pair's studied language through `measure.mjs` over that pair's route; writes `results-<pair>.jsonl`                                                                                                                                                                                                                                                                    |
| `../soak_engine.mjs`   | The soak (harden-lingua-translation-engine D4): a pair's route over the same corpus, reporting what trapped, the time per sentence and the memory high-water mark                                                                                                                                                                                                                                                                                 |
| `results-<pair>.jsonl` | Per selection: the translated sentence (`translation`) with its marks bracketed (`shown`), the fragment's own translation (`alone`, null when it trapped twice), and the experiment's mark (`gloss`, null when the pair's `tables/<pair>/gloss.tsv` is not committed); a sentence the engine trapped on twice is `trapped: true` with no translation — `results-en-fr.jsonl`, `results-es-fr.jsonl`, `results-es-en.jsonl`, `results-en-es.jsonl` |
| `judged-<pair>.tsv`    | Every mark judged correct, wrong or withheld, with a reason for each wrong one — `judged-en-fr.tsv`, `judged-es-fr.tsv`, `judged-es-en.tsv`, `judged-en-es.tsv`                                                                                                                                                                                                                                                                                   |
| `tier.mjs`             | D2's first tier and the count of a judged file, as read above; `test/translate-marks.spec.ts` holds `MARKED_PAIRS` to it                                                                                                                                                                                                                                                                                                                          |

## Running it again

One pair per run, named as the catalogue routes it; the corpus is its studied language's, the stop
words its native language's:

```bash
node tool/assemble_model_site.mjs /tmp/models                                                  # or read models.cymbra.app: omit --models
node --experimental-strip-types tool/measure_marks.mjs --pair en-fr --models /tmp/models      # ~15 s, writes results-en-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-fr --models /tmp/models      # writes results-es-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-en --models /tmp/models      # the Spanish selections, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair en-es --models /tmp/models      # the English selections, judged in Spanish
node tool/marks/select_corpus.mjs                                                             # only to rebuild corpus.json; deterministic
```

A request that traps the engine (a `WebAssembly.RuntimeError`, `isTrap`) is asked once more on a
fresh engine, as the extension asks it after a respawn (harden-lingua-translation-engine D2): the
instance that trapped is poisoned and put down. A sentence that traps twice is recorded
`trapped: true` — the reader would get no translation — and judged `withheld`, `trapped twice`; a
fragment that traps twice leaves the sentence's own marks unreconciled, as the extension shows
them. The run says how many engines it built and which ids trapped.

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

The criteria were written before the run (release-lingua-spanish-translation D3, then
measure-lingua-translation-matrix-marks D4 for any native language), and a pair is judged in its
**native language** — French for en-fr and es-fr, English for es-en, Spanish for en-es.

- **Correct:**
  - the marks cover the rendering, in the pair's native language, of the selected word in that
    sentence;
  - they may include the article, preposition or auxiliary it carries, and may be split;
  - an expression's rendering counts, as « sin embargo » → « Cependant » (es-fr), « sin embargo »
    → « However » (es-en), « gave up » → « se rindió » (en-es).
- **Wrong:** the marks cover another word, only a function word, or only part of a compound; or a
  word the translation omitted, so that the marks cover something else.
- **Withheld:** no mark — a sentence the engine trapped on twice included (`withheld`,
  `trapped twice`).

One judge reads a pair, as the shipped pairs were judged; a doubtful line says so in
`engine_reason`, and the owner may re-judge any line in the pull request — the figures are
recomputed from the committed judgments (`tier.mjs`). The rates read D2 as the study did:

- correct is the share of the shown marks;
- withheld is the share of all 100 selections.

## Results

Engine pinned by `engine-pin.json`; models `en-fr` base-memory 2.0, `es-en` 2.0 and `en-es` 2.1
(`model-manifest.json`). The pairs listed in `MARKED_PAIRS` (`src/translate/markup.ts`) are the
ones on the first tier here, and `test/translate-marks.spec.ts` holds the list to the judged files.
en-fr and es-fr were measured on 2026-10-05, judged in French; es-en and en-es on 2026-10-08
(measure-lingua-translation-matrix-marks), judged in English and in Spanish, one judge each.

|                                      | Correct (of shown marks) | Withheld | D2                                               |
| ------------------------------------ | ------------------------ | -------- | ------------------------------------------------ |
| **es-fr, engine (pivot es→en→fr)**   | **89 / 90 — 98.9 %**     | **10 %** | **First tier: marked**                           |
| en-fr, engine                        | 96 / 97 — 99.0 %         | 3 %      | (reference; the study found 83 / 87)             |
| **es-en, engine, judged in English** | **94 / 96 — 97.9 %**     | **4 %**  | **First tier: marked once change 34 ships it**   |
| **en-es, engine, judged in Spanish** | **96 / 96 — 100 %**      | **4 %**  | **First tier: marked once change 35 ships it**   |
| es-fr, gloss-located (experiment)    | 76 / 78 — 97.4 %         | 22 %     | —                                                |
| en-fr, gloss-located (experiment)    | 76 / 79 — 96.2 %         | 21 %     | —                                                |
| es-en, gloss-located (experiment)    | 75 / 78 — 96.2 %         | 22 %     | —                                                |
| en-es, gloss-located (experiment)    | —                        | —        | left empty: no `tables/en-es/gloss.tsv` yet (22) |

What these numbers say:

- **es-fr is on D2's first tier.** One wrong mark in 90 (« dos facciones enemigas » marked « deux »
  instead of « ennemies »).
- **It holds under a stricter reading:** if the five expression marks counted as wrong, es-fr would
  still score 84 / 90, 93 %.
- **es-en and en-es are on the first tier too**, at the figures the study predicted (94 / 96 and
  96 / 96). es-en's two wrong marks: « declaró » marked « he » instead of « said » (#25), and
  « enemigas » marked « two » instead of « enemy » (#79) — the mark es-fr inherited through the
  pivot. Its four withheld are three expressions the translation reshaped (« dejar caer », « dar
  por hecho », « consiguió vencer ») and one hyphenated compound (« franco-monegasco »). en-es has
  no wrong mark; its four withheld are two of en-fr's own (« officer », « the only one » — the same
  English selections), « alarm clock » → « despertador » and « captained », unmarked. Three en-es
  lines (#18, #70, #77) follow a mistranslation (« edge » → « ventaja »): the mark sits on the
  rendering the reader sees, so they count correct and are flagged `doubtful:` in `engine_reason`.
- **No request trapped** on es-en or en-es: one engine built per run, no `trapped: true` line and no
  unreconciled fragment — the trapped ids this README would list are none (`TRANSLATION.md`, where
  change 25 recorded the en-es soak, says the same).
- **Sampling error:** about ±6 % at 100 selections.

**The experiment** marks the first word of the sentence that matches the pack's gloss of the
selected word, in the pair's native language, its stop words left out (`stop-words.mjs`).

- It is about as precise as the engine, in English as in French (es-en: 75 / 78).
- It withholds about one selection in five: a translation often picks a synonym the gloss does not
  list.
- Where the engine withholds, it would have found 7 of Spanish's 10 missing marks for es-fr, and
  2 of es-en's 4.

A later change may try it as a fallback; nothing here ships it.
