# split-lingua-pack-format — a pack is built as a base per studied language and a layer per pair, assembled at load

## Why

Change 3 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 0, decision I3
(2026-10-21). A pack is one container per pair, and each container carries its studied language's
whole side — forms, lemmas, frequencies, levels, tag pool, paradigms, dictionary words — beside the
native side that is the pair's own. Measured on the six packs built from the committed tables that
day: 13,473,847 bytes, of which 7,975,182 (59 %) are studied sections, byte-identical across the
natives of one language (es-fr = es-en, fr-en = fr-es), except four bytes of English's tag pool:
a whole pack's pool ends with the tags only the pair's senses carry, and en-es's senses carry `NUM`
where en-fr's do not. Italian doubles the pairs, so the package would carry four studied sides
three times each, ≈ 27–30 MB; a base per studied language and a layer per pair carry each once,
≈ 15–18 MB, and 9.5 MB today instead of 13.5.

The matrix programme had written that a fourth language makes the split worth it; the owner first
accepted the growing package, then reversed it the same day on the measurement. This change makes
the split possible without moving what reads a pack, nor any table, pin or golden: the assembled
pack is today's pack, every named section byte for byte, and a whole pack still loads. Change 4
(`ship-lingua-pack-bases-once`) then switches the package, the pins and the loaders to the parts.

## What Changes

- **Two kinds of part, one container** (design D1, D2): a *base* holds a studied language's
  sections — its tag pool being the pinned pool and the readings' tags, nothing of any pair's
  senses — and metadata naming the language, its analyser version, a `base_version` digest and
  whether its levels are estimated; a *layer* holds a pair's sections — glosses, their sense runs
  and the tags only those senses carry, expressions, the pair's full NOTICE and licences — and
  metadata naming the pair, `pack_version` and the `base_version` it was built against. A whole
  pack keeps its metadata and its bytes.
- **Assembly at load** (D3): the engine assembles a base and a layer into the pack it loads today —
  the tag pool being the base's followed by the layer's sense tags, as a whole pack lays it out —
  refusing a layer of another studied language, a layer built against another `base_version`, and a
  part given to any reader of a whole pack, each by name. The engine, the pack set, the goldens and
  the agent plugin see no new type; a part dropped among whole packs is skipped as an unreadable
  file, never read as a pack.
- **The builder writes both** (D4, D8): one pass yields the whole pack and the two parts from the
  same intermediate tables, so they cannot drift; each part is held to the 5 MiB budget with a
  remedy of its own; `lingua-pack-build --split` writes a base and a layer.
- **Nothing in the data moves** (D5): no table, no pin, no pinned pool, no golden. The whole pack's
  pool — the pinned pool, the readings' tags, then the sense-only tags — is what the base and the
  layer reproduce between them; the sense ids a layer stores are those of the assembled pool, as
  today.
- **One base per studied language** (D6): the base built from any pair's inputs of a language is
  byte-identical to the base built from its reference pair's; the committed-tables checks fail,
  naming both pairs and the field, when not. This is the invariant change 4 ships on.
- **The wasm engine takes the parts** (D7): `assemble(base, layer)` beside `new(pack)`,
  `addParts(base, layer)` beside `addPack(pack)`. The extension keeps fetching whole packs until
  change 4.
- **The programme's wording follows**: the Italian programme called the part a *core*, the word
  the specs use for the engine crate; it is renamed *base* in the programme's decision I3, its
  Architecture, its rows 3 and 4 and its risk 7, and the Architecture bullet now says what the parts
  hold as this design fixes it (the pair's whole NOTICE in the layer, a digest of the base's bytes).
- Not in this change: the package layout, the extension's loader, `packs.json`, the pins of the
  parts, `build.sh`, `gen_pack.sh`, the variant check, the CI comparison, the pack-update workflow,
  the Apple natives derivation, the source archive (change 4); the agent plugin (M17, whole packs).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *Versioned pack container, keyed by language pair* — a pack is also
  built as a base and a layer, assembled at load into that same container's content (the
  requirement is held by the open change `add-lingua-spanish-expression-keys`, after which this
  change archives); MODIFIED *Size budget* — the budget applies to each part as to a whole pack,
  with a remedy per part. ADDED *A pack assembles from a base and a layer*; *A layer is built
  against its base*; *One base per studied language*. *A form's readings do not depend on the
  pack's native language* is untouched: the pool's order and every pinned pool stay as they are.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/packs/{meta.rs, pack.rs}`: the part metadata, `Pack::assemble`, the
  refusals; `Pack::load`, `studied_in` and `pair_in` unchanged for a whole pack and refusing a part
  by name. No analysis, no knowledge model, no backup moves.
- `crates/lingua-pack/src/{lib.rs, tags.rs, bin/lingua-pack-build.rs}`: `build_split`, the sense-tag
  block, `base_version`, the per-part budget, `--split`; `tests/committed_tables.rs` holds the six
  pairs' parity, the one-base invariant and the budget per part; `tests/pipeline_testdata.rs` the
  testdata pairs' round trip.
- `crates/lingua-wasm/src/lib.rs`: `assemble` and `addParts`; `tests/support` runs the English
  baseline through the parts.
- `scripts/lingua-data/SOURCES.md` describes a base and a layer; `docs/lingua/italian-programme.md`
  renames the part and states what it holds.
- The committed tables, every `pin.json`, the extension, the Safari host, the agent plugin, the
  workflows: untouched. The six goldens and S0 run unchanged.
