# The translation engine

« Traduction étendue » answers a reader's selection with **their whole sentence, translated, their
selection marked in its French place** — and a single word the pack has no gloss for, in its
sentence too. It is off until the reader ticks it in Réglages, per device, never synced
(`add-lingua-translation-delivery`).

What ships, and what does not:

|                                                                                                         | Where it comes from                                                    | In the package?                                         |
| ------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------- |
| The engine — `bergamot-translator.js` + `.wasm`                                                         | built from `mozilla/translations` at the commit `engine-pin.json` pins | **yes**, every variant: Chromium, Firefox and Safari    |
| The models — `base-memory` `en→fr` 2.0, `es→en` 2.0, `en→es` 2.1, `fr→en` 2.0, `it→en` 2.0, `en→it` 2.1 | Mozilla's registry, re-served by Cymbra (`model-manifest.json`)        | **never**: downloaded once the reader ticks the setting |

The stores count WebAssembly loaded from anywhere but the package as remote code, which a Manifest
V3 extension may not run; the model is data, which they allow. So the engine is packaged and only
the model travels.

## 1. The engine

`build.mjs` refuses to build Chromium or Firefox without it, and refuses any bytes but the pinned
ones. Fetch it — from the GitHub Release `lingua-engine-build` publishes once per pin (it never
expires), or, until that release exists, from the workflow's 90-day artefact:

```bash
cd apps/lingua-extension
yarn fetch:engine          # → engine/, checked against engine-pin.json
```

Or build it from source on Linux (the upstream script warns that it breaks on macOS AArch64):
`yarn build:engine` runs `tool/build_engine.sh`, the same recipe CI and Mozilla's reviewers use.
The build is reproducible — three CI runs a day apart gave identical bytes — which is why the pin
is a hash. At the same path only: the `.wasm` embeds its sources' absolute path, so the script
builds under `/home/runner/work/cymbra/cymbra`, where CI built the pinned bytes; one directory over,
the output differs by those strings. To move it, change `translationsCommit` in `engine-pin.json`: the build names the hashes
it got, and every measurement taken on the engine (size, memory, latency, the mark) has to be
taken again.

## 2. The model

`model-manifest.json` is the whole contract, a catalogue (`generalise-lingua-translation-catalogue`):

