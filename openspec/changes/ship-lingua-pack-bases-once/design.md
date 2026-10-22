# Design

## Context

See proposal.md — Why. What constrains the shape:

- Change 3 gives the builder `build_split` and `lingua-pack-build --split`, the engine
  `Pack::assemble`, the wasm engine the static factory `assemble(base, layer)` and `addParts(base,
  layer)`, and proves on every pull request that the bases of one language are byte-identical and
  every part under budget; a split needs the language's pinned pool (`tags.tsv`), and a base
  writes its `tags` whenever its language has one. Its `SOURCES.md` *Build* section is written by
  its implementation, which has not landed.
- The package: `packs.json` lists the pairs; `build.mjs` copies `assets/packs/<pair>.lingua` into
  the package and folds the list into `__LINGUA_PACKS__`; `tool/manifests.mjs` exposes each pack as
  a web-accessible resource; `tool/check_variants.mjs` holds the packaged files to exactly the
  listed pairs and no file above 8 MB; `tool/packs.mjs` reads a pack's metadata at offset 14 and
  `assertPacksMatchEngine` holds each pack's studied language, native language and analyser
  version to its pair and to lingua-core's constant; `gen_pack.sh` builds each pair through
  `build.sh` (whole packs, checked against the pin); `.gitignore` ignores `assets/packs/`.
- The engine (`src/analyzer/engine.ts`): `fetchPack(pair)` reads `packPath(pair)`; the default
  pair's pack builds the engine, another pair's is added once on first need, a failed load
  forgotten; one pack per studied language, all of one native. The TypeScript seam types
  `WasmModule.LinguaEngine` as a constructor over pack bytes and `WasmEngine` with `addPack`;
  `build.mjs`'s `assertWasmMatchesEngine` holds the glue to `WasmEngine`'s instance methods, by the
  glue's four-space-indented method lines, so a static method is outside its sight.
- The Safari host declares its app's localisations from the pair names under
  `dist-safari/assets/packs` (`tool/app_localizations.sh`, `lingua-apple-build.yml`), and copies the
  built extension whole.
- The pins: `pin.json` records `pack {sha256, size}`; `build.sh` build mode refuses a pack that
  does not match; `pack_sources.py record-build --pin --pack --reducer` writes it in the reduce,
  update and dry modes, after `build_pack`, saving a fixed tuple of keys; `check-reducer` runs
  `check_studied_tables`, which holds a reader pair's record of the studied tables to the committed
  ones; `pack_report.py` compares `pack.sha256` between two records and, under `--identical`, lets
  `pack.sha256`, `reducer.sha256` and `reducer.files` move and nothing else; `lingua-pack-update.yml`
  builds a whole pack per pair for the report and the measurement.
- `lingua-extension-check.yml` builds each pair's whole pack from the committed tables (for the
  goldens and the reducer checks), runs `yarn gen:pack` and `yarn build` on the testdata packs, and
  — from the reviewer's source archive — runs `yarn gen:pack:real` and compares each packaged pack
  with the whole pack built before. `tool/make_source_archive.sh` writes into the archive's README
  each pair's recorded `pack.sha256` against `assets/packs/<pair>.lingua`.
- The agent plugin loads whole packs from its directory (M17); `lingua-pack-measure` and
  `pack_report.py --pack` read a whole pack.
- The testdata fixtures: en-fr, en-es, es-fr and es-en carry `grammar.tsv` and `senses.tsv`; fr-en
  and fr-es carry neither; none carries `tags.tsv`. `pipeline_testdata.rs` pins the bytes of the
  four English and Spanish fixtures' whole packs, not the French ones'.

## Goals / Non-Goals

**Goals:**

- The package carries each base once and each layer once; nothing else of it moves.
- Every part the package carries is the one the committed tables build, checked on every pull
  request, recorded in the pins, and named to the store reviewer.
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
stays the list of pairs; `.gitignore` gains `assets/bases/`.

*Alternative.* One folder with `<pair>.layer.lingua` and `<studied>.base.lingua`: every reader of
the folder would learn two shapes for nothing.

### D2 — The engine fetches a base and a layer

`fetchBase(studied)` reads `basePath(studied)`, `fetchLayer(pair)` reads `packPath(pair)`; the
default pair's two parts build the engine through `assemble`, another pair's are added through
`addParts`, under the same `added` map keyed by language, so a pending load is shared and a failed
one forgotten, as today — if either fetch fails, the language's load is forgotten whole. A base is
fetched once per engine by construction: the engine holds one pair per studied language and the
map is keyed by it; no cache is needed, and the engine built for a backup of another native
language (`restore`) fetches its own parts as it fetched its own packs. The seam's types follow:
`WasmModule.LinguaEngine` gains the static `assemble(base, layer)`, `WasmEngine` gains
`addParts(base, layer)`, and `assertWasmMatchesEngine` learns the static factory — the glue
writes it as a `static` method, which its instance-method pattern does not see — so that a `pkg`
older than the extension still fails the build by name rather than at run time.

### D3 — The pins record the parts, and a reader pair's base is its reference's

