-- Constrain the scope/role vocabulary in the database (change: harden-module-
-- boundaries, task 1.5).
--
-- `user_roles.scope` and `.role` carried the vocabulary as a COMMENT
-- (`-- global | music | live`) while `role_grants.action` next door has a real
-- CHECK. The Rust side validates on the grant path (`validate_scope_role`), but that
-- is one writer among several: `backend/scripts/seed_admin.sh` INSERTs straight into
-- this table, and it is the only path that ever creates a privileged role in
-- production. A typo there produced a row no code would ever match, silently.
--
-- Constrained here is the authorization STATE. `role_grants` is deliberately left
-- alone: it is an append-only audit, and history must stay writable exactly as it
-- happened, even for a vocabulary that later changes.
--
-- Consequence, and it is intended: adding a product scope (`lingua`) now needs a
-- migration. Widening who may hold authority should be a reviewed act, not a string
-- that happens to reach an INSERT.

-- Fail with the offending values rather than a bare constraint violation, so a
-- deploy that trips this says what to fix.
DO $$
DECLARE
    bad TEXT;
BEGIN
    SELECT string_agg(DISTINCT format('(%L, %L)', scope, role), ', ')
      INTO bad
      FROM user_roles
     WHERE scope NOT IN ('global', 'music', 'live')
        OR role  NOT IN ('user', 'admin', 'moderator');
    IF bad IS NOT NULL THEN
        RAISE EXCEPTION
            'user_roles holds rows outside the recognized vocabulary: %. '
            'Fix or delete them, then re-run this migration.', bad;
    END IF;
END $$;

ALTER TABLE user_roles
    ADD CONSTRAINT user_roles_scope_vocabulary
    CHECK (scope IN ('global', 'music', 'live'));

ALTER TABLE user_roles
    ADD CONSTRAINT user_roles_role_vocabulary
    CHECK (role IN ('user', 'admin', 'moderator'));
