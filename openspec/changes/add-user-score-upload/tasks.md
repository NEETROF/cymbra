## 1. Shared MusicXML core (client + server parity)

- [ ] 1.1 Extract the pure parser from `apps/music/rust/src/api/musicxml_core.rs` into a workspace crate `crates/musicxml-core` (enable the `crates/*` member glob in the root `Cargo.toml`); keep `apps/music/rust` depending on it via the FFI seam so app behavior is unchanged.
- [ ] 1.2 Add `.mxl` (zip) decoding to the shared core: read `META-INF/container.xml` → rootfile → underlying MusicXML bytes; add a bounded `zip` dependency with a max-decompressed-size guard.
- [ ] 1.3 Add a `validate(bytes) -> Result<ScoreSummary, RejectReason>` entry point that decodes (plain or `.mxl`), parses, and confirms the score contains playable piano notes; return typed rejection reasons (undecodable, unparseable, no-notes, too-large).
- [ ] 1.4 Host-testable unit tests for the shared core: valid plain XML, valid `.mxl`, corrupt zip, unparseable XML, empty/no-note score, oversized/zip-bomb guard (keep coverage ≥ 80%, per `CLAUDE.md`).

## 2. Object storage (greenfield)

- [ ] 2.1 Add an S3-compatible object-store client crate to the workspace matching the `tls-rustls`/no-OpenSSL stack (e.g. `aws-sdk-s3` or `object_store`).
- [ ] 2.2 Add `CYMBRA_SCORE_S3_*` config (bucket, endpoint, region, credentials) to the typed `backend/platform/src/config.rs` `Config::from_env`, failing fast when required keys are missing.
- [ ] 2.3 Document the new keys in `backend/.env.example` and wire a local MinIO/S3-compatible service in `backend/docker-compose.yml` with dev defaults.
- [ ] 2.4 Implement a small storage port (put/get/delete by key) with a fake for tests; keep the real S3 glue in a coverage-excluded seam.

## 3. Backend `score` module

- [ ] 3.1 Scaffold `backend/score-port/` (`proto/score.proto`, `build.rs`, `ScorePort` trait + domain structs) mirroring `backend/user-port/`.
- [ ] 3.2 Define the gRPC surface in `score.proto`: `UploadScore(bytes, filename, difficulty, authorship_ack) -> ScoreRecord`, `ListMyScores() -> [ScoreRecord]`, `DeleteScore(id)`; regenerate app stubs (`apps/music/lib/src/grpc/`) and run `flutter_rust_bridge_codegen generate` if the Rust public API changed.
- [ ] 3.3 Add migration `backend/score/migrations/0001_init.sql`: `user_scores(id UUID pk, owner_id UUID not null, object_key text unique not null, title text, difficulty text not null check (…), authorship_ack boolean not null, size_bytes int, content_hash text, created_at timestamptz not null default now())` in the module's own schema; **no cross-schema FK** on `owner_id`.
- [ ] 3.4 Implement `score/src/module.rs`, `repo.rs` (+ `FakeScoreRepo`), `pg.rs` (`PgScoreRepo`, runtime `sqlx::query(...).bind(...)`), and `lib.rs` (`MIGRATOR`), following the `user` module.
- [ ] 3.5 Implement `score/src/grpc.rs`: read `AuthIdentity` from request extensions; reject unauthenticated calls; scope every query to `owner_id`; owner-only delete (compare `AuthIdentity.user_id` to `owner_id`).
- [ ] 3.6 Upload handler: server-side `validate()` (task 1.3) on received bytes → reject invalid/oversized before storage; on success put canonical bytes under `user-scores/{owner_id}/{uuid}.musicxml`, then insert the record; reject non-affirmative authorship or out-of-set difficulty.
- [ ] 3.7 Delete handler: remove the record, then delete the object; if the object delete fails, enqueue an idempotent object-delete job so no record ever points at a missing object.
- [ ] 3.8 Provision the module's least-privilege DB role/schema (extend `backend/db/init/roles.sql.tpl`) per `ops-db-access`.
- [ ] 3.9 Wire the module in the composition root `backend/server/src/main.rs` (own pool, run `MIGRATOR`, `add_service`).

## 4. Backend cleanup & account erasure

