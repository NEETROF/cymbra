# Tasks

## 1. Dictionary words (lingua-core)

- [x] 1.1 `section::LEXICAL` is read as a bitset when present; a wrong length is refused as `Malformed("lexical")`. `Pack::is_dictionary_word` reads the bit, else gloss presence (D1, D2). Host tests:
  - without a section, `is_dictionary_word` equals gloss presence and `dictionary_words` is unchanged;
  - with one, the bit decides: a glossed lemma whose bit is off is not a dictionary word, and an unglossed ranked lemma whose bit is on is;
  - a levelled lemma still counts;
  - an unknown lemma is not a dictionary word;
  - a wrong-length section is refused.
- [x] 1.2 `dictionary_words` and `document_names` read dictionary words (D2), and the doc comments stop citing gloss presence as the rule. Host tests:
  - with a lexical table, a capitalised mid-sentence lemma that is a dictionary word without a gloss stays a word;
  - a glossed lemma whose bit is off is set aside;
  - the existing Augusto/Dios tests pass unchanged.

## 2. The builder (lingua-pack)

- [x] 2.1 An optional `lexical.tsv` (`PackInputs`, `inputs_from_dir`). The section is written only when it differs from the glossed lemmas. When it is written, a dictionary word or glossed lemma outside forms ∪ ranks is refused (D1, D3). Tests:
  - a `lexical.tsv` equal to the glosses builds byte for byte the pack without it;
  - a different one writes the section deterministically;
  - both refusals name the lemma;
  - the testdata fixture still builds.
- [x] 2.2 An optional `tags.tsv` pin and the ordered pool: the pin, then other reading tags, then sense-only tags. Without a pin, the pool stays as before (D4). Tests:
  - a pack with a sense tag no reading uses stores its readings as the pack without it;
  - a reading tag outside the pin builds and reads back;
  - the archived Romance scenario still passes;
  - testdata packs are unchanged.
- [x] 2.3 Noun gender from readings (D5). Tests:
  - a bare `NOUN` run of a noun read with one gender gains it;
  - a noun of both genders gets a bare run, whatever gender its sense table names;
  - a noun read with no gender keeps its runs as its sense table writes them;
  - a contradicting run is refused, naming the noun.
- [x] 2.4 `tags.tsv` committed for en-fr and es-fr: today's pools, in their order. `build.sh` and `pack_sources.py` keep it when a pair is reduced again. Both packs rebuild to the sha256 in their `pin.json`. A test fails when a committed pair lacks its studied language's pin, or when two committed pairs of one studied language disagree on their pin, their studied tables (`forms.tsv`, `freq.tsv`, `level.tsv`, `grammar.tsv`) or their dictionary words.

## 3. Invariance

- [x] 3.1 `crates/lingua-wasm/tests/cross_native.rs` (D6): for English and Spanish, it compares a second-native pack over the reference's studied tables with the reference, probe by probe, glosses, senses and expressions stripped. Studied sections must be byte-equal.
- [x] 3.2 `english_baseline`, `spanish_baseline` and `backup_format` pass without `LINGUA_BLESS`. `git diff origin/main -- crates/lingua-wasm/tests/baseline crates/lingua-core/tests/fixtures` is empty, and no `scripts/lingua-data/reduce*.py` file changes.

## 4. Gates and docs

- [x] 4.1 Gates:
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`;
  - `cargo llvm-cov` at or above 80 %;
  - in `apps/lingua-extension`: `yarn test`, then `yarn build` and `yarn check:variants`, since the packs are rebuilt;
  - `scripts/lingua-data/SOURCES.md` documents `lexical.tsv` and `tags.tsv`.
- [x] 4.2 `openspec validate add-lingua-pack-lexical-layer --strict` passes and `python3 scripts/openspec_archive_order.py add-lingua-pack-lexical-layer` exits 0. Change 5 is marked done in `docs/lingua/language-matrix-programme.md`.
