# Tasks

## 1. Editions (scripts/lingua-data)

- [x] 1.1 `Edition` in `reduce_common.py`, and the French edition in `reduce_edition_fr.py` holding today's rules by identity (D1). Every cleaning and classifying function takes an edition as a required keyword (`reduce_common.py` imports none, D2); the French-native reducers bind the French one. es-fr's native-side helpers move to the shared module, and es-fr keeps aliases. `test_reduce_en_fr.py` and `test_reduce_es_fr.py` pass unchanged (additions only).
- [x] 1.2 The English and Spanish editions in `reduce_edition_en.py` and `reduce_edition_es.py` (D1). Tests on recorded senses from real kaikki data:
  - an English « plural of », an untagged « synonym of », an `alt-of` sense and a no-gloss sense;
  - a Spanish « Forma del plural de », « Participio pasado del verbo (to) read », a sense-link subscript and « Femenino. » kept as a meaning;
  - English glosses keep their lower case.

## 2. Rules, sources and the digest

- [x] 2.1 `rule_files` reads the `reduce_*` modules a pair's reducer loads (D2). Tests:
  - en-fr's and es-fr's rules name the shared and French-edition modules, not the English or Spanish ones;
  - a reducer that loads a module its record does not name fails `check-reducer`, naming it.
- [x] 2.2 `derive()` reads `senses[].translations` (D3). Tests:
  - a sense-level fixture keeps the table's sense;
  - an entry-level line is byte for byte as before.

## 3. The proof

- [x] 3.1 `pack_report.py --identical` (D4). `test_pack_report.py` covers each file kind that may and may not move.
- [x] 3.2 The `reduce` job in `lingua-extension-check.yml`, filtered on `scripts/lingua-data/**` and its own workflow file, re-reduces every pair from its pinned sources and fails on any diff.
- [x] 3.3 `lingua-pack-update.yml`: `pair: all` (reduce mode only, one branch, goldens blessed once) and `expect: identical`, failing also on a coverage-JSON change or a golden line other than `### pack` / `### beside`. The commit carries the run URL.

## 4. The re-pin (owner-approved)

- [x] 4.1 en-fr and es-fr reduced again from their pinned sources with the new rules (D5). The seven tables, `tags.tsv` and NOTICE are byte-identical; `manifest.json` moves `meta.pack_version` only; `pin.json` moves `pack.sha256` and `reducer.*` only. `pack_report.py --identical` passes for both.
- [x] 4.2 Both goldens re-blessed: exactly three lines move, in their `pack_version` token. `cross_native`, `committed_tables` and `backup_format` pass.
- [ ] 4.3 The owner approves the re-bless in the pull request.

## 5. Gates and docs

- [ ] 5.1 Gates:
  - the Python tests (`python3 -m unittest discover -s scripts/lingua-data`);
  - `cargo test -p lingua-core -p lingua-pack -p lingua-wasm`;
  - in `apps/lingua-extension`: `yarn test`, `yarn build` and `yarn check:variants`, since the packs are rebuilt;
  - the new `reduce` job, green on this pull request (the first Linux reproduction);
  - `lingua-pack-update` dispatched on this branch with `mode=reduce`, `pair=all` and `expect=identical`: it reports every pair identical and proposes nothing.
- [x] 5.2 Docs:
  - `SOURCES.md`, both tables' `README.md` and `apps/lingua-extension/REVIEWERS.md` name the rule modules and describe the editions;
  - `reduce_common.py`'s docstring stops saying « every <studied>→FR pair »;
  - `openspec validate generalise-lingua-gloss-reducer --strict` passes;
  - change 6 is marked done in `docs/lingua/language-matrix-programme.md`.
