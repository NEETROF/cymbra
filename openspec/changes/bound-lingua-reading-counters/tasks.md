# Tasks — bound-lingua-reading-counters

## 1. Engine

- [x] 1.1 `KnowledgeState::promotable_by_exposure` — the promotion rule minus the distinct-day count; `promote_by_exposure` reuses it
- [x] 1.2 `ExposureCounters::retain` + `cap_by_recency`, with unit tests (dropping a language empties it; the cap keeps the most recent)
- [x] 1.3 `lingua-wasm`: record only promotable lemmas; prune (90 days, 5,000 lemmas) on record and on restore

## 2. Extension

- [x] 2.1 `storageFull` category (`authErrorOf` / `isStorageFull`) ahead of the TypeError branch, with copy in Réglages and in the account flows

## 3. Gates

- [x] 3.1 `cargo test -p lingua-core -p lingua-wasm`, `cargo fmt --check`, `clippy -D warnings`; extension `typecheck`/`lint`/`format:check`/`test`/`build`/`check:variants`
- [x] 3.2 `openspec validate bound-lingua-reading-counters --strict`
