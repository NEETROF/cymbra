## Context

Two mechanisms in the reader both talk about "exposure" and have drifted apart:

- **Daily `exposures`** (`state/dailystats.ts`, pushed as `DailyStat.exposures`, charted in the back office): `ReadingSession.repaint()` adds `stats.counted` — every counted token of the whole document — once per document, on the first analysis (`session.ts` `exposuresRecorded`). Reset on `detach()`, so every reload/navigation re-adds the whole page. Content that arrives later (a feed) is never counted.
- **Per-word exposure** (`reading/exposure-tracker.ts`): an `IntersectionObserver` (threshold 0.5) confirms a block container after a 1.5 s dwell, once per container, and feeds `onExposed(lemmas)` → the engine's promotion counters.

Observed: one tester, 300k–700k "exposures" a day. The number measures pages opened. Nothing measures the difficulty of what is read.

Constraints: `stats.proto` is a client contract gated by `buf breaking` (FILE); installed extensions keep pushing the old shape; the admin API is aggregates-only (privacy allow-list in `lingua_admin.proto`); per-device upsert key `(user, day, language, device)` replaces the row on each push; `SyncEngine` pushes every locally stored day.

Product decisions (maintainer, 2026-09-27): outdated extensions are simply not counted until they update; no history is deleted or reset (no users yet); the new counter is for the back office only.

## Goals / Non-Goals

**Goals:**
- Daily words read = occurrences in blocks confirmed seen, by the same dwell signal as per-word exposure.
- A second daily counter, new words seen (Unknown + Learning occurrences in seen blocks), pushed for the back office.

**Non-Goals:**
- Changing the per-word exposure counters, the dwell, the threshold or promotion.
- Showing new words seen or comprehension in the extension.
- Deleting or resetting existing daily statistics, on the server or on the device.
- Distinct-word counts per day (would need a per-day lemma set on the device — the storage blow-up of #473 again).
- Counting agent (Claude Code plugin) sessions — still local only.

## Decisions

### D1 — One observer, a richer payload
`ExposureTracker.track()` receives, per container, `{ lemmas, read, unknown }` instead of `lemmas` only; `confirm()` hands the whole payload to the session. The session keeps feeding lemmas to the per-word path unchanged and bumps the daily counters with `read`/`unknown` (`recordReading(store, day, read, unknown)`, batched with the existing flush timer / `visibilitychange` / `detach` flush). `scan.ts` gains a pure `readingByContainer(blocks, analysis)` computing per-container counted occurrences and Unknown+Learning occurrences — same inclusion rule as the engine's `counted`, so words read on a fully-seen page equals `stats.counted`.
*Alternative*: a second IntersectionObserver for stats. Rejected — two dwell clocks that could disagree, twice the observers.

### D2 — Counts snapshotted at confirmation, containers counted once per document
`track()` already skips containers in `exposed`, and re-tracking an unexposed container overwrites its payload, so the counts at confirmation reflect the latest analysis (a word marked known meanwhile is not "new"). A confirmed container is never counted again for this document. The whole-document `recordExposures` call and `exposuresRecorded` flag are removed. Today `track()` skips a container with no lemmas; the check becomes "no counted occurrence and no lemma".

### D3 — Presence of `unknown_seen` is the version marker; absent ⇒ dropped
`DailyStat.unknown_seen = 7` is `optional uint32` (presence tracked). `UpsertDailyStats` filters out every stat without it, then upserts the rest; the response is the usual acknowledgement so an outdated extension does not retry or surface an error. The column is `unknown_seen INTEGER NOT NULL DEFAULT 0` — after the filter every stored row reports it, so no NULL semantics are needed downstream.
`ConsolidatedStat` (the extension's `GetStats`) does not gain the field: the extension does not display it.
*Alternatives*: store NULL and compute comprehension over reporting rows only — rejected by the product decision (old clients are not counted at all, which is simpler and keeps every aggregate on one definition); a minimum-client-version header — rejected, field presence already carries it without a new mechanism.

### D4 — No history rewrite
Migration `0004` only adds the column. Existing rows keep their whole-document `exposures` and read as 0 new words seen; the updated extension keeps its local daily statistics and pushes them with `unknown_seen` set (0 for past days), so they pass the D3 filter. Accepted: the only history is the maintainer's test data (no users before 2026-09-25), and it ages out of the default 30-day window.
*Alternative*: delete the server rows and reset the local key — rejected as needless machinery with no users to protect.

### D5 — Admin: counts only, comprehension formatted in the UI
`AdminGetLinguaUsage` adds `words_read` and `new_words_seen` to the response and to `LinguaLanguageUsage`; the UI computes `1 − new ÷ read` and shows "—" when read is 0. `LinguaSeriesMetric` gains `LINGUA_SERIES_UNKNOWN_SEEN = 3`; `LINGUA_SERIES_EXPOSURES` keeps its name (renaming an enum value is a FILE break) and is relabelled in the UI only. No per-day comprehension series — ratios over tiny daily denominators are noise at our volume.

### D6 — Names in code vs labels
Wire/DB names stay `exposures` (compat) and add `unknown_seen`. User-facing labels only: « Mots lus » / "Words read" (back office and the extension's existing card), « Mots nouveaux vus » / "New words seen" and « Compréhension » / "Comprehension" (back office). Comments on the field in `stats.proto` and `dailystats.ts` are rewritten to the new definition.

## Risks / Trade-offs

- [Test history stays inflated, with 0 new words seen] → the comprehension of old days reads high; test data only, out of the default window within 30 days.
- [Outdated extensions vanish from all Lingua aggregates, active accounts included] → intended; store auto-update closes the gap within days. The screen's "synced accounts" note stays true.
- [A container taller than twice the viewport never reaches the 0.5 threshold] → never counted, same as per-word exposure today. Blocks are paragraphs in practice; if measured otherwise, fix the tracker for both paths at once.
- [No declared level: every word reads as unknown] → new words seen ≈ words read for those readers, lowering comprehension. Accepted: the HUD asks for a level on the first page; the state is transient.
- [Reload re-counts seen blocks] → the reader did see them again; a small overcount, far below today's whole-page recount.

## Migration Plan

1. Backend: migration `0004` (column), proto field, drop-without-field filter, admin additions. From then on, outdated extensions' stats are dropped.
2. Back office: regenerate, new tiles/series/column. Empty until updated extensions push.
3. Extension release (Chromium, Firefox, Safari host from the same build): viewport-gated counting, new field.
Rollback: every step is additive; reverting the extension alone makes its stats dropped again by the filter.
