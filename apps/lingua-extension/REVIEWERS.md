# Cymbra Lingua — building the submitted package from this source

This archive is the human-readable source of the reviewed add-on. It is submitted because
the package is produced by a build step three times over: the JavaScript is bundled with
esbuild, `wasm/lingua_wasm_bg.wasm` is compiled from the Rust in `crates/lingua-core` and
`crates/lingua-wasm`, and `engine/bergamot-translator.wasm` and `engine/bergamot-translator.js`
— the translation engine — are compiled with Emscripten from Mozilla's own source (see
[The translation engine](#the-translation-engine)).

## What is here

| Path                                         | What it is                                                           |
| -------------------------------------------- | -------------------------------------------------------------------- |
| `apps/lingua-extension/`                     | the add-on: `src/` (TypeScript), `manifest.json`, `build.mjs`        |
| `crates/lingua-core/`                        | the analysis engine, in Rust                                         |
| `crates/lingua-wasm/`                        | the `wasm-bindgen` wrapper the add-on loads                          |
| `crates/lingua-pack/`                        | the builder that writes the language pack, in Rust                   |
| `scripts/lingua-data/`                       | the pipeline that produces the bundled language pack                 |
| `backend/*/proto/`                           | the `.proto` files the add-on's sync client is generated from        |
| `apps/lingua-extension/engine-pin.json`      | the translation engine's upstream commit and the sha256 of each file |
| `apps/lingua-extension/tool/build_engine.sh` | how the translation engine is built from that commit                 |
| `apps/lingua-extension/model-manifest.json`  | the translation models' files (address, size, sha256), and routes    |
| `Cargo.toml`                                 | the Rust workspace root, reduced to the three crates above           |
| `Cargo.lock`                                 | the dependency versions the submitted `.wasm` was built with         |

Inside the add-on, `apps/lingua-extension/vendor/foliate-js/` is third-party source, unmodified:
the EPUB renderer of the book reader ([foliate-js](https://github.com/johnfactotum/foliate-js),
MIT), vendored at the commit `vendor/VENDOR.md` records because it publishes no release. It is
bundled by the same esbuild step; its zip reader comes from the npm package `@zip.js/zip.js`
(`yarn.lock`), not from a prebuilt copy.

The add-on lives in a larger repository. This archive is cut from it by
`apps/lingua-extension/tool/make_source_archive.sh`, and every pull request rebuilds the
add-on from the archive alone, with the commands below, so that they keep working.

## Build it

Requires Node 22, Yarn (via Corepack), a stable Rust toolchain with the
`wasm32-unknown-unknown` target, `wasm-pack`, `protoc`, and Python 3.12.

```sh
cd apps/lingua-extension
corepack enable
yarn install --immutable
yarn gen:wasm     # compiles crates/lingua-wasm → src/wasm/pkg
yarn gen:proto    # generates the gRPC-web client stubs from the .proto files
pip install wordfreq
yarn gen:pack:real  # builds assets/packs/en-fr.lingua from public corpora — see the next section
tool/build_engine.sh  # builds engine/ from mozilla/translations — Linux only, see below
LINGUA_GRPC_WEB_URL=https://api.cymbra.app yarn build:firefox
```

The result is `dist-firefox/`, which is what was submitted. The archive's copy of this file
ends with the sign-in client ids the package was built with; without them the build is the
same add-on with its sign-in buttons hidden.

## The language pack

`assets/packs/en-fr.lingua` — the English→French language pack, about 1.5 MB of frequency and
translation data, the one pair `packs.json` lists. It is **generated, not authored**: the built
file is not in the archive, but everything it is built from is. `yarn gen:pack:real` runs
`scripts/lingua-data/build.sh en-fr assets/packs/en-fr.lingua` for it, which builds it from the
reduced tables in `scripts/lingua-data/tables/en-fr/` (the French glosses) and
`scripts/lingua-data/tables/en/` (English's forms, frequencies, levels, readings, pinned tag pool
and dictionary words, kept once for every pack studying English) — **offline, with no download and no Python** — and checks the
result against the sha256 recorded in `tables/en-fr/pin.json` (also printed at the end of this
README): the build fails unless it produces the very bytes the package carries.

Those tables were reduced from public corpora — Kaikki's French Wiktionary extract of English
entries, the `wordfreq` distribution, ESDB's inflections (SCOWLv2), and the CEFR-J and Octanove
vocabulary profiles — by `scripts/lingua-data/reduce-en-fr.py` and the rule modules it imports,
all in this archive: `scripts/lingua-data/reduce_common.py`, the rules every pair shares, and
`scripts/lingua-data/reduce_edition_fr.py`, the French Wiktionary's — which of its senses only point
at another word, and the notes it writes for its own readers, taken out of a gloss.
`reduce_edition_en.py` and `reduce_edition_es.py`, the English and Spanish Wiktionaries' rules, are
in the archive too, with `reduce-es-en.py` and `tables/es-en/` (Spanish glossed in English, which
reads Spanish's tables in `tables/es/` and loads the English Wiktionary's rules); they reduce no
table this package ships, and `packs.json` lists no pack built from them. So are `reduce-fr-en.py`,
`tables/fr/` and `tables/fr-en/` (French's forms and ranks, which the checks build into a pack
studying French): they reduce nothing this package carries either.
`pin.json` names each raw source at a fixed commit or snapshot, with its sha256, so the
tables can be reduced again from the same bytes (`build.sh --reduce`); the reviewer does not
need to.

`yarn gen:pack` builds a small test pack instead, which is enough to load the add-on and
exercise it.

## The translation engine

« Traduction étendue », a setting that is off until the reader turns it on, translates the
reader's selection in its sentence with Mozilla's Bergamot engine — the one Firefox Translations
uses — entirely on the device. The engine's code is **part of this package**:
`engine/bergamot-translator.js` (Emscripten's glue) and `engine/bergamot-translator.wasm`.

They are compiled from `mozilla/translations` at the commit `engine-pin.json` names (Bergamot
v0.6.0, Emscripten 3.1.8, pinned by the upstream submodules), with Mozilla's own build script and
no patch. `tool/build_engine.sh` is the whole recipe: it fetches that commit, runs
`inference/scripts/build-wasm.py`, and checks the result against the sha256 in `engine-pin.json`.
The build is reproducible — CI runs the same script and gets the same bytes — and `build.mjs`
refuses to package any engine whose files do not match those hashes. One condition: the `.wasm`
embeds the absolute path of its sources (145 times, in assertion messages), so the exact bytes
come out only when the sources sit where CI put them, `/home/runner/work/cymbra/cymbra/translations`
— the script builds there by default. Built anywhere else, the engine differs by those path strings
and nothing else. It needs Linux x86-64 (the upstream script warns that macOS AArch64 breaks it),
Python 3.11, cmake and a C++ toolchain, and takes about eight minutes.

The engine runs in a dedicated worker (`src/translate/host/engine-worker.ts`), started by the event
page only when a translation is coming — the reader asks for one, begins a selection, or returns to
a page that translated in the last ten minutes — and only once the reader has turned the setting on
and its model is on the device; it is stopped after ten idle minutes. Firefox for Android offers
the setting exactly as Firefox desktop does.

**No code is fetched.** The worker loads the glue with `importScripts` and the `.wasm` with
`fetch`, both from the package's own files. The only thing the setting downloads is models — the
network's weights, which are data: each model's three files, listed in `model-manifest.json` with
the sha256 of their contents, checked before anything uses them (`src/translate/host/model-download.ts`).
Nothing in a model is executed; it is read by the engine as a parameter file.

`model-manifest.json` lists three models, Mozilla's `base-memory` models, under the Mozilla Public
License 2.0 and redistributed unmodified:

| Model                   | Translates | Route it serves                                            |
| ----------------------- | ---------- | ---------------------------------------------------------- |
| `en-fr/base-memory/2.0` | `en→fr`    | `en-fr`; and `es-fr`, second                               |
| `es-en/base-memory/2.0` | `es→en`    | `es-fr`, first (Spanish goes through English); and `es-en` |
| `en-es/base-memory/2.1` | `en→es`    | `en-es`                                                    |

Their files come from Mozilla's translation model registry (`sourceBase` in `model-manifest.json`),
not from `mozilla/firefox-translations-models`: each file lists its path there and the sha256 of
the gzip file Mozilla serves, beside the sha256 of its decompressed bytes that the add-on checks.
The package downloads none of them until the reader turns « Traduction étendue » on, and then only the models its pairs' routes need. This
package ships the pairs `en-fr` and `es-fr` (`packs.json`), so it downloads at most `en-fr` and
`es-en`; the `es-en` and `en-es` routes serve pairs it does not ship, and nothing is downloaded for
those pairs.

## Where the add-on reaches the network

The analysis is local: the pack and the WebAssembly engines run in the browser, and no page
content ever leaves the machine. A reader who never signs in and never turns on « Traduction
étendue » makes no request at all. The book reader (`reader.html`) opens the reader's own files
from the device and requests nothing.

There are four origins, all of them in the source:

- `https://models.cymbra.app` — the translation models, downloaded once, and only after the reader
  turns « Traduction étendue » on in the settings (`src/translate/host/model-download.ts`). Each
  request asks for one file at a fixed address and carries no cookie, no referrer and nothing of
  the reader's; turning the setting off deletes the models. The addresses are in
  `model-manifest.json`.
- `https://api.cymbra.app` — the reader's own account, and only once they have signed in: the
  word statuses and review history they asked to synchronise across their devices. Set at
  build time by `LINGUA_GRPC_WEB_URL` (`src/net/transport.ts`), and it lands in the manifest's
  `host_permissions`.
- `https://accounts.google.com` and `https://appleid.apple.com` — the sign-in pages, opened in
  the browser by the reader's own click (`src/state/oidc.ts`). The add-on receives the
  provider's token through the redirect and exchanges it with the backend; it never sees a
  password.
- `https://cymbra.app` — a link in the account screen to the account-deletion page
  (`src/account/flow.ts`). A link, not a request.
