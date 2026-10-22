# Tasks

## 1. The module

- [ ] 1.1 `scripts/lingua-data/reduce_edition_it.py` (D1–D9): `IT = common.Edition(code="it", form_of=…, notes=…, letter=…, capitalised=False)`; `typography`; `read_as_meanings`, the pre-pass reading pointers by the word their wording names (D3), a `form-of` tag on a meaning (D4), nested glosses (D5), « vedi italiano » alone as the headword (D6), obsolete and heraldic senses last (D8); « contrazione di » kept a meaning (D2).
- [ ] 1.2 `test_reduce_editions.py`: one fixture per rule (each scenario of the ADDED requirement, the letter, the notes, the typography, a heraldic sense last, `could`'s nested gloss, `hear`); the Italian edition in the editions' list; a test that no committed reducer imports `reduce_edition_it.py`, and that every committed pair's `rule_files`, `rules_sha256` and `pack_version` are as pinned.

## 2. Elsewhere

- [ ] 2.1 `docs/lingua/italian-programme.md` (its *Measured figures* rows for en-it, fr-it and es-it done in this proposal's pull request): row 35's status, and the figures re-measured on the committed tables by changes 36, 47 and 51.
- [ ] 2.2 `openspec/changes/add-lingua-pack-it-es/{design.md, tasks.md}` (D12, done in this proposal's pull request): `reduce_italian_es.py` drops the Italian Wiktionary's notes when reading its Spanish section backwards.

## 3. Gates

- [ ] 3.1 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `python3 scripts/lingua-data/gloss_coverage.py --check`; every committed pair's tables byte for byte.
- [ ] 3.2 `openspec validate add-lingua-italian-edition-rules --strict`; the MODIFIED block re-copied as the main spec then stands.
