## 1. Migration & proto

- [ ] 1.1 Add `backend/music/migrations/0015_soundfont_review_feedback.sql` (see `migrations-note.md`): `soundfonts.review_reason` + `resubmission_note` (additive, idempotent). Extend the soundfont repo row/model + Postgres and in-memory-fake repos with both columns.
- [ ] 1.2 In `backend/music/proto/score.proto`: add a privileged `uploader_display_name` field to the admin soundfont listing message; add a privileged `resubmission_note` field there too; add a public `contributor_credit` field to `ProtoSoundFont`; add an optional `reason` to `SetSoundFontModerationStatusRequest`.
- [ ] 1.3 Regenerate gRPC stubs (backend + Flutter).

## 2. Backend resolution

- [ ] 2.1 Ensure the soundfont gRPC/delivery path holds a `cymbra_user_port::UserPort` (wire the concrete port in `server`, `MockUserPort` in tests); reuse it if `add-score-catalog-proposal` already wired one.
- [ ] 2.2 Admin/console read: resolve each font's `uploaded_by` → `uploader_display_name` via `UserPort` (batched per page, dedupe ids); leave it empty for a seeded font with no uploader and for any normal-caller path. Include `resubmission_note` on the privileged read.
- [ ] 2.3 Public read (`ListSoundFonts` + the `GET /soundfonts/:id` delivery attribution): populate `contributor_credit` for an `accepted` font by resolving the uploader's profile — include the handle/display name only when `visibility == Public` (fail-closed: omit on private/unresolvable/handle-less/no-uploader); keep the licence `attribution` field unchanged; never expose the raw id.
- [ ] 2.4 Extend `set_moderation_status` to accept an optional reason: on `rejected` store `review_reason`; on any other status clear it.
- [ ] 2.5 Extend the propose path with a `resubmission_note`: on a `rejected` content match, require a non-empty note (else refuse), reopen the row (→ `pending`, re-attribute `uploaded_by`, clear `review_reason`, store the note); first proposal / non-`rejected` dedup unchanged.
- [ ] 2.6 Surface `review_reason` to the uploader through the private-font proposal-status resolution.

## 3. Backend tests (≥ 80% lines)

- [ ] 3.1 Admin read (via `MockUserPort`): a font with an uploader shows its pseudo; a seeded font shows none; a normal-caller path never carries the uploader pseudo/id.
- [ ] 3.2 Public read (via `MockUserPort`): `contributor_credit` present for a `Public` uploader; omitted for private/unresolvable/handle-less/seeded and for non-`accepted` fonts; raw `uploaded_by` never present publicly; licence attribution still present.
- [ ] 3.3 Reject stores/clears `review_reason`; re-propose of a `rejected` font reopens the row (→ pending, re-attributed, reason cleared, note stored) and is refused without a justification; the uploader's proposal status carries the rejection reason.

## 4. App (Flutter)

- [ ] 4.1 Surface the public soundfont `contributor_credit` in the soundfont picker/catalog entry when present; show nothing when absent.
- [ ] 4.2 Show the rejection reason on a rejected private font and require a justification field before a re-propose can submit (mirror the score propose sheet).
- [ ] 4.3 Add localized ARB strings (en/fr/es/it) for the contributor credit, rejection reason, and re-proposal justification.
- [ ] 4.4 Widget tests: the picker/catalog shows the credit when present/absent; a rejected font shows its reason and gates re-propose on a justification (fake service).

## 5. Back office (Vue)

- [ ] 5.1 Surface the privileged `uploader_display_name` + `resubmission_note` in the soundfont store/type (behind the existing injectable client seam).
- [ ] 5.2 Show the uploader's pseudo (+ resubmission note when reopened) on the soundfont review-queue rows; add a rejection-reason input to the reject action.
- [ ] 5.3 Vitest/Playwright: a font with an uploader renders the pseudo (+ resubmission note when reopened); a seeded font does not; rejecting sends the reason; via the gated fake-client seam.

## 6. Validation & housekeeping

- [ ] 6.1 `openspec validate add-soundfont-uploader-attribution --strict` passes.
- [ ] 6.2 `melos run analyze` + `dart format` clean; `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean; back-office lint/typecheck clean.
- [ ] 6.3 Coverage ≥ 80% (Rust `cargo llvm-cov` + Flutter `flutter test --coverage` + back-office vitest); `dart run custom_lint` clean.
