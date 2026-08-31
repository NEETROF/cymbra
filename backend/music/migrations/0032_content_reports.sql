-- music module — in-app content reports (change: add-content-reporting).
--
-- Google Play's user-generated-content policy requires an **in-app** way to
-- report objectionable content and users, for any app that hosts UGC. Cymbra
-- hosts it (proposed catalog scores, contributed sound fonts, public handles on
-- the leaderboards) and had no intake at all: reporting went through an e-mail
-- address in the terms. This table is that intake.
--
-- Deliberately ONE table for every reportable kind rather than a table per kind:
-- the moderator's queue is one queue, and `target_kind` + `target_id` keeps a
-- new reportable surface a code change rather than a migration. No FK to the
-- target for the same reason — and because a report must OUTLIVE the takedown it
-- causes (the same reasoning as `user_score_takedowns` in 0031).
--
-- `reporter_id` is nullable so a report survives its author's account erasure:
-- `purge_user` nulls it instead of deleting the row, or a moderator would lose
-- the backlog every time a reporter closes their account.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path. New tables inherit music_svc's privileges
-- from ALTER DEFAULT PRIVILEGES (backend/deploy/provision-music-role.sql).

CREATE TABLE IF NOT EXISTS music.content_reports (
    id           UUID PRIMARY KEY,                  -- UUID v7, app-side
    target_kind  TEXT        NOT NULL
                 CHECK (target_kind IN ('catalog_score', 'soundfont', 'profile')),
    target_id    TEXT        NOT NULL CHECK (length(btrim(target_id)) > 0),
    reporter_id  UUID,                              -- NULL once the reporter erases their account
    reason       TEXT        NOT NULL
                 CHECK (reason IN ('copyright', 'inappropriate', 'wrong_content', 'other')),
    note         TEXT        CHECK (note IS NULL OR length(note) <= 2000),
    status       TEXT        NOT NULL DEFAULT 'open'
                 CHECK (status IN ('open', 'reviewed', 'dismissed')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at  TIMESTAMPTZ,
    reviewed_by  UUID
);

-- The moderator's queue: open reports, oldest first.
CREATE INDEX IF NOT EXISTS content_reports_open_idx
    ON music.content_reports (created_at)
    WHERE status = 'open';

-- "Show me everything reported about this item", and the per-target count the
-- moderation screens surface.
CREATE INDEX IF NOT EXISTS content_reports_target_idx
    ON music.content_reports (target_kind, target_id);

-- One open report per reporter per target: a user hammering the button must not
-- flood the queue, and re-reporting the same item is not new information. A
-- partial unique index rather than a constraint, so a *closed* report never
-- blocks a genuine new one after the item changes.
CREATE UNIQUE INDEX IF NOT EXISTS content_reports_one_open_per_reporter_idx
    ON music.content_reports (target_kind, target_id, reporter_id)
    WHERE status = 'open' AND reporter_id IS NOT NULL;
