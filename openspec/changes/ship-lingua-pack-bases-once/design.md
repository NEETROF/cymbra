# Design

## Context

See proposal.md — Why. What constrains the shape:

- Change 3 gives the builder `build_split` and `lingua-pack-build --split`, the engine
  `Pack::assemble`, the wasm engine `assemble(base, layer)` and `addParts(base, layer)`, and proves
  on every pull request that the bases of one language are byte-identical and every part under
  budget; a split needs the language's pinned pool, which the testdata fixtures lack.
- The package: `packs.json` lists the pairs; `build.mjs` copies `assets/packs/<pair>.lingua` into
  the package and folds the list into `__LINGUA_PACKS__`; `tool/manifests.mjs` exposes each pack as
  a web-accessible resource; `tool/check_variants.mjs` holds the packaged files to exactly the
  listed pairs and no file above 8 MB; `tool/packs.mjs` reads a pack's metadata at offset 14 and
  `assertPacksMatchEngine` holds each pack's studied language, native language and analyser
  version to its pair and to lingua-core's constant; `gen_pack.sh` builds each pair through
  `build.sh` (whole packs, checked against the pin).
- The engine (`src/analyzer/engine.ts`): `fetchPack(pair)` reads `packPath(pair)`; the default
  pair's pack builds the engine, another pair's is added once on first need, a failed load
  forgotten; one pack per studied language, all of one native.
- The Safari host declares its app's localisations from the pair names under
  `dist-safari/assets/packs` (`tool/app_localizations.sh`, `lingua-apple-build.yml`), and copies the
  built extension whole.
- The pins: `pin.json` records `pack {sha256, size}`; `build.sh` build mode refuses a pack that
  does not match; `pack_sources.py record-build` writes it in update and reduce modes, and
  `check_studied_tables` holds a reader pair's record of the studied tables to the committed ones;
  `pack_report.py` reports a pin's moved keys; `lingua-pack-update.yml` builds a whole pack per
  pair for the report and the measurement.
- `lingua-extension-check.yml` builds each pair's pack from the committed tables, then compares
  the package's with it, twice: in the monorepo and from the reviewer's source archive.
- The agent plugin loads whole packs from its directory (M17); `lingua-pack-measure` and
  `pack_report.py --pack` read a whole pack.

## Goals / Non-Goals

**Goals:**

- The package carries each base once and each layer once; nothing else of it moves.
- Every part the package carries is the one the committed tables build, checked on every pull
  request and recorded in the pins.
- The host app, the agent plugin, the measurement and the report keep reading what they read.

**Non-Goals:**

- The format (change 3).
- Fetching a base from the host or a server: the parts are embedded, as the packs were.
- A pin without the whole pack's record: the whole pack is still built, recorded and used.

## Decisions

### D1 — Layers keep the pair's path; bases get their own folder

`assets/packs/<pair>.lingua` holds the pair's layer; `assets/bases/<studied>.lingua` the base. Two
things read pair names under `assets/packs`: the host app's localisation script, which declares the
app's languages from the natives it finds there, and its build workflow's check of the same. With
the layer at the pair's path, both see the pair names they saw; a base file beside them, named
without a dash, would have failed the script's « `<studied>-<native>` » rule. `__LINGUA_PACKS__`
stays the list of pairs.

*Alternative.* One folder with `<pair>.layer.lingua` and `<studied>.base.lingua`: every reader of
the folder would learn two shapes for nothing.

### D2 — The engine fetches a base and a layer

`fetchBase(studied)` reads `basePath(studied)`, `fetchLayer(pair)` reads `packPath(pair)`; the
default pair's two parts build the engine through `assemble`, another pair's are added through
`addParts`, under the same `added` map keyed by language, so a pending load is shared and a failed
one forgotten, as today — if either fetch fails, the language's load is forgotten whole. A base is
fetched once per engine by construction: the engine holds one pair per studied language and the
map is keyed by it; no cache is needed, and the engine built for a backup of another native
language (`restore`) fetches its own parts as it fetched its own packs.

### D3 — The pins record the parts, and a reader pair's base is its reference's

