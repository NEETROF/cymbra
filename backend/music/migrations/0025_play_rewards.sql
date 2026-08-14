-- music module — playing and practising earn points (change: add-play-rewards).
--
-- The points economy paid for RATING only (`coverage`/`honesty`/`adjustment`/
-- `redeem`, migration 0016). This adds the two PLAY award reasons to the same
-- append-only ledger, so lifetime points, the spendable balance, levels, the shop
-- and the activity feed all keep working unchanged — they simply have more than
-- one source of income.
--
-- Everything here is ADDITIVE. Existing rows carry NULL in both new columns and
-- are untouched by the new indexes; every existing read (lifetime, balance,
-- coverage-today, curator metrics, the activity feed) either filters on
-- `award_kind` or sums the lot, so none of them changes behaviour. Reverting the
-- backend leaves the rows written by the new kinds in the ledger, still counting
-- toward lifetime — which is correct, since points already earned are never taken
-- back (design "Migration Plan").
--
-- Three additions:
--
--   1. Two new `award_kind` values. `performance` = one scored run that passed the
--      quality floor; `practice` = the once-a-day showing-up award for a scoreless
--      measure-range run.
--
--   2. `award_key` — the durable IDEMPOTENCY key of the event that caused the
--      award, with a partial unique index per user (the shape
--      `curation_points_coverage_once_idx` already uses). Session ingest is
--      at-least-once by design (the app retries from a durable outbox until
--      acked), so the award MUST be keyed on the event rather than on "did the
--      session row insert" — a crash between the two writes would otherwise lose
--      the award forever (design D4). A performance award keys on the client
--      session id; a practice award keys on the player's LOCAL day.
--
--   3. `piece_id` — the piece a performance award was paid for, so the per-piece
--      diminishing curve can read "how many times has this piece already paid this
--      user". Deliberately NOT `catalog_score_id`: that column FKs to
--      music.catalog_scores, and a played piece is just as often a bundled slug or
--      a `music.user_scores` upload — both of which must be counted, or replaying
--      your own upload would be an infinite well. Opaque TEXT, exactly like the
--      `score_id` the play ingest already stores.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- 1. The two new award kinds. The CHECK is replaced rather than added to (a
-- constraint cannot be extended in place); dropping IF EXISTS first keeps the
-- migration re-runnable.
ALTER TABLE music.curation_points
    DROP CONSTRAINT IF EXISTS curation_points_award_kind_check;
ALTER TABLE music.curation_points
    ADD CONSTRAINT curation_points_award_kind_check
    CHECK (award_kind IN ('coverage', 'honesty', 'adjustment', 'redeem',
                          'performance', 'practice'));

-- 2. The idempotency key + the piece an award was paid for. Both NULL on every
-- existing row and on every rating-side award.
ALTER TABLE music.curation_points
    ADD COLUMN IF NOT EXISTS award_key TEXT,   -- e.g. 'performance:<session uuid>' / 'practice:2026-08-14'
    ADD COLUMN IF NOT EXISTS piece_id  TEXT;   -- played piece (catalog uuid / upload uuid / bundled slug)

-- Exactly-once under at-least-once ingest: a retried session re-attempts its
-- award and this index turns the second insert into a no-op. Partial, so the
-- rating-side awards (which carry no key) are unaffected.
CREATE UNIQUE INDEX IF NOT EXISTS curation_points_award_key_once_idx
    ON music.curation_points (user_id, award_key)
    WHERE award_key IS NOT NULL;

-- Backs the per-piece diminishing curve's count ("how many times has this piece
-- already paid this user") on the play ingest's hot write path. Partial to the
-- performance rows, which are the only ones the curve counts.
CREATE INDEX IF NOT EXISTS curation_points_piece_paid_idx
    ON music.curation_points (user_id, piece_id)
    WHERE award_kind = 'performance';
