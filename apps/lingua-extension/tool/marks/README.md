# The marks measurement

How often the translated sentence marks the right words for the reader's selection, per pair: en-fr
and es-fr (release-lingua-spanish-translation), es-en and en-es
(measure-lingua-translation-matrix-marks), fr-en and fr-es (add-lingua-french-translation), it-fr
(measure-lingua-it-fr-translation-marks), it-en (measure-lingua-it-en-translation-marks). es-fr,
fr-es and it-fr go through English — es-en then en-fr, fr-en then en-es, it-en then en-fr — so their
marks cross two alignments. A measurement is a pair's — es-fr's says nothing of es-en, measured on its
own route and judged in English (generalise-lingua-translation-routes-by-pair D6), nor fr-es's of
fr-en's, nor it-fr's of it-en, the it-en model alone, measured on its own and judged in English.
Decision D2 of the Spanish programme fixed what the result decides before anything was measured:

- **≥ 90 % correct and ≤ 25 % withheld:** marked;
- **75–90 %:** unmarked;
- **below that:** withdrawn.

The corpus is per **studied language** — a selection is of the text it was made in, and es-en is
measured on the same Spanish selections as es-fr, en-es on the same English ones as en-fr, fr-es on
the same French ones as fr-en, it-en on the same Italian ones as it-fr — while the results and the
judgments are filed per **pair**, and the judge reads the pair's **native language**, the one its
route translates into. Every studied language's selections come from the same sentences by one rule
(add-lingua-french-translation D4): French, added, moved no English or Spanish selection, nor any
committed result; Italian, added after it (measure-lingua-it-fr-translation-marks D1), moved no
English, Spanish or French selection, nor any committed result.

| File                   | What it holds                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `select_corpus.mjs`    | The selection rule: every tenth PUD sentence, the same in every language, one word each (noun, verb, noun, adjective in turn)                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `pud.mjs`              | PUD English, Spanish, French and Italian (French at add-lingua-french-forms-tables' pin, Italian at `master`'s head on 2026-10-22, the corpus's own pin), fetched at pinned commits and checked by sha256 (CC BY-SA, never committed)                                                                                                                                                                                                                                                                                                                                                       |
| `corpus.json`          | The 400 selections, 100 per studied language (`lang`: en, es, fr, it), each step's four from one sentence: sentence id, token, word, offsets                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `engine.mjs`           | The pinned engine in Node with a route's models built as `engine-worker.ts` builds them: one markup string in, its translation out — shared by the harness and the soak                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `measure.mjs`          | The loop: each selection marked exactly as `relay.ts` marks it — tagged sentence, fragment alone, reconciled — and the experiment's mark; a trap answered per request, as the extension answers it                                                                                                                                                                                                                                                                                                                                                                                          |
| `stop-words.mjs`       | The experiment's stop words per native language (fr, en, es): the function words skipped when the gloss is looked for in the sentence; the harness picks the route's native language's                                                                                                                                                                                                                                                                                                                                                                                                      |
| `../measure_marks.mjs` | The harness: the pinned engine and the catalogue's models, the corpus of a pair's studied language through `measure.mjs` over that pair's route; writes `results-<pair>.jsonl`                                                                                                                                                                                                                                                                                                                                                                                                              |
| `../soak_engine.mjs`   | The soak (harden-lingua-translation-engine D4): a pair's route over the same corpus, reporting what trapped, the time per sentence and the memory high-water mark                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `results-<pair>.jsonl` | Per selection: the translated sentence (`translation`) with its marks bracketed (`shown`), the fragment's own translation (`alone`, null when it trapped twice, `""` for an empty fragment), and the experiment's mark (`gloss`, null when the pair's `tables/<pair>/gloss.tsv` is not committed or holds no gloss); a sentence the engine trapped on twice is `trapped: true` with no translation — `results-en-fr.jsonl`, `results-es-fr.jsonl`, `results-es-en.jsonl`, `results-en-es.jsonl`, `results-fr-en.jsonl`, `results-fr-es.jsonl`, `results-it-fr.jsonl`, `results-it-en.jsonl` |
| `judged-<pair>.tsv`    | Every mark judged correct, wrong or withheld, with a reason for each wrong one — `judged-en-fr.tsv`, `judged-es-fr.tsv`, `judged-es-en.tsv`, `judged-en-es.tsv`, `judged-fr-en.tsv`, `judged-fr-es.tsv`, `judged-it-fr.tsv`, `judged-it-en.tsv`                                                                                                                                                                                                                                                                                                                                             |
| `tier.mjs`             | D2's first tier and the count of a judged file, as read above; `test/translate-marks.spec.ts` holds `MARKED_PAIRS` to it                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

