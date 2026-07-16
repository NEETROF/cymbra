## Context

`ScoreSummary::from_document` (`crates/musicxml-core/src/meta.rs`) is the single derivation
of a score's facets, consumed by the app preview, the upload record, and the crawler's
`catalog_scores` row — so facets never drift across client/server/crawler. It already yields
`is_piano`, `staves`, `key_fifths`, `time_sig`, `measure_count`, `note_count`. The streaming
parser (`lib.rs`) already emits per-note `type` (note value), `chord`, `dot`, `pitch`
(step/octave/alter), `staff`, and `dynamics` events. So the facets below are derivable from
the *same* parse with no new source data.

Persisted catalog columns today: `is_piano`, `key_fifths`, `time_sig`, `measure_count`
(+ provenance). `note_count`/`staves` are computed but **not** persisted. The Score Hub
(`score-hub-search` change) filters by text/author/level only.

**Locked with the user:** more relevant filters is better; existing rows are **backfilled by
re-reading the already-stored objects** (not by re-crawling from source), preserving ids;
`is_piano` is **forced `true` by the front** for now (piano-only corpus) but must exist as a
search parameter.

## Goals / Non-Goals

**Goals**
- Derive a rich, reliable set of *musical* facets at ingest from the existing parse, persist
  them, and expose them as conjunctive Score-Hub filters — headline being the "fastest note
  value" (no sixteenths / no eighths) rhythmic-granularity filter.
- One derivation (`ScoreSummary`) for every consumer; pure and host-testable.

**Non-Goals**
- No new source formats or re-parsing beyond the single existing parse.
- No ML/heuristic difficulty re-estimation (keep `level` as-is).
- No *fabricated* tempo: when a score carries no tempo marking, `tempo_bpm` stays `null` (the
  player still falls back to its default 90 BPM for playback, unchanged) — we never store the
  playback default as if it were the score's tempo (decision A).
- No keyset/relevance changes to search beyond adding filter predicates.

## Decisions

### D1 — The facet set + wire representations
Derived on `ScoreSummary` and persisted (all nullable — an old/edge file may lack a signal):
- `min_note_value: u8` — the **smallest** note value present as a power-of-two denominator
  (`1`=whole, `2`=half, `4`=quarter, `8`=eighth, `16`=sixteenth, `32`, `64`). The filter is
  "nothing faster than X" → `min_note_value <= X`. This is the croche/double-croche filter.
- `has_tuplets: bool` — any `time-modification`/`tuplet` (triplets, …).
- `has_dotted: bool` — any `<dot>`.
- `has_chords: bool` — any `<chord>` note (simultaneous pitches).
- `lowest_midi: u8`, `highest_midi: u8` — pitch ambitus (step+octave+alter → MIDI). Derived
  filter: **span** = `highest-lowest` → "fits within N octaves".
- `staff_count: u8` — persist `staves` (1 vs 2 grand staff).
- `note_count: i32` — persist the existing count (density = `note_count/measure_count`).
- `is_minor: Option<bool>` — from `<key><mode>` (major/minor); `None` when absent
  (complements `key_fifths`, which alone can't tell relative major/minor).
- `tempo_bpm: Option<u16>` — the primary tempo in quarter-note BPM: prefer the first
  `<sound tempo="…">`, else convert the first metronome mark (`<metronome>` beat-unit +
  per-minute) to quarter-BPM. `None` when the score carries no tempo marking (common) — the
  filter then excludes it. A representative single value (the opening tempo), not a per-section
  range, keeps the facet and its index simple.
- Expressivity flags: `has_dynamics: bool`, `has_ornaments: bool`, `has_articulations: bool`,
  `has_pedal: bool` — pedagogically useful include/exclude toggles.

### D2 — Derive in the shared `ScoreSummary` (single source of truth)
Add the fields to `ScoreSummary` and compute them in `from_document`. Extend the parser to
carry the few not-yet-captured signals (`time-modification`/`tuplet`, `<mode>`, `<ornaments>`,
`<articulations>`, `<pedal>`) onto the parsed model; `type`/`chord`/`dot`/`pitch`/`staff`/
`dynamics` are already parsed. Keep derivation **pure** and unit-tested against XML fixtures.

### D3 — `min_note_value`: `<type>` first, duration fallback
Prefer the notated `<type>` (map name → denominator). When a note omits `<type>` (some
exports only give `<duration>`), fall back to `round(divisions_per_quarter*4 / duration)` to
the nearest power-of-two denominator. Rests are ignored (playable notes only, matching
`note_count`). Grace notes (zero duration) are ignored for the fallback.

### D4 — DB: nullable columns + indexes, repopulate by truncate + re-crawl
- Module migration adds the facet columns to `music.catalog_scores` **and** `music.user_scores`
  (parity so "mes partitions" can filter identically), all nullable, guarded, fully-qualified.
- Indexes for the columns used as filters: btree on `min_note_value`, `staff_count`,
  `is_minor`, and the ambitus columns; the boolean flags are low-cardinality so a partial
  index (`WHERE has_x`) or none (bitmap scan) suffices — pick per column at implementation.
- **Backfill (re-read, not re-crawl):** the score objects are already in the store, so existing
  rows are updated in place without re-downloading. A one-shot maintenance command streams
  `SELECT id, object_key FROM …` for both tables, reads each object via the `ObjectStorage`
  seam, runs `decode_canonical` (catalog objects are `.mxl`; user-score objects are already
  plain XML — both pass through), parses it, derives the facets via `ScoreSummary`, and
  `UPDATE`s that row's facet columns by id. No network, no `TRUNCATE`, ids (and users'
  `user_library` references) preserved. The command is idempotent (re-running recomputes the
  same values) and resumable (skip rows whose facets are already set, or `WHERE min_note_value
  IS NULL`). The crawler fills the columns for new ingests going forward.

