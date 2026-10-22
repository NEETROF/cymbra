# Tasks

## 1. The module

- [ ] 1.1 `scripts/lingua-data/reduce_edition_it.py` (D1–D9): `IT = common.Edition(code="it", form_of=…, notes=…, letter=…, capitalised=False)`; `typography`; `read_as_meanings`, the pre-pass taking out template residue before the pointer test (D6), reading pointers by the word their wording names (D3), a `form-of` tag on a meaning (D4), glosses nested under a pointer (D5), « vedi italiano » alone or followed by a qualifier as the headword (D6), obsolete and heraldic senses last and a headword heraldic only, word or expression, glossed by nothing (D8); « contrazione di » never a pointer, « abbreviazione di » only when its word lends a gloss (D2); gender marks never taken after « di » or « of » (D9).
- [ ] 1.2 `test_reduce_editions.py`: one fixture per scenario of the ADDED requirement, and the notes, the template residue before a pointer, a heraldic sense last, `could`'s nested gloss; the Italian edition in the editions' list; a test, as the English and Spanish editions have, that editing the Italian edition's rules moves only the pairs whose definitions come from the Italian Wiktionary, and that no pair glossed from another edition loads `reduce_edition_it.py`; every committed pair's `rule_files`, `rules_sha256` and `pack_version` as pinned.
- [ ] 1.3 `scripts/lingua-data/SOURCES.md`: `reduce_edition_it.py` in the table of edition modules.

## 2. Elsewhere

- [ ] 2.1 `docs/lingua/italian-programme.md` (its *Measured figures* rows for en-it, fr-it and es-it done in this proposal's pull request): row 35's status, and the figures re-measured on the committed tables by changes 36, 47 and 51.
- [x] 2.2 `openspec/changes/add-lingua-pack-it-es/{design.md, specs, tasks.md}` (D12, done in this proposal's pull request): `reduce_italian_es.py` takes the Italian Wiktionary's notes out of its Spanish section read backwards, and drops a sense left empty.

## 3. Gates

- [ ] 3.1 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `python3 scripts/lingua-data/gloss_coverage.py --check`; every committed pair's tables byte for byte.
- [ ] 3.2 `openspec validate add-lingua-italian-edition-rules --strict`; the MODIFIED block re-copied as the main spec then stands.