## Running it again

One pair per run, named as the catalogue routes it; the corpus is its studied language's, the stop
words its native language's. The catalogue routes twelve pairs — the six of the matrix, and Italian's
six since `add-lingua-italian-translation-models` (it-en, en-it, it-fr, it-es, fr-it, es-it) — and
the harness measures the eight below: the Italian selections are in the corpus since the Italian
programme's row 19 (`measure-lingua-it-fr-translation-marks`), which measured it-fr on them; row 24
(`measure-lingua-it-en-translation-marks`) measured it-en on the same selections, row 29 measures
it-es on them, and the pairs glossed in Italian need the Italian stop words, row 38's
(`measure-lingua-en-it-translation-marks`) — each Italian pair is measured by its own row (19, 24, 29,
38, 49, 53) before it ships:

```bash
node tool/assemble_model_site.mjs /tmp/models                                                  # or read models.cymbra.app: omit --models
node --experimental-strip-types tool/measure_marks.mjs --pair en-fr --models /tmp/models      # ~15 s, writes results-en-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-fr --models /tmp/models      # writes results-es-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-en --models /tmp/models      # the Spanish selections, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair en-es --models /tmp/models      # the English selections, judged in Spanish
node --experimental-strip-types tool/measure_marks.mjs --pair fr-en --models /tmp/models      # the French selections, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair fr-es --models /tmp/models      # the same, through English, judged in Spanish
node --experimental-strip-types tool/measure_marks.mjs --pair it-fr --models /tmp/models      # the Italian selections, through English, judged in French
node --experimental-strip-types tool/measure_marks.mjs --pair it-en --models /tmp/models      # the same, through the it-en model alone, judged in English
node tool/marks/select_corpus.mjs                                                             # only to rebuild corpus.json; deterministic
```

A request that traps the engine (a `WebAssembly.RuntimeError`, `isTrap`) is asked once more on a
fresh engine, as the extension asks it after a respawn (harden-lingua-translation-engine D2): the
instance that trapped is poisoned and put down. A sentence that traps twice is recorded
`trapped: true` — the reader would get no translation — and judged `withheld`, `trapped twice`; a
fragment that traps twice leaves the sentence's own marks unreconciled, as the extension shows
them (so does an empty fragment, never asked: `alone` is `""` for it, null for one that trapped
twice). The run says how many engines it built and which ids trapped. The fresh-engine retry was
exercised once against the real engine on 2026-10-08 — a factory throwing a trap-shaped error from
the first instance's first request made the es-en run build a second engine, which answered every
request to the committed figures, byte for byte — and remains fake-tested in the suite
(`test/translate-measure.spec.ts`).

## Soaking a route

