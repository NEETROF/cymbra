## 1. Split the reducer

- [ ] 1.1 `scripts/lingua-data/reduce_common.py`: move the French-side and form rules out of `reduce-en-fr.py` whole (by AST, comments kept), with a frozen `Studied` (code, token, form_of_target, coordinators) passed keyword-only to the functions that read one of the four English values and to their two callers
- [ ] 1.2 `reduce-en-fr.py`: keep the English rules and `main`; `EN = Studied(...)`; bind the shared functions to it under their existing names (`functools.partial`); re-export the rest; `python3 -m unittest discover` green with the existing tests unchanged

## 2. Rule set and gate

- [ ] 2.1 `pack_sources.py`: `rule_files` (pair reducer + every `reduce_*.py`), `rules_sha256`, `record-build` writes `reducer.sha256` and `reducer.files`, `check-reducer` compares the set, `rules` prints the digest; tests for a shared module entering every pair's set and another pair's reducer staying out
- [ ] 2.2 `build.sh --reduce`: pack version from `pack_sources.py rules`
- [ ] 2.3 Source registry: ESDB and pinned CSV lists optional per pair; a pair with no registry fails with a message naming what to add

## 3. Tables

- [ ] 3.1 NOTICE credits wordfreq to Robyn Speer with its licence and the Google Books Ngram acknowledgement; unit test
- [ ] 3.2 Manifest `analyzer_version` read from lingua-core's `ANALYZER_VERSION`; unit test that the committed manifest matches
- [ ] 3.3 Reference run: `build.sh --reduce en-fr` with the unchanged reducer reproduces the committed pack (sha256 and size); then the same with this change: the seven data tables byte-identical, only NOTICE, `pack_version` and `pin.json` differ; commit the new tables and pin

## 4. Workflows and docs

- [ ] 4.1 `lingua-extension-check.yml`: build and `check-reducer` every folder of `tables/`; the reviewer-archive comparison reads the per-pair pack
- [ ] 4.2 `lingua-pack-update.yml`: `pair` input (en-fr by default and for the schedule); pair in the release, branch, commit, notes and concurrency names
- [ ] 4.3 SOURCES.md (shared rules; "adding a pair" corrected), `tables/en-fr/README.md` (rule files, branch name), REVIEWERS.md (shared module; ESDB instead of AGID)

## 5. Gates

- [ ] 5.1 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` green on Python 3.12 with `requirements-reduce.txt`; `bash scripts/lingua-data/build.sh en-fr <out>` matches `pin.json`
- [ ] 5.2 `openspec validate generalise-lingua-pack-reducer --strict`
