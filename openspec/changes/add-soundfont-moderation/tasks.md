## 1. Database migrations

- [x] 1.1 Migration on `music.soundfonts`: add `moderation_status TEXT NOT NULL DEFAULT 'pending' CHECK IN ('pending','accepted','rejected')`, `reviewed_by UUID`, `reviewed_at TIMESTAMPTZ`, `uploaded_by UUID`, `content_sha256 TEXT`; index on `moderation_status`
- [x] 1.2 Backfill in the same migration: `upright-piano-kw` → `accepted`; all other existing rows → `pending`
- [x] 1.3 Backfill `content_sha256` for existing rows (compute from stored objects, or leave NULL + note that dedup only guards new uploads until re-hashed)
- [x] 1.4 New table `music.user_soundfonts` (owner `user_id`, `label`, `object_key`, `content_sha256`, `size_bytes`, timestamps) with an index on `user_id`; additive + reversible

## 2. Backend — public catalog moderation (soundfont-moderation)

- [x] 2.1 Extend `FontEntry` + `SoundFontRepo` (backend/music) with `moderation_status`, `reviewed_by`, `reviewed_at`, `uploaded_by`, `content_sha256`
- [x] 2.2 Repo reads: public `list_accepted` returns `accepted` only; `list` (admin) returns any status; `lookup` any status; `FakeSoundFontRepo` in parity
- [x] 2.3 `set_moderation_status(id, status, reviewer_id)` stamping `reviewed_by`/`reviewed_at`; reject unknown id; `rejected` keeps row + object
- [x] 2.4 Content-identity lookup by `content_sha256` (against non-`rejected` rows) used before insert
- [x] 2.5 Host-tested pure logic + repo tests (fake) for gating, status transitions, dedup

## 3. Backend — delivery + upload gating (soundfont-delivery)

- [x] 3.1 Delivery `decide()`: normal caller → `accepted` only (else not-found); moderator/admin → any status. (private-library owner path in phase 4)
- [x] 3.2 Upload handler: compute SHA-256, refuse byte-identical duplicate (409 + existing id), branch status by role (admin → `accepted`, else → `pending`), record `uploaded_by`; back-office-audience uploads stay admin/moderator-only
- [x] 3.3 App uploads target the private library via the new `/me/soundfonts` routes (open to any authenticated user), separate from the admin catalog upload
- [x] 3.4 Handler tests: accepted-only gate, moderator audition, duplicate refusal, role branching (audience routing in 3.3)

## 4. Backend — private library + proposal (user-soundfont-library)

- [x] 4.1 `UserSoundFontRepo` + private per-user storage prefix (`user/{uid}/{id}.sf2`); list/import/get scoped by `user_id`; idempotent import by `content_sha256`
- [x] 4.2 Per-user quota (max 5 fonts for non-moderator/admin; mod/admin exempt) enforced on import (403)
- [x] 4.3 Propose route `POST /me/soundfonts/:id/propose` (license + attribution + attestation): copies bytes to a public key, creates a `pending` catalog row attributed to the proposer; refuses missing licence/attestation (400), non-owned (404), duplicate content/id (409)
- [x] 4.4 Routes: `GET/POST /me/soundfonts` (list + import) + `GET /me/soundfonts/:id` (owner delivery); propose in 4.3
- [x] 4.5 Tests: owner-only visibility + listing, idempotent import, quota (proposal validation + dedup with 4.3)

## 5. gRPC / proto

- [x] 5.1 Gate `ListSoundFonts` to `accepted`; `AdminListSoundFonts` now carries moderation status/reviewer/uploader/content_sha256
- [x] 5.2 Add `SetSoundFontModerationStatus(id, status)` (music-scope moderator/admin)
- [ ] 5.3 Add private-library + propose messages/RPCs (or document the HTTP routes if kept HTTP)
- [ ] 5.4 Regenerate proto/bridge/clients (backend proto regenerated via tonic-build; Dart `gen-grpc` + Vue gRPC-web still to do)

## 6. Back office (moderation-console)

- [x] 6.1 Sound fonts screen: status badges (AppTag pending/accepted/rejected) + a back-office-only status filter
- [x] 6.2 Pending review queue view (default filter = pending, with a count badge)
- [x] 6.3 Audition before deciding: `fontBytes` fetches a non-`accepted` font (moderator privilege) via the existing edit-drawer preview
- [x] 6.4 Accept/Reject actions wired to `setSoundFontModerationStatus`; failures land in `op` (error), access control server-side + router-gated
- [x] 6.5 Store tests for the moderation action (success re-lists; denied → op error) via the fake-client seam

## 7. Flutter app

- [x] 7.1 `PrivateSoundFontService` (HTTP over `/me/soundfonts`); `ImportedSoundFonts.importSoundFont` uploads on import; picker sources the synced registry + accepted public catalog
- [x] 7.2 Cross-device sync: `build()` migrates local imports up + pulls the server library down, caching bytes to a local file for the FFI
- [x] 7.3 Propose flow: `proposeToPublicCatalog` + a licence/attribution/attestation dialog in the sound manager (submit disabled until licence + attestation); errors via snackbar
- [x] 7.4 Migration of device-local imports handled in the sync (idempotent by content SHA-256); `remove` also deletes server-side (new `DELETE /me/soundfonts/:id`) so it doesn't re-sync
- [x] 7.5 Notifier tests (import/propose/remove/sync via a fake service) + widget picker tests; `flutter analyze` + `custom_lint` clean; 678 tests pass

## 8. Cross-cutting: coverage, quotas, docs

- [x] 8.1 Backend `cargo test` (136 music + 40 server) + `clippy -D warnings` clean; Flutter 678 tests pass; back-office 165 vitest. (llvm-cov / very_good_coverage run in CI)
- [x] 8.2 Quota = **5 fonts** per non-moderator/admin user (`USER_LIBRARY_MAX_FONTS`), mod/admin exempt
- [x] 8.3 `backend/scripts/soundfonts.json` documents that admin-token uploads stay auto-accepted while moderator/user contributions go through the review queue
- [x] 8.4 `openspec validate add-soundfont-moderation --strict` passes