- [ ] 4.1 Add an idempotent `purge_score_object` worker job (`backend/worker/src/handlers.rs` + register in `backend/jobs/src/registry.rs`) that deletes a stored object by key.
- [ ] 4.2 Extend the existing `purge_user` job to also delete the user's `user_scores` rows (via the admin pool) and enqueue their object deletions, satisfying account-deletion erasure.

## 5. Backend tests

- [ ] 5.1 Module/handler tests with `FakeScoreRepo` + fake storage: upload happy path, invalid/oversized rejection (nothing stored), list isolation across users, owner-only delete, non-owner delete rejected, authorship/difficulty validation.
- [ ] 5.2 Test the delete partial-failure path (object delete fails → record gone, cleanup job enqueued) and the account-erasure purge.
- [ ] 5.3 `cargo llvm-cov` ≥ 80% for the new host-testable logic; `cargo fmt` + `clippy -D warnings` clean.

## 6. App — upload dependencies & services

- [ ] 6.1 Add `file_picker` to `apps/music/pubspec.yaml`; wrap it in an injectable `filePickerProvider` (faked in tests).
- [ ] 6.2 Add a `scoreUploadService` provider wrapping the generated score gRPC stub via `authedCall` (`apps/music/lib/services/grpc_client.dart`), with a fake for tests.
- [ ] 6.3 Add a backend-backed score source (paralleling `apps/music/lib/services/score_asset_source.dart`) that fetches a contributed score's bytes for the player.

## 7. App — contribution wizard

- [ ] 7.1 `ScoreUploadNotifier` (`@riverpod` + Freezed state) modelling the `pickFile → validating → previewing → confirming → submitting → done/error` state machine with step gating.
- [ ] 7.2 Build the three-step screen (`Navigator.push` from the library), gated on `SessionAuthenticated` (`sessionNotifierProvider`); hide/disable the entry point when signed out.
- [ ] 7.3 Upload step: pick `.musicxml`/`.xml`/`.mxl`, run client-side validation via `notationEngineProvider` (reuse shared core over FFI), surface typed rejection reasons, and gate submit behind the mandatory authorship CGU checkbox.
- [ ] 7.4 Verification step: render the score horizontally with `StaffPainter`, playback locked to the score's own tempo (`playerProvider` at `speed = 1.0`, `notationToTimedNotes`), with all tempo/Wait/practice controls hidden.
- [ ] 7.5 Confirmation step: require choosing a `PracticeLevel` (Beginner/Intermediate/Advanced) before finalizing; submit bytes + difficulty + authorship via `scoreUploadService`; reflect success/typed failure without losing inputs.

## 8. App — library integration & deletion

- [ ] 8.1 Extend `score-library` (`apps/music/lib/state/score_catalog.dart`) to list the signed-in user's contributed scores (via `ListMyScores`) as a section distinct from the bundled catalog; no section when signed out.
- [ ] 8.2 Allow selecting a contributed score to open the player, loading bytes via the backend source (task 6.3) through the existing `SelectedScore` → `NotationProvider` → player path.
- [ ] 8.3 Offer a delete action only on owned contributed scores (never on bundled entries); on confirm, call `DeleteScore` and remove the entry on success.

## 9. App tests

- [ ] 9.1 Unit/notifier tests for `ScoreUploadNotifier` (step gating, CGU gate, difficulty gate, validation rejections, submit success/failure) with fake picker/engine/service.
- [ ] 9.2 Widget tests for the three-step screen (auth gating, horizontal tempo-locked preview shows no practice controls) and for the library contributed-scores section + owner-only delete.
- [ ] 9.3 `flutter test --coverage` ≥ 80% (exclusions per `CLAUDE.md`); `melos run analyze` + `dart format` + `custom_lint` clean.

## 10. Verification & spec hygiene

- [ ] 10.1 Integration test (`apps/music/integration_test/`) driving upload → verify → confirm with a fixture score against fakes; run `melos run integration`.
- [ ] 10.2 Manual end-to-end against a local backend + MinIO: upload a real `.mxl`, verify preview, confirm, see it in the library, play it, delete it.
- [ ] 10.3 `openspec validate add-user-score-upload --strict` passes.
