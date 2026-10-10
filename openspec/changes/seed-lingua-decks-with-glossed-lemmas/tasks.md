# Tasks

## 0. Owner, before the implementation

- [ ] 0.1 [manual] The owner approves the re-bless of `en-fr.golden` (the programme's rule « en-fr and es-fr output does not move ») and of `en-es.golden`, 9 probes each — `review-current first`, `review-current second`, `tracked-count`, `analyse reader homographs`, `phrase-gloss reader She will lead the expedition`, `vocabulary-estimate reader`, `export-status-ops`, `export-card-ops`, `backup` (design *Measured*, D5) — and, should change 51's `fr-es.golden` be on `main` first, its 6.
- [ ] 0.2 [manual] The owner settles the design's open questions: 1, the control's message when only words without a translation are left (recommended: leave it); 2, cards without a gloss already in readers' decks (recommended: leave them).

## 1. The core (crates/lingua-core)

- [ ] 1.1 `decks/review.rs`: `Deck::seed_lemmas` skips a candidate with no gloss, before the cap counts it, as it skips a lemma already carded or holding an explicit status, and writes no card and no status for it; its comment says the three reasons a lemma is skipped and that the next one takes its place (D1, D2).
- [ ] 1.2 `decks/card.rs`: `Card::seeded(lemma, gloss: String, gloss_language, at)` takes the gloss itself; `seeded_card_uses_import_and_has_no_sentence` follows (D1).
- [ ] 1.3 Tests in `decks/review.rs`: `seed_lemmas_caps_and_skips_tracked_lemmas` seeds glossed candidates only (its `quixotic` and `arcane` given glosses); `seed_lemmas_skips_a_lemma_without_a_gloss_and_takes_the_next` — cap 2 over `quixotic` (none), `nuance`, `arcane` (none), `lucid`, `zeal`: `nuance` and `lucid` carded, `zeal` capped, `quixotic` and `arcane` with no card and no status (*The rarest words of a level have no gloss*); `a_level_whose_glossed_lemmas_are_tracked_seeds_nothing` — 0 added, nothing written (*Only lemmas without a gloss are left*); and in `knowledge/` that a lemma left unseeded keeps its presumption below a declared level and its place in the estimate (D3).

## 2. The engine and the invariance tests (crates/lingua-wasm)

- [ ] 2.1 `src/lib.rs`: `seedLevel`'s comment says it skips a lemma the pack does not gloss and takes the next; no signature moves.
- [ ] 2.2 `tests/support/mod.rs`: `render_with` unchanged; `render_unseeded` leaves out the two `seed-level` probes and their seedings; `follows_seeding(names, name)`, the probes from `start-review` on; the module's comment says why the seeded deck is the native side (D4).
- [ ] 2.3 `tests/cross_native.rs`: `assert_probes_alike` answers the scenario through `render_unseeded`, its count of compared probes unchanged in form; `seeding_follows_each_packs_glosses` (*Seeded cards through another native language*) — for en-fr / en-es, es-fr / es-en and fr-en / fr-es, every level, `common` and `rare`, a fresh engine seeding 50: the cards are the first 50 of the level's lemmas in that order (read from the reference pack) that the pack glosses, each with the pack's gloss and its native's label; `part` at A1 seeded through fr-en and not through fr-es while it is unglossed there. Its comments name D4.
- [ ] 2.4 `tests/en_es_baseline.rs`, `tests/es_en_baseline.rs`: `the_golden_is_the_*_one_on_the_studied_side` compares the shared probes before `start-review`, both seeding counts included; its count of compared probes takes off the ones that follow the seeding; the module comments say the rest is `cross_native.rs`'s, through engines.
- [ ] 2.5 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline --test en_es_baseline`: `en-fr.golden` and `en-es.golden` move on exactly the 9 probes of task 0.1 (the C1 rarest-first cards `digitalize`, `exhilarate`, `impishly` and `eclectically`, `irately`, `tastebud`), no other; then every baseline without `LINGUA_BLESS`: `spanish_baseline`, `es_en_baseline` and `french_baseline` pass unchanged (*A pack that glosses every levelled lemma*; D5).
- [ ] 2.6 Should change 51's `fr_es_baseline.rs` be on `main` first: its golden-to-golden comparison stops at the seeding as in 2.4, and `fr-es.golden` is re-blessed on its 6 probes (C1's `carreau` for `cheminement`); otherwise nothing here, and 51 uses `follows_seeding`.
- [ ] 2.7 In the pull request: every moved probe with its before and after, the reason it moved (the seeded lemma, then the review card marked known), and the design's *Measured* tables measured again on the implementation for the six committed pairs.

## 3. Gates and docs

- [ ] 3.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`, and `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`.
- [ ] 3.2 The WASM lane: `wasm-pack test --node crates/lingua-wasm` passes.
- [ ] 3.3 In `apps/lingua-extension`: `yarn gen:wasm`, then `yarn test`, `yarn lint`, `yarn typecheck`, `yarn build` and `yarn check:variants` — no source and no snapshot moves (`word-card-en-es.txt`, `word-card-es-en.txt` read the grammar probes alone).
- [ ] 3.4 `openspec validate seed-lingua-decks-with-glossed-lemmas --strict` passes, and `python3 scripts/openspec_archive_order.py seed-lingua-decks-with-glossed-lemmas` exits 10 naming `add-lingua-pack-fr-es` and `add-lingua-french-levels` (0 once they are archived).
- [ ] 3.5 Row 49c of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 4. Owner, after the implementation

- [ ] 4.1 [manual] The owner releases the extension (Chrome Web Store, addons.mozilla.org, the Safari host app) with the rule, before or with change 52's release of French; change 52 adds this change to its `archiveAfter` when it next moves.
