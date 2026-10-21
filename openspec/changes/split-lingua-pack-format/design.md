# Design

## Context

See proposal.md — Why. What constrains the shape:

- One container envelope (`LINGUAPK`, format 1): a JSON meta, then named sections. Meaning lives in
  the reader (`packs::pack::section`), which reads a whole pack's required sections (`forms`,
  `lemmas`, `freq`, `gloss.zst`, `notice`) and the optional ones leniently.
- The sections already split by side: `forms`, `lemmas`, `freq`, `levels`, `tags`, `paradigms.zst`
  and `lexical` are built from `tables/<studied>/` alone (*A studied language's tables are kept
  once*, M24); `gloss.zst`, `senses.zst`, `expr`, `expr.zst`, `expr.names.zst`, `expr.others` and
  `notice` from `tables/<pair>/`. Measured on 2026-10-21, the studied side is byte-identical across
  natives, except English's tag pool: the builder appends the tags only senses carry, and en-es's
  senses use `NUM`, which no en-fr sense and no English reading does (413 against 417 bytes).
- `senses.zst` indexes the tag pool, so a layer's senses depend on the core's pool.
- The whole pack's `NOTICE` and `licences` are the pair's full attribution stack, both sides; the
  reducer writes them as one file, not as two halves.
- The engine holds one pack per studied language, all of one native (`PackSet`); the wasm engine
  takes bytes (`new`, `addPack`); the agent plugin loads whole files from a directory
  (`lingua-agent-capture`, M17); the goldens build their packs from the committed tables at test
  time (`tests/support/committed.rs`), and five tests embed a whole test pack (`fixtures/pack.lingua`).
- `pack_version` is the pair's (`<snapshot>+<rules>.<studied>`); the pins record the whole pack's
  sha256 and size, and `build.sh` refuses a pack that does not match.

## Goals / Non-Goals

**Goals:**

- The assembled pack is the whole pack, section for section, so the engine, the goldens, the card,
  the review and the backups see nothing new.
- A whole pack keeps loading and keeps its bytes — but en-fr's, which moves by one pool line.
- The parts cannot drift from the whole: one builder pass, one digest linking a layer to its core.
- One core per studied language, enforced by the committed-tables checks, so change 4 can ship it
  once.

**Non-Goals:**

- The package layout, the extension's fetches, the pins of the parts, `build.sh`'s split mode, the
  CI comparison, the variant check, the pack-update workflow and the source archive: change 4.
- A core that several *layers' senses* extend at load (a layer-local tag pool): refused by D5.
- Loading a layer without its core, or a core alone, as a reduced pack.
- The agent plugin's files (whole packs stay its format, M17).

## Decisions

### D1 — Two parts, one envelope, a `part` field

