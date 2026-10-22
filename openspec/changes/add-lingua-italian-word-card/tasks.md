# Tasks

## 1. The renderers (apps/lingua-extension)

- [ ] 1.1 `src/i18n/index.ts`: `StudiedLanguageCode` gains `it` (D1); `StudiedLanguage` untouched.
- [ ] 1.2 `src/i18n/{fr,en,es}/grammar.ts`: `TENSES.it` (D2's table, its order) and `GERUNDS.it` (D4); the doc comments name this change; unit tests: `parlò`, `parliamo`, `parli`, `parlando`, `parlante`, `parlata` in each interface, as the spec's scenarios say.
- [ ] 1.3 Every form of the six other committed packs rendered before and after, byte for byte (D7); the other card snapshots pass without `-u`.

## 2. Review (crates/lingua-wasm)

- [ ] 2.1 `src/lib.rs` `pack_gloss`: the Italian no-space branch (D5); a test: a card `d'accordo` reviewed over an it-en fixture keyed `di accordo` shows its gloss, a card `perché` read as a word; `cargo test -p lingua-wasm` passes.

## 3. The golden and the snapshot

- [ ] 3.1 `crates/lingua-wasm/tests/support/italian.rs`: the card's probes of D6; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`, the pull request listing the probes added; verify the test passes without re-blessing afterwards.
- [ ] 3.2 `test/word-card-it-fr.spec.ts` and `test/baseline/word-card-it-fr.txt` (D6); `yarn test` passes, and passes again without `-u`.

## 4. Owner and gates

- [ ] 4.1 [manual] The owner reads `word-card-it-fr.txt` (the French terms, the merges, the order) before the implementation merges.
- [ ] 4.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.
- [ ] 4.3 `openspec validate add-lingua-italian-word-card --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-word-card` names its prerequisites; `docs/lingua/italian-programme.md` row 18's status names this change's pull requests.