- `models`: each model the package may download, under its id (`en-fr/base-memory/2.0`), with the
  languages it translates between, its licence, its mirror release and its three files. For each
  file: its content-addressed path under `base`, its size as served (Mozilla's gzip), its size once
  decompressed (`unpacked`), the sha256 of its **decompressed** bytes, and where Mozilla publishes it;
- `routes`: for each pair, keyed `<studied>-<native>`, the models that translate its studied language
  into its native language, in order (`generalise-lingua-translation-routes-by-pair`). The parser
  reads both languages from the key and refuses a route that does not start from the one or end in
  the other.

| Route   | Models, in order                                                        | Pair shipped?                           |
| ------- | ----------------------------------------------------------------------- | --------------------------------------- |
| `en-fr` | `en-fr/base-memory/2.0`                                                 | yes                                     |
| `es-fr` | `es-en/base-memory/2.0`, then `en-fr/base-memory/2.0` — through English | yes                                     |
| `es-en` | `es-en/base-memory/2.0`                                                 | yes, since change 34 (English speakers) |
| `en-es` | `en-es/base-memory/2.1`                                                 | yes, since change 35 (Spanish speakers) |
| `fr-en` | `fr-en/base-memory/2.0`                                                 | yes, since change 52 (French studied)   |
| `fr-es` | `fr-en/base-memory/2.0`, then `en-es/base-memory/2.1` — through English | yes, since change 52 (French studied)   |
| `it-en` | `it-en/base-memory/2.0`                                                 | not yet: the Italian programme's row 25 |
| `en-it` | `en-it/base-memory/2.1`                                                 | not yet: the Italian programme's row 45 |
| `it-fr` | `it-en/base-memory/2.0`, then `en-fr/base-memory/2.0` — through English | not yet: the Italian programme's row 20 |
| `it-es` | `it-en/base-memory/2.0`, then `en-es/base-memory/2.1` — through English | not yet: the Italian programme's row 30 |
| `fr-it` | `fr-en/base-memory/2.0`, then `en-it/base-memory/2.1` — through English | not yet: the Italian programme's row 50 |
| `es-it` | `es-en/base-memory/2.0`, then `en-it/base-memory/2.1` — through English | not yet: the Italian programme's row 54 |

English and Spanish are translated into each other directly, one model each
(`add-lingua-translation-matrix-models`). Mozilla publishes no fr↔es model. en-es is pinned at
2.1, the registry's one `Release` entry: the decompressed sha256 of its three files are those
Firefox's Remote Settings publish for en→es 2.1, the model's the registry's `uncompressedHash`.
A route is needed only by a reader whose pairs include its pair, and a reader's pairs are the
shipped pairs of their native language (`packs.json`): es-en's route is an English-speaking reader's
since change 34 ships es-en, en-es's a Spanish-speaking reader's since change 35 ships en-es, fr-en's
an English-speaking reader's of French and fr-es's a Spanish-speaking reader's since change 52 ships
them, and a French-speaking reader still holds the es-en model only as es-fr's first model.

French is translated into English directly and into Spanish through English, as Spanish is into
French (`add-lingua-french-translation`). fr-en is pinned at 2.0, the registry's one fr-en entry
(`base-memory`, `Release`, run `retrain_hr_EFgIftH_RrCyzl5gjemVNg`): 26,234,715 bytes to download,
37,200,311 on the device, the decompressed sha256 of its three files those Firefox's Remote
Settings publish for fr→en 2.0 (`translations-models-v2` lists the same bytes as 3.0, as it relabels
every model pinned here), the model's the registry's `uncompressedHash`. fr-es downloads fr-en's and
en-es's files, 51,608,069 bytes. Since change 52 lists fr-en and fr-es in `packs.json`, an
English-speaking reader of French needs fr-en's route and a Spanish-speaking one fr-es's; a
French-speaking reader, whom no pair teaches French, needs neither and fetches nothing of fr-en.

Italian is translated to and from English directly, and to and from French and Spanish through
English (`add-lingua-italian-translation-models`, the [Italian programme](../../docs/lingua/italian-programme.md)'s
change 2). Mozilla publishes it-en and en-it, and no model between Italian and French or Spanish.
it-en is pinned at 2.0 and en-it at 2.1, the registry's one entry each (`base-memory`, `Release`,
runs `retrain_hr_d1nPDwVeSD6t-HdtWA4xNg` and `retrain_hr_fix_names_GPSr_u8PSmC9fOx1ZDCHoA`) and the
newest versions Firefox's Remote Settings publish (it→en 1.0 and 2.0; en→it 1.0, 2.0 and 2.1):
it-en is 26,242,301 bytes to download and 37,088,127 on the device, en-it 26,509,835 and
36,507,703. The six files were downloaded from the registry on 2026-10-22; the decompressed sha256
of each is the one Remote Settings publish for it→en 2.0 and en→it 2.1, and each model's the
registry's `uncompressedHash`. it-fr downloads it-en's and en-fr's files (51,994,773 bytes), it-es
it-en's and en-es's (51,615,655), fr-it fr-en's and en-it's (52,744,550), es-it es-en's and
en-it's (52,750,887) — two models each, as es-fr and fr-es. No pair of `packs.json` studies Italian
or is glossed in it, so no reader's pairs need these six routes: nothing is downloaded, kept or
loaded for them until the Italian programme ships a pair — it-fr in the programme's row 20
(`enable-lingua-italian`), it-en in row 25, it-es in row 30, en-it in row 45, fr-it in row 50 and
es-it in row 54. The model host must serve both models before any package or App Store build whose
catalogue names them is released: the owner dispatches `lingua-model-deploy` right after the merge,
and until then the release gates refuse (`add-lingua-italian-translation-models` D6).

