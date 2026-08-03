-- music module — user-score catalog proposal + proposer attribution
-- (change: add-score-catalog-proposal).
--
-- A signed-in user's uploaded score lives in the PRIVATE `music.user_scores`
-- library. This change adds the ONE explicit, opt-in bridge into the public
-- catalog: `ProposeScore` materialises a `music.catalog_scores` row from a private
-- score. Two additive columns support that bridge and its attribution:
--
--   1. `catalog_scores.proposed_by` — the proposer's `AuthIdentity.user_id` for a
--      user-proposed row (a PLAIN uuid, no cross-schema FK — matching every other
--      `music` table). NULL for crawler-ingested rows. Resolved to a pseudo at read
--      time via the user directory (privileged console read + opt-in public credit);
--      never stored denormalised.
--   2. `user_scores.proposed_catalog_id` — links a private score to the catalog row
--      created from it, so the app can show the proposal status and the server can
--      guard against re-proposing an already-`pending`/`accepted` score. NULL until
--      the score is proposed.
--   3. `catalog_scores.review_reason` — the moderator's motive when a proposal is
--      rejected, surfaced back to the proposer so they know why. NULL while pending /
--      when accepted without a note.
--   4. `catalog_scores.resubmission_note` — the proposer's mandatory justification when
--      RE-proposing a previously-`rejected` score. Because `sha256` is UNIQUE, a
--      re-proposal reuses (re-opens) the existing rejected row (status → `pending`,
--      `proposed_by` re-attributed) rather than inserting a second row; this note
--      travels to the moderator on re-review. NULL until a resubmission.
--
-- A user-proposed row is also tagged `source = 'user-proposal'` (a plain value on the
-- existing NOT NULL `source` column — no DDL needed) so the back office distinguishes
-- it from a crawler dataset origin.
--
-- Additive + reversible at the schema level; no backfill (no existing private score is
-- proposed). Idempotent DDL + fully-qualified names, matching the existing migrations.

ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS proposed_by       UUID,   -- proposer's user id; NULL for crawler rows
    ADD COLUMN IF NOT EXISTS review_reason     TEXT,   -- moderator's rejection motive; shown to proposer
    ADD COLUMN IF NOT EXISTS resubmission_note TEXT;   -- proposer's justification on re-proposal

ALTER TABLE music.user_scores
    ADD COLUMN IF NOT EXISTS proposed_catalog_id UUID;  -- linked catalog row; NULL until proposed

-- Attribution lookups in the privileged review queue (who proposed what).
CREATE INDEX IF NOT EXISTS catalog_scores_proposed_by_idx
    ON music.catalog_scores (proposed_by);

-- Re-propose guard + status join on the owner's contributions list.
CREATE INDEX IF NOT EXISTS user_scores_proposed_catalog_id_idx
    ON music.user_scores (proposed_catalog_id);
