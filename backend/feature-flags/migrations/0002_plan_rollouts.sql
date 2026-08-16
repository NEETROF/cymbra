-- Plan- and beta-scoped rollouts (change: add-premium-subscription).
--
-- `rollout_scope` gains `premium_only` (effective plan premium + staff) and
-- `beta:<campaign key>` (active members of that beta campaign + staff). The
-- two-value CHECK becomes a pattern check; existing rows are unaffected.
ALTER TABLE flag_overrides DROP CONSTRAINT IF EXISTS flag_overrides_rollout_scope_check;
ALTER TABLE flag_overrides
    ADD CONSTRAINT flag_overrides_rollout_scope_check
    CHECK (rollout_scope ~ '^(global|staff_only|premium_only|beta:[a-z0-9-]+)$');
