## 1. Contract

- [x] 1.1 `backend/lingua/proto/deck.proto`: `string language = 11` on `CardOp` (comment: empty = `en`), `repeated string languages = 2` on `PullCardsRequest` (comment: empty = English only); `lingua_data.proto`: `bool card_language = 2` on `GetDataStateResponse`. Re-run `buf breaking` locally against `main`: no break, no `!` marker needed
- [x] 1.2 Migration `backend/lingua/migrations/0005_lingua_card_language.sql`: `ADD COLUMN IF NOT EXISTS language TEXT NOT NULL DEFAULT 'en'`, drop and recreate the primary key as `(user_id, language, client_id)`, idempotent and schema-qualified like 0001–0004, header comment naming this change

## 2. Host-tested logic

- [x] 2.1 `backend/lingua/src/language_core.rs`: `normalise(raw: &str) -> String` (trim, lowercase, primary subtag on `-`/`_`, cap 8 bytes, empty → `en`) and `accepted_languages(list: &[String]) -> Vec<String>` (normalised, deduplicated, empty → `["en"]`); unit tests for `es-ES`, `ES`, `es_419`, `spa-ESP`, blanks, an over-long value, and the empty list
- [x] 2.2 `deck.rs`: `Card.language`; `DeckRepo::changes_since(user, cursor, languages)`; `DeckModule::pull_cards(user, cursor, languages)` defaults the set through `accepted_languages` and keeps the cursor rule (max returned sequence, else the request's); fake repo keyed by `(client_id, language)`
- [x] 2.3 `deck.rs` tests: no-language push stored as `en`; same client id in `en` and `es` are two cards with independent LWW; pull with an empty set returns English only and the cursor stops at the last English card; pull with `[en, es]` returns both in sequence order; a `pt` card is returned only when `pt` is accepted

## 3. Adapters

- [x] 3.1 `pg_deck.rs`: insert and conflict target on `(user_id, language, client_id)`, `language` selected and bound, `changes_since` filters with `language = ANY($3)`
- [x] 3.2 `deck_grpc.rs`: map `CardOp.language` through `normalise` on push and back on pull; pass `PullCardsRequest.languages` through `accepted_languages`
- [x] 3.3 `known_words_grpc.rs` and `stats_grpc.rs`: normalise the language of status ops, declared-level ops and daily stats on the way in; normalise `GetStatsRequest.language` only when non-empty (empty still means every language)
- [x] 3.4 `data_grpc.rs`: `card_language: true` on `GetDataStateResponse`

## 4. Integration and gates

- [x] 4.1 `backend/lingua/tests/convergence.rs` (Postgres): seed cards in two languages with the same client id, assert two rows and independent LWW; a pull without `languages` returns English only; a pull with both returns both; migration 0005 applies on top of 0001–0004 on an empty database and on one holding English cards
- [x] 4.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`
- [x] 4.3 `openspec validate add-lingua-card-language --strict`; `python3 scripts/check_ci_units.py --list` unchanged (no new unit)

## 5. Release (owner)

- [x] 5.1 [manual] Rehearse the migration on a copy of production; replay a sync from the published extension (1.4.0) and the Apple app against the rehearsed server: English cards unchanged, no request refused
- [x] 5.2 [manual] Backend release and production deployment; verify from outside that `GetDataState` answers `card_language: true` and that a pull without `languages` returns the same cards as before. Only then may `add-lingua-language-sync-client` be built for a store
