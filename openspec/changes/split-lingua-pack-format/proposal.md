# split-lingua-pack-format — a pack is built as a core per studied language and a layer per pair, assembled at load

## Why

Change 3 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 0, decision I3
(2026-10-21). A pack is one container per pair, and each container carries its studied language's
whole side — forms, lemmas, frequencies, levels, tag pool, paradigms, dictionary words — beside the
native side that is the pair's own. Measured on the six packs built from the committed tables that
day: 13,473,847 bytes, of which 7,975,182 (59 %) are studied sections, byte-identical across the
natives of one language (es-fr = es-en, fr-en = fr-es; en-fr = en-es but for one tag of the pool).
Italian doubles the pairs, so the package would carry four studied sides three times each,
≈ 27–30 MB; a core per studied language and a layer per pair carry each once, ≈ 15–18 MB, and
9.5 MB today instead of 13.5.

The matrix programme had written that a fourth language makes the split worth it; the owner first
accepted the growing package, then reversed it the same day on the measurement. This change makes
the split possible without moving what reads a pack: the assembled pack is today's pack, section for
section, and a whole pack still loads. Change 4 (`ship-lingua-pack-cores-once`) then switches the
package, the pins and the loaders to the parts.

## What Changes

- **Two kinds of part, one container** (design D1, D2): a *core* holds a studied language's
  sections and a meta naming the language, its analyser version, a `core_version` digest and whether
  its levels are estimated; a *layer* holds a pair's sections — glosses, senses, expressions, the
  pair's full NOTICE and licences — and a meta naming the pair, `pack_version` and the `core_version`
  it was built against. A whole pack keeps its meta and its bytes.
- **Assembly at load** (D3): the core assembles a core and a layer into the pack it loads today,
  refusing a layer of another studied language, a layer built against another `core_version`, and a
  part loaded alone as a whole pack — each by name. The engine, the pack set, the goldens and the
  agent plugin see no new type.
- **The builder writes both** (D4): one pass yields the whole pack and the two parts from the same
  intermediate tables, so they cannot drift; each part is held to the 5 MiB budget with the same
  remedies; `lingua-pack-build --split` writes a core and a layer.
- **A layer's sense tags come from its core's pool** (D5): the pool of a core is the studied
  language's pinned pool and its readings' tags; a layer whose senses carry a tag outside it fails
  the build, naming the tag and the studied language's `tags.tsv`. English's pinned pool gains `NUM`
  — the one tag en-es's senses carry and no English reading does — so that en-es's layer builds and
  the whole en-fr pack equals its assembly. **en-fr's pack moves by four bytes** (its pool gains a
  28th line, after every tag it had: no id moves, no analysis moves, S0 holds); its pin records the
  new sha256 and size, with the owner's approval in this pull request.
- **One core per studied language** (D6): the core built from any pair's inputs of a language is
  byte-identical to the core built from its reference pair's; the committed-tables checks fail,
  naming both pairs and the field, when not. This is the invariant change 4 ships on.
- **The wasm engine takes the parts** (D7): `assemble(core, layer)` beside `new(pack)`,
  `addParts(core, layer)` beside `addPack(pack)`. The extension keeps fetching whole packs until
  change 4.
- Not in this change: the package layout, the extension's loader, `packs.json`, the pins of the
  parts, `build.sh`, `gen_pack.sh`, the variant check, the CI comparison, the pack-update workflow,
  the Apple natives derivation, the source archive (change 4); the agent plugin (M17, whole packs).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: *Versioned pack container, keyed by language pair* — a pack is also built as
  a core and a layer, assembled at load into that same container's content; *Size budget* — the
  budget applies to each part as to a whole pack; *A form's readings do not depend on the pack's
  native language* — a studied language's pool is its pinned pool and its readings' tags, a sense
  tag outside it is refused rather than appended, and English's pinned pool holds `NUM`. ADDED: *A
  pack assembles from a core and a layer*; *A layer is built against its core*; *One core per
  studied language*.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/packs/{meta.rs, pack.rs}`: the part metas, `Pack::assemble`, the
  refusals; `Pack::load` unchanged for a whole pack. No analysis, no knowledge model, no backup
  moves.
- `crates/lingua-pack/src/{lib.rs, tags.rs, bin/lingua-pack-build.rs}`: `build_split`, the per-part
  budget, the sense-tag rule, `--split`; `tests/committed_tables.rs` holds the six pairs' parity,
  the one-core invariant and the budget per part.
- `crates/lingua-wasm/src/lib.rs`: `assemble` and `addParts`; `tests/support` runs the English
  baseline through the parts.
- `scripts/lingua-data/tables/en/tags.tsv` gains `NUM`; `tables/en-fr/pin.json` records the moved
  pack; `SOURCES.md` describes a core and a layer.
- The extension, the Safari host, the agent plugin, the workflows and the pins of the other pairs:
  untouched. The six goldens and S0 run unchanged.