With the matrix's six routes, every native language's shipped pairs need two models — the engine's
bound (see [What the engine holds](#what-the-engine-holds)):

| Native  | Pairs        | Models       | Download               |
| ------- | ------------ | ------------ | ---------------------- |
| French  | en-fr, es-fr | en-fr, es-en | 51,993,524 B (52,0 Mo) |
| English | es-en, fr-en | es-en, fr-en | 52,475,767 B (52.5 MB) |
| Spanish | en-es, fr-es | en-es, fr-en | 51,608,069 B (51,6 MB) |

With Italian's six routes, the pairs glossed in each native language need three models — still
loaded two at a time, a route's (`add-lingua-italian-translation-models` D2). None of Italian's
pairs ships yet, so no reader holds a third model today:

| Native  | Pairs               | Models              | Download               |
| ------- | ------------------- | ------------------- | ---------------------- |
| French  | en-fr, es-fr, it-fr | en-fr, es-en, it-en | 78,235,825 B (78,2 Mo) |
| English | es-en, fr-en, it-en | es-en, fr-en, it-en | 78,718,068 B (78.7 MB) |
| Spanish | en-es, fr-es, it-es | en-es, fr-en, it-en | 77,850,370 B (77,9 MB) |
| Italian | en-it, fr-it, es-it | en-it, fr-en, es-en | 78,985,602 B (79,0 MB) |

**A file two models share.** en-es 2.1's vocabulary decompresses to the same bytes as es-en 2.0's
(`5ae254fa…58ad`; Mozilla serves two different gzip files). The device stores a file under the
sha256 of its decompressed bytes, so a reader who holds one model and then needs the other — only
once the native language can change (change 20) — does not download that file again, and pruning
keeps it while either model is kept. The setting's cost is the sum over the files of the models
the reader's pairs need, so it may count that file's 409,312 bytes although it is not fetched.
fr-en 2.0's vocabulary is en-fr 2.0's the same way (`783abf3a…002a`, from two gzip files): no
native language's pairs hold both, so it matters only when a French-native reader of English becomes
an English- or Spanish-native reader of French — the vocabulary is kept, fr-en's download fetches
25,825,009 bytes, and the setting states 26,234,715. it-en 2.0's and en-it 2.1's vocabularies are one
file the same way (`3ef0211d…2bc5`, 812,724 bytes, from two gzip files of 407,598 bytes; Remote
Settings publish one hash for both): no native language's pairs hold both either, so it matters only
when an English-native reader of Italian becomes an Italian-native reader of English, or the other
way round — the vocabulary is kept, the second model's download fetches 407,598 bytes fewer than the
setting states (26,102,237 of en-it's 26,509,835; 25,834,703 of it-en's 26,242,301).

A page asks in the document's language and never names a pair: the background forms the pair from
that language and the reader's native language, read from their stored profile, gates on it — the
device records which pairs are ready — and asks the engine for that pair's route. A pair the
catalogue lists no route for is unavailable, and the engine is not started for it. A route is
reached only once a pack glossed in that native language ships, since a reader is served the pairs
of their native language alone: change 25 lists es-en's and en-es's routes, and changes 34 and 35
ship their pairs. Marks are measured per pair (`MARKED_PAIRS`, `tool/marks/README.md`), each on
its own route and judged in its native language — es-fr's measurement says nothing of es-en's.
en-fr, es-fr, es-en, en-es, fr-en and fr-es are measured and listed; es-en and en-es were listed
ahead of their readers, inert until changes 34 and 35 shipped them, and fr-en and fr-es until
change 52 shipped them. it-fr is measured through it-en then en-fr on the Italian selections and
judged in French (the Italian programme's row 19, `measure-lingua-it-fr-translation-marks`): 92 of
its 97 shown marks correct, 3 % withheld, on the first tier — listed ahead of its readers, inert
until row 20 (`enable-lingua-italian`) lists it-fr in `packs.json`. Rows 24, 29, 38, 49 and 53
measure the other Italian pairs through their routes before they ship.

It is bundled, so the reviewed package decides what is accepted; the host only serves bytes. The
setting's cost (« Télécharge 25,8 Mo une fois ») is computed from it, and so is what the build, the
variant check, the host's assembly, its mirror releases (`tool/mirror_models.mjs`) and its check
cover: every model of the catalogue.

When the reader ticks the setting, the background asks the engine's host (the offscreen document on
Chromium, the event page on Firefox) to download: a worker of its own fetches each missing file with
no cookie and no referrer, decompresses it, and stores it in the `lingua-model` IndexedDB database
only if its sha256 matches. A file that does not — the static site's 200-with-the-home-page trap
included — is discarded. Unticking deletes the database.

The host, `models.cymbra.app`, is filled by the `lingua-model-deploy` workflow with
`tool/assemble_model_site.mjs`, which checks both digests of every file and takes it from Mozilla's
registry, else from our own copy (a GitHub Release the same workflow creates once), else from the
deployed host itself — Mozilla has moved these files once already. `tool/check_model_host.mjs`
then checks the host from outside, and `lingua-extension-release` runs the same check before it
submits a package: a package whose model cannot be downloaded is never sent to a store.

### Before that host exists, or to test a download

Serve the files yourself and point a **development** build at them:

```bash
node tool/assemble_model_site.mjs /tmp/models        # fetched from Mozilla, both digests checked
npx http-server /tmp/models -p 8765 --cors            # any static server that sends CORS
LINGUA_MODEL_BASE_URL=http://127.0.0.1:8765/ yarn build:chromium
```

`check_variants.mjs` refuses such a build, so it cannot be shipped by mistake — and a later `yarn
build` without the variable silently replaces it (same `dist-<target>/`): reload the extension
after every build.

## What you will see

Réglages → Traduction → « Traduction étendue ». Before ticking it says what it costs (25,8 Mo once,
about 200 Mo of memory while translating). The memory is that of the models the reader's pairs need
together: « environ 340 Mo » for two — a French-speaking reader of Spanish, whose route goes through
English, or an English-speaking reader of Spanish and French, two direct routes — « environ 200 Mo »
for one (enable-lingua-french D6). Ticking it shows the download's progress, with Annuler;
then « Prête ». A failure says why in French and offers Réessayer; a download whose host was torn
down says « interrompu » and offers Reprendre; a model the browser removed offers Télécharger à
nouveau. Nothing restarts on its own.

Select a fragment inside a sentence: the expression card shows the pack's answer at once, then
**« Dans votre phrase — traduction automatique »**: the sentence, translated, the fragment in the
answer colour. `gave up` comes back `a abandonné`. Click or select a word the pack cannot answer —
`disambiguation` on Wikipedia — and its card gains the same line. A word the pack glosses keeps its
dictionary card, and a proper noun outside the lexicon is never sent.

Without a model ready — off, downloading, failed, interrupted, removed — every card is exactly what
it was before the engine existed, with no line saying a translation is on its way.

## Where it runs, and why nowhere else

The engine blocks for as long as a sentence takes, so it runs in a worker of its own and never on
a thread that paints. On Chromium an offscreen document owns that worker, because a service
worker cannot construct one; on Firefox the event page does. `test/lint-translator-placement.spec.ts`
walks the import graph from every entry point that paints and fails if one can reach the
engine's host.

The worker is **classic**, not a module: Mozilla's glue assumes sloppy mode, so under
`importScripts` the artefact runs as built, unpatched.

It is loaded when a translation is coming — never when the setting is ticked, the download ends
or a page opens — and put down ten minutes after the last one, giving its ~195 MiB back; on
Chromium the offscreen document then closes too. "Coming" means one of three things
(add-lingua-translation-android): a translation asked; a selection that **begins**, so the cold
start runs while the handles move rather than after them (`SelectionWatcher.onBegin` → `warm`);
and a page that translated within those ten minutes becoming **visible again**, because Firefox
for Android tears the engine down while a tab is frozen in the background (`keepWarm`). A warm
loads the engine and translates nothing, and only with a model ready. A reading tab's keep-warm
ping keeps Firefox's event page (and the loaded engine) alive between two selections, but it is not
a translation: it never holds the engine past those ten minutes.

A page keeps the translations it has received (`answer-memory.ts`, 32 of them, never stored): the
same sentence with the same selection is not asked twice — not while the handles come back to it,
and not while the first request is still being answered.

### When the engine traps

The engine can trap under an input — a WebAssembly "memory access out of bounds", measured through
the en-es model (harden-lingua-translation-engine) — and a trapped instance poisons every model
built after it. The worker tells a trap (`WebAssembly.RuntimeError`, or the glue's abort, whose
message starts with `Aborted(`) from an ordinary refusal — no model, a route too long, not loaded
— reports it as one, and closes itself. Its owner puts it down, starts a fresh worker, loads the
route again and asks the request once more. Once: a request that traps the fresh worker too is
unavailable, and the card behaves as without an engine. Everything in flight on the worker that
trapped — a marked selection's two requests, a load for another pair, a warm — is asked again the
same way, each once. The console says what happened with the pair and the markup's length, never
the text. An ordinary refusal is passed on as it always was, and the worker is kept. The reader
pays one respawn, not ten minutes.

### What the engine holds

Two models at most, what one route needs: a route chains two models (es-fr goes through English),
and a French reader of English and Spanish holds en-fr and es-en, en-fr shared. Before a route is
loaded whose models would make a third, the worker deletes — with the memory it was built from —
the least recently used model the route does not need (a load and a translation both count as
uses), and drops every route that went through it; a route whose models are held costs nothing.
The decision is `model-residency.ts`, pure and tested apart from the engine; the worker runs loads
one at a time, so two cannot decide against a bound the other is about to move. A reader who
alternates three languages pays the third route's load each time it comes back (≈ 200 ms on a Mac,
3–4 s on the measured tablet); the bound is the measured problem, not the load. Measured: one model
is 195.4 MiB in the worker, es-fr's two 321.8 MiB — and, before the bound, the models of a language
the reader had left made 463 MiB. What the worker holds is the union of its routes, after every
load and every failed one: a route dropped with a deleted model takes its other model with it.

A deletion does not lower what the worker holds in the operating system's eyes — a wasm instance's
linear memory never shrinks — the bound caps growth, with the freed blocks reused by the next model
built. The catalogue lists six models, but every native language's shipped pairs need two at most
(the first table above), so a reader who keeps their native language never makes a third today: a Spanish-native
reader of English and French holds en-es and fr-en, en-es serving both routes; an English-native
reader of Spanish and French, es-en and fr-en. The deletion runs only when the native language
changes while the worker lives — French to Spanish deletes en-fr and es-en for fr-es's two models;
French to English deletes es-en for fr-en, then en-fr for es-en — and the worker holds two models
after every load (`test/model-residency.spec.ts`, through the committed routes). Since change 35
ships en-es it can run in production: a French-native reader who holds en-fr and es-en and chooses
Spanish needs en-es, a third, and the least recently used of the two is deleted; since change 52
the same holds of French's routes — an English-native reader of Spanish and French who chooses
Spanish keeps fr-en, which fr-es goes through, and es-en is deleted for en-es. Once Italian's pairs
ship, each native language's routes need three models (the second table above), loaded two at a
time, and a reader who keeps their native language makes the worker delete one only when a third
pair's route needs the room (`add-lingua-italian-translation-models` D2): a French-native reader of
English, Spanish and Italian keeps en-fr, which their three routes go through, es-en and it-en taking
turns beside it; a Spanish-native reader keeps en-es the same way, fr-en and it-en taking turns; an
Italian-native reader keeps en-it, fr-en and es-en taking turns; an English-native reader holds two
of es-en, fr-en and it-en. A translation goes to the engine at once over a route the worker holds; over
one it deleted since the channel loaded it, the worker answers `reload`, and the channel loads the
route again — under the start bound — and asks once more.

### Soaking a route by hand

`tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]` runs the real engine
through a pair's route over the committed corpus of its studied language, in Node, and reports the
inputs that trapped by their corpus id, the count translated, the time per sentence and the
process's memory high-water mark; without `--isolate` the run stops at the first trap, since the
instance is poisoned from then on. It is how a model is tried before it ships — en-es before
change 35, now that change 25 pins its route, fr-en and fr-es before change 52, now that change
50 routes them, and the en-it model through en-it, fr-it and es-it before any Italian speaker is
offered it, now that the Italian programme's change 2 pins it (`add-lingua-italian-translation-models`
D4), and the it-en model through it-en and it-fr over the Italian selections, which the programme's
row 19 (`measure-lingua-it-fr-translation-marks` D2) added to `tool/marks/corpus.json` (below) — and
it never runs in CI: the programme's M25 recommends a manual tool. The memory
figure is Node's RSS, not the worker's (the ≈ 322 MiB of es-fr above was measured in the browser),
and the soak says nothing of the two-model bound: a run loads one route and deletes nothing. `tool/marks/README.md` says how.

