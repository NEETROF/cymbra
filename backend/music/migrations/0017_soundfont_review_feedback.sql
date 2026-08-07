-- music module — soundfont rejection reason + motivated re-proposal
-- (change: add-soundfont-uploader-attribution).
--
-- Brings the soundfont moderation loop to parity with score proposals
-- (add-score-catalog-proposal): a moderator's rejection carries a human-readable
-- MOTIVE surfaced back to the uploader, and re-proposing a rejected font requires a
-- JUSTIFICATION shown to the moderator on re-review.
--
--   * `review_reason`     — moderator's rejection motive; NULL while pending / when
--                           accepted. Surfaced to the uploader via the private-library
--                           proposal status.
--   * `resubmission_note` — uploader's justification when re-proposing a previously
--                           `rejected` font (which REOPENS that row: status -> pending,
--                           uploaded_by re-attributed, review_reason cleared). NULL until
--                           a resubmission.
--
-- Additive + reversible; no backfill. Idempotent DDL + fully-qualified names.

ALTER TABLE music.soundfonts
    ADD COLUMN IF NOT EXISTS review_reason     TEXT,   -- moderator's rejection motive; shown to uploader
    ADD COLUMN IF NOT EXISTS resubmission_note TEXT;   -- uploader's justification on re-proposal