`pin.json` gains `base {sha256, size}` and `layer {sha256, size}` beside `pack`; `record-build`
writes the three from one `build_split` and one `build_pack` over the same inputs, in the update and
reduce modes `build.sh` already runs it in. `build.sh --parts <pair> <base.lingua> <layer.lingua>`
builds the parts in build mode (`lingua-pack-build --split`) and refuses either file that does not
match its record, naming it; in testdata mode it builds the fixtures' parts (D6).
`check_studied_tables` gains one line: a reader pair's `base.sha256` must be its reference's, so a
reference whose reduction moved the base names the pair left behind, as the studied record does.
The six pins are re-recorded in this change with their two new keys; `pack` does not move.

*Alternative.* Recording the base in `tables/<studied>/studied.json` once: the pair's pin is what
`build.sh` checks and what `pack_report.py` diffs, and a pin that names what its pair ships is
read in one place.

### D4 — The build tooling reads parts

`tool/packs.mjs`: `baseFile(studied)`, `shippedStudied(pairs)` (the listed pairs' studied
languages, once each, in order), `packMeta` reads `part` and `baseVersion` beside `studied`,
`native` and `analyzerVersion` — the metadata still starts at offset 14 — and
`assertPacksMatchEngine` holds each layer's `studied` and `native` to its pair and its
`baseVersion` to its base's, and each base's `analyzerVersion` to lingua-core's constant for its
language. `gen_pack.sh` builds, per pair, the layer into `assets/packs/` and the base into
`assets/bases/` through `build.sh --parts`; a language's second pair rebuilds its base and the
script refuses a byte that differs (change 3's invariant, checked where the files are written).
`build.mjs` copies both folders; `tool/manifests.mjs` exposes the bases after the layers;
`tool/check_variants.mjs` holds the packaged files to the listed pairs' layers and their languages'
bases, no whole pack among them, each under 8 MB.

### D5 — The checks compare parts; the whole pack stays for what reads it

`lingua-extension-check.yml` builds each pair's parts into the runner's temp folder with
`build.sh --parts` beside the whole pack it builds today, and after `yarn gen:pack:real` compares
each packaged layer and base with them, byte for byte, in the monorepo and from the reviewer's
source archive. `lingua-pack-update.yml` changes nothing visible: `build.sh` records the parts
where it records the pack; the report and the measurement keep reading the whole pack the
workflow builds. The agent plugin keeps loading whole packs, built as today.

### D6 — The fixtures gain a pin, and keep their bytes

Each `testdata/<pair>/` gains `tags.tsv`: the sorted union of its readings' and senses' tags, the
pool its whole pack carries today, so that the whole pack's bytes do not move and `build_split`
accepts the inputs. `yarn gen:pack` (the test packages) then builds parts as `gen_pack:real` does,
and `pipeline_testdata.rs` holds the fixtures' whole packs to their former bytes.

### D7 — Documents

`SOURCES.md` (*Build*: what a package carries, `--parts`, the pins' three records),
`REVIEWERS.md` (the reviewer's rebuild yields bases and layers; what to compare), the extension's
`README.md` where it names the pack, and the programme's row 4.

### D8 — What a package weighs

Today 13.5 MB of packs; after this change 9.5 MB of parts (4.0 MB of bases, 5.5 MB of layers),
each file under the variant check's 8 MB. The store listings say nothing of size and do not move.

## Risks / Trade-offs

- [A base and a layer out of step in a package] → `assertPacksMatchEngine` holds the layer's
  `baseVersion` to its base's at build time, and the engine refuses the pair at load otherwise.
- [A language's second pair writes a different base] → `gen_pack.sh` refuses it, and
  `committed_tables.rs` (change 3) already fails on it in `cargo test`.
- [The reviewer's archive rebuild diverges] → the same `build.sh --parts` over the same committed
  tables; the comparison is byte for byte on both parts.
- [`app_localizations.sh` meets `assets/bases`] → it reads `assets/packs` only; the workflow's
  check reads the same folder.
- [A whole pack dropped in `assets/packs` by habit] → the variant check refuses a packaged file
  that is not a listed layer or base, and `packMeta` names a whole pack by its absent `part`.

## Migration Plan

Merging ships nothing: the next release's packages carry the parts, built from the same committed
tables; an installed extension keeps its packs until it updates, and a reader's data holds no
pack. Rollback is a revert: the pins' `pack` records never moved, and `build.sh` without `--parts`
is today's build.
