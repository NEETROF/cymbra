# Design

## Context

See proposal.md — Why. What constrains the shape:

- One container envelope (`LINGUAPK`, format 1): magic, format version, the JSON metadata with its
  length, a section count, then each section as its name and its payload, each with its length.
  Meaning lives in the reader (`packs::pack::section`): a whole pack must hold `forms`, `lemmas` and
  `freq`; `gloss.zst`, `levels`, `lexical`, the expression sections, the grammar sections and
  `notice` are read when present. The reader keeps no section order.
- The sections already split by side: `forms`, `lemmas`, `freq`, `levels`, `tags`, `paradigms.zst`
  and `lexical` are built from `tables/<studied>/` alone (*A studied language's tables are kept
  once*, M24); `gloss.zst`, `senses.zst`, `expr`, `expr.zst`, `expr.names.zst`, `expr.others` and
  `notice` from `tables/<pair>/`. The builder interleaves the two sides when it writes a whole pack.
- The tag pool (`tags`) is laid out as the studied language's pinned pool in its own order, then
  the readings' tags outside it, then the tags only the pair's senses carry, each part sorted (*A
  form's readings do not depend on the pack's native language*). Measured on 2026-10-21: en-fr's
  pool ends where en-es's has one more line, `NUM`; fr-en's and fr-es's end with the same fifteen
  bare tags their senses carry, since French's pin holds its seventy-nine reading tags and nothing
  else (*French's pinned tag pool*, an open change); Spanish's pin holds the bare tags, so es-fr's
  and es-en's pools are the pin. `senses.zst` indexes that pool.
- The whole pack's `notice` and `licences` are the pair's full attribution stack, both sides; the
  reducer writes them as one file, not as two halves.
- The engine holds one pack per studied language, all of one native (`PackSet`); the wasm engine
  takes bytes (`new`, `addPack`); the agent plugin reads every file of its directory through
  `Pack::pair_in` and keeps those it accepts, the first of a language winning, loading them later
  with `Pack::load` and skipping what fails (`lingua-agent-capture`, M17); the goldens build their
  packs from the committed tables at test time (`tests/support/committed.rs`), and five tests embed
  a whole test pack (`fixtures/pack.lingua`).
- `pack_version` is the pair's; every pin records the whole pack's sha256 and size, `pin.json`'s
  `studied` record the studied tables' digests, and `build.sh` refuses a pack that does not match.

## Goals / Non-Goals

**Goals:**

- The assembled pack is the whole pack, every named section byte for byte, so the engine, the
  goldens, the card, the review and the backups see nothing new.
- A whole pack keeps loading and keeps its bytes; no table, pin, pinned pool or golden moves.
- The parts cannot drift from the whole: one builder pass, one digest linking a layer to its base.
- One base per studied language, enforced by the committed-tables checks, so change 4 can ship it
  once.

**Non-Goals:**

- The package layout, the extension's fetches, the pins of the parts, `build.sh`'s split mode, the
  CI comparison, the variant check, the pack-update workflow and the source archive: change 4.
- Loading a layer without its base, or a base alone, as a reduced pack.
- The agent plugin's files (whole packs stay its format, M17).
- Re-pinning any tag pool: the pools are what they are, and the parts reproduce them.

## Decisions

### D1 — Two parts, one envelope, a `part` field; every whole-pack reader refuses a part

A base and a layer are containers of today's envelope, told apart by a `part` field in their
metadata (`"base"` or `"layer"`); a whole pack's metadata has none. Two metadata shapes:
`BaseMeta { part, studied, analyzer_version, base_version, levels_estimated }` and
`LayerMeta { part, studied, native, pack_version, base_version, licences }`. The reader of a whole
pack's metadata probes `part` before it deserialises `PackMeta` — a part lacks fields a whole pack
requires, so without the probe it would fail on a missing field — and refuses a part as
`PackError::NotAWholePack { part, needs }`. `Pack::load`, `Pack::studied_in` and `Pack::pair_in`
all go through it: the agent plugin, which keeps what `pair_in` accepts and loads it later, keeps
skipping a part as it skips any file it cannot read, and never files a layer as a pair's pack. The
part-aware readers are new names (`BaseMeta::read`, `LayerMeta::read`), for change 4's loaders.

*Alternatives.* A new format version (2) for the parts: nothing in the envelope changes, and a
version bump would make every reader of the envelope — `tool/packs.mjs` reads the metadata at
offset 14 — check a number that means nothing to it. Telling a part by its sections (no `native` →
base): implicit, and a layer without glosses would read as a base. Letting `pair_in` read a layer
as its pair: the agent would then file a layer as a pack and evict the whole one (the first file of
a language wins), silently.

### D2 — What each part holds

Base: `forms`, `lemmas`, `freq`, then the optional `levels`, `lexical`, `tags`, `paradigms.zst`;
its `tags` is the pinned pool followed by the readings' tags outside it, sorted — the whole pack's
pool without its last block. Layer: `gloss.zst`, then the optional `senses.zst`, `senses.tags` (the
tags only the pair's senses carry, outside the base's pool, sorted, one per line, as the whole
pack's pool ends; absent when there are none, as for es-fr and es-en), `expr`, `expr.zst`,
`expr.names.zst`, `expr.others`, and `notice`.

The layer's `notice` and `licences` are the pair's full stack, exactly the whole pack's: the
reducer writes one NOTICE per pair and the Attributions page shows it whole, so splitting it into
halves would invent a boundary no source has, and the assembled pack would not be the whole pack.
The base carries no notice; its sources' licences are in every layer's. `levels_estimated` is a
property of the studied language's levels, so it lives in the base's metadata, read from the pair's
manifest the base is built from; D6 holds every pair of a language to the same value.

### D3 — `Pack::assemble(base, layer)`, section by name

Reads both metadata, checks `part`, the studied language (the layer's must be the base's), the
analyser version (the base's, against the running engine, as `load` does), the native language
(the layer's, as `load` does), `base_version` (D4), then gathers the sections by name — the
assembled `tags` being the base's `tags` and, when the layer has one, its `senses.tags`, joined as
the whole pack's pool is encoded — into the path `load` takes once its sections are gathered, so
the parsed pack is the same type with the same invariants. Parity with the whole pack is by name:
the builder interleaves the sides in a whole container and lays the parts out in another order, so
the container bytes differ while every named section's bytes are the same, and `load` keeps no
order. The assembled `PackMeta` is `{ studied, native: layer, pack_version: layer,
analyzer_version: base, licences: layer, levels_estimated: base }`.

*Alternative.* A `Pack` that holds a reference to a shared base (one base in memory for several
layers): the engine holds one pack per studied language, all of one native, so no two packs ever
share a base in one engine — the sharing would save nothing and cost a lifetime.

### D4 — `base_version`, a digest any reader can recompute

The builder computes `base_version` as the sha256 of the base container's bytes from its section
count to its end — the count, then each section's name length, name, payload length and payload,
in the container's order — so the digest contains no metadata and a reader that has the file
(change 4's `tool/packs.mjs`, which already finds the metadata's end) can recompute it. It is
written in the base's metadata and copied into the layer's; assembly compares the two strings, no
hashing at load, nothing new in the wasm build. A layer and a base are always written by one
`build_split` run over one pair's inputs (D6 makes the base the same whichever pair's run wrote it).

*Alternative.* The studied tables' digest `pin.json` records: it names the inputs, not the bytes
the builder made of them, so a builder change would leave a stale layer assembling with a new base.

### D5 — The pool is reproduced, not re-pinned

Today's `tag_pool` and its order stay: the pinned pool, the readings' tags outside it, the tags
only the pair's senses carry. `build_split` computes the same pool, writes its first two blocks as
the base's `tags` and its last block as the layer's `senses.tags`, and stores the sense ids of
`senses.zst` as indices of the whole pool, as today. So the base depends on the studied tables
alone — the same base whichever pair of the language built it — and the assembled pool is the
whole pack's, byte for byte. No pinned pool changes, no pin moves, no requirement on a pool moves.

*Alternatives.* Pinning the sense-only tags in each language's `tags.tsv` so the base holds the
whole pool: French's pin is specified as its readings' tags and nothing else, in byte order (an
open change holds it, and the committed-tables checks hold the order), the fifteen bare tags its
pairs' senses carry would have to be pinned out of order, en-fr's and en-es's pins and bytes would
move, and `pin.json`'s record of the studied tables with them. Refusing a sense tag outside the
pin: fr-en and fr-es carry fifteen. Neither buys anything the split needs.

### D6 — One base per studied language, checked over the committed tables

`committed_tables.rs` builds every committed pair whole and split, and asserts: the assembled parts
equal the whole pack section by name and metadata for metadata; the bases of one studied language
are byte-identical (`en-fr` = `en-es`, `es-fr` = `es-en`, `fr-en` = `fr-es`); each part is under
`MAX_PACK_BYTES`. A pair's manifest that disagrees with its language's reference on
`levels_estimated` or `analyzer_version` fails there, naming both pairs and the field. This is what
change 4 relies on to ship `tables/<studied>`'s base once: the invariant is proven before any
package carries it.

### D7 — The wasm engine takes parts beside whole packs

`LinguaEngine.assemble(base, layer)` (a static factory; wasm-bindgen allows one constructor, kept
for the whole pack) and `addParts(base, layer)`, both assembling through D3 then taking the path
`new` and `addPack` take. The extension keeps `new`/`addPack` until change 4; `tests/support`
gains `committed::split(pair)` and runs the English baseline through `assemble`, so S0 holds on the
parts before change 4 ships them.

### D8 — The builder's split is the whole build's second output

`build_split(inputs) -> (base, layer)` shares `build_pack`'s intermediate blobs — the FST, the
ranks, the grammar blobs, the gloss and expression sections — and writes two containers instead of
one; `build_pack` stays and is what `build.sh`, the pins and the agent keep using. The budget is
checked per part, with a remedy the part carries: a layer names its expression table when it has
one (longest entries, then rarest), else its glossed lemmas; a base names the readings of its
rarest dictionary forms. A base without readings is under the budget by construction at today's
lemma counts (the largest, Spanish's, is 1.6 MB with its readings) and, should it ever exceed it,
fails naming its lemma count, the only lever left once the FST and the frequencies are what every
analysis depends on. Nothing is written when either part is over. `lingua-pack-build --split
<studied-dir> <pair-dir> <base.lingua> <layer.lingua>` exposes it — a testdata folder holds both
sides and is passed twice; `build.sh` learns it in change 4.

## Risks / Trade-offs

- [A reader of the envelope outside the engine — `tool/packs.mjs`'s `packMeta` — meets a part] →
  nothing ships a part before change 4; `packMeta` reads `studied`, `native` and
  `analyzerVersion`, and change 4 teaches it the parts.
- [`build_pack` and `build_split` drift] → one pass, one set of intermediate blobs, and the parity
  assertion over every committed pair on every pull request.
- [A layer's sense ids assume a base of the same pool] → `base_version` ties the layer to the base
  it was built with; a layer never assembles with another.
- [The agent plugin meets a part] → refused by `pair_in` as by `load`, skipped as today's unreadable
  files are.
- [Coverage] → every new path is in host-testable crates (`lingua-core`, `lingua-pack`); the wasm
  glue is excluded from the gate as before.

## Migration Plan

Nothing deploys, nothing is released and no data moves: the package keeps shipping whole packs
until change 4, every table and pin is as it was, and the goldens are not re-blessed. Rollback is a
revert: no stored state, no backup, no server row is touched.
