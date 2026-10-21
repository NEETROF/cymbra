# Tasks

## 1. The pool is the studied language's (D5)

- [ ] 1.1 `tag_pool` no longer appends the tags only senses carry; a sense run tagged outside the pinned pool and the readings' tags fails with `BuildError::SenseTagOutsidePool { tag, pool_file }` naming `tables/<studied>/tags.tsv`; unit tests in `crates/lingua-pack/src/tags.rs` cover the refusal by name and the unchanged order (pinned, then readings' tags, each sorted), and `cargo test -p lingua-pack` passes.
- [ ] 1.2 Append `NUM` to `scripts/lingua-data/tables/en/tags.tsv` as its last line; verify with `bash scripts/lingua-data/build.sh en-es /tmp/en-es.lingua` that en-es's pack keeps the sha256 its pin records, and that en-fr's pool, read from a rebuilt pack, is its former pool with `NUM` as id 27.
- [ ] 1.3 Re-record en-fr's pin (`pack.sha256`, `pack.size` = former size + 4) with `pack_sources.py record_build`; verify `bash scripts/lingua-data/build.sh en-fr /tmp/en-fr.lingua` passes and `cargo test -p lingua-wasm --test english_baseline` passes without re-blessing. [owner] The owner approves the moved sha256 in this pull request.
- [ ] 1.4 `SOURCES.md`: the pool rule (pinned pool and readings' tags only; a sense tag outside them is pinned, at the pool's end), and `NUM` in English's pool since this change; verify the paragraph names the error and the remedy.

## 2. The core assembles a core and a layer (D1–D4)

- [ ] 2.1 `packs/meta.rs`: `CoreMeta` and `LayerMeta` with their `part` field, and `PackMeta` refusing a meta that carries `part` at `Pack::load` (`PackError::NotAWholePack { part, needs }`); unit tests round-trip each meta and pin that a whole pack's meta JSON is unchanged byte for byte.
- [ ] 2.2 `packs/pack.rs`: `Pack::assemble(core, layer)` gathering both parts' sections into the path `load` takes after reading its sections, with the refusals of D3 (another studied language, `core_version` mismatch, a part given to `load`, an unknown or studied-equal native) each a named `PackError`; `studied_in` reads either part, `pair_in` a layer; unit tests build a test core and layer in memory with `write_container` and assert the assembled pack equals the whole pack built from the same sections (sections, meta, gloss, rank, grammar), and each refusal by name.
- [ ] 2.3 `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure; `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` pass.

## 3. The builder writes the parts (D4, D6, D8)

- [ ] 3.1 `lingua-pack`: `build_split(inputs) -> Result<SplitPack { core, layer }, BuildError>` sharing `build_pack`'s intermediate blobs, writing the core's `core_version` (sha256 of the core's sections, names and payloads in order) into both metas, and checking each part against `MAX_PACK_BYTES` with the whole pack's remedy order; unit tests assert the parts assemble into a pack equal to `build_pack`'s on the testdata inputs, that an over-budget part fails naming the part and its remedy, and that `build_pack`'s bytes are unchanged on the testdata pairs.
- [ ] 3.2 `bin/lingua-pack-build.rs`: `--split <studied-dir> <pair-dir> <core.lingua> <layer.lingua>`, writing neither file when either part fails; verify the usage line and a run over `scripts/lingua-data/testdata/en-fr` that writes both files.
- [ ] 3.3 `tests/committed_tables.rs`: for every committed pair, the assembled parts equal the whole pack section for section and meta for meta; the cores of one studied language are byte-identical (en, es, fr), with a failure naming both pairs and the field on a disagreeing `levels_estimated` or `analyzer_version`; every part is under budget; `cargo test -p lingua-pack --test committed_tables` passes on the committed tables.
- [ ] 3.4 `tests/pipeline_testdata.rs`: every testdata pair builds split, assembles, and round-trips through the reader as the whole pack does; `cargo test -p lingua-pack` passes.

## 4. The wasm engine takes the parts (D7)

- [ ] 4.1 `lingua-wasm/src/lib.rs`: `LinguaEngine.assemble(core, layer)` and `addParts(core, layer)`, each assembling through `Pack::assemble` then taking `new`'s and `addPack`'s path; an assembly error reaches JavaScript as the named error; `wasm-pack build` (`yarn gen:wasm` in `apps/lingua-extension`) succeeds.
- [ ] 4.2 `tests/support/committed.rs`: `split(pair)` building the parts from the committed tables; `tests/english_baseline.rs` runs S0 once through `assemble(core, layer)` beside the whole pack and asserts the same golden; `tests/languages.rs` adds es-fr through `addParts` onto an engine built from en-fr's parts and asserts the pair set and a Spanish analysis equal the whole-pack engine's; `cargo test -p lingua-wasm` passes with no golden re-blessed.

## 5. Documentation and integration

- [ ] 5.1 `SOURCES.md` — *Build*: what a core and a layer hold, `core_version`, the per-part budget, and that the package still ships whole packs until `ship-lingua-pack-cores-once`; verify the section names `lingua-pack-build --split` as implemented.
- [ ] 5.2 `docs/lingua/italian-programme.md`: row 3's status names this change's pull request and the en-fr pin move; verify the programme table still parses (one row per change, numbers contiguous).
- [ ] 5.3 Integration: `cargo test --workspace` passes; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes; `cd apps/lingua-extension && yarn gen:pack:real && yarn test` passes with the extension untouched (whole packs); the six goldens and S0 pass unchanged.