Measured on a Galaxy Tab S6 Lite (Firefox for Android, 4 GB): a cold start costs 4.1–4.7 s there
(0.2–0.3 s on a Mac), a warm translation 0.4–1 s, and the loaded engine about 180 MB.

#### en-es, soaked before it ships (2026-10-08)

`node --experimental-strip-types tool/soak_engine.mjs --pair en-es --models <dir> --isolate`, over
the 100 English selections of `tool/marks/corpus.json`, with the models assembled locally from
Mozilla's registry by `tool/assemble_model_site.mjs` (every file kept, every digest held)
(`add-lingua-translation-matrix-models` D4).

|                        |                                                                                                               |
| ---------------------- | ------------------------------------------------------------------------------------------------------------- |
| Engine                 | `engine-pin.json`: `mozilla/translations` `a6310e24669df32a9098faadccbb1206448b51cb`, `.wasm` `7ef4b3fd…3122` |
| Model                  | `en-es/base-memory/2.1`                                                                                       |
| Machine                | MacBook Pro, Apple M2 Max (12 cores), 64 GB; macOS 26.5.2 (Darwin 25.5.0, arm64); Node 22.22.2                |
| Selections             | 100 run, **100 translated, 0 trapped, 0 timed out**                                                           |
| Time per sentence      | median 145 ms, mean 149 ms, max 216 ms — each in a fresh child, its first translation                         |
| Memory high-water mark | 392.3 MiB (maxRSS, the highest child's; Node's, not the worker's)                                             |
| Whole run              | 46 s                                                                                                          |

No selection of the committed English corpus trapped en-es 2.1 in Node (V8) with this engine build,
isolated or in one instance; the browsers' workers (SpiderMonkey, JavaScriptCore) were not run, and
the study's trapping inputs and the en-es version it used were never committed. Beside it, on the same machine: en-es in one
instance (without `--isolate`, closer to the worker, which translates sentence after sentence)
translated all 100 too, median 35 ms, maxRSS 417.5 MiB; en-fr under `--isolate` gave no trap,
median 151 ms, maxRSS 397.3 MiB — in this tool, en-es costs what en-fr costs. Change 35 reads these
figures; they decide nothing here, and a trap on another input still costs the reader one respawn.

The marks measurement of change 26 (`tool/measure_marks.mjs --pair en-es` and `--pair es-en`,
2026-10-08, the same machine, engine and models) asked every selection twice — the sentence tagged,
then the fragment alone — through one engine per run, a request that traps being asked once more on
a fresh engine: no request trapped on either pair, so `tool/marks/README.md`, where a trapped
selection would be listed by its id, lists none.

#### fr-en and fr-es, soaked before they ship (2026-10-09)

`node --experimental-strip-types tool/soak_engine.mjs --pair fr-en --models <dir>` and
`--pair fr-es`, each with `--isolate` and in one instance, over the 100 French selections of
`tool/marks/corpus.json`, with en-fr, es-en, en-es and fr-en assembled locally from Mozilla's registry by
`tool/assemble_model_site.mjs` (every file kept, every digest held) (`add-lingua-french-translation`
D5). The same engine (`a6310e24…`, `.wasm` `7ef4b3fd…3122`) and machine as en-es's soak above.

|                        | fr-en `--isolate`           | fr-es `--isolate`           | fr-en, one instance        | fr-es, one instance        |
| ---------------------- | --------------------------- | --------------------------- | -------------------------- | -------------------------- |
| Models                 | `fr-en/base-memory/2.0`     | fr-en 2.0, then en-es 2.1   | `fr-en/base-memory/2.0`    | fr-en 2.0, then en-es 2.1  |
| Selections translated  | **100 / 100**               | **100 / 100**               | **100 / 100**              | **100 / 100**              |
| Trapped, timed out     | **0, 0**                    | **0, 0**                    | **0, 0**                   | **0, 0**                   |
| Time per sentence      | 146 / 150 / 230 ms          | 227 / 234 / 396 ms          | 39 / 43 / 207 ms           | 68 / 76 / 304 ms           |
| Memory high-water mark | 395.3 MiB (highest child's) | 546.0 MiB (highest child's) | 415.4 MiB (this process's) | 557.1 MiB (this process's) |
| Whole run              | 46 s                        | 74 s                        | 5.0 s                      | 9.0 s                      |

Times are median / mean / max; maxRSS is Node's, not the worker's. Beside them, in the same session:
es-fr 68 / 75 / 298 ms and 558.5 MiB in one instance, 220 / 226 / 311 ms and 547.5 MiB isolated;
en-fr 35 / 39 / 180 ms and 416.6 MiB in one instance — in this tool, French's routes cost what the
shipped ones cost, fr-es what es-fr does. The machine was not idle (other work kept its one-minute
load average between 4.8 and 7.0 during the runs); a first session under a load average near 25 gave
the same counts, memory within 7 % and median times up to twice as long. No selection of the
committed French corpus trapped fr-en 2.0 or fr-es in Node (V8) with this engine build; the
browsers' workers were not run. Change 52 reads these figures; they decide nothing here, and a trap on another input
still costs the reader one respawn.

The marks measurement of change 50 (`tool/measure_marks.mjs --pair fr-en` and `--pair fr-es`, the
same day, machine, engine and models) asked every French selection twice through one engine per
run: no request trapped on either pair, and `tool/marks/README.md` lists no trapped selection.

#### en-it, fr-it and es-it, soaked before they ship (2026-10-22)

`node --experimental-strip-types tool/soak_engine.mjs --pair en-it --models <dir> --isolate`, then
`--pair fr-it` and `--pair es-it`, each also in one instance, over the 100 English, French and
Spanish selections of `tool/marks/corpus.json` — each route's studied language's — with the six
models assembled locally from Mozilla's registry by `tool/assemble_model_site.mjs` (all eighteen
files kept, every pin held) (`add-lingua-italian-translation-models` D4). The en-it model is the new
one on every route: en-it alone, then second after fr-en and after es-en. The same engine
(`a6310e24…`, `.wasm` `7ef4b3fd…3122`) and machine as the soaks above (MacBook Pro `Mac14,6`, Apple
M2 Max, 12 cores, 64 GB; macOS 26.5.2; Node 22.22.2).

