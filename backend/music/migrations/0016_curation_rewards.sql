-- music module — curation rewards points economy (change: add-curation-rewards).
--
-- Adds the points backbone that rewards broad + honest rating (#2/#3) without
-- rewarding raw volume. Everything here is ADDITIVE — disabling the award hooks
-- and hiding the surfaces leaves ratings and moderation fully functional, and the
-- data below is inert if unused (design "Migration Plan" / Rollback).
--
-- Five additions:
--
--   1. `curation_points` — an APPEND-ONLY ledger of every points event (a
--      `coverage` award, a `honesty` award, a re-settlement `adjustment`, or a
--      shop `redeem`). Lifetime points, spendable balance, and level are all
--      DERIVED from it (design D3/D5); nothing here is ever UPDATEd/DELETEd.
--      `amount` is signed — awards are positive, `redeem` is negative — so the
--      spendable balance is simply SUM(amount) and lifetime is SUM(amount) over
--      the non-`redeem` rows. `catalog_score_id` ties an award to its score (NULL
--      for a redemption); `reward_key` names the redeemed reward (NULL for awards).
--
--   2. Per-rating honesty settlement state on `score_ratings` (`settled_at`,
--      `settled_source`) — the idempotency guard for the honesty bonus (design
--      D2): a rating's honesty is awarded AT MOST ONCE, and a `consensus`
--      settlement may be superseded ONCE by a later `moderator` decision, which
--      appends a correcting `adjustment` to the ledger.
--
--   3. `score_consensus_settlements` — the per-score FROZEN community truth
--      (design D2): once a score crosses the consensus minimum its aggregate is
--      frozen here (idempotent by PK), so the worker sweep settles each rating
--      against a stable truth even as later ratings arrive.
--
--   4. `curation_grants` — durable per-(user, key) grants: redeemed rewards
--      (pianos/SoundFonts, feeding piano-sound-selection) and earned badges. The
--      PRIMARY KEY (user_id, key) is the redemption/badge idempotency guard — a
--      reward is charged and granted exactly once.
--
--   5. `score_engagements` — the engagement signal (design D4 / task 2.2): a
--      rater must have PREVIEWED/opened a score before rating it to earn coverage
--      points. A rating without a matching row records normally but earns no
--      coverage points ("no engagement, no coverage").
--
-- user_id is a PLAIN uuid (no cross-schema FK to the user module — module-role
-- isolation, exactly as score_ratings/leaderboard_bests/play_sessions). Account
-- deletion erases these rows explicitly in the `purge_user` worker job (no
-- cross-schema cascade is possible). catalog_score_id DOES FK to catalog_scores
-- (same `music` schema), so a purged score drops its ledger/consensus/engagement
-- rows via ON DELETE CASCADE.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- 1. Append-only points ledger (task 1.1).
CREATE TABLE IF NOT EXISTS music.curation_points (
    id               BIGSERIAL   PRIMARY KEY,
    user_id          UUID        NOT NULL,   -- rater's AuthIdentity.user_id; no cross-schema FK
    award_kind       TEXT        NOT NULL
        CHECK (award_kind IN ('coverage', 'honesty', 'adjustment', 'redeem')),
    amount           INTEGER     NOT NULL,   -- signed: awards > 0, redeem < 0
    catalog_score_id UUID
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,  -- award's score; NULL for redeem
    reward_key       TEXT,                    -- redeemed reward key; NULL for awards
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Balance/lifetime/level + the recent-activity read all group by the user; back
-- that hot path (newest-first for the activity feed).
CREATE INDEX IF NOT EXISTS curation_points_user_idx
    ON music.curation_points (user_id, created_at DESC);

-- Coverage is awarded AT MOST ONCE per (user, score): re-rating (an upsert on
-- score_ratings) must not re-award. This partial unique index is the guard — the
-- coverage insert is `ON CONFLICT (user_id, catalog_score_id) WHERE award_kind =
-- 'coverage' DO NOTHING` against it.
CREATE UNIQUE INDEX IF NOT EXISTS curation_points_coverage_once_idx
    ON music.curation_points (user_id, catalog_score_id)
    WHERE award_kind = 'coverage';

-- 2. Per-rating honesty settlement state (task 1.2, idempotency guard design D2).
ALTER TABLE music.score_ratings
    ADD COLUMN IF NOT EXISTS settled_at     TIMESTAMPTZ,   -- when honesty was settled; NULL = unsettled
    ADD COLUMN IF NOT EXISTS settled_source TEXT
        CHECK (settled_source IS NULL OR settled_source IN ('consensus', 'moderator'));

-- 3. Per-score frozen community truth (task 1.2, design D2). One row per score,
-- written once the score crosses the consensus minimum; frozen thereafter
-- (idempotent by PK). `truth_positive` is TRUE (liked), FALSE (disliked), or NULL
-- (ambiguous — within the neutral band → every rater gets only the floor).
CREATE TABLE IF NOT EXISTS music.score_consensus_settlements (
    catalog_score_id UUID             PRIMARY KEY
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    truth_positive   BOOLEAN,                     -- TRUE like / FALSE dislike / NULL ambiguous
    avg_effective    DOUBLE PRECISION NOT NULL,   -- the frozen aggregate value on the 1–5 scale
    rater_count      BIGINT           NOT NULL,   -- distinct raters at freeze time
    settled_at       TIMESTAMPTZ      NOT NULL DEFAULT now()
);

-- 4. Durable grants: redeemed rewards + earned badges (task 1.3, design D5).
CREATE TABLE IF NOT EXISTS music.curation_grants (
    user_id    UUID        NOT NULL,   -- grantee; no cross-schema FK
    grant_kind TEXT        NOT NULL CHECK (grant_kind IN ('reward', 'badge')),
    key        TEXT        NOT NULL,   -- reward key (piano/soundfont id) or badge key
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, key)          -- one grant per (user, key): redemption/badge idempotency guard
);

-- 5. Engagement signal (task 2.2, design D4). Recorded when the rating-deck
-- preview bytes are served (GetRatingPreviewBytes). One row per (user, score);
-- re-preview just bumps the timestamp. Coverage checks for a row here before
-- awarding — a rating without prior engagement earns no coverage points.
CREATE TABLE IF NOT EXISTS music.score_engagements (
    user_id          UUID        NOT NULL,
    catalog_score_id UUID        NOT NULL
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    engaged_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, catalog_score_id)
);

-- 6. Reward-shop pricing on the existing SoundFont catalog (task 1.4 / 4.2). The
-- shop items ARE the SoundFont catalog — rather than a parallel item registry, a
-- font carries its own point cost and redeemability, keyed by its stable TEXT id
-- (which doubles as the `curation_grants.key` / ledger `reward_key`):
--   * `point_cost` = 0  → free/default, available to everyone as today (no grant
--     needed). > 0 → a shop item: usable only once redeemed (a `curation_grants`
--     row) or when its cost has been spent.
--   * `redeemable` = FALSE → listed as "coming later" and NOT redeemable now (the
--     future temporary-premium tier); it is shown in the shop but RedeemReward
--     refuses it.
-- Defaults keep every existing/seeded font free + available, so this is inert
-- until a font is priced.
ALTER TABLE music.soundfonts
    ADD COLUMN IF NOT EXISTS point_cost INTEGER NOT NULL DEFAULT 0
        CHECK (point_cost >= 0),
    ADD COLUMN IF NOT EXISTS redeemable BOOLEAN NOT NULL DEFAULT TRUE;
