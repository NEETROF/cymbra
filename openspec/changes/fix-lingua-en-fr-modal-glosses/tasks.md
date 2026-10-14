# Tasks

## 1. The curated word glosses

- [ ] 1.1 `scripts/lingua-data/reduce-en-fr.py`: add `WORD_GLOSSES` (lemma → ordered `(UPOS, sense)` list) beside `LOCUTIONS`, with the ten entries of design D3 as the owner approves them, and a pure function that applies it to the `glosses`/`runs` `native_tables` returns — joined gloss, runs counted from the entry — refusing, with the lemma named, an unglossed lemma, more than eight senses, a sense over 300 characters or a gloss over 800, a separator inside a sense, a part of speech split across two runs, an unknown UPOS, or `PROPN` senses alone (design D2). Call it in `main()` before `resolve_forms` and every write. Verify with the tests of 1.2.
- [ ] 1.2 `scripts/lingua-data/test_reduce_en_fr.py`: one test per entry pinning its exact gloss and runs (`would` opens on « Conditionnel », holds no `INTJ`; `must` is `VERB:2 NOUN:3`; `can` opens on « Pouvoir »); a test that the table holds exactly the ten lemmas; one test per refusal; a test that every key the table does not name is equal before and after on a fixture (design D4). Verify: `python3 -m unittest discover -s scripts/lingua-data` passes.
- [ ] 1.3 Reduce en-fr again from the pinned snapshot (`scripts/lingua-data/build.sh --reduce en-fr <out.lingua>`) and commit `tables/en-fr/` and `pin.json`. Verify by diffing against the previous tables: exactly ten changed lines in `gloss.tsv` and ten in `senses.tsv` (the ten lemmas), no change in `mwe.tsv`, NOTICE or any `tables/en/` file, a new `pack_version` and rule digest; update the en-fr README's or `SOURCES.md`'s figures only if one moved.
- [ ] 1.4 `crates/lingua-pack/tests/committed_tables.rs`, on the pack built from the committed tables: `would`'s gloss is one verb group opening on « Conditionnel : would come = viendrait »; `can`'s first group is the verb; `must` is a verb group then a noun group. Verify: `cargo test -p lingua-pack` passes.

## 2. Gates

- [ ] 2.1 Checks pass: the Python suites of `scripts/lingua-data`; `pack_sources.py check-reducer` for every pair (only en-fr's digest moved) and `check-pack` for en-fr; `cargo test -p lingua-pack -p lingua-core -p lingua-wasm`; `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`; the extension's `yarn gen:pack:real` builds the en-fr pack matching `pin.json`.
- [ ] 2.2 Dogfood: with an extension built on the new pack, the word card of `would` in « knowing it would not come » reads « Conditionnel : would come = viendrait » first, and `could`, `should`, `must`, `can` open on their modal sense.
- [ ] 2.3 `openspec validate fix-lingua-en-fr-modal-glosses --strict` passes.
