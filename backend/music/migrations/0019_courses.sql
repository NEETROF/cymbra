-- Interactive courses (change: add-notation-courses).
--
-- A course is a self-describing, versioned JSON manifest (a block DSL) stored as
-- an opaque JSONB blob: the backend serves it without understanding its blocks —
-- all block semantics + forward-compatibility live in the client, so new block
-- types ship without a backend release, and (later) community-authored courses
-- are just more rows. Delivered by ListCourses/GetCourse; first-party courses are
-- seeded. Idempotent DDL with fully-qualified names (search_path-independent).
--
-- `title` is the inline-localized title object ({en, fr, es, it}); `content` is
-- the full manifest. `instrument`/`track`/`level` let the home screen group tiles
-- (piano now; a later drums track reuses the format, swapping interactive blocks).

CREATE TABLE IF NOT EXISTS music.courses (
    id             text PRIMARY KEY,
    -- Lifecycle: `published` is served to clients. Room for `draft` now and, with
    -- 2c (community courses), `pending`/`accepted`/`rejected` under moderation.
    status         text NOT NULL DEFAULT 'published',
    instrument     text NOT NULL DEFAULT 'piano',
    track          text NOT NULL DEFAULT 'solfege',
    level          text NOT NULL DEFAULT 'beginner',
    sort_order     integer NOT NULL DEFAULT 0,
    -- Manifest schema version; the client declines a version it cannot handle.
    schema_version integer NOT NULL,
    title          jsonb NOT NULL DEFAULT '{}'::jsonb,
    content        jsonb NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now()
);

-- The listing reads published courses grouped by track/level in display order.
CREATE INDEX IF NOT EXISTS courses_listing_idx
    ON music.courses (status, track, level, sort_order);