### D5 — Search: optional facet params, validated + conjunctive
Extend `SearchCatalogRequest` / `CatalogSearchParams` with optional:
`is_piano`, `max_note_value` (fastest allowed → `min_note_value <= v`), `has_chords`,
`has_tuplets`, `has_dotted`, `max_ambitus_semitones`, `staff_count`, `is_minor`,
`min_bpm`/`max_bpm` (tempo range → `tempo_bpm BETWEEN`, null tempo excluded). Each is
`None` = no constraint; supplied values compose conjunctively with query/author/level and each
other. `max_note_value`/`staff_count` are validated against the allowed sets; unknown → typed
`InvalidArgument`. Only rows whose facet is non-null and satisfies the predicate match (a null
facet is excluded when that filter is active — an unknown trait can't be asserted absent).
Pure clamping/validation stays in `ScoreModule::search_catalog`; the Pg adapter just adds
`AND (…)` predicates.

### D6 — App: advanced-filters panel; front pins `is_piano = true`
The hub keeps its simple default (search + author + level + "mes partitions"); the new facet
controls live behind an expandable **"Filtres avancés"** section to avoid clutter. Controls:
a rhythmic-granularity chip row ("≤ noire / ≤ croche / ≤ double / tout"), toggles
(accords / triolets / pointés), an ambitus chip ("≤ 1 / ≤ 2 octaves / tout"), mode
(majeur/mineur/tout), and a tempo band ("Lent < 76 / Modéré 76–120 / Rapide > 120 / tout" →
`min_bpm`/`max_bpm`). `CatalogSearch` state + `CatalogService.search` gain the params; the
production call always sends `is_piano: true` (the corpus is piano-only) while the parameter
remains overridable for the future. "Mes partitions" (uploads) filters client-side on the
same facets.

## Risks / Trade-offs

- **[Extraction accuracy]** tuplet/ornament/articulation detection depends on exports tagging
  them; absence yields `false`, not "unknown" — acceptable for include-only toggles, but note
  it (a `false` may be under-reported). Mitigation: keep flags conservative; document semantics.
- **[Missing `<type>`/`<mode>`]** some files omit them → `min_note_value` uses the duration
  fallback; `is_minor` stays `Null` and is excluded when the mode filter is active.
- **[Null-excluding filters]** applying a facet filter drops rows whose facet is null (not-yet-
  backfilled rows). After the backfill every stored row has the facets, so this only bites a
  partially-migrated DB. Mitigation: run the backfill before relying on the filters.
- **[Missing object on backfill]** a row whose object is gone from the store can't be
  re-derived. Mitigation: the backfill logs and skips it (leaves facets null), never aborts the
  whole run.
- **[Index selectivity]** boolean flags are low-cardinality; over-indexing wastes writes.
  Mitigation: index only the scalar/range columns; leave flags to bitmap/partial indexes.
- **[Coverage]** all derivation/validation/compose logic stays in host-testable modules
  (`meta.rs`, `module.rs`, the fake search repo, the notifier) to hold ≥ 80% both ecosystems.

## Migration Plan

1. Extend the parser + `ScoreSummary` with the facet fields (+ unit tests on XML fixtures).
2. Module migration: nullable facet columns + indexes on `catalog_scores` and `user_scores`.
3. Thread facets through `CatalogEntry`/`ScoreMetadata` (crawler) + `PgCatalogRepo` insert and
   `UserScore`/upload persistence.
4. Extend `score.proto` `SearchCatalogRequest`; regenerate stubs; add filter params to the
   port/adapter + `ScoreModule::search_catalog`.
5. App: service + notifier + advanced-filters UI; pin `is_piano = true` in the production call.
6. Backfill existing rows: run the re-read maintenance command (catalog + user scores) — no
   re-crawl.
7. **Rollback:** additive — reverting the app hides the advanced filters; the columns/indexes
   are safe to drop (nothing else consumes them).

## Open Questions

- Exact bucket boundaries for the ambitus chip (≤ 1 / ≤ 2 octaves) and the density filter —
  settle from real corpus distribution after the first full crawl.
- Whether to expose `has_dynamics/ornaments/articulations/pedal` as UI filters now or keep them
  persisted-but-unfiltered until there's demand (lean: persist all, surface the highest-value
  few first).
