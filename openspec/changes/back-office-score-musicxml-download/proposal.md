## Why

Moderators and admins reviewing the score catalog in the back office can preview a
score's notation and audio, but they have no way to pull the underlying MusicXML file
onto their own machine — needed to inspect it in external notation software, reproduce
a conversion/ingestion bug, or archive a reference copy. The bytes are already fetched
and decoded server-side for the existing preview path, so exposing a download is a small,
high-value addition to the catalog table they already work in.

## What Changes

- Add a per-row **Download MusicXML** action to the back-office catalog table
  (`CatalogTable` / `CatalogView`), available to moderators/admins, that saves the
  score's canonical MusicXML to the operator's machine.
- Clicking the action fetches the score's decoded MusicXML bytes on demand (reusing the
  existing `GetCatalogScoreBytes` RPC), wraps them in a Blob, and triggers a browser
  download named from the score (`<title-or-id>.musicxml`).
- Keep the RPC/data-fetch in the Pinia store (per the repo's Vue architecture rule); the
  component only triggers the DOM download once the bytes resolve.
- **Provenance / authorization**: the download is served only to an authenticated
  back-office moderator/admin in the `music` audience — the same gating that already lets
  moderators fetch any-status score bytes — and the download control is rendered only for
  such operators. No new backend RPC is introduced; the existing byte-serving guard is the
  provenance check.
- Surface per-row loading/error state for the download without disrupting the rest of the
  table (a failed or empty-object score reports a localized error, never a raw gRPC
  string).

## Capabilities

### New Capabilities
- `back-office-score-download`: A back-office affordance for a moderator/admin to download
  the canonical MusicXML of any catalog score (any moderation status) to their local
  machine from the catalog table, with proper authorization (provenance) and localized
  per-row feedback.

### Modified Capabilities
<!-- No existing requirement changes: moderator/admin byte access is already granted by
     moderation-access-control, and the feature reuses GetCatalogScoreBytes unchanged. -->

## Impact

- **Front-end (`apps/back-office`)**: `components/CatalogTable.vue` (new per-row action +
  loading/error affordance), `views/CatalogView.vue` (wire the action to the store),
  `stores/catalog.ts` (new `downloadBytes(catalogId, filename)` action reusing
  `api().score.getCatalogScoreBytes`, tracked as an `Async<T>` per row), and localized
  strings for the button/tooltip/errors.
- **Backend**: none required — reuses `ScoreService.GetCatalogScoreBytes`
  (`backend/music/src/grpc.rs`, `module.get_catalog_bytes`), which already resolves the
  `object_key`, streams from `ObjectStorage`, and `decode_canonical`-decompresses `.mxl`
  to uncompressed MusicXML. Confirm the moderator/admin `allow_unvalidated` gating covers
  the download path.
- **Proto/API**: no proto change.
- **Tests**: back-office store + component tests (fake client seam) for success, empty
  bytes, and error; verify the download control is gated to moderators/admins.
- **Note**: served bytes are decoded/uncompressed canonical MusicXML, so the download is
  named `.musicxml` (not the stored `.mxl`).
