## Context

The public catalog (`music.catalog_scores`) is auto-published: the crawler inserts
rows via `PgCatalogRepo::insert` (`backend/music/src/pg.rs`) and the hub's
`search_catalog` (gRPC, `backend/music/src/grpc.rs` → `PgCatalogSearchRepo::search`)
returns every matching row. The only quality signals are `confidence`
(`verified`/`unverified`, a licensing signal) and `conversion_status` — neither gates
visibility. There is no moderation/review lifecycle.

Backend constraints that shape this design:
- The public API is **gRPC over tonic only**; `backend-service` spec forbids REST.
- **RBAC already exists**: `user_roles(user_id, scope, role)` with scopes
  `global|music|live` and roles `user|admin`; tokens carry `roles`; guards
  `require_admin`/`require_role` exist (`backend/platform/src/guard.rs`) reading the
  `AuthIdentity` injected by the auth interceptor. There is **no `moderator` role yet**.
- The crawler is a **synchronous CLI**, not a worker job; the insert default lives in
  the storage layer it calls.

This change is #1 of 3. It delivers only the data model + read-side gating + crawler
default. The app rating UI (#2) and the back office / evaluate action / `moderator`
role (#3) build on the columns and the gated filter introduced here.

## Goals / Non-Goals

**Goals:**
- Add a persisted moderation status (`pending`/`accepted`/`rejected`, default
  `pending`) and a review audit trail (reviewer id + timestamp) to catalog scores.
- Reset all existing rows to `pending` so the hub shows nothing until validated.
- Make `search_catalog` return only `accepted` scores to normal callers, enforced
  server-side so the Flutter front needs no change and cannot see unvalidated scores.
- Make the crawler insert new scores as `pending`, never auto-validated.
- Add a privileged, back-office-only status filter to `search_catalog`, refused for
  non-admin/moderator callers.

**Non-Goals:**
- The write path to change a score's status (accept/reject) — that RPC lands in #3.
- The `moderator` role and role-granting — #3 (this change gates on `admin` only).
- Any Flutter UI change, and the swipe/star rating feature — #2.
- Moderation of user uploads (`music.user_scores`); scope is the public catalog only.

## Decisions

### D1 — Status model: `moderation_status` enum with audit columns on `catalog_scores`
Add three columns to `music.catalog_scores`:
`moderation_status TEXT NOT NULL DEFAULT 'pending' CHECK (moderation_status IN ('pending','accepted','rejected'))`,
`reviewed_by UUID` (nullable; the moderator's `users.id`), `reviewed_at TIMESTAMPTZ`
(nullable). Add a partial/plain index on `moderation_status` for the hub's hot path
(`WHERE moderation_status = 'accepted'`).
- **Why a status enum over a boolean `validated`**: the request needs a third state
  (`rejected`) that is distinct from "not yet reviewed" (`pending`) so the back office
  can filter and so rejections are traceable and not re-surfaced. A boolean can't
  express that.
- **Why audit columns on the row (not a separate table)**: #1 only needs "who/when
  last reviewed". A single reviewer+timestamp on the row is enough to trace rejections
  and keeps the change small. If a full review history/log is wanted later, #3 can add
  a `score_moderation_events` table without reworking this.
- **Alternative considered**: reuse `confidence`. Rejected — `confidence` is a
  licensing signal with different semantics and values; overloading it would conflate
  legal confidence with editorial validation.

### D2 — Existing rows backfill to `pending`, no bulk-accept
The migration sets `DEFAULT 'pending'`; because the column is added to a populated
table, existing rows also become `pending`. This empties the hub until moderation
runs, exactly as requested.
- **No bulk-validation** of the existing corpus: every score is reviewed individually
  through the back office (#3). The migration deliberately does NOT pre-accept any
  subset (not even `confidence='verified'`), because licensing confidence is not
  editorial validation (see D5 / the `confidence` distinction).
- **Consequence**: the hub is empty on deploy until scores are validated. This is an
  accepted, intentional trade-off — sequence the deploy so #3's review tooling is
  available (see Risks).

### D3 — Read gating in `PgCatalogSearchRepo::search`, not in the app
`search_catalog` always applies `moderation_status = 'accepted'` unless the caller is
authorized AND explicitly passes a status filter.
- **Why server-side**: the request is explicit that the status filter must be
  back-office-only and "surtout pas front Flutter". Enforcing in SQL guarantees an
  unvalidated score can never reach the app even if a client is modified. `HIT_COLS`
  need not expose the status to normal callers.
- **Reviewer access**: an admin/moderator caller MUST be able to both list/search AND
  **open** (fetch bytes of) a `pending`/`rejected` score to evaluate it. So the same
  identity check that authorizes the privileged status filter (D4) also authorizes
  fetch-bytes for non-`accepted` ids; normal callers get not-found. Admin-only until
  the `moderator` role lands in #3.

### D4 — Privileged status filter: optional proto field + `require_admin` gate
Add an optional `moderation_status` field to `SearchCatalogRequest` (proto). In
`search_catalog` (`grpc.rs`): if the field is **set**, call `require_admin(identity)`
(from `backend/platform/src/guard.rs`) first — if it fails, return `PERMISSION_DENIED`
and do not run the query. If the field is **unset**, behave as a normal caller
(accepted-only). Normal app requests never set it.
- **Why gate only when the field is present**: keeps the existing app flow (no field)
  fully unauthenticated-of-status and unchanged, while any status-scoped query is
  privileged.
- **Role today vs #3**: #1 authorizes `admin` only (the only privileged role that
  exists). #3 replaces the guard with "admin OR music-scoped moderator" when the
  `moderator` role is introduced — a one-line guard swap, no proto/schema change.
- **Alternative considered**: a separate `SearchCatalogModeration` RPC. Rejected for
  #1 — it duplicates the whole search/facet surface; reusing the field keeps the "same
  filters as the hub" requirement (from #3) trivially satisfied. Revisit in #3 if the
  moderation table view needs materially different projections.

### D5 — Crawler default via the storage layer
`CatalogEntry` (`backend/music/src/repo.rs`) gains no status field the crawler sets;
instead `PgCatalogRepo::insert` writes `moderation_status = 'pending'` unconditionally
(the DB default also guarantees it). The crawler needs no logic change beyond
recompiling against the model.
- **Why**: ingestion must never auto-validate; making it structurally impossible for
  the insert path to set anything but `pending` is safer than relying on the caller.

## Risks / Trade-offs

- **Empty hub on deploy** → The migration hides the entire existing catalog until
  moderators validate it, one score at a time (no bulk-accept, by decision). Mitigation:
  sequence the rollout so change #3's review tooling (back office) is available before
  #1's migration runs in production, so moderators can work the backlog down. Flag the
  empty-hub window to stakeholders before the prod migration.
- **Client sending the privileged field** → A tampered client could set
  `moderation_status`. Mitigation: the `require_admin` gate rejects it; a normal token
  lacks `admin`, so the request fails closed with `PERMISSION_DENIED`.
- **Index/perf** → adding `AND moderation_status='accepted'` to every hub query.
  Mitigation: index on `moderation_status`; the accepted set is the common path.
- **Coverage** → new SQL/guard branches must keep Rust line coverage ≥ 80%. Mitigation:
  unit-test `search` (accepted-only default, privileged filter honored, filter refused
  for non-admin) and `insert` (defaults to pending) in host-testable modules.

## Migration Plan

1. Add the migration (new columns + CHECK + index + backfill to `pending`). Forward
   migration is additive; existing reads keep working (they just see `pending` rows
   until search is updated in the same release).
2. Update `CatalogEntry`/`insert` and `HIT_COLS`/`search` + proto in the same release
   so the gate is live the moment the column exists.
3. **Rollback**: drop the three columns (data-lossy for review audit only) or, safer,
   set the migration to leave columns and revert the search WHERE clause to unfiltered.
   Because the app sends no new field, rolling back the server alone restores prior
   behavior (all rows visible) without a client release.

## Resolved Questions

- **User uploads — out of scope (decided).** `music.user_scores` are owner-private: a
  user's uploads are visible only to their owner (via `myContributedScoresProvider`),
  not publicly browsable by others, so there is nothing to moderate for public
  visibility. Moderation applies to the **public catalog only**. If publicly-browsable
  user uploads ever become a feature, they get their own gating at that point — not
  here.
