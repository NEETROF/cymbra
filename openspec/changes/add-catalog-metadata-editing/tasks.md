## 1. Data model (backend/music migrations)

- [x] 1.1 Add a `catalog_edits` append-only audit table migration: `id`, `catalog_score_id` (FK), `editor` (identity), `field`, `old_value`, `new_value`, `edited_at` (default now). Index on `catalog_score_id`. No updates/deletes (append-only).
- [x] 1.2 Add provenance columns to `music.catalog_scores`: `edited_by TEXT NULL`, `edited_at TIMESTAMPTZ NULL` (set on manual edit; also the future refresh-skip condition). Idempotent DDL, fully-qualified names, matching the existing migration style. Title-backfill (`backfill.rs`/`pg.rs`) now skips `edited_by IS NOT NULL` rows.

## 2. Edit operation (backend/music)

- [x] 2.1 Add `UpdateCatalogScore(score_id, { title?, composer?, arranger?, level? })` to the score proto (only curatorial fields — no derived facets); regenerate bindings.
- [x] 2.2 Implement the module logic (pure/host-testable where possible): validate `level` against the enum; compute the **diff** (changed fields, old→new); recompute `title_norm`, `composer_norm`, `work_key` via `cymbra_musicxml_core::normalize_text` when title/composer changed (crawler parity); a no-op edit is an idempotent success (no audit rows). (Corpus-only is structural — `catalog_scores` has no `owner_id`, so there is no owned-score to refuse.)
- [x] 2.3 Persist in **one transaction** (`pg.rs`): UPDATE the curatorial + recomputed normalized columns + provenance (`edited_by`/`edited_at`), and INSERT one `catalog_edits` row per changed field. Reject unknown score.
- [x] 2.4 Wire the gRPC handler in `grpc.rs` guarded by `require_moderator_or_admin` (same guard as `SetModerationStatus`); map errors (permission/not-found/invalid-arg) to statuses; no raw errors leaked.

## 3. Console edit form (apps/back-office)

- [x] 3.1 Add a `catalog.updateCatalogScore(id, fields)` store action behind the `api()` seam, returning an `Async<void>` submit state (ts-pattern), refreshing the hit on success. (Submit `Async` + hit refresh live in the view, mirroring the existing `decide()` pattern; the store method is the thin RPC seam.)
- [x] 3.2 Add a gated edit form on the score detail view (moderator/admin only): editable title/composer/arranger/level; the derived facets (timeSig/notes/measures/staves/tempo/piano/facets) shown **read-only**. Reachable from review mode via the detail view. Submit → store action; show the submit `Async` state; localized errors only.
- [x] 3.3 Hide the form entirely for signed-in non-moderators (UX gate; the server guard is the real boundary).

## 4. Tests & verification

- [x] 4.1 Rust: edit writes the curatorial fields + recomputes the 3 derived keys (`catalog_edit` pure tests assert `work_key`/`title_norm`/`composer_norm` recompute from the final values); audit row per changed field (fake `edit_calls`); refused for non-moderator (`update_catalog_score_requires_moderator_or_admin`); invalid level + empty title refused; `normalize_text` parity. (Owned-score case is structural — `catalog_scores` has no owner, so there is nothing to refuse.) 106 music tests pass.
- [x] 4.2 Vitest: the store action calls the RPC (`catalog.spec.ts`); the form is gated to moderator/admin + saves+refreshes (`edit.spec.ts` view gate); derived facets are read-only; submit error surfaces a localized message. All 91 back-office tests pass; new files 100% line coverage.
- [x] 4.3 `cargo fmt`/`clippy` clean; regenerated proto bindings (`yarn gen`); back-office `typecheck`/`lint` clean (app untouched — no `melos run analyze` needed).
- [x] 4.4 `openspec validate add-catalog-metadata-editing --strict` passes.

## 5. Follow-ups (tracked, not in v1)

- [ ] 5.1 User-uploaded scores: an owner-aware edit path (permission/consent model) — deferred.
- [ ] 5.2 License/source editing (provenance/CGU-sensitive) — deferred, add with extra care.
- [ ] 5.3 Optimistic concurrency (updated_at check) if multi-moderator edit conflicts become real — last-write-wins + audit for now.
