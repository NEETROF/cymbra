## 1. Before any code

- [ ] 1.1 Decide the open questions of design.md: the refresh opens the pull request itself or prints the pin; a raw-source snapshot or none; a refresh cadence or on demand
- [ ] 1.2 Confirm the builder's determinism across machines: build the current tables on macOS and on `ubuntu-24.04`, compare the pack sha256 (zstd level 19, `Cargo.lock` versions)

## 2. The pin and the pinned build

- [ ] 2.1 `scripts/lingua-data/pack-pin.json` (design D2), empty for `en-fr` until the first snapshot; a small reader shared by the build and the checks
- [ ] 2.2 `build.sh` pinned mode (D3): fetch the tables asset of the pinned release, check its sha256, unpack, run `lingua-pack-build`, check the pack's sha256; clear errors naming the release and the expected hashes
- [ ] 2.3 `build.sh --live` mode (D3): today's fetch and reduce, recording each source's url, date, Last-Modified, size and sha256; `--pack-version <snapshot>` passed to the reducer (D5)
- [ ] 2.4 `yarn gen:pack:real` runs the pinned mode; a separate script runs the live mode; `yarn gen:pack` (testdata) unchanged
- [ ] 2.5 Tests: the pinned mode refuses a tables asset or a pack whose sha256 differs; the live mode records the provenance

## 3. The refresh

- [ ] 3.1 `requirements-refresh.txt` with `wordfreq==3.1.1` and its dependencies, hash-pinned; the refresh pins its Python version
- [ ] 3.2 A diff report between two table sets: lemmas, glosses, levels, expressions added / removed / changed (counts and samples), pack size against the 5 MB budget; tests on small fixtures
- [ ] 3.3 `lingua-pack-refresh` workflow (manual): live build, report in the run summary, new release `lingua-pack-en-fr-<date>` (never replacing one), pin change proposed as decided in 1.1
- [ ] 3.4 Add the workflow to `scripts/check_ci_units.py`'s view if it watches a unit; name it per the repository's `<target>-<verb>` convention

## 4. Every lane uses the pin

- [ ] 4.1 `lingua-extension-release`: pinned `gen:pack:real`; remove the raw-source `actions/cache` and `pip install wordfreq`; replace the "at least 1 MB" guard by the pack sha256
- [ ] 4.2 `lingua-apple-release`: the same
- [ ] 4.3 `lingua-extension-check`: the pinned asset exists and matches; `reduce-en-fr.py` matches `reducer.sha256` (fails asking for a refresh)
- [ ] 4.4 `make_source_archive.sh` carries the pinned `tables.tgz`; `REVIEWERS.md` rebuilds the pack from it with no download and states the expected sha256; the check lane's archive rebuild does exactly that and compares the hash

## 5. First snapshot

- [ ] 5.1 Dispatch the refresh once; review its report (first snapshot = what the current pipeline produces)
- [ ] 5.2 Merge the pin change; switch both release lanes to the pinned mode if not done in 4.x
- [ ] 5.3 Update `scripts/lingua-data/SOURCES.md` and the extension's README: sources are read only by the refresh; where the snapshot lives

## 6. Gates

- [ ] 6.1 `openspec validate pin-lingua-pack-sources --strict`
- [ ] 6.2 Reducer unit tests, `cargo test -p lingua-pack`, extension check lane green; a release dry run (dispatch without tag) builds the pinned pack with no request to kaikki, GitHub raw or PyPI
