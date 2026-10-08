# The translation engine

« Traduction étendue » answers a reader's selection with **their whole sentence, translated, their
selection marked in its French place** — and a single word the pack has no gloss for, in its
sentence too. It is off until the reader ticks it in Réglages, per device, never synced
(`add-lingua-translation-delivery`).

What ships, and what does not:

|                                                                  | Where it comes from                                                    | In the package?                                         |
| ---------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------- |
| The engine — `bergamot-translator.js` + `.wasm`                  | built from `mozilla/translations` at the commit `engine-pin.json` pins | **yes**, every variant: Chromium, Firefox and Safari    |
| The models — `base-memory` `en→fr` 2.0, `es→en` 2.0, `en→es` 2.1 | Mozilla's registry, re-served by Cymbra (`model-manifest.json`)        | **never**: downloaded once the reader ticks the setting |

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

| Route   | Models, in order                                                        | Pair shipped?                         |
| ------- | ----------------------------------------------------------------------- | ------------------------------------- |
| `en-fr` | `en-fr/base-memory/2.0`                                                 | yes                                   |
| `es-fr` | `es-en/base-memory/2.0`, then `en-fr/base-memory/2.0` — through English | yes                                   |
| `es-en` | `es-en/base-memory/2.0`                                                 | not yet: change 34 (English speakers) |
| `en-es` | `en-es/base-memory/2.1`                                                 | not yet: change 35 (Spanish speakers) |

English and Spanish are translated into each other directly, one model each
(`add-lingua-translation-matrix-models`). Mozilla publishes no fr↔es model. en-es is pinned at
2.1, the registry's one `Release` entry: the decompressed sha256 of its three files are those
Firefox's Remote Settings publish for en→es 2.1, the model's the registry's `uncompressedHash`.
A route is needed only by a reader whose pairs include its pair, and a reader's pairs are the
shipped pairs of their native language (`packs.json`): until changes 34 and 35 ship es-en and
en-es, no reader needs either route, and the es-en model is held only as es-fr's first model.

**A file two models share.** en-es 2.1's vocabulary decompresses to the same bytes as es-en 2.0's
(`5ae254fa…58ad`; Mozilla serves two different gzip files). The device stores a file under the
sha256 of its decompressed bytes, so a reader who holds one model and then needs the other — only
once the native language can change (change 20) — does not download that file again, and pruning
keeps it while either model is kept. The setting's cost is the sum over the files of the models
the reader's pairs need, so it may count that file's 409,312 bytes although it is not fetched.

A page asks in the document's language and never names a pair: the background forms the pair from
that language and the reader's native language, read from their stored profile, gates on it — the
device records which pairs are ready — and asks the engine for that pair's route. A pair the
catalogue lists no route for is unavailable, and the engine is not started for it. A route is
reached only once a pack glossed in that native language ships, since a reader is served the pairs
of their native language alone: change 25 lists es-en's and en-es's routes, and changes 34 and 35
ship their pairs. Marks are measured per pair (`MARKED_PAIRS`, `tool/marks/README.md`): es-fr's
measurement says nothing of es-en's.

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
about 200 Mo of memory while translating). Ticking it shows the download's progress, with Annuler;
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
built. The catalogue lists three models, but today's shipped pairs need two at most (en-fr and
es-en, for a reader of French with English and Spanish), so no route makes a third and the deletion
never runs in production: it is for the matrix's readers, once changes 34 and 35 ship es-en and
en-es. A translation goes to the engine at once over a route the worker holds; over one it deleted
since the channel loaded it, the worker answers `reload`, and the channel loads the route again —
under the start bound — and asks once more.

### Soaking a route by hand

`tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]` runs the real engine
through a pair's route over the committed corpus of its studied language, in Node, and reports the
inputs that trapped by their corpus id, the count translated, the time per sentence and the
process's memory high-water mark; without `--isolate` the run stops at the first trap, since the
instance is poisoned from then on. It is how a model is tried before it ships — en-es before
change 35, now that change 25 pins its route — and it never runs in CI: the programme's M25
recommends a manual tool. The memory figure is Node's RSS, not the worker's (the ≈ 322 MiB of es-fr
above was measured in the browser), and the soak says nothing of the two-model bound: a run loads
one route and deletes nothing. `tool/marks/README.md` says how.

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

The committed English corpus holds no input that traps en-es: the study's trapping inputs were not
committed, and none of these 100 selections is one. Beside it, on the same machine: en-es in one
instance (without `--isolate`, closer to the worker, which translates sentence after sentence)
translated all 100 too, median 35 ms, maxRSS 417.5 MiB; en-fr under `--isolate` gave no trap,
median 151 ms, maxRSS 397.3 MiB — in this tool, en-es costs what en-fr costs. Change 35 reads these
figures; they decide nothing here, and a trap on another input still costs the reader one respawn.

## What never happens

- No code is fetched: the engine is in the package.
- Nothing is downloaded for a reader who does not tick the setting. Every variant offers it the
  same way: Chromium, Firefox desktop and Android, and Safari on iPhone, iPad and Mac
  (add-lingua-translation-safari), whose event page hosts the engine as Firefox's does.
- A machine translation is never stored: no gesture carries it, so it cannot reach a card.
