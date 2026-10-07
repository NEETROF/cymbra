# Tasks

## 1. Contract (backend/lingua/proto)

- [ ] 1.1 `deck.proto` `CardOp.gloss_language = 12` and `PullCardsRequest.any_gloss_language = 3`, `stats.proto` `DailyStat.native_language = 8`, `lingua_data.proto` `GetDataStateResponse.language_labels = 3`, each commented as the precedent's fields are (D1, D4, D7). `buf breaking` reports no break; `tests/privacy_allow_list.rs` passes.

## 2. Storage (backend/lingua)

- [ ] 2.1 Migration `0006_lingua_native_language.sql`: `ADD COLUMN IF NOT EXISTS gloss_language TEXT NOT NULL DEFAULT 'fr'` on `lingua.cards`, `native_language` on `lingua.daily_stats`; a header naming this change (D2).
- [ ] 2.2 `language_core.rs`: `normalise_or(raw, default)`, `gloss_language(raw)` and `native_language(raw)`; `normalise` delegates with `en`; tests named after *A regional code on a label* and *A label the server has never seen*, plus the empty value and an over-long code (D3).
- [ ] 2.3 `deck.rs` `Card.gloss_language`, `deck_grpc.rs` from/to proto through the named readers, `pg_deck.rs` insert, conflict update and select; `stats_core.rs` `DailyStat.native_language`, `stats_grpc.rs` from proto through the named reader, `pg_stats.rs` upsert and select; `ConsolidatedStat` unchanged (D5). `data_grpc.rs` answers `language_labels: true` (D4).
- [ ] 2.4 The pull filter: `DeckRepo::changes_since` and `DeckModule::pull_cards` take `any_gloss_language`; `pg_deck.rs` withholds the cards glossed other than `fr` when it is unset, the cursor being the highest sequence returned (D7).

## 3. Tests (backend/lingua)

- [ ] 3.1 `deck.rs` module tests on `FakeDeckRepo`: each scenario of *A card carries the language of its gloss* (the empty value through the named reader, `en`, two devices with the French one last, the round trip, the pull from a client that predates labels, the pull from one that reads them).
- [ ] 3.2 `stats.rs` module tests on `FakeStatsRepo`: each scenario of *A daily statistic carries the native language of its device*.
- [ ] 3.3 `tests/convergence.rs`: a card labelled `en` converges across two devices with its label; `tests/pg_deck_it.rs` (extended) and `tests/pg_stats_it.rs` (new), `#[ignore]`, run by `backend-it`: 0006 applies over 0005, a row written before it by SQL that names no label reads `fr`, a labelled row round-trips, a pull without `any_gloss_language` withholds a non-French gloss; `data_grpc` answers true (*The server states that it stores…*, both scenarios: the false case is the wire's default, decoded from bytes without the field).

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p cymbra-lingua`, `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `backend-it` green on the pull request; `python3 scripts/check_ci_units.py --list` unchanged (no new unit).
- [ ] 4.2 `openspec validate add-lingua-native-language-server --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-server` exits 0; change 10 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Release (owner)

- [ ] 5.1 [manual] Rehearse migration 0006 on a copy of production; replay a sync from the published extension and the Apple app against the rehearsed server: cards and statistics unchanged, no request refused.
- [ ] 5.2 [manual] Backend release and production deployment; verify from outside that `GetDataState` answers `language_labels: true`, that a card pushed without a gloss language comes back with `fr`, that a pull without `any_gloss_language` returns the same cards as before, and that a statistic pushed without a native language is stored. Only then may `add-lingua-card-gloss-language` and `add-lingua-native-language-sync-client` be built for a store.
