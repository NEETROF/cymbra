# generalise-lingua-pack-build — one list of shipped pairs, a pack per pair

## Why

The engine serves several languages (#626) and the extension asks in one (#628). The
extension's build still knows one pack, at one path:
- `gen:pack` and `gen:pack:real` write `assets/pack.lingua` from the en-fr sources;
- `build.mjs` checks that file against English's analyser version, copies it, and the manifest
  exposes it by name;
- the engine fetches it by that name;
- the reviewers' source archive and its check lane name the en-fr pin.

A second pair has nowhere to go, and nothing would check its analyser version against its own
language's. This is change 8 of the Spanish programme (`docs/lingua/spanish-programme.md`), still
within R2, a silent English release. It makes the shipped pairs one list and builds a pack per
pair. The list stays `en-fr`, and a check refuses any other until Spanish is enabled.

## What Changes

- **One list of shipped pairs.** `apps/lingua-extension/packs.json` lists them (`["en-fr"]`).
  The first pair is the default language, the engine's first pack. Every tool reads the list
  through one helper module, `tool/packs.mjs`.
- **A pack per pair, at a path per pair**: `assets/packs/<pair>.lingua`. `gen:pack` builds each
  listed pair from its testdata sources, and `gen:pack:real` from its committed tables against
  its own `pin.json`. A pair missing those inputs fails with a message naming what to add.
- **`build.mjs` checks each pack against its own language.** It reads the studied language and
  analyser version from each pack's metadata, and compares them with lingua-core's version for
  that language (English: `ANALYZER_VERSION`, Spanish: `SPANISH_ANALYZER_VERSION`). It copies
  every listed pack, exposes each one in the manifest, and tells the bundle the list (an esbuild
  define).
- **The engine fetches the default pair's pack** by its new path. Loading the other pairs is
  `package-lingua-packs-per-pair`.
- **The shipped list is a gate.** `check_variants.mjs` refuses a package whose packs differ from
  the list, and a list other than `["en-fr"]`, until `enable-lingua-spanish` widens that
  expectation.
- **Review and release follow the list.**
  - The source archive carries each listed pair's tables and pin, and its README names each
    pack's sha256.
  - The check lane's reviewer rebuild compares each pack.
  - REVIEWERS.md and the README give the per-pair commands.

  The release lanes keep calling `gen:pack:real`, which now builds every listed pair.
- **English does not move.** The en-fr pack keeps its bytes, the sha256 in `pin.json`. Only its
  path inside the package changes, and what a reader sees is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The shipped pairs are one list*. Each listed pair's pack is built
  from its committed tables against its own pin, and checked against its language's analyser
  version. Every package carries exactly the listed packs, and the list is en-fr until Spanish is
  enabled. *Reproducible offline build* is not rewritten: it holds for each pack. The only open
  change on this capability (`add-lingua-expression-table`) holds another requirement.

## Impact

- **Products.** Cymbra Lingua only. In `apps/lingua-extension`, this touches:
  - the build and tools: `build.mjs`, `tool/gen_pack.sh`, `tool/check_variants.mjs`,
    `tool/make_source_archive.sh`, and a new `tool/packs.mjs` and `packs.json`;
  - `src/analyzer/engine.ts` (the pack path);
  - the manifest, `.gitignore`, the docs, and the `lingua-extension-check` reviewer rebuild.

  The Safari app ships the same build. The agent plugin keeps its own pack path. The engine, the
  pack format and the tables are unchanged. ID, Music, Live, the back office and the site are
  untouched.
- **Release.** Part of R2, a silent English release: the same en-fr pack, at a new path. A
  developer rebuilds their local pack with `yarn gen:pack` or `yarn gen:pack:real`, because the
  old `assets/pack.lingua` is no longer read.
