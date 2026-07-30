## Why

Moderators can accept/reject catalog scores, but not **fix** them: crawled corpus and
user uploads often carry wrong or messy descriptive metadata (a misspelled composer, a
bad title, an off difficulty). Today the only recourse is rejection, which loses an
otherwise-good score. Letting a moderator/admin correct the **curatorial** fields turns
"reject" into "clean up and keep". The catch is that some catalog fields are **derived
from the MusicXML** (time signature, note count, tempo, the musical facets) and must never
be hand-edited — editing them would make the metadata lie about the actual score — and the
search index is built on **derived normalized keys**, so an edit that forgets to recompute
them silently desyncs search from what's displayed. This change adds correct, audited
editing of the curatorial fields only.

## What Changes

- Add an authenticated **`UpdateCatalogScore(id, { title?, composer?, arranger?, level? })`**
  operation on the music service, guarded by `require_moderator_or_admin` (the same guard
  as `SetModerationStatus`). It edits **only curatorial/attribution fields** — never the
  MusicXML-derived facts (time signature, note/measure/staff counts, tempo, piano/chords/
  tuplet facets), which stay read-only and authoritative from the score bytes.
- **Recompute the derived search keys in the same update**: `title_norm`, `composer_norm`
  and `work_key` (`composer_norm::title_norm`) via the shared `normalize_text` (crawler
  parity), so the trigram search index and the same-work grouping key stay consistent with
  the edited values. (Editing `title`/`composer` without this would leave the score
  unfindable by its new name.)
- Validate on the server: `level` against its enum, non-derived fields only, and a no-op
  edit (nothing changed) is rejected or a no-op success.
- **Audit every edit** in a new append-only trail (score id, editor, field, before/after,
  timestamp), queryable — mirroring the role-grant audit and the `reviewed_by` traceability.
- Record a **manual-edit provenance marker** on the row (who/when last edited), so edited
  rows are identifiable. (The crawler upserts `ON CONFLICT (sha256) DO NOTHING`, so it does
  not overwrite existing rows today — the marker is provenance + future-proofing against any
  later metadata-refresh path, not a fix for a current clobber.)
- **Console edit form** (moderator/admin) on the score detail view (reachable from review
  mode): the curatorial fields are editable, the derived facets are shown **read-only**;
  behind the store/api seam with `Async<T>` state, per the front-end rules.
- **Scope v1 to the crawled public corpus.** User-uploaded scores (which have an
  `owner_id`) are out of scope — editing an owner's metadata is a heavier permission
  question deferred to a follow-up.

## Capabilities

### New Capabilities
- `catalog-metadata-editing`: an authenticated, moderator/admin-only operation to edit a
  catalog score's curatorial fields (title/composer/arranger/level) that keeps the derived
  search keys consistent, refuses to touch MusicXML-derived facts, records provenance, and
  writes an append-only audit trail — plus the gated console edit form that drives it.

### Modified Capabilities
<!-- None. Catalog search is unchanged (the edit maintains its existing derived
     columns); the crawler is unchanged (already non-clobbering via ON CONFLICT DO
     NOTHING); the console gains the form as part of the new capability. -->

## Impact

- **`backend/music`**: new `UpdateCatalogScore` proto method + handler (`grpc.rs`, guarded
  by `require_moderator_or_admin`); module logic to validate, recompute `title_norm`/
  `composer_norm`/`work_key` via `cymbra_musicxml_core::normalize_text`, diff for the audit,
  and update the row (`pg.rs` `META_COLS`/update); a **`catalog_edits` audit migration** +
  a `edited_by`/`edited_at` (or boolean) provenance column on `catalog_scores`.
- **`crates/musicxml-core`**: reuse `normalize_text` (no change); the diff/validation logic
  is pure and host-tested.
- **`apps/back-office`**: an edit form on the detail view (moderator/admin gate), a store
  action `updateCatalogScore`, `Async<T>` state, and read-only display of derived facets;
  Vitest for the store action + the gating.
- **Provenance/audit**: the `catalog_edits` trail is queryable ("who changed what, when"),
  independent of the current row state.
- **Unchanged**: the MusicXML-derived facets and the score bytes; the crawler's sha256
  dedup + ingest; the native gRPC/gRPC-web surfaces; user-uploaded scores (v1).
- **Sequencing**: builds on the moderation console + `require_moderator_or_admin`
  (`add-moderation-back-office`); land after it.
