-- Shared runtime feature-flag & config store (change: add-runtime-feature-flags).
--
-- Runs as `flags_svc`, the owner of the `feature_flags` schema (search_path =
-- feature_flags), so these unqualified tables land there. This crate owns its
-- own schema/migrations on the model of `jobs`/`music`.
--
-- The DB is the source of truth for OVERRIDES only: every key is declared in code
-- with a typed default, and an absent row resolves to that code default. So this
-- table never lists every key — only the ones an admin has overridden. `value` is
-- the typed override as JSON (`value_type` disambiguates int vs number), scoped by
-- `(app, key)`.

CREATE TABLE flag_overrides (
    app           TEXT        NOT NULL,                 -- all | music | live | …
    key           TEXT        NOT NULL,                 -- declared registry key
    value_type    TEXT        NOT NULL
        CHECK (value_type IN ('bool', 'int', 'number', 'string', 'json')),
    value         JSONB       NOT NULL,                 -- typed override, JSON-encoded
    rollout_scope TEXT        NOT NULL DEFAULT 'global'
        CHECK (rollout_scope IN ('global', 'staff_only')),
    -- Denormalized snapshot of the registry's sensitivity for display; the code
    -- registry stays authoritative for gating.
    sensitive     BOOLEAN     NOT NULL DEFAULT FALSE,
    updated_by    UUID        NOT NULL,                 -- admin who last set it
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (app, key)
);

-- Append-only change audit, mirroring `role_grants` / `catalog_edits`: who
-- changed which key from what to what, and when. No foreign keys (the audit must
-- survive account deletion) and never consulted for evaluation. `old_value` is
-- NULL when the key had no prior override (a first set); a clear records
-- `new_value = '(default)'`.
CREATE TABLE feature_flag_changes (
    id         BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    app        TEXT        NOT NULL,
    key        TEXT        NOT NULL,
    old_value  TEXT,                                    -- display string, or NULL
    new_value  TEXT        NOT NULL,                    -- display string
    actor      UUID        NOT NULL,                    -- acting admin's account id
    at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Recent changes are listed newest-first, optionally filtered by app.
CREATE INDEX feature_flag_changes_at_idx ON feature_flag_changes (at DESC);
CREATE INDEX feature_flag_changes_app_idx ON feature_flag_changes (app, at DESC);