A core and a layer are containers of today's envelope, told apart by a `part` field in their meta
(`"core"` or `"layer"`); a whole pack's meta has none. Two metas: `CoreMeta { part, studied,
analyzer_version, core_version, levels_estimated }` and `LayerMeta { part, studied, native,
pack_version, core_version, licences }`. `Pack::load` refuses a container whose meta has `part`,
naming the part and the one it needs: a layer dropped beside `pack.lingua` tells the agent's user
what it is instead of failing on a missing `forms`.

*Alternatives.* A new format version (2) for the parts: nothing in the envelope changes, and a
version bump would make every reader of the envelope — `tool/packs.mjs` reads the meta at offset
14 — check a number that means nothing to it. Telling a part by its sections (no `native` → core):
implicit, and a layer without glosses would read as a core.

### D2 — What each part holds

Core: `forms`, `lemmas`, `freq`, then the optional `levels`, `lexical`, `tags`, `paradigms.zst`.
Layer: `gloss.zst`, then the optional `senses.zst`, `expr`, `expr.zst`, `expr.names.zst`,
`expr.others`, and `notice`. The layer's `notice` and `licences` are the pair's full stack, exactly
the whole pack's: the reducer writes one NOTICE per pair and the Attributions page shows it whole,
so splitting it into halves would invent a boundary no source has, and the assembled pack would not
be the whole pack. The core carries no notice; its licences are in the layer's.

`levels_estimated` is a property of the studied language's levels, so it lives in the core's meta,
read from the pair's manifest the core is built from; D6 holds every pair of a language to the same
value.

### D3 — `Pack::assemble(core, layer)`

Reads both metas, checks `part`, the studied language (the layer's must be the core's), the
analyser version (the core's, against the running core, as `load` does), the native language (the
layer's, as `load` does), `core_version` (D4), then builds the `Pack` from the union of the
sections — the same code path `load` takes once the sections are gathered, so the parsed pack is
the same type with the same invariants. The assembled `PackMeta` is `{ studied, native: layer,
pack_version: layer, analyzer_version: core, licences: layer, levels_estimated: core }`.
`Pack::studied_in` works on either part; `Pack::pair_in` on a layer (a core has no pair).

*Alternative.* A `Pack` that holds a reference to a shared core (one core in memory for several
layers): the engine holds one pack per studied language, all of one native, so no two packs ever
share a core in one engine — the sharing would save nothing and cost a lifetime.

### D4 — `core_version`, a digest written by the builder

The builder computes `core_version` as the sha256 of the core's bytes without its meta — the
sections' names and payloads in order, so the digest does not contain itself — writes it in the
core's meta, and copies it into the layer's. Assembly compares the two strings; no hashing at load,
nothing new in the wasm build. A layer and a core are always written by one `build_split` run over
one pair's inputs (D6 makes the core the same whichever pair's run wrote it).

*Alternative.* The studied tables' digest `pin.json` records: it names the inputs, not the bytes
the builder made of them, so a builder change would leave a stale layer assembling with a new core.

### D5 — The pool is the core's; a sense tag outside it fails the build

`tag_pool` keeps its order — the pinned pool, then the readings' tags outside it, each part sorted —
and no longer appends the tags only senses carry: a sense run whose tag is outside both fails the
build, `BuildError::SenseTagOutsidePool { tag, pool_file }`, naming `tables/<studied>/tags.tsv`.
The one case today is `NUM` in en-es's senses, pinned in `tables/en/tags.tsv` as its 28th line:
every existing id stays, en-es's pack keeps its bytes (its pool already ended with `NUM`), en-fr's
gains four bytes. The pin of en-fr is re-recorded in this change, with the owner's approval: the
English invariance baseline is on the analysis, which does not move.

*Alternatives.* A layer-local pool appended at assembly: `senses.zst` would then index two pools,
the assembled `tags` section would differ from the whole pack's, and a pack built whole would have
to reproduce the layer's order — D3's "section for section" would be lost for the one section that
is cheapest to pin. Leaving the whole-pack build as it is and enforcing the rule for the split only:
then `build_pack` and `build_split` disagree on en-fr's bytes, and the parity test (D8) is the whole
point.

### D6 — One core per studied language, checked over the committed tables

`committed_tables.rs` builds every committed pair whole and split, and asserts: the assembled parts
equal the whole pack section for section and meta for meta; the cores of one studied language are
byte-identical (`en-fr` = `en-es`, `es-fr` = `es-en`, `fr-en` = `fr-es`); each part is under
`MAX_PACK_BYTES`. A pair's manifest that disagrees with its language's reference on
`levels_estimated` or `analyzer_version` fails there, naming both pairs and the field. This is what
change 4 relies on to ship `tables/<studied>`'s core once: the invariant is proven before any
package carries it.

### D7 — The wasm engine takes parts beside whole packs

`LinguaEngine.assemble(core, layer)` (a static factory; wasm-bindgen allows one constructor, kept
for the whole pack) and `addParts(core, layer)`, both assembling through D3 then taking the path
`new` and `addPack` take. The extension keeps `new`/`addPack` until change 4; `tests/support`
gains `committed::split(pair)` and runs the English baseline through `assemble`, so S0 holds on the
parts before change 4 ships them.

### D8 — The builder's split is the whole build's second output

`build_split(inputs) -> (core, layer)` shares `build_pack`'s intermediate blobs — the FST, the
ranks, the grammar blobs, the gloss and expression sections — and writes two containers instead of
one; `build_pack` stays and is what `build.sh`, the pins and the agent keep using. The budget is
checked per part with the whole pack's remedy order (`REDUCE_EXPRESSIONS` for a layer that carries
expressions, `REDUCE_READINGS` for a core that carries paradigms, `REDUCE_GLOSSES` otherwise), and
nothing is written when either part is over. `lingua-pack-build --split <studied-dir> <pair-dir>
<core.lingua> <layer.lingua>` exposes it; `build.sh` learns it in change 4.

## Risks / Trade-offs

- [en-fr's bytes move, and so does its pin] → four bytes at the end of the pool, no id moves, the
  English baseline runs on the parts and on the whole; the owner approves the new sha256 in this
  pull request (task 1.3), as the matrix's change 6 did.
- [A reader of the envelope outside the core — `tool/packs.mjs`'s `packMeta` — meets a part] →
  nothing ships a part before change 4; `packMeta` reads `studied`, `native` and
  `analyzerVersion`, and change 4 teaches it the parts.
- [A future sense tag outside a pinned pool] → the build fails by name at the next reduction, and
  the remedy is one line in `tags.tsv` at the pool's end, which moves no id.
- [`build_pack` and `build_split` drift] → one pass, one set of intermediate blobs, and the parity
  assertion over every committed pair on every pull request.
- [Coverage] → every new path is in host-testable crates (`lingua-core`, `lingua-pack`); the wasm
  glue is excluded from the gate as before.

## Migration Plan

Nothing deploys and nothing is released: the package keeps shipping whole packs until change 4.
Merging re-records en-fr's pin; the next `lingua-pack-update` run of any pair reads the new pool
rule and fails by name if a pair's senses need a tag its language's pool lacks. Rollback is a
revert: no stored state, no backup, no server row is touched.
