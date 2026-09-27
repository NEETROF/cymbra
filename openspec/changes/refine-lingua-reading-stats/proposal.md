## Why

The daily `exposures` counter — the "Expositions" chart in the back office and « Mots rencontrés » in the extension's stats — adds **every counted token of a whole document** the first time the reader analyses it, and adds it again on every reload or navigation. A single tester reaches 300k–700k a day: the figure measures how much text was *opened*, not how much was *read*. It also says nothing about the thing Lingua exists for — the reader meeting words they do not know yet.

The extension already knows what was actually read: the viewport-gated exposure tracker (a block counts once it stays on screen for a dwell) feeds the per-word promotion counters. The daily figure should use the same signal, and a second figure should count the unknown words the reader saw, so the back office can tell *how much people read* apart from *how hard what they read is*.

## What Changes

- **Redefine `exposures`** (daily stat): the occurrences counted in the blocks the reader actually saw (viewport + dwell), added when each block is confirmed read — no longer the whole document at first analysis. The field name and wire number are unchanged; only its meaning is. Back-office label « Mots lus » / "Words read"; the extension's stats card « Mots rencontrés » is relabelled « Mots lus » to match.
- **Add `unknown_seen`** (daily stat, **back office only**): among those occurrences, the ones classed unknown or being learned (the words the reader sees highlighted — words presumed known under the declared level are excluded). Pushed with the daily stats; not shown in the extension. Labelled « Mots nouveaux vus » / "New words seen" in the back office.
- **Sync contract** (`stats.proto`): additive `optional uint32 unknown_seen` on `DailyStat`. **A daily stat pushed without it (an extension that predates this change) is acknowledged and not stored** — outdated extensions stop being counted until they update.
- **Local reset on update**: the updated extension discards, once, the daily statistics it stored under the old whole-document counting, so none of them is pushed back. Word statuses, deck, level, calibration and per-word exposure counters are kept. The server history is left as is (no users besides the maintainer; outdated rows age out of the window).
- **Backend**: column `lingua.daily_stats.unknown_seen`; the upsert and `GetStats` carry it.
- **Back office** (`lingua_admin.proto`, additive): a `LINGUA_SERIES_UNKNOWN_SEEN` series, words-read and new-words-seen totals, and a **reading comprehension** figure (share of read words that were known) in the tiles and the per-language breakdown. The "Expositions" chart is relabelled « Mots lus ».
- The per-word exposure counters and exposure-confirmed promotion are **unchanged**.
- Not breaking on the wire (`buf breaking` passes: fields and an enum value added, nothing renumbered). Behaviourally, outdated extensions' daily stats are dropped — intended.

Products impacted:
- **Lingua (extension, Chromium/Firefox/Safari from one source)** — new: viewport-gated daily counting, the new pushed counter, one-time local reset of daily statistics, card relabel.
- **Lingua backend** — new: column, proto field, drop of stats without the counter, admin series and totals. Consumes the existing sync path and erasure rules unchanged.
- **Back office** — new: chart, tiles, breakdown column; consumes the existing `admin-lingua-console` screen, store and scoped gating.
- ID / Music / Live / site: not impacted (the privacy policy already lists "daily stats" as synced; no new category of data).

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `lingua-browser-extension`: adds the requirement defining the daily reading statistics (what counts as a word read and a new word seen, when it is counted, what is pushed, the one-time local reset, and that only words read is shown in the extension) — today undefined in the specs and implemented as whole-document counting.
- `admin-lingua-console`: the "Lingua usage aggregates" requirement gains the words-read and new-words-seen series/totals and the comprehension figure, and states that only daily stats carrying the new counter are counted (older ones are dropped at the sync boundary).

## Impact

- `apps/lingua-extension/src/reading/{session,exposure-tracker,scan}.ts`, `src/state/{dailystats,store}.ts`, `src/stats/view.ts` (label only), `src/sync/sync.ts`, regenerated `src/gen/stats_pb.ts`.
- `backend/lingua/proto/{stats,lingua_admin}.proto`, new migration `0004_…`, `src/{stats_core,pg_stats,admin_core,admin_grpc,pg_admin}.rs`, tests.
- `apps/back-office`: regenerated `src/gen/lingua_admin_pb.ts`, `src/stores/lingua.ts`, `src/views/LinguaView.vue`, `src/lib/e2e-seam.ts`, en/fr locales, e2e.
- Gates: `buf breaking` (additive only), Rust ≥ 80 % coverage, extension + back-office tests.
