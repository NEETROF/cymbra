# Design — add-lingua-french-translation

## Context

See proposal.md (Why). What exists, on `main` at `80af4f72`:

| Where | What |
|---|---|
| `apps/lingua-extension/model-manifest.json` | `base` `https://models.cymbra.app/`, `sourceBase` Mozilla's registry; models `en-fr/base-memory/2.0`, `es-en/base-memory/2.0`, `en-es/base-memory/2.1` (change 25), each `from`, `to`, `licence`, `mirror`, `files.{model,lex,vocab}` with `path` (`<id>/<decompressed sha256>/<file>.gz`), `size`, `unpacked`, `sha256` (decompressed), `source.{path, sha256}`; routes `en-fr`, `es-fr` (es-en then en-fr), `es-en`, `en-es` |
| `src/translate/host/model-manifest.ts` | `parseCatalogue` (a route keyed `<studied>-<native>`, known ids, starts from the studied language, chains, ends in the native one), `routeOf`, `modelsFor` (the union of the pairs' routes, each model once) |
| `src/analyzer/pairs.ts`, `packs.json` | `SHIPPED_PAIRS` `en-fr`, `es-fr`; `readerPairs(languages, native)` keeps shipped pairs only — a route of a pair not shipped is needed by no reader |
| `src/translate/host/model-db.ts`, `model-controller.ts` | files stored by decompressed sha256, a stored file not fetched again; `prune(needed)` keeps every file a needed model names, stored or not |
| `src/translate/host/model-residency.ts` (change 9) | the worker holds two models at most; above, the least recently used model the new route does not need is deleted with its routes; `test/model-residency.spec.ts` already uses the placeholder ids `fr-en/base-memory/2.0` and routes `fr-en`, `fr-es` = fr-en then en-es |
| `tool/assemble_model_site.mjs`, `mirror_models.mjs`, `check_model_host.mjs`, `.github/workflows/lingua-model-deploy.yml` | read the catalogue: the host is assembled from Mozilla's registry, then the mirror, then the host, keeping a file only when every pin holds; one mirror release per model, created once; the host checked from outside before a submission or an App Store delivery |
| `build.mjs`, `tool/check_variants.mjs` | bundle the catalogue without `sourceBase`/`mirror`/`source`; hold the bundled copy to the committed one, models and routes |
| `tool/marks/` (change 26) | `corpus.json` (100 English, 100 Spanish selections from PUD by `select_corpus.mjs`: every tenth sentence, NOUN, VERB, NOUN, ADJ in turn, « the next sentence, for both languages »), `pud.mjs` (English and Spanish PUD pinned), `engine.mjs`, `measure.mjs` (a trap asked once more on a fresh engine), `stop-words.mjs` (fr, en, es), `tier.mjs`; results and judgments of en-fr, es-fr, es-en, en-es; `README.md` (criteria, figures) |
| `src/translate/markup.ts` | `MARKED_PAIRS = ["en-fr", "es-fr", "es-en", "en-es"]`; a pair outside it is translated untagged |
| `tool/soak_engine.mjs` (change 9) | `--pair` (a pair the catalogue routes), `--models`, `--limit`, `--isolate`; the en-es soak recorded in `TRANSLATION.md` |
| `apps/site/src/lib/lingua-pairs.ts` (change 30) | reads the catalogue's route of each **shipped** pair (direct, through English, none) |
| Change 43 (proposed) | pins UD French-PUD at `db260db10fe728853c549760801229ef4e7b16e1`, sha256 `4dfed37b…3c10`, and hands the same pin to this change (its D9); commits `tables/fr/forms.tsv` and an **empty** `tables/fr-en/gloss.tsv`, filled by change 48 |
| The model host, 2026-10-09 | `models.cymbra.app` serves en-fr's and es-en's files and answers 404 for en-es's; the release `lingua-model-en-es-base-memory-2.1` does not exist — change 25's task 4.1 has not run |

How it was measured: Mozilla's registry (`db/models.json`, generated 2026-10-09T00:54:11Z) and
Firefox's Remote Settings (`translations-models`, `fromLang=fr&toLang=en`) read on 2026-10-09; the
three files downloaded once from the registry; a scratch copy of `apps/lingua-extension/tool` and
`src` (never committed) with the prototype catalogue entry, the three-language selection rule and
French PUD, run with the pinned engine (`a6310e24…`, `.wasm` `7ef4b3fd…3122`) on a MacBook Pro,
Apple M2 Max (12 cores), 64 GB, macOS 26.5.2, Node 22.22.2 — the machine change 25's soak used.

## Goals / Non-Goals

**Goals:**
- fr-en 2.0 pinned as every model is, verifiable byte for byte from Mozilla's two publications.
- fr-en's route direct and fr-es's through English, inert until change 52.
- French selections for the marks, without moving a single English or Spanish selection.
- fr-en's and fr-es's marks measured and judged as the four other pairs' were, every judgment
  committed; the soak of both routes recorded.

**Non-Goals:**
- Shipping fr-en or fr-es, or offering their translation (change 52, M15).
- The listings' and the site's sentences, and their sizes (change 53).
- A French–Spanish model: Mozilla publishes none.
- Re-measuring en-fr, es-fr, es-en or en-es: their figures and files stand.
- A selection inside an elided word (change 51, handed over by change 40's D7).

## Decisions

### D1 — The entry is read from the files, and checked against two publications

Mozilla's registry lists one fr-en entry: `base-memory`, `releaseStatus: "Release"`, run
`models/fr-en/retrain_hr_EFgIftH_RrCyzl5gjemVNg/exported/`, the model's `uncompressedSize`
31,561,787 and `uncompressedHash` `15f997bc…2b90`; flores200-plus BLEU 42.95 and COMET-22 0.886
(en-fr 48.85 / 0.865, es-en 26.85 / 0.857, en-es 27.52 / 0.854). Firefox's Remote Settings list
fr→en 1.0 and 2.0, nothing later. Downloaded once and measured, every pin agrees:

| File | Served (gzip) | gzip sha256 | Decompressed | sha256 (decompressed) = Remote Settings 2.0 |
|---|---|---|---|---|
| `model.fren.intgemm.alphas.bin.gz` | 23,175,075 | `06b1eeed…a2ee` | 31,561,787 | `15f997bc0d13808b0b0fbd0786e684a3c8a52adcd8071844b76123fdacbf2b90` (= the registry's `uncompressedHash`) |
| `lex.50.50.fren.s2t.bin.gz` | 2,649,934 | `395aa776…642a` | 4,824,120 | `87c6752ea908f5f0347c10ac0cf7d80d9c2f4f20c81c90168f3e8230b56d4440` |
| `vocab.fren.spm.gz` | 409,706 | `8d15b219…53a7` | 814,404 | `783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a` (= en-fr 2.0's) |

The download is 26,234,715 B (« 26,2 Mo »), 37,200,311 B on the device. The id is
`fr-en/base-memory/2.0` — the one `test/model-residency.spec.ts` already uses — the paths
`fr-en/base-memory/2.0/<decompressed sha256>/{model.bin.gz,lex.bin.gz,vocab.spm.gz}`, the mirror
`https://github.com/NEETROF/cymbra/releases/download/lingua-model-fr-en-base-memory-2.0/`, the
licence MPL-2.0, `source.path` the registry's paths and `source.sha256` the gzip digests above
(full values in the implementation; the prototype entry holds them). `assemble_model_site.mjs`,
run over the prototype catalogue, took all twelve files of the four models from the registry,
« every pin matches », in 7 s. A mismatch between the registry and Remote Settings stops the change,
as change 25's D1 says. The largest file, 23,175,075 B, is under Cloudflare Pages' 25 MiB per file,
as the three models already served are.

### D2 — Two routes: French direct to English, through English to Spanish

`"fr-en": ["fr-en/base-memory/2.0"]` and `"fr-es": ["fr-en/base-memory/2.0",
"en-es/base-memory/2.1"]`, appended after the four routes; the models listed in the order they
were pinned (en-fr, es-en, en-es, fr-en). Both satisfy the parser (each starts from French, chains
English, ends in the pair's native language), so the bundled catalogue loads — checked by
`parseCatalogue` on the prototype entry. fr-es goes through English because Mozilla publishes no
French–Spanish model, as es-fr does; its download is fr-en's and en-es's, 51,608,069 B
(« 51,6 Mo », es-fr's 52,0), and the setting already states the pivot's memory when a needed route
has two models (« environ 340 Mo », `translation-setting.ts`).

A route is needed only when its pair is a reader's pair, and a reader's pairs are shipped pairs:
until change 52 lists fr-en and fr-es in `packs.json`, no reader needs either route, and the
controller downloads, keeps and loads exactly what it does today. Both routes are added here for the
reason change 25 gave for es-en's: the harness measures a pair through the catalogue's route, and
a route added by the enable change would leave this measurement without one.

The site reads a route only for a pair `lingua-coverage.json` lists, which are `packs.json`'s, so
its pages do not move; once change 52 ships the pairs, it says fr-en is translated directly and
fr-es through English without another change.

### D3 — Two models per native language, and when the bound evicts

With the six routes, every native language's pairs need two models together (`modelsFor` over the
prototype catalogue):

| Native | Pairs | Models | Stated download |
|---|---|---|---|
| French | en-fr, es-fr | en-fr, es-en | 51,993,524 B (52,0 Mo, as today) |
| English | es-en, fr-en | es-en, fr-en | 52,475,767 B (52.5 MB) |
| Spanish | en-es, fr-es | en-es, fr-en | 51,608,069 B (51.6 MB) |

So a reader who stays in one native language never makes the worker evict: change 9's bound of
two (`model-residency.ts`) holds every model their pairs need. Run through `toLoad`/`loaded` on the
prototype catalogue, a Spanish-native reader alternating English and French pages loads en-es,
then fr-en beside it, and nothing is deleted after; an English-native reader the same with es-en
and fr-en. The bound evicts only when the native language changes while the worker lives — French
to Spanish deletes en-fr and es-en for fr-es's two models, French to English deletes es-en for
fr-en, then en-fr for es-en — and the worker holds two models after every load. This change adds a
test of those sequences on the committed catalogue's routes, beside the placeholders the residency
spec already uses.

**A file two models share.** fr-en 2.0's vocabulary decompresses to en-fr 2.0's bytes
(`783abf3a…002a`; Mozilla serves two gzip files, `5dcadaad…` and `8d15b219…`), as en-es's and
es-en's do (change 25's D3). No native language's pairs hold both en-fr and fr-en, so it matters only
when a French-native reader of English becomes an English- or Spanish-native reader of French: the
controller's `prune(needed)` keeps every file a needed model names, stored or not, so the vocabulary
stays and fr-en's download fetches 25,825,009 B, while the setting states 26,234,715 B — the cost
sums the files of the models needed, as the requirement *The setting's cost comes from the catalogue*
states it, and as en-es's shared file already does.

**Memory.** Two `base-memory` models of the same shape (31,561,787 B each, 31,248,966 parameters
in the registry) load for fr-es as for es-fr, whose 321.8 MiB was measured in the worker. Node's
figure is not the worker's, but like for like it shows French's routes cost what the shipped ones
do: fr-es 557.0 MiB in one instance against es-fr's 559.3, 547.2 isolated against 545.9; fr-en
415.1 against en-fr's 415.7. The worker's own figure is measured on devices in change 52's dogfood.

### D4 — French selections: one rule, the same sentences, nothing else moves

`pud.mjs` pins UD French-PUD as change 43 does: `UD_French-PUD` at
`db260db10fe728853c549760801229ef4e7b16e1`, `fr_pud-ud-test.conllu`, sha256
`4dfed37b83d76e77fd2e9963d0be00d723e9a010e7e2a746f8b7640c48063c10` (1,818,011 B, CC BY-SA 3.0, kept
in the git-ignored work directory, never committed) — downloaded again on 2026-10-09 at that commit,
the same bytes. `parseConllu` reads all 1,000 sentences, every word's offsets found in the text
(French PUD writes `'`, never `’`).

`select_corpus.mjs` runs its rule over English, Spanish and French at once: per step, the first
sentence from the tenth where **every** language has a word by the rule (« the next sentence, for
every language »), one word each. Measured:

- **No English or Spanish selection moves.** The first sentence where all three languages have a
  word is, for each of the 100 steps, the one the committed corpus chose for two; regenerated, the
  200 English and Spanish items equal the committed ones, and the file's diff is 1,006 lines added
  and the rule's text (« both languages » → « every language »).
- **French takes a word on every one of the 100 sentences**: 58 nouns, 24 verbs, 18 adjectives, 11
  of them by the rule's fallback part of speech; every selection's offsets slice its word out of the
  sentence. The rule's « letters only, not in a multiword token » leaves out an elided piece (`l'`)
  and `au`/`du`/`des`, as it leaves out Spanish's `al`/`del`; 8 selections sit right after an elided
  piece (`d'enseigner`, `l'extrême`, `l'état`, `D'importantes`, `L'armée`, `l'université`,
  `l'instant`, `D'autre`), so the engine is asked `l'<b>armée</b>`.
- The three languages share each step's sentence, so a French selection reads beside the English
  and Spanish ones of the same text, as es-en reads beside es-fr.

The selection is UD's word, not the extension's (change 40's pre-pass decides what a reader's
click selects); the engine sees a sentence and two offsets either way.

### D5 — The soak, by hand

`tool/soak_engine.mjs --pair fr-en` and `--pair fr-es`, `--models` a locally assembled site, over
the 100 French selections, never in CI (M25's recommendation). The tool needs no change: it reads
the catalogue's routes and the corpus of the pair's studied language. Prototype:

| | fr-en `--isolate` | fr-es `--isolate` | fr-en, one instance | fr-es, one instance |
|---|---|---|---|---|
| Selections | 100 run, **100 translated, 0 trapped, 0 timed out** | **100, 0, 0** | **100, 0, 0** | **100, 0, 0** |
| Time per sentence (median / mean / max) | 144 / 147 / 263 ms | 220 / 224 / 305 ms | 35 / 39 / 182 ms | 68 / 75 / 297 ms |
| maxRSS (Node's, not the worker's) | 390.2 MiB | 547.2 MiB | 415.1 MiB | 557.0 MiB |
| Whole run | 45 s | 71 s | 4.5 s | 8.7 s |

Beside them on the same run: es-fr 68 / 75 / 295 ms and 559.3 MiB in one instance, 219 / 225 / 312
ms and 545.9 MiB isolated; en-fr 36 / 39 / 185 ms and 415.7 MiB — French's routes cost what the
shipped routes cost. The implementation runs it again and records it in `TRANSLATION.md` beside
en-es's soak. A run that finds no trap says so; a trap on another input still costs a reader one
respawn (change 9).

### D6 — fr-en and fr-es measured, judged in their native languages

`measure_marks.mjs --pair fr-en` and `--pair fr-es`, as change 26 measured es-en and en-es: the
French selections, the pair's route, the pair's native language's stop words; a request that traps
asked once more on a fresh engine. The four pairs already measured are **not** run again and their
files not rewritten: their engine columns reproduce exactly through the extended catalogue and
corpus (prototype: every line of the four results equal but for the experiment's column, which the
scratch copy ran without tables, and the `trapped` key, which en-fr's and es-fr's committed lines
predate — en-es's file byte for byte), so a rewrite would only churn them.

The criteria are kept word for word (change 26's D4) and gain, before any result is read, a French
example per native language — « A travers » → « Throughout » (fr-en), « Par conséquence » →
« Como resultado » (fr-es) — and the README says fr-es crosses two alignments through English, as
es-fr does. One judge reads each pair; a doubtful line says so in `engine_reason`; the owner may
re-judge any line in the pull request, the figures recomputed by `judgedCounts`.

**The gloss experiment fills in only from a pair's committed glosses.** `measure_marks.mjs` reads
the experiment's table when `tables/<pair>/gloss.tsv` exists; change 43 commits fr-en's empty, and
on an empty table every line's experiment would read « no gloss », a column that looks measured and
is not. The harness reads the table only when it holds a gloss, and leaves the columns empty
otherwise (`existsSync` kept, a test on an empty table added). With changes 48 and 49 merged first,
the experiment fills from `tables/fr-en/gloss.tsv` (with `tables/fr/forms.tsv`, change 43) and
`tables/fr-es/gloss.tsv`; otherwise a later run may fill it without touching the engine's columns
(change 26's D3).

Prototype, one judge per pair as for the four others, the criteria above, the judgments kept in the scratch directory (never committed):

| | Correct (of shown marks) | Withheld | D2 |
|---|---|---|---|
| **fr-en, engine, judged in English** | **95 / 96 — 99.0 %** | **4 %** | **First tier** |
| **fr-es, engine (pivot fr→en→es), judged in Spanish** | **90 / 91 — 98.9 %** | **9 %** | **First tier** |
| es-en, committed (same sentences, Spanish) | 94 / 96 — 97.9 % | 4 % | first tier |
| es-fr, committed (pivot) | 89 / 90 — 98.9 % | 10 % | first tier |

- **fr-en's one wrong mark**: « intérêt principal » marked « their » instead of « main » (k 15).
  Its four withheld: « la plus grande partie de » → « most of » (k 0), « Les orages » (k 48),
  « Autrement dit » → « In other words » (k 57), « Pour l'instant » → « For now » (k 79).
- **fr-es's one wrong mark**: « la dernière fois » marked « la » instead of « vez » (k 30). Its nine
  withheld are fr-en's four and five the pivot loses (« discordance » rendered « dispensación »,
  « au mieux », « Au troisième siècle » → « En el siglo III », « stéréotypé », « représente »), as
  es-fr withholds 10 where es-en withholds 4.
- **Reconciliation matters on both routes**: the fragment translated alone moved the engine's mark
  to the right word on 2 fr-en lines (« last » → « time », « hit » → « song ») and 4 fr-es lines
  (« su » → « principal », « protección » → « limitada », « es … de » → « canción », « , se » →
  « se reunieron »), and trimmed it on 3 and 4 others (« would want » → « want »).
- **Doubtful, counted correct**: fr-en « limitée » → « for limited » (the mark carries « for », k 39)
  and « traces » → « The blood trails » (the English compound carries « sang »'s rendering, k 92);
  fr-es « voudrait » → « el … querría” » (a stray « el », « gobierno »'s article, k 25). Counted
  wrong, fr-en is 93 / 96 (96.9 %) and fr-es 89 / 91 (97.8 %): the tier holds either way.
- The 8 selections right after an elided piece: 7 correct on both pairs, « l'instant » withheld on
  both. No request trapped; one engine built per run; every fragment reconciled.
- **The study** gave 94 / 96 and 90 / 91: within the ± 6 % of 100 selections.

The engine and models are deterministic: two runs gave byte-identical results for both pairs.

### D7 — The list

A pair on the first tier joins `MARKED_PAIRS` after en-es (`["en-fr", "es-fr", "es-en", "en-es",
"fr-en", "fr-es"]` if both reach it, as the prototype does); it is inert until change 52 ships the
pair, since no reader has it. A pair short of the tier stays out and translates without a mark;
under 75 % correct or over 30 % withheld, the figures go to the owner, who settles under M15
whether change 52 offers its translation unmarked or not at all — this change decides nothing for
it, nor M15 for either pair.

### D8 — Documents and tests

- `test/model-manifest.spec.ts`: four models in pin order, six routes; fr-en's digests, sizes,
  paths, source and mirror (*Pinned against Mozilla's publications*); fr-en's vocabulary equal to
  en-fr's once decompressed, its gzip file not; fr-es's download 51,608,069 B; for each native
  language, every route keyed by it needs two models together (*Every native language's pairs*);
  the no-route test keeps `de-fr` and `en` alone and drops `fr-en`/`fr-es`; *A route of a pair not
  shipped* covers fr-en and fr-es.
- `test/model-residency.spec.ts`: the sequences of D3 through the committed catalogue's routes.
- `test/translate-marks.spec.ts`: the corpus's languages are en, es and fr, each step's three items
  share `k` and `id` (*A studied language added to the corpus*); fr-en's and fr-es's totals and tier;
  nothing filed by studied language (`fr` added to the loop); `MARKED_PAIRS` exactly the judged pairs
  on the tier. `test/translate-relay.spec.ts`: the list's exact value, and a French selection through
  fr-en marked in the English sentence (*A French selection, judged in English*). A test of
  `measure_marks.mjs`'s table rule on an empty `gloss.tsv`.
- `TRANSLATION.md`: the routes table (fr-en direct, fr-es through English, « not yet: change
  52 »), the shared vocabulary, D3's per-native table, the soak (D5), the measured pairs.
  `tool/marks/README.md`: three studied languages, the French pin, the files, the criteria's
  examples, the results table and what the figures say. `REVIEWERS.md`: four models, which routes
  each serves, the package downloading none for French. `soak_engine.mjs`'s and `measure_marks.mjs`'s
  usage comments name the six routes.

## What moves, and what cannot

Nothing in `crates/`, `scripts/lingua-data/` or `apps/site/` changes, so no golden can move:
`fr-en.golden` (change 39's French baseline, an engine over the hand-written fixture pack, no
translation involved), S0, the es-fr, es-en and en-es goldens, the extension's snapshots
(`test/baseline/*.txt`) and the site's pinned build all stay as committed, run without re-blessing.
en-fr, es-fr, es-en and en-es cannot move here: their catalogue entries and routes are byte for
byte (the test holds their three models and four routes as before), their readers' needs are
unchanged (`readerPairs` keeps shipped pairs), their corpus items are byte for byte (D4), their
results and judgments are not rewritten (D6), and `MARKED_PAIRS` keeps their four entries in order.
The bundled catalogue grows by 1,147 B per package (3,354 → 4,501 B as built) and `MARKED_PAIRS` by
two pair names; nothing else.

## Risks / Trade-offs

- **Submissions refused until the host serves fr-en** → the owner dispatches `lingua-model-deploy`
  after the merge (task 5.1). The host already refuses today for en-es's files (change 25's 4.1 not
  run on 2026-10-09); one dispatch assembles every model of the catalogue and creates both missing
  mirror releases.
- **Remote Settings' 2.0 is not the registry's file** → D1 refuses the entry; the change waits.
- **One judge** → the criteria fixed before the run, every judgment committed and re-judgeable, the
  doubtful lines named; the tier holds with them counted wrong.
- **PUD is news and Wikipedia prose, selected word by word** → as for every pair; elided pieces are
  never selected. Literary French and phrase selections are checked by eye in change 52's dogfood.
- **The pivot reads English through** (« tué au combat » → « asesinado en acción », English's
  « killed in action ») → as es-fr already goes through English; the card says it is a machine
  translation, and the mark follows the rendering the reader sees.
- **Memory on iOS** → fr-es's two models are es-fr's size, already dogfooded on the iPhone
  (2026-10-05); change 52's dogfood measures it again.

## Migration Plan

No release: a catalogue entry, two inert routes, a corpus extended and two measurements. The owner
deploys the model host after the merge; nothing a reader has stored moves.

## Open Questions

1. **The doubtful lines** (D6: fr-en k 39 and k 92, fr-es k 25): the owner may re-judge them in the
   pull request; the tier holds either way.
2. **M15 for French** (open): whether change 52 offers fr-en's and fr-es's translation, everywhere at
   once, with the marks this change measures — the owner's, before change 52; this change lists the
   pairs that reach the tier and decides nothing else.
