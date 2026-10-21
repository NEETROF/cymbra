# ship-lingua-pack-bases-once — the package carries each studied language's base once, and each listed pair's layer

## Why

Change 4 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 0, decision I3.
Change 3 (`split-lingua-pack-format`) lets a pack be built as a base per studied language and a
layer per pair, assembled at load into today's pack, and proves on every pull request that one
base serves every pair of a language. Nothing ships it yet: the package still carries one whole
pack per pair, each with its language's studied side — 13.5 MB for the six pairs, of which 4 MB
are the same three studied sides repeated. With Italian's six pairs the repetition would be the
larger part of ≈ 27–30 MB; carried once, the parts are ≈ 15–18 MB, and 9.5 MB today.

This change makes the package carry the parts: a base per shipped studied language, a layer per
listed pair, each fetched by the engine the first time its language is needed. It is the second
and last change the split needs before any Italian pack; after it, adding a pair adds its layer and,
for a new language, one base.

## What Changes

- **The package layout** (design D1): a layer keeps the pair's path, `assets/packs/<pair>.lingua`;
  a base lives at `assets/bases/<studied>.lingua`. The Safari host's localisation script and its
  build workflow, which read the shipped natives from the pair names under `assets/packs`, see what
  they saw.
- **The engine fetches the parts** (D2): the default pair's base and layer at start, another listed
  pair's base and layer the first time its language is needed, assembled through change 3's
  `assemble` and `addParts`; a load that fails is forgotten, as today. Each studied language's base
  is fetched once per engine, by construction: an engine holds one pair per studied language.
- **The pins record the parts** (D3): each pair's `pin.json` records the sha256 and size of its base
  and its layer beside the whole pack's; `build.sh --parts <pair> <base> <layer>` builds and checks
  both; a reader pair's base is checked equal to its reference's.
- **The build tooling** (D4): `gen_pack.sh` writes the layers and, once per studied language, the
  bases; `tool/packs.mjs` reads a part's metadata and the build's check holds each layer to its
  pair and its base, each base to its language's analyser version; the variant check holds the
  packaged files to the listed pairs' layers and their languages' bases, and the manifests expose
  the bases as they expose the packs.
- **The checks** (D5): `lingua-extension-check` compares the packaged parts with the parts built
  from the committed tables, in the monorepo and from the reviewer's source archive;
  `lingua-pack-update` records the parts when it records a pack. The whole pack stays what the
  pins also record, what the agent plugin loads, and what the measurement and the report read.
- **The test fixtures** (D6): each testdata pair gains a `tags.tsv`, the sorted pool it carries
  today, so that a test package is built as parts too; their whole packs keep their bytes.
- **The documents**: `SOURCES.md`, `REVIEWERS.md` (the reviewer's rebuild yields the parts), the
  extension's `README` where it names the pack, and the programme's row 4.
- Not in this change: the pack format (change 3); the agent plugin (M17, whole packs); any Italian
  pack; the host app's embedding, which copies the built extension whole.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *Reproducible offline build* — the tables record the sha256 of the
  base and the layer they build beside the whole pack's, every package carries those parts, and a
  build whose parts do not match fails; ADDED *A package carries each base once and each listed
  pair's layer* — the composition of a package and where each part lives. *The shipped pairs are
  one list* is held by four open changes (`enable-lingua-spanish`, `enable-lingua-english-speakers`,
  `enable-lingua-spanish-speakers`, `enable-lingua-french`) and is not modified: the added
  requirement states what the list's pairs put in the package; its scenario « that pack and no
  other » reads, once the parts ship, as that pair's layer and its language's base.
- `lingua-browser-extension`: MODIFIED *A language's pack is loaded the first time it is needed* —
  the engine starts with the default pair's base and layer and fetches another listed pair's base
  and layer on first need, every scenario kept.

This change archives after `split-lingua-pack-format`, whose requirements it ships.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension`: `src/analyzer/engine.ts` (the fetches, `assemble`, `addParts`),
  `src/analyzer/pairs.ts` (`basePath`), `tool/packs.mjs`, `tool/manifests.mjs`, `tool/gen_pack.sh`,
  `tool/check_variants.mjs`, `build.mjs`; `test/engine.spec.ts`, `test/packs.spec.ts`,
  `test/manifests.spec.ts`, `test/pairs.spec.ts`; `README.md`, `REVIEWERS.md`.
- `scripts/lingua-data`: `build.sh` (`--parts`), `pack_sources.py` (`record-build`, `check-parts`,
  the studied record), `pack_report.py` (the new pin keys), their tests; every pair's `pin.json`
  gains two records; each `testdata/<pair>/` gains `tags.tsv`; `SOURCES.md`.
- `.github/workflows/lingua-extension-check.yml` (the comparisons), `lingua-pack-update.yml` (the
  records).
- `docs/lingua/italian-programme.md`: row 4.
- Untouched: `crates/*` (change 3's engine and builder are used as they are), the Safari host, the
  agent plugin, the six goldens, every whole pack's bytes and pin record.
