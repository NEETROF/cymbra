-- Course units (change: add-notation-courses).
--
-- A unit is a display grouping *within* a track/level section — the listing
-- shows courses bucketed under a unit heading. `unit` is a stable slug (empty =
-- ungrouped, so pre-unit rows and manifests stay valid); `unit_title` is the
-- inline-localized heading object ({en,fr,es,it}), carried opaquely like
-- `title`. Grouping only: listing order stays (track, level, sort_order, id).

ALTER TABLE music.courses
    ADD COLUMN IF NOT EXISTS unit text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS unit_title jsonb NOT NULL DEFAULT '{}'::jsonb;
