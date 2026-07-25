## Why

Today every score in `music.catalog_scores` is publicly visible in the app hub the
moment it exists — the crawler auto-publishes everything it ingests, and the
`search_catalog` query returns all rows with no visibility gate. There is no way to
keep an unreviewed or low-quality score out of the hub. We want catalog scores to be
**hidden from the hub until a human validates them**, and newly crawled scores to
arrive **unvalidated by default**. This change introduces the moderation status and
the read-side gating that later changes (app swipe-rating, back-office review tool)
build on.

## What Changes

- **New moderation status on catalog scores** — a `moderation_status` of
  `pending` | `accepted` | `rejected` (default `pending`), plus an audit trail of
  **who** reviewed a score and **when** (so a rejected score is traceable to a
  moderator). Only `accepted` scores are "validated".
- **BREAKING (data): all existing catalog scores reset to `pending`** — a migration
  backfills every current row to `pending`, so the hub is empty until scores are
  validated one by one. This is intentional and matches the request: no automatic
  bulk-validation of the existing corpus.
- **Hub only shows validated scores** — `search_catalog` returns only `accepted`
  rows to normal callers. No app-side change is required; the gate is enforced
  server-side so unvalidated scores simply never reach the Flutter front.
- **Crawler ingests as unvalidated** — newly ingested scores are inserted as
  `pending`, never auto-validated, regardless of licensing `confidence`.
- **Back-office-only status filter (server-gated)** — `search_catalog` gains an
  optional `moderation_status` filter that the server **accepts only from an
  admin/moderator identity and rejects (PERMISSION_DENIED) from anyone else**. The
  Flutter app never sends it; it exists for the future back office. Until the
  `moderator` role lands (change #3), the gate authorizes `admin` only.

Out of scope (later changes): the app-side swipe/star rating UI and the rating→queue
priority signal (change #2); the moderator role, the evaluate (accept/reject) write
RPC, role-granting, and the Vue back-office app (change #3). This change delivers the
data model, the read-side gating, and the crawler default that those depend on.

## Capabilities

### New Capabilities
- `score-moderation`: The moderation lifecycle of catalog scores — the
  `pending`/`accepted`/`rejected` status with `pending` as the default, the invariant
  that only `accepted` scores are publicly exposed, the review audit trail
  (reviewer + timestamp), the one-time backfill of existing rows to `pending`, and the
  authorization rule that only admins/moderators may query by status.

### Modified Capabilities
- `catalog-search`: `search_catalog` MUST exclude non-`accepted` scores for normal
  callers, and MUST accept an optional privileged `moderation_status` filter only from
  an authorized (admin/moderator) identity, rejecting it otherwise.
- `corpus-ingestion`: newly ingested catalog scores MUST be persisted as `pending`
  (unvalidated) and MUST NOT be auto-validated on the basis of licensing confidence.

## Impact

- **DB migration** (`backend/music/migrations/`): new `moderation_status`,
  `reviewed_by`, `reviewed_at` columns on `music.catalog_scores` (CHECK-constrained
  status, default `pending`, index for status queries); backfill of existing rows to
  `pending`.
- **Rust storage/model** (`backend/music/src/repo.rs`, `pg.rs`): `CatalogEntry` and
  `PgCatalogRepo::insert` set `pending`; `HIT_COLS`/`PgCatalogSearchRepo::search`
  (`backend/music/src/pg.rs`) add the `WHERE moderation_status = 'accepted'` default
  and the optional privileged status filter.
- **gRPC surface** (`backend/music/src/grpc.rs`, score proto): `SearchCatalogRequest`
  gains an optional status filter field; `search_catalog` gates it via
  `require_admin` (`backend/platform/src/guard.rs`) reading `AuthIdentity`.
- **Crawler** (`crates/score-crawler/`): ingestion path (`catalog.rs` →
  `PgCatalogRepo::insert`) carries the `pending` default; no auto-validation.
- **No Flutter app change** and **no new client field is sent** by the app. Coverage
  gates (Rust ≥ 80%) apply to the new storage/search/guard logic.
