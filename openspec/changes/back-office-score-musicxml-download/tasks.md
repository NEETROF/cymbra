## 1. Backend provenance check (confirmation, likely no code)

- [x] 1.1 Confirm `GetCatalogScoreBytes` (`backend/music/src/grpc.rs:291`, `module.get_catalog_bytes` at `backend/music/src/module.rs:557`) gates any-status bytes to `music` moderator/admin via `allow_unvalidated` (`id.is_admin() || id.has_role("moderator")`). Guard already covers the download path — no backend change needed.
- [x] 1.2 Verified no proto change is needed: `get_catalog_bytes` ends with `decode_canonical(&raw)`, returning uncompressed canonical MusicXML. Documented in `lib/download.ts` (`MUSICXML_MIME` comment) that served bytes are `.musicxml`, not the stored `.mxl`.

## 2. Store: on-demand byte fetch for download

- [x] 2.1 Added `downloadBytes(catalogId)` to `stores/catalog.ts` calling `api().score.getCatalogScoreBytes` (via the existing `fetchBytes`) through `run(toRef(downloads, id), …)`; returns the `Uint8Array` on success, `null` on failure.
- [x] 2.2 Per-row state tracked in `reactive downloads: Record<string, Async<Uint8Array>>` keyed by `catalogId`; success drops the entry (bytes not pinned), errors are retained per-row so one failure never blocks the table.
- [x] 2.3 Added `musicxmlFileName(title, id)` in `lib/download.ts` — sanitizes the title (`\p{Cc}` + path/reserved chars → space, collapse, trim, cap 120), falls back to the identifier, suffixes `.musicxml`.

## 3. UI: per-row download action in the catalog table

- [x] 3.1 Added a per-row download button (Actions column) to `CatalogTable.vue`, rendered only when the `canDownload` prop is set (the view passes `auth.isModerator`); emits `download` with the hit, `@click.stop` so it never triggers row-select.
- [x] 3.2 Wired `@download` in `CatalogView.vue` to `store.downloadBytes(hit.id)`, then `saveBytesAsFile` (Blob `application/vnd.recordare.musicxml+xml` + transient `<a download>` + `revokeObjectURL`) in `lib/download.ts`; filename from `musicxmlFileName(hit.title, hit.id)`.
- [x] 3.3 Per-row `Async` reflected via the `downloads` prop: the active row's button shows a spinner and is disabled; other rows stay interactive; dynamic `colspan` on the empty row.
- [x] 3.4 Failures surface a localized message (the `run`/`humanError` path maps `FailedPrecondition` → `errors.notAvailable`), shown inline on the row and as the button title; the raw cause is only `console.error`-logged, never displayed.

## 4. Localization

- [x] 4.1 Added `table.actions`, `table.download`, `table.downloading` to `en.json` and `fr.json` (parity gate passes); error text reuses the existing localized `errors.*` catalog.

## 5. Tests

- [x] 5.1 `test/catalog.spec.ts`: `downloadBytes` success returns bytes and clears per-row state; RPC rejection maps to a localized per-row error (never the raw cause); concurrent downloads are independent (one failure keeps its own error, another still succeeds).
- [x] 5.2 `test/components.spec.ts`: control hidden without `canDownload`, rendered per row with it; click emits `download` with the hit and does not `select`; a `downloads` map drives per-row disabled/loading + inline error without affecting other rows.
- [x] 5.3 `test/download.spec.ts`: `musicxmlFileName` sanitization, punctuation kept, whitespace/trailing-dot trim, identifier fallback, length cap; plus `saveBytesAsFile` Blob type + named-anchor + `revokeObjectURL` cleanup.

## 6. Verification & polish

- [ ] 6.1 Manual verification in a running back office (needs a live backend + moderator session — not runnable headless here): download a `pending`, `rejected`, and `accepted` score and confirm the saved `.musicxml` opens in external notation software.
- [x] 6.2 `yarn lint`, `yarn typecheck` (clean for all changed files; remaining errors are the pre-existing `@/wasm/pkg*` codegen seams, unrelated), and `yarn vitest run` — the download/catalog/components/i18n suites pass (32 new/edited assertions; 104 total non-wasm tests green). New code is fully exercised by tests.
- [x] 6.3 `openspec validate back-office-score-musicxml-download --strict` → valid.
