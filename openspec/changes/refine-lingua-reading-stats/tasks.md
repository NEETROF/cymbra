## 1. Sync contract & backend

- [ ] 1.1 `stats.proto`: add `optional uint32 unknown_seen = 7` to `DailyStat`; rewrite the `exposures` comment (occurrences in blocks seen, viewport + dwell) and document that a stat without `unknown_seen` is dropped
- [ ] 1.2 Migration `backend/lingua/migrations/0004_lingua_reading_stats.sql`: `ADD COLUMN IF NOT EXISTS unknown_seen INTEGER NOT NULL DEFAULT 0` (no data change)
- [ ] 1.3 `stats_core.rs`: filter out stats without `unknown_seen` before the upsert (pure, unit-tested); `pg_stats.rs` writes the column
- [ ] 1.4 Tests: outdated push acknowledged and nothing stored (statuses/cards unaffected), up-to-date push with `unknown_seen = 0` stored, mixed batch keeps only reporting stats
- [ ] 1.5 Erasure: the existing daily-stats delete/drop still applies (assertion in the existing test)

## 2. Admin RPCs

- [ ] 2.1 `lingua_admin.proto`: `LINGUA_SERIES_UNKNOWN_SEEN = 3`; add `words_read`, `new_words_seen` to `AdminGetLinguaUsageResponse` and `LinguaLanguageUsage` (new field numbers only)
- [ ] 2.2 `admin_core.rs` / `admin_grpc.rs` / `pg_admin.rs`: new metric mapping (`COALESCE(SUM(unknown_seen),0)`), totals `SUM(exposures)` / `SUM(unknown_seen)` overall and per language
- [ ] 2.3 Tests: metric value mapping, totals per language
- [ ] 2.4 `buf breaking` passes locally against `main`

## 3. Extension — viewport-gated counting

- [ ] 3.1 `scan.ts`: pure `readingByContainer(blocks, analysis)` → per container `{ lemmas, read, unknown }` (read = tokens entering the percentage, unknown = `Unknown` + `Learning`); unit tests incl. "fully seen page ⇒ Σ read = stats.counted"
- [ ] 3.2 `exposure-tracker.ts`: carry the payload; `confirm()` passes it; update tests (dwell, fast scroll, once per container, re-track overwrites payload before confirmation)
- [ ] 3.3 `session.ts`: feed lemmas to the per-word path unchanged; accumulate `read`/`unknown` and flush with the existing exposure flush; remove the whole-document `recordExposures` call and `exposuresRecorded`
- [ ] 3.4 `state/dailystats.ts`: `unknownSeen` field (missing in stored entries ⇒ 0), `recordReading(area, day, read, unknown)`; rewrite the definitions comment
- [ ] 3.5 `sync/sync.ts`: push `unknownSeen` for every day; regenerate `src/gen/stats_pb.ts`
- [ ] 3.6 Session tests: long page read at the top only, fast scroll, content arriving later, re-analysis does not recount (spec scenarios)

## 4. Extension — stats screen

- [ ] 4.1 `stats/view.ts`: relabel « Mots rencontrés » → « Mots lus »; no new card (the model ignores `unknownSeen`)

## 5. Back office

- [ ] 5.1 Regenerate `src/gen/lingua_admin_pb.ts`; `e2e-seam.ts` fake data with the new fields
- [ ] 5.2 `stores/lingua.ts`: fetch the new series inside the existing `Async<T>`; expose comprehension (null when words read is 0)
- [ ] 5.3 `LinguaView.vue`: tiles words read / new words seen / comprehension ("—" when unavailable), « Mots lus » and « Mots nouveaux vus » charts, breakdown columns
- [ ] 5.4 en/fr locales aligned; vocabulary lint (no "lemma"); no « Expositions » / "Exposures" label left
- [ ] 5.5 Store unit tests + Playwright e2e for the new tiles and the unavailable state

## 6. Gates

- [ ] 6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, coverage ≥ 80 %
- [ ] 6.2 Extension and back-office lint/type-check/tests green
- [ ] 6.3 `openspec validate refine-lingua-reading-stats --strict`
