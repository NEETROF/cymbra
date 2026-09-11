-- Add the `lingua` product scope to the role vocabulary (change: add-lingua-back-
-- office, task 2.1).
--
-- Migration 0009 constrained `user_roles.scope` to (global, music, live) and warned in
-- its own header that "adding a product scope (`lingua`) now needs a migration". This is
-- that migration: the Lingua ops console gates on `admin` in the `lingua` scope
-- (`require_admin_in_scope(id, "lingua")`), so `lingua/admin` must be a grantable row.
--
-- It restates the FULL vocabulary (scope + role) as the single current source of truth:
-- the Rust `SCOPES`/`ROLES` drift test reads the latest vocabulary migration, so widening
-- the scope set stays a reviewed act in exactly one authoritative place. The role set is
-- unchanged; it is restated only so this file — not the superseded 0009 — is that place.

-- Fail with the offending values rather than a bare constraint violation, so a deploy
-- that trips this says what to fix. (After 0009 every row already conforms to the old
-- vocabulary, so this only ever fires on a hand-edited row.)
DO $$
DECLARE
    bad TEXT;
BEGIN
    SELECT string_agg(DISTINCT format('(%L, %L)', scope, role), ', ')
      INTO bad
      FROM user_roles
     WHERE scope NOT IN ('global', 'music', 'live', 'lingua')
        OR role  NOT IN ('user', 'admin', 'moderator');
    IF bad IS NOT NULL THEN
        RAISE EXCEPTION
            'user_roles holds rows outside the recognized vocabulary: %. '
            'Fix or delete them, then re-run this migration.', bad;
    END IF;
END $$;

ALTER TABLE user_roles DROP CONSTRAINT user_roles_scope_vocabulary;
ALTER TABLE user_roles
    ADD CONSTRAINT user_roles_scope_vocabulary
    CHECK (scope IN ('global', 'music', 'live', 'lingua'));

ALTER TABLE user_roles DROP CONSTRAINT user_roles_role_vocabulary;
ALTER TABLE user_roles
    ADD CONSTRAINT user_roles_role_vocabulary
    CHECK (role IN ('user', 'admin', 'moderator'));
