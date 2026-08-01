## Context

Back-office moderators/admins work primarily in the catalog browse table
(`apps/back-office/src/views/CatalogView.vue` + `components/CatalogTable.vue`), backed by
`stores/catalog.ts` (`useCatalogStore`) which talks to `ScoreService` over the gRPC-web
seam in `lib/api.ts`. The detail view already fetches and decodes a score's MusicXML bytes
via `store.fetchBytes(id)` → `api().score.getCatalogScoreBytes({ catalogId })`, which the
backend serves from object storage after `decode_canonical`-decompressing the stored
`.mxl` (`backend/music/src/module.rs::get_catalog_bytes`, guarded so `moderator`/`admin`
callers get `allow_unvalidated`). The user asked for the download to live **in the catalog
table** (per-row), reusing the existing byte-serving RPC while **verifying the request's
provenance** (i.e. it must come from an authorized back-office operator).

The repo's Vue architecture rule (see the `vue-frontend-architecture` skill) requires: a
component NEVER calls the API directly — only the store does, behind the injectable client
seam; and async state is modeled as one `Async<T>` discriminated union, never scattered
`loading`/`error` refs. The `no-raw-technical-errors-in-ui` convention forbids surfacing
gRPC/exception strings to users.

## Goals / Non-Goals

**Goals:**
- One-click download of a catalog score's canonical MusicXML from the catalog table row,
  without opening the detail view.
- Reuse `GetCatalogScoreBytes` — no proto/backend RPC changes.
- Enforce provenance: only `music` moderator/admin operators can download; the control is
  hidden for others and the byte path stays guarded.
- Per-row, non-blocking loading/error state with localized messages.

**Non-Goals:**
- No new backend RPC, no signed-URL / direct-object-storage download path.
- No download of the original compressed `.mxl` (served bytes are decoded canonical
  MusicXML; a raw-object download would need a new RPC and is out of scope).
- No bulk / multi-select download, no download from the detail view (table only, per the
  chosen scope).
- No dedicated server-side download audit log (reusing the existing RPC; provenance is the
  moderator/admin guard, not a new audit trail).

## Decisions

### Reuse `GetCatalogScoreBytes`, fetch on demand per click
The catalog list rows carry metadata only, not bytes, so the download must fetch bytes on
click via the existing `api().score.getCatalogScoreBytes({ catalogId })`. Chosen over
prefetching bytes for every visible row (wasteful — most rows are never downloaded) and
over adding a new RPC (unnecessary; the existing one already returns decoded MusicXML and
is correctly guarded). Alternative considered — download only from the detail view where
bytes are already loaded — rejected because the user specifically wants it in the catalog
table.

### Store action owns the fetch + Blob; component only triggers the save
Add `useCatalogStore.downloadBytes(catalogId, fileNameBase)` that runs the RPC through the
`Async<T>` helper (`run`/`idle` from `lib/async.ts`) and returns the resolved
`Uint8Array`. The DOM save (create `Blob`, `URL.createObjectURL`, click a transient
`<a download>`, revoke the URL) lives in the component, since building a Blob/anchor is a
browser-DOM concern, not data access — but the network call stays in the store to honor
the "components never call the API directly" rule. The filename is sanitized and suffixed
`.musicxml`.

### Per-row async state keyed by catalog id
Track download state as a map from `catalogId` to an `Async<Uint8Array>` (or a `Set` of
in-flight ids plus a per-id error) so each row shows its own spinner/error and one row's
failure or latency never blocks the table. Chosen over a single store-wide `downloading`
flag, which would disable the whole table during any one download.

### Provenance = existing moderator/admin guard, surfaced in the UI gate
The backend already restricts any-status byte access to `moderator`/`admin`
(`allow_unvalidated` in `get_catalog_score_bytes`; see the `moderation-access-control`
spec). The download reuses that guard as its provenance check — no new server code beyond
confirming the guard covers this path. The client renders the download control only when
`auth.isModerator` (matching how `ScoreDetailView` gates moderator actions), so
non-moderators never see it, and an unauthorized byte request is still refused server-side.

### Errors localized, raw cause logged only
Map any RPC failure (including the typed `FailedPrecondition` "bytes not available yet")
to a localized i18n message shown on the row; the raw error is logged, never displayed,
per `no-raw-technical-errors-in-ui`.

## Risks / Trade-offs

- [Large scores fetched fully into memory before saving] → Acceptable: MusicXML scores are
  small (KB–low MB) and this mirrors the detail-view preview which already loads full
  bytes; no streaming needed.
- [Fetch-on-click adds a round trip per download] → Acceptable and expected; avoids
  prefetching bytes for rows that are never downloaded. Per-row loading state covers the
  latency.
- [Downloaded file is decoded `.musicxml`, not the stored `.mxl`] → Documented behavior;
  file is named `.musicxml` to match. If admins later need the original `.mxl`, that is a
  separate new RPC (out of scope).
- [Filename derived from user/corpus-supplied title] → Sanitize the title (strip path
  separators / control chars, cap length) before using it as a filename to avoid odd or
  unsafe names; fall back to the identifier.

## Open Questions

- Should the download also be offered from the detail view for convenience? Deferred —
  current scope is the catalog table only; the same store action could be reused later.
