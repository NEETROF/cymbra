# Tasks

## 1. Core: the mark, the tombstones, the edit

- [ ] 1.1 `crates/lingua-core/src/decks/card.rs` (D4, D5): `gloss_by_reader: bool`, serde default `false`, skipped when `false`. Verify with unit tests: a card without the mark serialises byte for byte as before; a marked card round-trips; a card JSON without the field reads unmarked.
- [ ] 1.2 `decks/review.rs` (D7): `Deck.tombstones` (per language, lemma → seconds), serde default, skipped when empty; `upsert` clears a tombstone of its key; `apply_card_lww` refuses a card not later than a tombstone of its key and clears the tombstone when later; a new `apply_card_removal_lww` removes a card not changed after the removal and records the tombstone (also without a card); `export_tombstones`. Verify with unit tests for each scenario of *A card removed by an edit stays removed on every device* (stale copy refused, newer copy back, removal of a card graded later ignored, removal recorded without a card, re-capture clears it) and a deck without tombstones serialising as before.
- [ ] 1.3 `apply_card_lww` (D4): keep the local `gloss_by_reader` when the incoming card has none and the same gloss text. Verify with unit tests: same text kept, different text cleared, incoming mark applied.
- [ ] 1.4 The form rule (D3): the normalisation (NFC, trim, collapse, edge punctuation by the rule of `fix-lingua-selection-card-punctuation` — introduced here as one shared function if that change has not landed, else called), the 120/200 limits, the error kinds. Verify with unit tests: « four-poster bed, » → « four-poster bed », blank and punctuation-only refused, 121 characters refused, case kept.
- [ ] 1.5 The edit in `decks/review.rs` (D4, D6, D8, D9): rename or merge of the session's current card at `now` — gloss settled in D4's order (the pack lookup handed in by the caller), merge by D6 (target matched letter case aside, review state by reps then last review then target, gloss order, edited card's sentence/source, earlier `captured_at`, target's media), encountered form recomputed by D8, tombstone of the old key, statuses by D9 (old `learning` cleared, new `learning` when none, known/ignored ⇒ retired and advanced), session re-keyed (current key, queue without the target's later place, answered-in-session carried over, snapshot). Verify with unit tests for every scenario of *An edit settles the card's gloss, the reader's own first*, *An edited form already in the deck merges the two cards* and *A renamed card moves its word's status*, plus a no-op save writing nothing and an edit counting no review.
- [ ] 1.6 `decks/backup.rs` (D5): the mark and tombstones in the backup, no schema version change. Verify with tests: a deck without an edit backs up byte for byte as the pinned baseline; round trip of a renamed card and a marked gloss; `read_backup` of the previous version's reader accepting the new fields (a backup with them restored by a reader that ignores unknown fields).

## 2. WASM bindings

- [ ] 2.1 `crates/lingua-wasm/src/lib.rs` (D3, D10): `reviewEdit(form, gloss, now)` returning `{ok:true, merged?}` or `{ok:false, error}`, looking up the new form's gloss as `readable_gloss` reads a lemma. Verify with binding tests: a rename, a merge answering `merged`, each error kind, a gloss from the pack, a reader gloss.
- [ ] 2.2 `readable_gloss`/`review_current` (D4): a reader-written gloss returned as written whatever its label. Verify with the scenario « A gloss the reader wrote in another language » and the English baseline views unchanged.
- [ ] 2.3 `exportCardOps`/`applyCardOps` (D5, D7): tombstones exported as `deleted: true` ops, `gloss_by_reader: true` emitted only when set; deleted ops applied through the removal LWW instead of skipped; the mark read. Verify with binding tests, and the pinned export of a deck without edits unchanged byte for byte.

## 3. Backend: the additive field

- [ ] 3.1 `backend/lingua/proto/deck.proto`: `bool gloss_by_reader = 13;` on `CardOp`, documented. Verify with `buf breaking` against `main` passing (the `proto` workflow) and `buf lint`.
- [ ] 3.2 `backend/lingua/migrations/0008_lingua_card_gloss_by_reader.sql`: `ALTER TABLE lingua.cards ADD COLUMN IF NOT EXISTS gloss_by_reader BOOLEAN NOT NULL DEFAULT false`, idempotent, schema-qualified. Verify by running the migrations twice on the test database.
- [ ] 3.3 `backend/lingua/src/{deck.rs,deck_grpc.rs,pg_deck.rs}`: the field stored and returned as received, no logic. Verify with the deck repository tests (mockall for the repo trait; the Postgres adapter in the integration tests): a marked card round-trips, an unmarked push stores `false`, a tombstone still wins and is pulled.

## 4. Extension: port, sync, review UI

- [ ] 4.1 `src/analyzer/{port.ts,engine.ts,messaging-port.ts}`: `CardOp.gloss_by_reader?`, `reviewEdit` on the port and its message. Verify with `yarn typecheck` and the port specs.
- [ ] 4.2 `src/sync/sync.ts`: `glossByReader` pushed and pulled; deleted ops passed to `applyCardOps`. Verify with `test/sync.spec.ts`: a marked card pushed with the field, a pulled one applied marked, a pulled tombstone removing the card.
- [ ] 4.3 `src/review/session.ts` + `review-page.ts` (D10): `ReviewController.edit(form, gloss)`, persisted with `saveBackup` as after an answer. Verify with `test/review-page.spec.ts`: the backup saved after an edit, a merged outcome shown once, a known target advancing to the next card.
- [ ] 4.4 `src/review/view.ts` (D1, D2): « Modifier » in the links row before and after the reveal; the form in place of the answer and the actions (labels, `lang`, input attributes, focus, Enter/Escape, review keys ignored in inputs, errors with `aria-invalid`/`aria-describedby`); the front after a save marking the new form or showing it above the sentence; no machine translation prefilled. Verify with `test/view.spec.ts` for every scenario of *The reader can edit the card under review* and *An edited form is checked before it is saved*.
- [ ] 4.5 `src/i18n/{fr,en,es}/review.ts`: `edit`, the two field labels, `save`, `cancel`, the three errors and the merged note, in the three catalogues. Verify with the catalogue alignment lint (`yarn lint`) and `test/review-stats-copy.spec.ts`.
- [ ] 4.6 `src/styles/review.css`: the form from `tokens.css`, sizes from `--cymbra-lingua-ui-scale`, no transition. Verify with `yarn lint` (tokens and scale lints).
- [ ] 4.7 The review section of `apps/lingua-extension/README.md`: editing a card, the gloss precedence, merges, and that a rename synchronises as a removal. Verify by reading it against the specs.

## 5. Cymbra Lingua app

- [ ] 5.1 `src/state/app-relay.ts` and `src/app/session.ts`: no shape change; tombstones and the mark pass through `appOpsSince`/`applyAppOps`. Verify with `test/app-relay.spec.ts` and `test/review-relay.spec.ts`: an edit in the app's session removes the old card and adds the marked new one in the extension's engine; an extension card changed after the app's edit survives it.

## 6. Integration and dogfood

- [ ] 6.1 Gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass.
- [ ] 6.2 Dogfood in the Cymbra Lingua app on iPhone: « colloquially referred to as » → « referred to as », a translation written for a card without one, the keyboard not covering the form; then Safari holds the result. Verify by noting the results in the pull request.
- [ ] 6.3 Dogfood in Chrome's side panel signed in, with a second device: a merge into an existing card, a rename reaching the second device after synchronisation, the old card not coming back. Verify by noting the results in the pull request.
- [ ] 6.4 Dogfood on the e-ink reader (Firefox for Android): opening and closing the form are one repaint each. Verify by noting the result in the pull request.
