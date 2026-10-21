# Tasks

## 1. The pins and the scripts (D3, D6)

- [ ] 1.1 `scripts/lingua-data/build.sh`: `--parts <pair> <base.lingua> <layer.lingua>` in build mode (through `lingua-pack-build --split`, each file checked against its pin record, named on a mismatch) and in testdata mode; verify `bash scripts/lingua-data/build.sh --parts en-fr /tmp/en.lingua /tmp/en-fr.lingua` passes and that a wrong record fails naming the file.
- [ ] 1.2 `pack_sources.py`: `record-build` records `base` and `layer` (sha256, size) beside `pack`; `check-parts --pin P --base B --layer L`; `check_studied_tables` holds a reader pair's `base.sha256` to its reference's, naming both on a mismatch; `test_pack_sources.py` covers the three records, the check and the mismatch messages; `python3 -m pytest scripts/lingua-data/test_pack_sources.py` passes.
- [ ] 1.3 Re-record the six pins with `base` and `layer` (`pack` unchanged); verify `bash scripts/lingua-data/build.sh <pair> /tmp/<pair>.lingua` and `--parts` pass for the six pairs, and `python3 scripts/lingua-data/pack_sources.py check-studied-tables` passes for the reader pairs.
- [ ] 1.4 `pack_report.py` reports the new keys as it reports `pack.size` and `pack.sha256`; `test_pack_report.py` covers a moved base and a moved layer; the tests pass.
- [ ] 1.5 Each `scripts/lingua-data/testdata/<pair>/tags.tsv`: the sorted union of the fixture's readings' and senses' tags; verify `cargo test -p lingua-pack --test pipeline_testdata` holds every fixture's whole pack to its former sha256 (recorded in the test before the pin is added) and that `build_split` now accepts each fixture.

## 2. The build tooling (D1, D4)

- [ ] 2.1 `tool/packs.mjs`: `baseFile(studied)`, `shippedStudied(pairs)`, `packMeta` reading `part` and `baseVersion`, `assertPacksMatchEngine` holding each layer to its pair and its base's `baseVersion`, each base to lingua-core's analyser constant; `test/packs.spec.ts` covers a layer of another pair, a layer of another base, a base of another analyser, a whole pack in the folder, and the paths; `yarn test test/packs.spec.ts` passes.
- [ ] 2.2 `tool/gen_pack.sh` writes each pair's layer into `assets/packs/` and, once per studied language, its base into `assets/bases/` through `build.sh --parts`, refusing a second pair's base that differs by a byte; verify `yarn gen:pack` and `yarn gen:pack:real` write nine files and no whole pack.
- [ ] 2.3 `build.mjs` copies `assets/bases/`; `tool/manifests.mjs` exposes the bases after the layers; `test/manifests.spec.ts` pins the exposed list; `yarn build` passes for the three targets.
- [ ] 2.4 `tool/check_variants.mjs` holds the packaged files to the listed pairs' layers and their languages' bases — no whole pack, each under 8 MB; `yarn check:variants` passes and refuses a package holding a whole pack.

## 3. The engine's loader (D2)

- [ ] 3.1 `src/analyzer/pairs.ts`: `basePath(studied)`; `src/analyzer/engine.ts`: `fetchBase`, `fetchLayer`, the engine built through `assemble` and extended through `addParts`, a failed fetch of either part forgetting the language's load; `test/engine.spec.ts` fakes `assemble` and `addParts` and asserts the fetches (the default pair's two parts at start, one base and one layer per added language, none twice, none for a language nothing ships, a refetch after a failure); `test/pairs.spec.ts` covers `basePath`; `yarn test` passes.

## 4. The checks and the documents (D5, D7)

- [ ] 4.1 `.github/workflows/lingua-extension-check.yml`: the parts built into the runner's temp folder beside the whole pack, and compared byte for byte with the packaged layers and bases after `yarn gen:pack:real`, in the monorepo step and in the reviewer's archive step; verify with `actionlint` and a pull request run.
- [ ] 4.2 `.github/workflows/lingua-pack-update.yml`: a dry run's report shows the three records per pair unchanged; verify on a `dry` dispatch of one pair after the merge [owner-dispatched, the owner reads the report].
- [ ] 4.3 `SOURCES.md` (*Build*), `REVIEWERS.md` (the rebuild yields bases and layers; what to compare), `apps/lingua-extension/README.md` where it names the pack; `docs/lingua/italian-programme.md` row 4's status; verify each names `assets/bases/` and `--parts`.
- [ ] 4.4 Integration: in `apps/lingua-extension`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants` pass; `cargo test -p lingua-pack` passes; `bash apps/lingua-apple/tool/test_app_localizations.sh` passes; `dist-chromium/assets` holds ≈ 9.5 MB of parts; the six goldens and S0 pass unchanged; `openspec validate ship-lingua-pack-bases-once --strict` passes and `scripts/openspec_archive_order.py` names `split-lingua-pack-format` alone.