|                        | en-it `--isolate`           | fr-it `--isolate`           | es-it `--isolate`           |
| ---------------------- | --------------------------- | --------------------------- | --------------------------- |
| Corpus                 | 100 English selections      | 100 French selections       | 100 Spanish selections      |
| Models                 | `en-it/base-memory/2.1`     | fr-en 2.0, then en-it 2.1   | es-en 2.0, then en-it 2.1   |
| Selections translated  | **100 / 100**               | **100 / 100**               | **100 / 100**               |
| Trapped, timed out     | **0, 0**                    | **0, 0**                    | **0, 0**                    |
| Time per sentence      | 143 / 153 / 373 ms          | 218 / 224 / 303 ms          | 222 / 224 / 301 ms          |
| Memory high-water mark | 396.1 MiB (highest child's) | 568.2 MiB (highest child's) | 566.4 MiB (highest child's) |
| Whole run              | 46 s                        | 68 s                        | 69 s                        |

|                        | en-it, one instance        | fr-it, one instance        | es-it, one instance        |
| ---------------------- | -------------------------- | -------------------------- | -------------------------- |
| Selections translated  | **100 / 100**              | **100 / 100**              | **100 / 100**              |
| Trapped, timed out     | **0, 0**                   | **0, 0**                   | **0, 0**                   |
| Time per sentence      | 32 / 36 / 166 ms           | 66 / 74 / 328 ms           | 65 / 72 / 279 ms           |
| Memory high-water mark | 413.7 MiB (this process's) | 578.5 MiB (this process's) | 576.9 MiB (this process's) |
| Whole run              | 4.2 s                      | 8.5 s                      | 8.4 s                      |

Times are median / mean / max; maxRSS is Node's, not the worker's. Beside them, in the same session
and in one instance: en-es 33 / 36 / 168 ms and 412.8 MiB, fr-es 68 / 75 / 289 ms and 555.5 MiB —
in this tool en-it costs what en-es costs, and fr-it and es-it what fr-es costs (and es-fr, above). The
machine's one-minute load average stayed between 2.8 and 4.8 during the runs. No selection of the
committed English, French or Spanish corpus trapped en-it 2.1, fr-it or es-it in Node (V8) with this
engine build, isolated or in one instance; the browsers' workers were not run. The rows that ship
these pairs (45, 50 and 54) read these figures; they decide nothing here, and a trap on another
input still costs the reader one respawn.

The it-en model is soaked below, over the Italian selections.

#### it-en and it-fr, soaked before it-fr ships (2026-10-23)

`node --experimental-strip-types tool/soak_engine.mjs --pair it-en --models <dir> --isolate`, then
`--pair it-fr`, each also in one instance, over the 100 Italian selections of
`tool/marks/corpus.json` — which the Italian programme's row 19 (`measure-lingua-it-fr-translation-marks`
D1) selected from UD Italian-PUD by the rule every language shares — with the six models assembled
locally from Mozilla's registry by `tool/assemble_model_site.mjs` (all eighteen files kept, every pin
held) (`measure-lingua-it-fr-translation-marks` D2). The it-en model is the new one on both routes:
it-en alone, then first before en-fr. These runs close change 2's D4, which handed the it-en soak to
row 19. The same engine (`a6310e24…`, `.wasm` `7ef4b3fd…3122`) and machine as the soaks above
(MacBook Pro `Mac14,6`, Apple M2 Max, 12 cores, 64 GB; macOS 26.5.2; Node 22.22.2).

