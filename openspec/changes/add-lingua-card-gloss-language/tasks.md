# Tasks

## 1. The card (crates/lingua-core)

- [ ] 1.1 `decks/card.rs`: `gloss_language: String`, default `fr`, out of the backup when `fr`; `Card::new` takes it (D1). Tests: a French card serialises without the field; an `en` card with it; a card without the field reads `fr`.
- [ ] 1.2 `decks/backup.rs` tests: *Every card today* (a version 1 and a version 2 backup unchanged byte for byte), *A card created on an engine glossed in English* (version 2, the field present, read back).

## 2. The engine (crates/lingua-wasm)

- [ ] 2.1 `add_card` labels with `nativeLanguage()`; `export_card_ops` emits `gloss_language`; `apply_card_ops` reads it, empty meaning `fr` (D2). Tests: *A card applied from a card operation*.
- [ ] 2.2 `review_current` substitutes the current pack's gloss by key — word or expression — when the card's label is not the engine's native, and keeps the card's text when the pack has none (D3, D4). Tests beside `tests/cross_native.rs`, on the pack glossed in another native: *A word glossed in another native language*, *An expression glossed in another native language*, *A gloss the pack has not*; `english_baseline` and `spanish_baseline` pass without `LINGUA_BLESS` (*The English baseline*).

## 3. The surfaces and the agent

- [ ] 3.1 `apps/lingua-extension/src/analyzer/port.ts` `CardOp.gloss_language`; `test/` fixtures that build card operations carry it; the sync client (`src/sync/sync.ts`) is unchanged (change 12).
- [ ] 3.2 `apps/lingua-agent/rust`: cards created with the followed pack's native language (D2); its tests.

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-wasm -p lingua-agent`; in `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.2 `openspec validate add-lingua-card-gloss-language --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-card-gloss-language` exits 0; change 11 is marked done in `docs/lingua/language-matrix-programme.md`.
