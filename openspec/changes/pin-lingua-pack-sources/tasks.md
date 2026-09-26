## 1. Before any code

- [ ] 1.1 Confirm the builder's determinism across machines: build the same tables on macOS and on `ubuntu-24.04`, compare the pack sha256 (zstd level 19, `Cargo.lock` versions)
- [x] 1.2 Check that the reducer's output is itself deterministic: two reductions of the same raw sources give identical tables (ordering, floating point, dict iteration)

## 2. Pinned raw sources and the record

- [x] 2.1 `scripts/lingua-data/tables/en-fr/pin.json` (design D2) and a small reader shared by `build.sh` and the checks
- [x] 2.2 AGID, CEFR-J and Octanove fetched at a commit (not a branch) and checked by sha256 (D3)
- [x] 2.3 `requirements-reduce.txt`: `wordfreq==3.1.1` and its dependencies, hash-pinned; a fixed Python version for every mode that reduces
- [x] 2.4 kaikki snapshot: zstd-compressed, published as release `lingua-pack-sources-en-fr-<snapshot>` (never replacing one), notes carrying its licence; fetched back and checked by sha256 in re-reduce mode
  - `lingua-pack-sources-en-fr-2026.09.26` published (14.2 MB, decompresses to the recorded sha256). Re-reduce mode run from a clean work folder: every pinned source fetched and checked, tables and pack byte-identical to the committed ones.

## 3. The three modes of `build.sh`

- [x] 3.1 Build mode (D4): `lingua-pack-build` on `tables/en-fr/`, pack sha256 checked against `pin.json`; `yarn gen:pack:real` runs it; offline, no Python
- [x] 3.2 Re-reduce mode: pinned raw sources → reducer → tables; `pack` and `reducer` updated in `pin.json`
- [x] 3.3 Update mode: live sources recorded in `pin.json`, new kaikki snapshot, reduce → tables
- [x] 3.4 `--pack-version <snapshot>` passed to the reducer instead of the constant `1.0.0` (D8)
- [x] 3.5 Tests: Build mode refuses a pack whose sha256 differs; re-reduce refuses a raw source whose sha256 differs; the record round-trips

## 4. Update workflow and monthly check

- [x] 4.1 A diff report between two table sets: lemmas, glosses, levels, expressions added / removed / changed (counts and samples), pack size against the 5 MB budget; tests on small fixtures
- [x] 4.2 `lingua-pack-update` workflow, manual (`mode`: `update` | `reduce`): pinned environment, kaikki release when new, branch `lingua-pack/<snapshot>` with the tables and `pin.json`, report and "open the pull request" link in the summary; never opens or approves a pull request (D6)
- [x] 4.3 Monthly schedule in dry mode (D7): report only; fails when a source cannot be fetched, the reducer fails, or a table loses more than the threshold (set from the first runs)
- [x] 4.4 Name per the `<target>-<verb>` convention; `python3 scripts/check_ci_units.py` still passes

## 5. Every lane uses the committed tables

- [x] 5.1 `lingua-extension-release`: Build mode; remove the raw-source `actions/cache`, `pip install wordfreq` and the "at least 1 MB" guard
- [x] 5.2 `lingua-apple-release`: the same
- [x] 5.3 `lingua-extension-check`: Build mode on every pull request touching the extension or the pipeline (sha256 and budget); fails when `reduce-en-fr.py` no longer matches `reducer.sha256`; keeps the testdata pack for the extension's tests
- [x] 5.4 `REVIEWERS.md`: the pack rebuilds from the committed tables with no download, expected sha256 stated; the check lane's archive rebuild does exactly that and compares the hash

## 6. First tables

- [x] 6.1 Dispatch the update once (first snapshot = what the current pipeline produces that day); open and merge its pull request
  - The workflow can only be dispatched once it is on `main`, and the release lanes switch to the committed tables in this same pull request — so the first snapshot (2026.09.26) was produced by the same Update mode run locally (Python 3.12, `requirements-reduce.txt`), and its kaikki snapshot is published as the release the workflow would have created. Every later update goes through `lingua-pack-update`.
- [x] 6.2 `scripts/lingua-data/tables/en-fr/README.md` (licences of the tables); `.gitignore` and `SOURCES.md` updated: raw sources and packs never committed, tables committed, sources read only by the update and the monthly check
- [x] 6.3 The extension's README section on the data pack (the real pack builds offline from the committed tables; packs themselves still never committed) and the comment of `scripts/lingua-data/.gitignore` ("only testdata/ and the scripts are tracked")

## 7. Gates

- [x] 7.1 `openspec validate pin-lingua-pack-sources --strict`
- [ ] 7.2 Reducer unit tests, `cargo test -p lingua-pack`, extension check lane green; a release dry run (dispatch without tag) builds the committed pack with no request to kaikki, GitHub raw or PyPI