Beside the measurement, by hand and never in CI (the programme's M25): the real engine through a
pair's route over the same corpus, to find the inputs that trap it — measured through the en-es
model in the study (2026-10-06), a trap poisons the instance for every model built after it — and
what a run costs in time and memory. It is how en-es is tried before it ships (change 35), now
that change 25 pins the en-es route in the catalogue, and fr-en and fr-es before change 52, now that
change 50 routes them, and the en-it model through en-it, fr-it and es-it, now that the Italian
programme's change 2 (`add-lingua-italian-translation-models`) pins it-en 2.0 and en-it 2.1 and
routes Italian's six pairs, and the it-en model through it-en and it-fr, over the Italian selections
row 19 (`measure-lingua-it-fr-translation-marks` D2) added to the corpus, which closes change 2's
D4 — their runs are recorded in `TRANSLATION.md`; a pair the catalogue does not route is refused.
A studied language the corpus held no selection of would be reported, not refused: « 0 selections
run » and « tool/marks/corpus.json holds no … selection: nothing was soaked », exit status 0.

```bash
node --experimental-strip-types tool/soak_engine.mjs --pair en-fr --models /tmp/models            # ~15 s
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models            # two models, through English
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models --limit 10 # the first ten selections
node --experimental-strip-types tool/soak_engine.mjs --pair en-es --models /tmp/models --isolate  # each sentence in a child process, ~45 s
node --experimental-strip-types tool/soak_engine.mjs --pair fr-es --models /tmp/models --isolate  # the French selections, through English
node --experimental-strip-types tool/soak_engine.mjs --pair en-it --models /tmp/models --isolate  # the English selections, through en-it
node --experimental-strip-types tool/soak_engine.mjs --pair fr-it --models /tmp/models --isolate  # the French selections, fr-en then en-it
node --experimental-strip-types tool/soak_engine.mjs --pair es-it --models /tmp/models --isolate  # the Spanish selections, es-en then en-it
node --experimental-strip-types tool/soak_engine.mjs --pair it-en --models /tmp/models --isolate  # the Italian selections, through it-en
node --experimental-strip-types tool/soak_engine.mjs --pair it-fr --models /tmp/models --isolate  # the same, it-en then en-fr
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
growth, with the freed blocks reused by the next model built. The catalogue lists six models
(en-fr, es-en, en-es, fr-en, it-en, en-it). Every native language's shipped pairs need two at most —
en-fr and es-en for French, es-en and fr-en for English, en-es and fr-en for Spanish — so a reader
who keeps their native language never makes a third today: the eviction runs when the native
language changes while the worker lives (add-lingua-french-translation D3), which production meets
since changes 34, 35 and 52 shipped the matrix's pairs. With Italian's six routes, the pairs glossed
in each native language need three — en-fr, es-en and it-en for French; es-en, fr-en and it-en for
English; en-es, fr-en and it-en for Spanish; en-it, fr-en and es-en for Italian — still loaded two at
a time, a route's: once Italian's pairs ship, a reader who keeps their native language makes the
worker delete a model only when a third pair's route needs the room
(add-lingua-italian-translation-models D2).

## Judging

The criteria were written before the run (release-lingua-spanish-translation D3, then
measure-lingua-translation-matrix-marks D4 for any native language; add-lingua-french-translation D6
added the French examples, before fr-en and fr-es were run, and moved no verdict;
measure-lingua-it-fr-translation-marks D3 added the Italian examples below, taken from the corpus,
before it-fr was run, and moved no verdict; measure-lingua-it-en-translation-marks D2 added their
English counterparts, anchored on verdicts already committed, before it-en was run, and moved no
verdict), and a pair is judged in its **native language** — French for en-fr, es-fr and it-fr,
English for es-en, fr-en and it-en, Spanish for en-es and fr-es. Through English, fr-es's mark
crosses two alignments, as es-fr's does, and so does it-fr's, it-en then en-fr: it is judged against
the rendering the reader sees, in the pair's native language, whatever the English one in between.
it-en's mark crosses one, the it-en model's alone: Italian is judged in English as French is (fr-en).

- **Correct:**
  - the marks cover the rendering, in the pair's native language, of the selected word in that
    sentence;
  - they may include the article, preposition or auxiliary it carries, and may be split;
  - an expression's rendering counts, as « sin embargo » → « Cependant » (es-fr), « sin embargo »
    → « However » (es-en), « gave up » → « se rindió » (en-es), « A travers » → « Throughout »
    (fr-en), « Par conséquence » → « Como resultado » (fr-es).
- **Wrong:** the marks cover another word, only a function word, or only part of a compound; or a
  word the translation omitted, so that the marks cover something else.
- **Withheld:** no mark — a sentence the engine trapped on twice included (`withheld`,
  `trapped twice`).

**Italian selections, judged in French** (measure-lingua-it-fr-translation-marks D3): the rules above,
with these examples from the corpus's Italian steps, written before it-fr was run.

- **A one-word past rendered as a compound one:** Italian's passato remoto is one word, French's
  rendering through English often a passé composé — steps 37 « vidi », 65 « conobbe », 69
  « rilasciò », 81 « accusò ». For step 65, a mark on « a connu » or « connu » is correct, on « a »
  alone it is not.
- **A subject the route adds:** Italian drops its subject, English and French write one — step 37's
  « A Pyongyang vidi alcune persone » alone: a mark on « ai vu » or « vu » is correct, on « j' » alone
  it is not.
- **A word after a fused preposition:** step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « cours » or « Au cours » is correct, on « Au » alone it is not; an expression's rendering
  (« Tout au long de ») counts.
- **A word after an elided piece:** step 86's « l'unico nel Governo », the only Italian selection of
  its kind (French had 8): the mark renders `unico` (« le seul »), never the article alone.
- **Gender or number English lost:** a French rendering whose gender or number the route lost still
  counts as the word's rendering — step 79 « nemiche », step 71 « stereotipata ».

**Italian selections, judged in English** (measure-lingua-it-en-translation-marks D2): the rules
above, with these examples from the same Italian steps, anchored on verdicts already committed,
written before it-en was run.

- **A subject the model adds:** step 25 « Ha detto all'Andrew Marr Show »: a mark on « told »,
  « said » or « He told » is correct, on « He » alone it is not — one of es-en's two wrong marks, k 25,
  marked « he » for « said »; es-fr's k 25 « il a déclaré » was judged correct for carrying the subject
  the rendering adds.
- **A compound past rendered as a simple one:** the passato remoto is one word in English as in
  Italian (es-en: k 65 « met », k 69 « released », k 81 « accused »), so the French case above does
  not arise; the reverse does — an Italian compound past's participle (steps 5, 13, 17, 25, 31) often
  renders as one English simple past: a mark on that word is correct, on « has » or « were » alone it
  is not.
- **A word after a fused preposition:** step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « course » or « In the course » is correct, on « In » alone it is not; an expression's rendering
  (« Throughout ») counts.
- **A word after an elided piece:** step 86's « l'unico nel Governo »: a mark on « only », « only
  one » or « the only (one) » is correct — the article comes from « l' » (es-en k 86 « the only
  one ») —, on « the » alone it is not.
- **A clitic pronoun beside the selected verb** (steps 9, 31, 33, 97; none reflexive): step 33 « Gli
  viene riconosciuto » → « He is credited »: « credited » or « is credited » is correct, « He » alone
  is not (es-en k 33 marked « credited »); step 31 « l'ho scelta » → « I chose it »: « chose » is
  correct, « it », « her » or « I » alone is not; step 97 « si incontrano » → « met »: « each other »
  alone is not; step 9 « ci fossero » → « there were »: « were » or « be » is correct, « there » alone
  is not.
- **One Italian word, an English compound:** step 54 « Il capodanno cinese » → « Chinese New Year »
  and step 78 « sveglia » → « alarm clock »: the whole compound is correct; « Year », « alarm » or
  « clock » alone is part of a compound, which the criteria count wrong — es-en's k 54 (« Year »
  judged correct) does not carry over, its Spanish selection being « Año » alone.
- **Gender or number English does not mark, and a numeral beside the word:** step 79 « le due fazioni
  nemiche » → « the two enemy factions »: a mark on « enemy » (or « hostile », « rival ») is correct,
  on « two » it is not, as es-en's k 79 was judged; step 71 « stereotipata » → « stereotypical » or
  « stereotyped ».

One judge reads a pair, as the shipped pairs were judged; a doubtful line says so in
`engine_reason`, and the owner may re-judge any line in the pull request — the figures are
recomputed from the committed judgments (`tier.mjs`). The rates read D2 as the study did:

- correct is the share of the shown marks;
- withheld is the share of all 100 selections.

## Results

Engine pinned by `engine-pin.json`; models `en-fr` base-memory 2.0, `es-en` 2.0, `en-es` 2.1,
`fr-en` 2.0 and `it-en` 2.0 (`model-manifest.json`). The pairs listed in `MARKED_PAIRS`
(`src/translate/markup.ts`) are the judged pairs on the first tier here, and
`test/translate-marks.spec.ts` holds the list to the judged files: every listed pair judged, a judged
pair listed exactly when its committed judgment reaches the tier (measure-lingua-it-fr-translation-marks
D4). en-fr and es-fr were measured on 2026-10-05, judged in French; es-en and en-es on 2026-10-08
(measure-lingua-translation-matrix-marks), judged in English and in Spanish; fr-en and fr-es on
2026-10-09 (add-lingua-french-translation), judged in English and in Spanish; it-fr on 2026-10-23
(measure-lingua-it-fr-translation-marks), judged in French; it-en on 2026-10-25
(measure-lingua-it-en-translation-marks), judged in English — one judge each.

|                                      | Correct (of shown marks) | Withheld | D2                                                                             |
| ------------------------------------ | ------------------------ | -------- | ------------------------------------------------------------------------------ |
| **es-fr, engine (pivot es→en→fr)**   | **89 / 90 — 98.9 %**     | **10 %** | **First tier: marked**                                                         |
| en-fr, engine                        | 96 / 97 — 99.0 %         | 3 %      | (reference; the study found 83 / 87)                                           |
| **es-en, engine, judged in English** | **94 / 96 — 97.9 %**     | **4 %**  | **First tier: marked (shipped since change 34)**                               |
| **en-es, engine, judged in Spanish** | **96 / 96 — 100 %**      | **4 %**  | **First tier: marked (shipped since change 35)**                               |
| **fr-en, engine, judged in English** | **95 / 96 — 99.0 %**     | **4 %**  | **First tier: marked (shipped since change 52)**                               |
| **fr-es, engine (pivot fr→en→es)**   | **90 / 91 — 98.9 %**     | **9 %**  | **First tier: marked (shipped since change 52)**                               |
| **it-fr, engine (pivot it→en→fr)**   | **93 / 97 — 95.9 %**     | **3 %**  | **First tier: marked (listed, inert until the Italian programme's change 20)** |
| **it-en, engine, judged in English** | **94 / 97 — 96.9 %**     | **3 %**  | **First tier: marked (listed, inert until the Italian programme's change 25)** |
| es-fr, gloss-located (experiment)    | 76 / 78 — 97.4 %         | 22 %     | —                                                                              |
| en-fr, gloss-located (experiment)    | 76 / 79 — 96.2 %         | 21 %     | —                                                                              |
| es-en, gloss-located (experiment)    | 75 / 78 — 96.2 %         | 22 %     | —                                                                              |
| en-es, gloss-located (experiment)    | —                        | —        | left empty: no `tables/en-es/gloss.tsv` yet (22)                               |
| fr-en, gloss-located (experiment)    | —                        | —        | left empty: no French gloss table yet (48)                                     |
| fr-es, gloss-located (experiment)    | —                        | —        | left empty: no French gloss table yet (49)                                     |
| it-fr, gloss-located (experiment)    | —                        | —        | left empty: no `tables/it-fr/gloss.tsv` yet (the Italian programme's 16)       |
| it-en, gloss-located (experiment)    | —                        | —        | left empty: no `tables/it-en/gloss.tsv` yet (the Italian programme's 22)       |

What these numbers say:

- **es-fr is on D2's first tier.** One wrong mark in 90 (« dos facciones enemigas » marked « deux »
  instead of « ennemies »).
- **It holds under a stricter reading:** if the five expression marks counted as wrong, es-fr would
  still score 84 / 90, 93 %.
- **es-en and en-es are on the first tier too**, at the figures the study predicted (94 / 96 and
  96 / 96). es-en's two wrong marks, by the TSV's `k`: « declaró » marked « he » instead of « said »
  (k 25), and « enemigas » marked « two » instead of « enemy » (k 79) — the mark es-fr inherited
  through the pivot. Its four withheld are three expressions the translation reshaped (« dejar
  caer », « dar por hecho », « consiguió vencer ») and one hyphenated compound
  (« franco-monegasco »). en-es has no wrong mark; its four withheld are two of en-fr's own
  (« officer », « the only one » — the same English selections), « alarm clock » → « despertador »
  and « captained », unmarked. Three en-es lines (k 18, k 70, k 77) follow a loose rendering, e.g.
  « edge » → « ventaja »: the mark sits on the rendering the reader sees, so they count correct and
  are flagged `doubtful:` in `engine_reason`.
- **No request trapped** on es-en or en-es: one engine built per run, no `trapped: true` line and no
  unreconciled fragment — the trapped ids this README would list are none (`TRANSLATION.md`, where
  change 25 recorded the en-es soak, says the same).
- **fr-en and fr-es are on the first tier**, at the figures the study predicted within its ± 6 %
  (94 / 96 and 90 / 91). fr-en's one wrong mark: « intérêt principal » marked « their » instead of
  « main » (k 15); its four withheld are « la plus grande partie de » → « most of » (k 0),
  « Les orages » (k 48), « Autrement dit » → « In other words » (k 57) and « Pour l'instant » →
  « For now » (k 79). fr-es's one wrong mark: « la dernière fois » marked « la » instead of « vez »
  (k 30); its nine withheld are fr-en's four and five the pivot loses (« discordance » rendered
  « dispensación », « au mieux », « Au troisième siècle » → « En el siglo III », « stéréotypé »,
  « représente »), as es-fr withholds 10 where es-en withholds 4.
- **The fragment alone mattered on both routes:** it moved the engine's mark to the right word on 2
  fr-en lines (« last » → « time », « hit » → « song ») and 4 fr-es lines (« su » → « principal »,
  « protección » → « limitada », « es … de » → « canción », « , se » → « se reunieron »), and trimmed
  it on 3 and 4 others (« would want » → « want »).
- **Three lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`: fr-en
  « limitée » → « for limited » (k 39) and « traces » → « The blood trails » (k 92), fr-es
  « voudrait » → « el … querría” » (k 25). Counted wrong, fr-en is 93 / 96 and fr-es 89 / 91: the
  tier holds either way.
- **The 8 French selections right after an elided piece** (`d'enseigner`, `l'armée`, …) are correct
  on both pairs but « l'instant », withheld on both. No request trapped on either French pair: one engine
  built per run, every fragment reconciled.
- **it-fr is on the first tier** through two models, it-en then en-fr: 93 of its 97 shown marks
  correct, 3 withheld — listed in `MARKED_PAIRS`, inert until the Italian programme's change 20
  (`enable-lingua-italian`) lists it-fr in `packs.json`. Its four wrong marks, by `k`: « isole »
  marked « granit » instead of « îles » (k 38), « battaglia » marked « au » alone instead of
  « combat » (k 52), « comprendono » marked « qui » (k 57, as en-fr's « make » was) and « rilasciò »
  marked the auxiliary « a » alone instead of « a sorti » (k 69). Its three withheld: « gran parte della »
  → « une grande partie de » (k 0), « Per quanto riguarda » → « Quant à » (k 61) and « stereotipata »,
  rendered « stéréotypée » unmarked (k 71).
- **The criteria's Italian cases** (Judging): the one-word pasts are marked on their compound
  rendering — « ai vu » (k 37, carrying the « j' » the route adds), « a rencontré » (k 65), « a
  accusé » (k 81) — but for k 69's auxiliary alone; after a fused preposition, « Tout | long »
  (k 6, « Tout au long de »), « mieux » (k 22) and « derniers » (k 91) are correct, k 82 doubtful;
  after the one elided piece, « seul » (k 86); gender or number never decided a verdict
  (« ennemies », k 79).
- **Four it-fr lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`:
  « discordanza » → « discorde » (k 14) and « coppia » → « paire » (k 20), loose renderings the
  mark follows; « Fa' cadere » → « Lâche le » (k 29), the mark carrying « micro »'s article; and
  « agli sgoccioli » rendered « son infarest" » (k 82), a garbled non-word the mark follows, as
  es-fr's k 15 (« indifféronteurs ») was judged. Counted wrong, it-fr is 89 / 97, 91.8 %: the tier
  holds either way.
- **No request trapped on it-fr**: one engine built, every fragment reconciled. The fragment alone
  moved the engine's mark to the right word once (« dilemmes » → « similaires », k 67) and trimmed
  it twice (k 7, k 96). Run twice, the results were byte for byte the same.
- **it-en is on the first tier** through its one model: 94 of its 97 shown marks correct, 3 withheld —
  listed in `MARKED_PAIRS`, inert until the Italian programme's change 25
  (`enable-lingua-italian-for-english-speakers`) lists it-en in `packs.json`. Its three wrong marks, by
  `k`: « comprendono », rendered « understand » in the wrong sense, marked « who » (k 57, as it-fr's
  « qui » and en-fr's « make » were); « sveglia » marked « alarm » alone, half of « alarm clock »
  (k 78); and « agli sgoccioli », rendered « in its infarest » — a garbled non-word —, marked « its »
  alone, the article, not the rendering (k 82). Its three withheld: « gran parte della » → « much of »
  (k 0), « isole » → « islands » unmarked (k 38) and « Per quanto riguarda » → « As for » (k 61).
- **The criteria's Italian cases in English** (Judging): the subject the model adds is left out on
  « told » (k 25) and carried by « I saw » (k 37) and « he played » (k 73); the compound pasts are
  marked on their one English word — « grew » (k 5), « posted » (k 13), « commented » (k 17),
  « told » (k 25), « chose » (k 31); after a fused preposition, « Throughout » (k 6), « best » (k 22,
  « At best ») and « last » (k 91) are correct, k 82 wrong; after the elided piece, « only one »
  (k 86); beside a clitic, « to be » (k 9, « there » left out), « chose » (k 31), « credited » (k 33)
  and « meet » (k 97); of the English compounds, « New Year » (k 54) is whole and « alarm » (k 78) is
  not; and « enemy » (k 79) is marked where es-en marked « two ».
- **Two it-en lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`:
  « discordanza » → « discord » (k 14) and « coppia » → « pair » (k 20), loose renderings the mark
  follows — it-fr's k 14 and k 20 too. Counted wrong, it-en is 92 / 97, 94.8 %: the tier holds either
  way.
- **No request trapped on it-en**: one engine built, every fragment reconciled. The fragment alone
  moved the engine's mark to the right word once (« in » → « battle », k 52) and trimmed it once
  (« originally from » → « originally », k 7). Run twice, the results were byte for byte the same.
- **Sampling error:** about ±6 % at 100 selections.

**The experiment** marks the first word of the sentence that matches the pack's gloss of the
selected word, in the pair's native language, its stop words left out (`stop-words.mjs`).

- It is about as precise as the engine, in English as in French (es-en: 75 / 78).
- It withholds about one selection in five: a translation often picks a synonym the gloss does not
  list.
- Where the engine withholds, it would have found 7 of Spanish's 10 missing marks for es-fr, and
  2 of es-en's 4.
- fr-en's and fr-es's columns are empty: change 43 commits fr-en's gloss table empty, and the harness
  reads a table only when it holds a gloss. Once changes 48 and 49 commit French's glosses, a later
  run may fill them without touching the engine's columns.
- it-fr's columns are empty: no `tables/it-fr/gloss.tsv` was committed at the run (the Italian
  programme's change 16 commits it); a later run may fill them the same way.
- it-en's columns are empty too: no `tables/it-en/gloss.tsv` was committed at the run (the Italian
  programme's change 22 commits it).

A later change may try it as a fallback; nothing here ships it.
