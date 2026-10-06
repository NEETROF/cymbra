# Tasks

## 1. The core card

- [ ] 1.1 `lingua-core` (`decks/card.rs`): `SentenceTranslation { text, marks }` and `Card::sentence_translation`, optional, skipped when absent (design D2). Verify with unit tests: a card without a translation serialises exactly as before; a card with one round-trips; a backup written before the field restores with none.
- [ ] 1.2 `decks/review.rs`: `apply_card_lww` keeps the local translation and marks when the pulled text is empty (D3). Verify with unit tests for « A device that predates the translation »: a newer pulled card without a translation keeps the local one; a newer pulled card with one replaces it.
- [ ] 1.3 `decks/backup.rs`: translated cards never change the version a backup is written in (1 for an English reader with the default profile, 2 otherwise). Verify with the « Backup round trip » scenario as a unit test (translated and untranslated cards, in an English-only state and in one holding another language), then `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.

## 2. The WASM engine

- [ ] 2.1 `lingua-wasm`: `addCard` accepts an optional translation; `reviewCurrent` adds `sentenceTranslation`; `exportCardOps` and `applyCardOps` carry `sentence_translation` and `sentence_translation_marks`. Verify with native tests in `crates/lingua-wasm/tests/deck.rs`: capture with and without a translation, the review view model, an op round trip, an op without the fields keeping the local translation.

## 3. The contract and the server

- [ ] 3.1 `backend/lingua/proto/deck.proto`: `TextSpan` and `CardOp` fields 12 and 13, commented like the others (D3). Verify that `buf lint` and `buf breaking` against `main` pass (the `proto` workflow's commands).
- [ ] 3.2 `backend/lingua/migrations/0006_lingua_card_sentence_translation.sql`: the two columns, idempotent and schema-qualified (D4). Verify by applying it twice on a local database without error.
- [ ] 3.3 `backend/lingua/src/deck.rs` and `pg_deck.rs`: store and return the translation and its marks; keep the stored ones when the incoming text is empty; `FakeDeckRepo` alike. Verify with module tests on the fake (both rules) and a `backend-it` case on Postgres: an op from an older client does not erase a kept translation.

## 4. The extension

- [ ] 4.1 Port and sync: `NewCard.sentenceTranslation`, `ReviewCard.sentenceTranslation`, the `addCard` signature in `analyzer/engine.ts`, `analyzer/messaging-port.ts` and the test fake; `sync/sync.ts` maps the two `CardOp` fields both ways. Verify with `yarn typecheck` and a sync spec: a pushed op carries the fields; a pulled op without them keeps the local translation.
- [ ] 4.2 Capture (D1): the word card and the selection card put the translation they show on the `Gesture`; `ReadingSession.onGesture` passes it to `addCard`. Verify with `test/reading-session.spec.ts` cases for « Captured with a translation » and « Captured without a translation » (off, and still translating).
- [ ] 4.3 Review (D5): `renderReview` shows the translated sentence under the gloss after the reveal, marked spans in bold, built from text nodes, invalid spans ignored; never before the reveal. Verify with view specs for « In review », a span out of range, and a translation holding markup shown as text.

## 5. The privacy annex

- [ ] 5.1 `apps/site/src/pages/confidentialite.md` and `apps/site/src/pages/en/privacy.md`: the deck row and the extended-translation paragraph (D6), French and English saying the same. Verify with the `site-check` build and a side-by-side read of the two annexes.

## 6. Integration and dogfood

- [ ] 6.1 Gates: `yarn typecheck && yarn lint && yarn test && yarn format:check && yarn build` in `apps/lingua-extension`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` at the root. Verify all pass.
- [ ] 6.2 Dogfood: capture a word with the extended translation on the e-ink reader, then review it on a phone that has no translation model. Verify by noting in the pull request that the translation shows with the answer, marked, in one repaint on the e-ink reader.