`pin.json` gains `base {sha256, size}` and `layer {sha256, size}` beside `pack`. In the reduce,
update and dry modes, `build.sh` runs `lingua-pack-build --split` beside `build_pack` over the same
inputs and passes `--base` and `--layer` to `record-build`, whose saved tuple gains both keys; a
new `record-parts --pin --base --layer` writes the two records alone, `pack` untouched, which is how
the six pins are recorded in this change without a reduction. `build.sh [--testdata] --parts <pair>
<base.lingua> <layer.lingua>` builds the parts — build mode checks each file against its record and
names a mismatch; testdata mode builds the fixtures' parts (D6) — the flag read after the mode's.
`check_studied_tables`, which `check-reducer` runs, gains one line: a reader pair's `base.sha256`
must be its reference's, so a reference whose reduction moved the base names the pair left behind,
as the studied record does.

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

`lingua-extension-check.yml`: the monorepo step that builds each pair's whole pack also builds its
parts into the runner's temp folder (`build.sh --parts`), the layer as `layer-<pair>.lingua` and
the base as `base-<studied>.lingua` — a language's second pair compares its base with the first
run's and fails on a differing byte, as `gen_pack.sh` does; the reviewer-archive step, after
`yarn gen:pack:real`, compares the nine packaged files — six layers, three bases — with them, byte
for byte, where it compared six packs. `tool/make_source_archive.sh` writes into the archive's
README, for each listed pair, the recorded `layer.sha256` against `assets/packs/<pair>.lingua` and
the recorded `base.sha256` against `assets/bases/<studied>.lingua`, so the reviewer compares what
the package carries. `pack_report.py`: `packs_moved` reads `pack.sha256` and `base.sha256`;
under `--identical`, `MAY_MOVE` gains `("layer", "sha256")` alone — a re-reduction under the same
sources moves `pack_version`, which the layer carries and the base does not — and a moved base
fails naming the pair. `lingua-pack-update.yml` changes nothing visible: `build.sh` records the
parts where it records the pack; the report and the measurement keep reading the whole pack the
workflow builds. The agent plugin keeps loading whole packs, built as today.

### D6 — The fixtures gain a pin; four keep their bytes, the two French ones gain a section

Each `testdata/<pair>/` gains `tags.tsv`. en-fr, en-es, es-fr and es-en carry grammar: their pin
is the sorted union of their readings' and senses' tags, the pool their whole pack carries today,
so those four whole packs keep their bytes. The English fixtures also gloss three lemmas their forms
do not hold (`leaf`, `can`, `put`), which change 3's `build_split` refuses as a base that would
depend on its pair: each gains the form row `<lemma><TAB><lemma>` in its `forms.tsv`, which moves
no byte of its whole pack (measured on change 3's implementation). `build_split` then accepts the
four inputs. fr-en and fr-es
carry no grammar and no senses: their pin is one canonical tag, `NOUN`, which no fixture reading
or sense uses; their whole packs gain a `tags` section of that line once change 3's implementation
makes `build_pack` follow the base's gate (a base holds its pool whenever its language has a pin;
today the builder writes no grammar section when readings and senses are both empty). Change 3's
`pipeline_testdata.rs` (`the_fixtures_build_whole_as_recorded_and_split_by_their_pin`) pins the six
fixtures' whole bytes, the French ones at their pinless digests, and checks each split by its data:
refused naming `tags.tsv` while a fixture has no pin, assembled into its whole pack once it has one.
This change edits the two French rows to their new sizes and digests, and nothing else in it. No fixture carries an empty pin, so change 3's reading of an empty file is never
exercised. `yarn gen:pack` (the test packages) then builds parts as `gen:pack:real` does.

*Alternative.* Grammar tables for the French fixtures: the French baseline moved to the committed
tables, and a fixture that carries readings it never exercises would be bytes nobody reads.

### D7 — Documents

The *Build* section of `SOURCES.md` that change 3 adds (what a package carries, `--parts`,
`record-parts`, the pins' three records), `REVIEWERS.md` (the reviewer's rebuild yields bases and
layers; what to compare), the archive's README (D5), the extension's `README.md` where it names the
pack, and the programme's row 4.

### D8 — What a package weighs

Today 13.5 MB of packs; after this change 9.5 MB of parts (4.0 MB of bases, 5.5 MB of layers),
each file under the variant check's 8 MB. The store listings say nothing of size and do not move.

## Risks / Trade-offs

- [A base and a layer out of step in a package] → `assertPacksMatchEngine` holds the layer's
  `baseVersion` to its base's at build time, and the engine refuses the pair at load otherwise.
- [A language's second pair writes a different base] → `gen_pack.sh` refuses it, and
  `committed_tables.rs` (change 3) already fails on it in `cargo test`.
- [The reviewer's archive names the wrong hash] → its README is written from the pins' `layer`
  and `base` records against the paths the package carries (D5).
- [The reviewer's archive rebuild diverges] → the same `build.sh --parts` over the same committed
  tables; the comparison is byte for byte on nine files.
- [`app_localizations.sh` meets `assets/bases`] → it reads `assets/packs` only; the workflow's
  check reads the same folder.
- [A stale `src/wasm/pkg` without `assemble`] → the build guard names it (D2).
- [A whole pack dropped in `assets/packs` by habit] → the variant check refuses a packaged file
  that is not a listed layer or base, and `packMeta` names a whole pack by its absent `part`.
- [Four open enable changes hold *The shipped pairs are one list*] → this change archives after
  them (`archiveAfter`); its implementation does not wait for them — it waits for change 3's.

## Migration Plan

Merging ships nothing: the next release's packages carry the parts, built from the same committed
tables; an installed extension keeps its packs until it updates, and a reader's data holds no
pack. Rollback is a revert: the pins' `pack` records never moved, and `build.sh` without `--parts`
is today's build.
