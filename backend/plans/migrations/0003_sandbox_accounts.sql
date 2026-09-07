-- Accounts whose SANDBOX store transactions are honoured (change:
-- scope-sandbox-to-marked-accounts, design D2). Replaces the process-wide
-- `CYMBRA_REVENUECAT_ALLOW_SANDBOX`, which accepted sandbox for EVERY account for
-- as long as it was on.
--
-- Presence is the mark; clearing deletes the row. History is not kept here —
-- `plan_admin_audit` already records both directions with the actor, and two
-- records of the same fact would drift.
--
-- The mark grants nothing by itself: it only decides whether a sandbox purchase
-- counts like a production one.
CREATE TABLE sandbox_accounts (
    user_id    UUID        PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by TEXT        NOT NULL
);
