# The translation engine

« Traduction étendue » answers a reader's selection with **their whole sentence, translated, their
selection marked in its French place** — and a single word the pack has no gloss for, in its
sentence too. It is off until the reader ticks it in Réglages, per device, never synced
(`add-lingua-translation-delivery`).

What ships, and what does not:

|                                                 | Where it comes from                                                    | In the package?                                         |
| ----------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------- |
| The engine — `bergamot-translator.js` + `.wasm` | built from `mozilla/translations` at the commit `engine-pin.json` pins | **yes**, every variant: Chromium, Firefox and Safari    |
| The model — `en→fr` `base-memory` 2.0           | Mozilla's registry, re-served by Cymbra (`model-manifest.json`)        | **never**: downloaded once the reader ticks the setting |

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

`model-manifest.json` is the whole contract: for each of the three files, its content-addressed
path under `base`, its size as served (Mozilla's gzip), the sha256 of its **decompressed** bytes,
and where Mozilla publishes it. It is bundled, so the reviewed package decides what is accepted;
the host only serves bytes.

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

Measured on a Galaxy Tab S6 Lite (Firefox for Android, 4 GB): a cold start costs 4.1–4.7 s there
(0.2–0.3 s on a Mac), a warm translation 0.4–1 s, and the loaded engine about 180 MB.

## What never happens

- No code is fetched: the engine is in the package.
- Nothing is downloaded for a reader who does not tick the setting. Every variant offers it the
  same way: Chromium, Firefox desktop and Android, and Safari on iPhone, iPad and Mac
  (add-lingua-translation-safari), whose event page hosts the engine as Firefox's does.
- A machine translation is never stored: no gesture carries it, so it cannot reach a card.