|                        | it-en `--isolate`           | it-fr `--isolate`           | it-en, one instance        | it-fr, one instance        |
| ---------------------- | --------------------------- | --------------------------- | -------------------------- | -------------------------- |
| Models                 | `it-en/base-memory/2.0`     | it-en 2.0, then en-fr 2.0   | `it-en/base-memory/2.0`    | it-en 2.0, then en-fr 2.0  |
| Selections translated  | **100 / 100**               | **100 / 100**               | **100 / 100**              | **100 / 100**              |
| Trapped, timed out     | **0, 0**                    | **0, 0**                    | **0, 0**                   | **0, 0**                   |
| Time per sentence      | 136 / 138 / 190 ms          | 217 / 220 / 314 ms          | 33 / 37 / 166 ms           | 67 / 73 / 271 ms           |
| Memory high-water mark | 389.1 MiB (highest child's) | 546.4 MiB (highest child's) | 411.5 MiB (this process's) | 555.0 MiB (this process's) |
| Whole run              | 41 s                        | 68 s                        | 5 s                        | 8 s                        |

Times are median / mean / max; maxRSS is Node's, not the worker's. Beside them, in the same session
and in one instance: es-fr 66 / 72 / 288 ms and 562.1 MiB, en-fr 34 / 37 / 181 ms and 415.1 MiB — in
this tool it-en costs what en-fr costs, and it-fr what es-fr costs. The machine's one-minute load
average stayed between 2.5 and 3.1 during the runs. No selection of the committed Italian corpus
trapped it-en 2.0 or it-fr in Node (V8) with this engine build, isolated or in one instance; the
browsers' workers were not run. Row 20 (`enable-lingua-italian`) reads these figures; they decide
nothing here, and a trap on another input still costs the reader one respawn.

The marks measurement of row 19 (`tool/measure_marks.mjs --pair it-fr`, the same day, machine,
engine and models) asked every Italian selection twice through one engine: no request trapped, and
`tool/marks/README.md` lists no trapped selection.

## What never happens

- No code is fetched: the engine is in the package.
- Nothing is downloaded for a reader who does not tick the setting. Every variant offers it the
  same way: Chromium, Firefox desktop and Android, and Safari on iPhone, iPad and Mac
  (add-lingua-translation-safari), whose event page hosts the engine as Firefox's does.
- A machine translation is never stored: no gesture carries it, so it cannot reach a card.
