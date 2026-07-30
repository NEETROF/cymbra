## Context

Catalog scores live in `music.catalog_scores` (backend/music). Descriptive fields
(`title`, `composer`, `arranger`, `level`) sit next to **MusicXML-derived** facts
(`time_sig`, `note_count`, `measure_count`, `staff_count`, `tempo_bpm`, piano/chords/tuplet
facets) computed at ingest from the score bytes. Search (`catalog-search`) is a **pg_trgm
GIN index over the derived normalized columns** `title_norm` / `composer_norm` (migration
0004); `work_key = composer_norm::title_norm` is the same-work grouping key (migration 0001,
plain non-unique index). All three normalized keys are produced by the shared Rust
`cymbra_musicxml_core::normalize_text` (NFD accent-strip + lowercase + whitespace-collapse),
with crawler parity. Dedup is by **content sha256** (`ON CONFLICT (sha256) DO NOTHING`), so
the crawler never updates an existing row.

Moderation writes already exist: `SetModerationStatus` (guarded `require_moderator_or_admin`,
records `reviewed_by`/`reviewed_at`) and the `role_grants` append-only audit are the
patterns to mirror. The moderation console (Vue) edits nothing today — it shows metadata
read-only and accept/rejects.

## Goals / Non-Goals

**Goals:**
- Moderator/admin can correct a catalog score's **curatorial** fields (title, composer,
  arranger, level) from the console, instead of only accept/reject.
- The edit keeps **search consistent**: recompute `title_norm`, `composer_norm`, `work_key`
  in the same update so the trigram index and grouping reflect the new values.
- Every edit is **audited** (who, field, before→after, when), queryable.
- Derived MusicXML facts stay **read-only and authoritative** from the bytes.
- Reuse the existing guard (`require_moderator_or_admin`), the shared `normalize_text`, and
  the store/api + `Async<T>` front-end seams.

**Non-Goals:**
- Editing **derived** fields (time/notes/tempo/facets) — they come from the score; fixing
  them means fixing the MusicXML, not the metadata.
- Editing **user-uploaded** scores (they have an `owner_id`) — deferred (permission
  question).
- Re-uploading/replacing the score bytes, merging duplicates, or a metadata-refresh
  crawler path.
- License/source editing beyond a decision below (provenance-sensitive).

## Decisions

### Decision: Curatorial allow-list, server-enforced
The update accepts only `{ title?, composer?, arranger?, level? }`; any attempt to set a
derived field is not representable in the request and rejected if smuggled. `level` is
validated against its enum (`beginner|intermediate|advanced` or null). Empty/whitespace
title/composer normalize to null-ish per `normalize_text`.
- *Why:* the derived facts must never diverge from the bytes; a narrow request shape makes
  "edit everything" impossible by construction. **Alternative:** a generic field map —
  rejected, too easy to desync content.

### Decision: Recompute the three derived keys in the same UPDATE
On any title/composer change, recompute `title_norm`, `composer_norm`, and
`work_key = normalize(composer)::normalize(title)` with `normalize_text`, in the same
statement, so Postgres updates the GIN index atomically.
- *Why:* the search index is on the *normalized* columns, not the raw ones — editing only
  `title` leaves the score findable by its **old** name and invisible under the new one.
  `work_key` is recomputed too so all three derived keys stay consistent with the displayed
  metadata (dedup is by sha256 today and `work_key` has no unique constraint, so there is no
  collision risk; keeping it in sync is the least-surprising choice for the future
  same-work grouping). **Alternative:** freeze `work_key` — rejected: it would let the
  grouping key drift from the shown composer/title for no benefit.

### Decision: Append-only `catalog_edits` audit
A new table records one row per changed field: `catalog_score_id`, `editor` (identity),
`field`, `old_value`, `new_value`, `edited_at`. Written in the same transaction as the
update. Queryable for "who changed what, when".
- *Why:* privileged writes on shared content need traceability, exactly like `role_grants`
  and rejection's `reviewed_by`. Per-field rows keep the diff explicit. **Alternative:** a
  JSON blob of the whole before/after — rejected, harder to query per field.

### Decision: Provenance marker, not an anti-clobber gate
Add `edited_by` / `edited_at` on `catalog_scores` (or a boolean `manually_edited`), set on
edit. The crawler already `DO NOTHING` on sha256 conflict, so it does not overwrite edited
rows today — the marker is **provenance** (identify hand-edited rows) and future-proofing
for any later metadata-refresh path, which must skip rows where the marker is set.
- *Why:* honest scoping — no current clobber to fix, but the marker is cheap and makes the
  invariant explicit for the future. **Alternative:** a full crawler-behavior change now —
  rejected, unnecessary (crawler is already non-clobbering).

### Decision: Corpus-only in v1
The operation targets crawled corpus rows (no `owner_id`). User-uploaded scores are refused
(or simply not offered) in v1.
- *Why:* editing an owner's metadata as a moderator is a heavier consent/permission
  question; corpus rows are anonymous public-domain-ish content where curation is clearly
  the moderator's job. **Open question below.**

### Decision: Console edit form behind the existing seams
An edit form on the score detail view, gated to moderator/admin, drives a
`catalog.updateCatalogScore` store action (behind the `api()` seam) with an `Async<T>`
submit state matched via ts-pattern; derived facets render read-only. On success it
refreshes the hit so the view reflects the edit.
- *Why:* follows the vue-frontend-architecture rules (no direct API in components, one
  Async union). Reachable from review mode by opening the detail view.

## Risks / Trade-offs

- **Search desync** (the headline risk) → the update recomputes all three derived keys via
  `normalize_text`; a module test asserts an edited title is findable by its new value and
  not its old.
- **Editing derived facts by mistake** → not representable in the request shape; the form
  shows them read-only.
- **Audit/txn atomicity** → the row update + `catalog_edits` insert + provenance marker are
  one transaction; a partial write must not happen.
- **Crawler drift (future)** → provenance marker documented as the skip condition for any
  future metadata-refresh; no behavior change now.
- **Normalize parity** → reuse `cymbra_musicxml_core::normalize_text` exactly (not a
  re-implementation), so crawler and edits agree; a test pins parity on an accented example.
- **User-upload scope creep** → v1 refuses `owner_id` rows server-side, so the boundary is
  enforced, not just UI.

## Migration Plan

Additive: a `catalog_edits` table + a provenance column on `catalog_scores` (nullable /
defaulted, backfilled trivially), a new proto method, a guarded handler, and a console form.
No change to existing rows' meaning; search/ingest untouched. Rollout: migrate, ship the
RPC (dormant until the form ships), then the console form. Rollback: drop the form, revert
the RPC + migration (the provenance column is nullable and harmless if left).

## Open Questions

- **User-uploaded scores** — corpus-only (recommended v1) vs. also allowing edits on
  `owner_id` scores (needs an owner-consent/permission model). Default: corpus-only.
- **License/source editing** — allow correcting `license`/`source` (provenance-sensitive,
  CGU-relevant) or keep them immutable? Lean: keep immutable in v1, add later with extra
  care.
- **No-op / concurrent edits** — reject a no-op, or accept idempotently? And do two
  moderators editing the same row need optimistic concurrency (updated_at check), or is
  last-write-wins acceptable at <50 users? Lean: last-write-wins + full audit.
- **Search reindex cost** — recomputing keys is a normal row UPDATE (GIN maintained
  incrementally); no bulk reindex. Confirm no trigger/materialized view needs a manual
  refresh.
