# Design — Private Score Catalog v1

## Context

`music.user_scores` already stores private scores server-side: owner-scoped
rows, server-derived metadata, `rights_basis IN ('own_work','public_domain')` +
`rights_ack`, per-owner SHA-256 dedup, objects under
`user-scores/{owner_id}/{uuid}.mxl` in the private OVH bucket, streamed by the
backend. The app's contribution wizard uploads exactly one file per pass
(pick → validate → attest → difficulty → confirm) and the library shows a flat
owner-scoped list with favorites. The in-flight `add-score-catalog-proposal`
change adds the opt-in ProposeScore path from a private score to the public
catalog.

Constraints: module isolation (no cross-schema FKs), UUID v7 app-side ids,
idempotent DDL, mockall/mockito test doubles, Riverpod 2 + Freezed codegen,
≥ 80 % coverage both ecosystems, UI strings FR/EN, no raw technical errors in
UI.

## Goals / Non-Goals

**Goals:**

- Import many files in one pass without N trips through the wizard.
- Organise the private library (collections, filter) with cross-device sync.
- An honest `private_use` rights basis that is *structurally* unshareable.
- The in-repo half of notice-and-takedown (admin lookup + reasoned removal +
  audit), so hébergeur diligence is demonstrable.

**Non-Goals:**

- Public catalog behavior, moderation, ratings, leaderboards — untouched.
- Organisations / établissements, any sharing between users.
- Web upload surface; offline mutation of collections.
- Editing a score's difficulty/metadata after import (future change).
- Blocking re-upload of taken-down content (audit only in v1).
- Owner notification e-mail on takedown (deferred; template infra is in flight).

## Decisions

### D1 — Batch import is a client-side loop over the existing upload RPC

The app uploads the selection sequentially through the existing single-file
upload operation; no new backend batch endpoint. Per-file server validation,
per-owner dedup (`owner_id, sha256` unique) and quota errors already have
defined semantics and map 1:1 onto a per-file result board
(imported / duplicate / invalid / quota-exceeded). Sequential keeps memory flat
and progress honest.

*Alternative rejected:* a multipart batch RPC — all-or-nothing semantics fight
per-file dedup/validation outcomes, complicates the size cap, and saves only
round-trips on a rare, user-initiated flow.

### D2 — One attestation and one difficulty for the whole batch

The batch flow asks once for the rights basis (applies to every file; recorded
per row via the existing `rights_basis`/`rights_ack` columns) and once for the
difficulty (applies to every file, `level_source = 'manual'`). Mixed-rights or
mixed-level selections are handled by running separate batches.

*Trade-off:* a wrong per-file level can't be fixed in v1 (no edit RPC) short of
delete + re-import. Accepted: keeps the flow to one screen; per-file metadata
editing is an explicit future change.

### D3 — `private_use` is enforced at every layer

- DB: the `rights_basis` CHECK gains `'private_use'` (drop + re-add, idempotent).
- Upload: server accepts the new basis (same `rights_ack = true` requirement).
- Propose: the ProposeScore path MUST reject a `private_use` score by reading
  the stored row's basis server-side — never trusting the client — so the guard
  holds regardless of whether `add-score-catalog-proposal` lands before or
  after this change. Client-side, propose affordances are simply absent for
  `private_use` rows.

Existing rows are untouched (no backfill; both legacy bases remain proposable).

### D4 — Collections are server-owned, many-to-many, name-scoped per owner

Two tables in the `music` schema (in-schema FKs are fine; the no-FK rule is
cross-schema only):

- `music.user_score_collections` — `id` UUID v7, `owner_id`, `name`,
  `created_at`; unique `(owner_id, lower(name))`.
- `music.user_score_collection_items` — `collection_id` FK → collections ON
  DELETE CASCADE, `user_score_id` FK → user_scores ON DELETE CASCADE,
  `added_at`; PK `(collection_id, user_score_id)`.

A score may belong to several collections (tag-like). Deleting a collection
never deletes scores; deleting a score silently leaves its collections. CRUD +
membership + "list my scores filtered by collection" are owner-scoped RPCs; the
app refetches on library load (same sync model as `saved-catalog-library` —
server is the source of truth, no offline queue).

*Alternative rejected:* a single `collection` text column on `user_scores`
(one collection per score, no rename semantics, migration pain later).

### D5 — Takedown = admin music-scope RPCs + persistent audit, hard delete

Two admin RPCs on the existing music admin surface (same role gating as other
music admin operations):

- lookup: find user scores by owner id/handle and/or title fragment (paged,
  minimal fields — no bulk enumeration beyond what moderation already allows).
- remove: given a user-score id and a **mandatory reason**, delete the S3
  object and the row, and write an audit row first
  (`music.user_score_takedowns`: admin id, owner id, score id, sha256, title,
  reason, created_at). Audit survives the deletion (that's its point);
  the removal is irreversible and the BO confirms it explicitly.

The intake (contact address in mentions légales, CGU illicit-content clause,
`private_use` CGU wording) is manual work in the cymbra-site repo, tracked in
tasks but not spec'd here.

### D6 — No feature flag

Batch import and collections are additive, owner-scoped UI with no blast radius
on existing flows; takedown is admin-gated by construction. A runtime flag would
add ceremony without a credible rollback scenario the flag uniquely enables.

## Risks / Trade-offs

- [Batch trips the upload quota mid-run] → the flow pre-checks remaining quota
  and warns when the selection exceeds it; per-file quota errors still land as
  per-file outcomes, never abort the whole batch.
- [One difficulty for heterogeneous batches] → accepted v1 limitation (D2);
  documented in the flow copy ("you can re-import a file to change its level").
- [`private_use` guard races the in-flight propose change] → guard reads the
  stored basis server-side and is specified defensively in
  `backend-score-storage`, so ordering doesn't matter (D3).
- [Takedown deletes user property] → mandatory reason, explicit BO
  confirmation, audit row written before deletion.
- [Collection name collisions/i18n] → uniqueness on `lower(name)` per owner;
  names are user data, never localized.

## Migration Plan

One idempotent migration (`0023_private_score_catalog.sql`): extend the
`rights_basis` CHECK, create both collection tables + indexes, create the
takedown audit table. Additive only — safe to deploy before the app ships;
old clients keep uploading with the two legacy bases. Rollback = revert the
deploy; the widened CHECK and empty tables are harmless.

## Open Questions

- Per-file difficulty adjustment (needs an edit RPC) — this change or a
  follow-up? Currently: follow-up.
- Should takedown eventually notify the owner (email template) once
  `template-backend-emails` lands? Currently: deferred.
