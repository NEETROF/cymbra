# The translation engine — development builds only

`add-lingua-translation-engine` brings the Bergamot neural engine into the extension so a
reader's selection is answered with **their whole sentence, translated, their selection marked
in its French place**. In this change it reaches no reader: no shipped build carries it, nothing
is downloaded, and no setting exists. This page is how a developer builds it in by hand.

`tool/check_variants.mjs` refuses any trace of the engine in a shipped build — permission, file
or code. If you see it pass after following this page, you ran it on a shipped build.

## 1. The engine

The engine is compiled from `mozilla/translations` at a pinned commit by the
`lingua-engine-build` workflow. Download its artefact into a directory of your choosing:

```bash
mkdir -p ~/lingua-engine
gh run download --repo NEETROF/cymbra --name lingua-translation-engine --dir ~/lingua-engine
```

That gives `bergamot-translator.js`, `bergamot-translator.wasm` and `TRANSLATIONS_COMMIT`, the
upstream commit they were built from. Do not build the engine on a Mac: the upstream build script
itself warns that it breaks on macOS AArch64.

## 2. The model

The `en→fr` `base-memory` model (Remote Settings version `2.0`), side-loaded by hand. Never the
`tiny` variant (version `1.0`): it saves about 9 MB and its mistakes would be memorised.

Mozilla serves it through Remote Settings; the attachment ids can move, the hashes cannot:

```bash
cd ~/lingua-engine
python3 - <<'PY'
import hashlib, json, urllib.request
base = "https://firefox.settings.services.mozilla.com/v1"
cdn = json.load(urllib.request.urlopen(f"{base}/"))["capabilities"]["attachments"]["base_url"]
records = json.load(urllib.request.urlopen(f"{base}/buckets/main/collections/translations-models/records"))["data"]
want = {  # file type -> (name on disk, sha256)
    "model": ("model.bin", "6322e296d4fecfe395a8d5723da4ec37ecbe6d7613bb1dfcf4b28e2a47498b68"),
    "lex": ("lex.bin", "2585ed98d3af0bc949865aedeb390493d591f56870814376e73e4144c41ed059"),
    "vocab": ("vocab.bin", "783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a"),
}
for r in records:
    if (r.get("fromLang"), r.get("toLang"), r.get("version")) != ("en", "fr", "2.0"): continue
    name, sha = want[r["fileType"]]
    data = urllib.request.urlopen(cdn + r["attachment"]["location"]).read()
    assert hashlib.sha256(data).hexdigest() == sha, f"{name}: not the model this change was measured on"
    open(name, "wb").write(data)
    print(f"{name}: {len(data)} bytes, sha256 ok")
PY
```

36 749 127 bytes in all. `metadata.json` in `mozilla/firefox-translations-models` gives the
model's own sha256 and its FLORES scores (BLEU 49.6, COMET 0.8697).

## 3. Build it in

```bash
cd apps/lingua-extension
LINGUA_TRANSLATION_ENGINE=~/lingua-engine LINGUA_ALL_URLS=1 yarn build
node tool/check_variants.mjs --engine
```

`LINGUA_ALL_URLS=1` injects the reader on every page on Chromium, so a selection is captured
without a click on the toolbar icon first. Load `dist-chromium/` (or `dist-firefox/`) unpacked.
Safari is out of scope and carries none of it.

Without the three model files the build warns and goes on: that is how the path where the engine
is present but cannot start is exercised. Every surface must then answer as it did before.

## What you will see

Select a fragment inside a sentence. The expression card shows the pack's answer as before, and
below it **« Dans votre phrase — traduction automatique »**: the sentence, translated, the
fragment in the answer colour. `gave up` comes back `a abandonné` — conjugated and agreed with
its subject — where the fragment alone would give an infinitive.

A single word keeps its dictionary card: only a selection of several words is translated.

## Where it runs, and why nowhere else

The engine blocks for as long as a sentence takes, so it runs in a worker of its own and never on
a thread that paints. On Chromium an offscreen document owns that worker, because a service
worker cannot construct one; on Firefox the event page does. `test/lint-translator-placement.spec.ts`
walks the import graph from every entry point that paints and fails if one can reach the
engine's host.

The worker is **classic**, not a module: Mozilla's glue assumes sloppy mode, so under
`importScripts` the artefact runs as built, unpatched.

## What never happens

- Nothing is fetched by the extension. The engine and the model are files you put there.
- A machine translation is never stored: no gesture carries it, so it cannot reach a card.
- No shipped build carries any of this.
