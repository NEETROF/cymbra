## 1. Data model & migration

- [x] 1.1 Add `backend/music/migrations/0014_score_catalog_proposal.sql` (additive, idempotent, fully-qualified): `catalog_scores.proposed_by UUID` (nullable) and `user_scores.proposed_catalog_id UUID` (nullable) + supporting index on `catalog_scores.proposed_by`.
- [x] 1.2 Extend the `catalog_scores` repo row/model with `proposed_by`, `review_reason`, `resubmission_note`; extend the `user_scores` repo row/model with `proposed_catalog_id`; update the Postgres and in-memory-fake repos.

## 2. Proto & codegen

- [x] 2.1 In `backend/music/proto/score.proto`: add `ProposeScore` RPC to `ScoreService` with `ProposeScoreRequest { score_id, license, rights_ack, attribution?, resubmission_note? }` / `ProposeScoreResponse`; add `optional string proposal_status` and `optional string rejection_reason` to `ScoreRecord`.
- [x] 2.2 Add privileged-only proposer fields to `CatalogHit`: `proposed_by` (id), `proposer_display_name` (pseudo), `resubmission_note`, and `review_reason` — populated for moderator/admin reads only, like `moderation_status`/`needs_review`.
- [x] 2.3 Add a public `contributor_credit` field to `CatalogHit` — populated for any caller only when the score is `accepted`, user-proposed, and the proposer's profile is `Public` (never the raw id).
- [x] 2.4 Add `optional string reason` to `SetModerationStatusRequest` (the moderator's rejection motive).
- [x] 2.5 Regenerate gRPC stubs (backend + Flutter) and flutter_rust_bridge if the public API surface requires it.

## 3. Backend propose logic

- [x] 3.1 In the music module, implement `propose(owner_id, score_id, license, rights_ack, attribution, resubmission_note)`: load the owned `user_scores` row (reject not-owned/unknown), require `rights_ack` + non-empty `license` (reject otherwise).
- [x] 3.2 Content dedup + status branch: look up the score's SHA-256 against `catalog_scores`. No row → fresh insert. Non-`rejected` match → refuse as duplicate (report the existing id). `rejected` match → the reopen path (3.3b).
- [x] 3.3 Re-propose guard: if the private score's `proposed_catalog_id` links a `pending`/`accepted` catalog row, refuse with a typed already-proposed error.
- [x] 3.3b Reopen path: for a `rejected` same-content row, require a non-empty `resubmission_note` (else refuse); transition the row back to `pending`, re-attribute `proposed_by`, clear `review_reason`, store `resubmission_note`; reuse the row (no second entry).
- [x] 3.4 Materialise a fresh catalog row (no prior row): copy the private score's bytes into a catalog object key, copy server-derived metadata, set `source='user-proposal'`, `source_url=''`, `source_item_id=<user_scores id>`, `confidence='unverified'`, `conversion_status='converted'`, `origin_format` from the stored file, `proposed_by=owner`.
- [x] 3.5 Branch initial `moderation_status` on the proposer's role: music-scope `admin` → `accepted`, else `pending`; never read a client-supplied status.
- [x] 3.6 Link back: set `user_scores.proposed_catalog_id` to the catalog id (fresh or reopened) in the same unit of work.
- [x] 3.7 Join the linked catalog row into `ListMyScores` so `ScoreRecord.proposal_status` reflects not-proposed / `pending` / `accepted` / `rejected`, and `ScoreRecord.rejection_reason` carries the moderator's reason for a `rejected` proposal.
- [x] 3.8 Extend `set_moderation_status` to accept an optional reason: on `rejected` store `review_reason`; on any other status clear it; stamp `reviewed_by`/`reviewed_at` as today.

## 4. Backend gRPC handler & privileged attribution

- [x] 4.1 Add the `propose_score` handler in `grpc.rs` (owner from the interceptor identity; pass `resubmission_note`; map module errors to typed gRPC statuses: duplicate → `ALREADY_EXISTS`, missing attestation/justification → `INVALID_ARGUMENT`, not-owned/unknown → `NOT_FOUND`).
- [x] 4.1b Pass `SetModerationStatusRequest.reason` through the `set_moderation_status` handler to the module (moderator/admin gate unchanged).
- [x] 4.2 Confirm the private-upload path (`upload_score`) is unchanged and still writes only to `user_scores`.
- [x] 4.3 Give the score/catalog gRPC service a `cymbra_user_port::UserPort` dependency (wire the concrete port in `server`, `MockUserPort` in tests), as `play_module`/`play_grpc` already do.
- [x] 4.4 On the privileged (moderator/admin) catalog read (review-queue/search), resolve `proposed_by` → `proposer_display_name` via `UserPort` and populate the privileged `CatalogHit` fields; leave them empty/absent for a normal caller so a proposer's identity never leaks through a public read path.
- [x] 4.5 On the public catalog read, populate `contributor_credit` for an `accepted` user-proposed score by resolving the proposer's profile via `UserPort::get_player_profile`: include the handle/display name only when `visibility == Public` (fail-closed — omit on private/unresolvable/handle-less); never include the raw `proposed_by` id.

## 5. Backend tests (≥ 80% lines)

- [x] 5.1 Unit-test the propose logic on the in-memory fake repos: happy path (pending) + admin (accepted), missing licence/attestation refused, not-owned/unknown refused.
- [x] 5.2 Test content dedup + reopen: byte-identical non-`rejected` → refused with existing id; re-propose of `pending`/`accepted` → already-proposed; `rejected` same-content → reopens the SAME row to `pending`, re-attributes `proposed_by`, clears `review_reason`, stores `resubmission_note`, no second row; reopen without a justification → refused.
- [x] 5.3 Test that `ProposeScore` sets `proposed_by`, links `proposed_catalog_id`, and that `ListMyScores` reports the joined `proposal_status` and `rejection_reason`; and that `set_moderation_status` stores the reason on reject and clears it otherwise.
- [x] 5.4 Assert an un-proposed score never appears via the catalog/search/review read paths (private stays private).
- [x] 5.5 Test the privileged read populates `proposer_display_name` (via `MockUserPort`) + origin for a user-proposed row, and that a normal-caller read leaves proposer id/pseudo empty.
- [x] 5.6 Test the public `contributor_credit` gating (via `MockUserPort`): present when the proposer is `Public`, omitted when `Private`/unresolvable/handle-less, and never for a non-`accepted` or crawler row; raw id never present publicly.

## 6. Flutter service & state

- [x] 6.1 Add a `propose(scoreId, {license, attestation, attribution, resubmissionNote})` method to the score upload/contributions service seam (injectable; overridable with a fake in tests) calling `ProposeScore`.
- [x] 6.2 Add `proposalStatus` + `rejectionReason` to the contributed-score model and a `proposeToPublicCatalog(id, {license, attestation, resubmissionNote})` notifier method that calls the service and optimistically tags the row `pending` (mirror `imported_soundfonts`).
- [x] 6.3 Map server refusals (duplicate / already-proposed / missing attestation) to localized messages — no raw gRPC/exception strings in the UI.

## 7. Flutter UI (two entry points)

- [x] 7.1 In the contributions list, show each score's proposal state tag (not proposed / pending / accepted / rejected) and offer the propose action only for a not-yet-proposed score (mirror the SoundFont management screen).
- [x] 7.2 Add an opt-in propose step at the end of the upload wizard (after a successful upload) that calls the same `proposeToPublicCatalog` seam; declining leaves the score private (no pre-ticked default).
- [x] 7.3 Add the shared propose sheet/dialog: licence declaration field + right-to-distribute attestation checkbox; submit disabled until both are provided; hide the action once submitted. For a `rejected` contribution, show the moderator's rejection reason and require a non-empty justification field before the re-propose can submit.
- [x] 7.4 Show the public `contributor_credit` ("proposé par @pseudo") on the score cover/detail when present; show nothing when absent.
- [x] 7.5 Add localized ARB strings (en/fr/es/it) for the propose action, the wizard step, status tags, attestation copy, the contributor credit, and refusal messages.

## 8. Flutter tests (≥ 80% lines)

- [x] 8.1 Widget/state tests: propose action gated on licence + attestation; hidden once proposed; status tag rendered per state; refusal surfaces a localized message (fake service).
- [x] 8.2 Wizard test: the opt-in propose step is offered after upload, declining keeps the score private, accepting calls the propose seam.
- [x] 8.3 Credit test: the score cover/detail shows the contributor credit when present and nothing when absent (fake catalog data).

## 9. Back office (Vue)

- [x] 9.1 Surface the new privileged `CatalogHit` fields (proposer id/pseudo, `resubmission_note`) in the catalog store/type (behind the existing injectable client seam).
- [x] 9.2 In the review-queue/catalog table, show a user-proposal origin badge (distinct from a crawler dataset origin), the proposer's pseudo, and a reopened score's resubmission justification.
- [x] 9.3 Add a rejection-reason input to the reject action (pass `SetModerationStatusRequest.reason` when rejecting).
- [x] 9.4 Vitest/Playwright: a user-proposed row renders the origin badge + pseudo (+ resubmission note when reopened); a crawler row does not; rejecting sends the reason; via the gated fake-client seam.

## 10. Validation & housekeeping

- [x] 10.1 `openspec validate add-score-catalog-proposal --strict` passes.
- [x] 10.2 `melos run analyze` + `dart format` clean; `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean; back-office lint/typecheck clean.
- [x] 10.3 Coverage ≥ 80% (Rust `cargo llvm-cov` + Flutter `flutter test --coverage` + back-office vitest); `dart run custom_lint` clean.
