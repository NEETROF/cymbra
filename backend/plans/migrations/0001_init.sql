-- Plans module (change: add-premium-subscription).
--
-- Runs as `plans_svc`, the owner of the `plans` schema (search_path = plans), so
-- these unqualified tables land there. Own crate, own schema, own migrations on
-- the model of `feature_flags` / `analytics`.
--
-- Two independent axes on an account (design D2):
--   * the PLAN — `plan_entitlements`: one row per (source, provider_ref);
--     premium while any row is active; identifiers only, never billing PII;
--   * BETA MEMBERSHIPS — `beta_campaigns` × `beta_memberships`: kind
--     `premium_trial` (per-tester end) or `feature` (closed by the operator).
-- `product` is carried on every table so a Live plan is a new value, not a
-- rename. No foreign key to `user_account` (different schema/role); the account
-- erasure job purges by user_id.

CREATE TABLE plan_entitlements (
    id            UUID        PRIMARY KEY,
    product       TEXT        NOT NULL DEFAULT 'music',
    user_id       UUID        NOT NULL,
    source        TEXT        NOT NULL
        CHECK (source IN ('apple', 'google', 'web', 'code', 'admin')),
    -- Apple originalTransactionId, Google subscription id, MoR subscription id,
    -- access-code id, admin grant id. Opaque; the only provider data kept.
    provider_ref  TEXT        NOT NULL,
    -- Set for premium-trial rows: the campaign that produced them.
    campaign_id   UUID,
    starts_at     TIMESTAMPTZ NOT NULL,
    -- NULL only for an open-ended admin grant (flagged in the console).
    ends_at       TIMESTAMPTZ,
    status        TEXT        NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'cancelled', 'billing_retry', 'ended', 'refunded', 'revoked')),
    revoked_at    TIMESTAMPTZ,
    -- Stamped once when this row's lapse has been withdrawn (design D13).
    withdrawn_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source, provider_ref)
);
CREATE INDEX plan_entitlements_user_idx ON plan_entitlements (user_id);
CREATE INDEX plan_entitlements_ends_idx ON plan_entitlements (ends_at) WHERE ends_at IS NOT NULL;
CREATE INDEX plan_entitlements_pending_withdrawal_idx
    ON plan_entitlements (ends_at) WHERE withdrawn_at IS NULL;

CREATE TABLE beta_campaigns (
    id                    UUID        PRIMARY KEY,
    product               TEXT        NOT NULL DEFAULT 'music',
    -- Stable key: flag rollouts `beta:<key>`, Discord channel mapping.
    key                   TEXT        NOT NULL UNIQUE
        CHECK (key ~ '^[a-z0-9-]+$'),
    name                  TEXT        NOT NULL,
    kind                  TEXT        NOT NULL
        CHECK (kind IN ('premium_trial', 'feature')),
    -- Trials only: premium for this many days from EACH tester's enrolment.
    duration_days         INTEGER
        CHECK (duration_days IS NULL OR duration_days > 0),
    -- Trials: stop new enrolments (nobody is shortened).
    enrollment_closes_at  TIMESTAMPTZ,
    -- Feature betas: the operator's lever — every membership ends here.
    closed_at             TIMESTAMPTZ,
    created_by            UUID        NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((kind = 'premium_trial') = (duration_days IS NOT NULL))
);

CREATE TABLE beta_memberships (
    campaign_id  UUID        NOT NULL REFERENCES beta_campaigns (id),
    user_id      UUID        NOT NULL,
    enrolled_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Trials: enrolled_at + duration; feature betas: NULL (ends when closed).
    ends_at      TIMESTAMPTZ,
    revoked_at   TIMESTAMPTZ,
    source       TEXT        NOT NULL CHECK (source IN ('code', 'admin')),
    PRIMARY KEY (campaign_id, user_id)
);
CREATE INDEX beta_memberships_user_idx ON beta_memberships (user_id);

CREATE TABLE access_codes (
    id              UUID        PRIMARY KEY,
    campaign_id     UUID        NOT NULL REFERENCES beta_campaigns (id),
    -- SHA-256 hex of the normalized clear text; the clear text is never stored.
    code_hash       TEXT        NOT NULL UNIQUE,
    issued_by       TEXT        NOT NULL,          -- admin id or 'discord:<user id>'
    issued_to_hint  TEXT,
    max_uses        INTEGER     NOT NULL DEFAULT 1 CHECK (max_uses > 0),
    uses            INTEGER     NOT NULL DEFAULT 0 CHECK (uses >= 0),
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX access_codes_campaign_idx ON access_codes (campaign_id);

CREATE TABLE access_code_redemptions (
    code_id      UUID        NOT NULL REFERENCES access_codes (id),
    user_id      UUID        NOT NULL,
    redeemed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (code_id, user_id)
);
CREATE INDEX access_code_redemptions_user_idx ON access_code_redemptions (user_id);

-- Idempotency ledger for provider notifications (design D3): a replayed event
-- id is a no-op. `payload_ref` is a pointer/short digest, never the raw payload
-- with personal data.
CREATE TABLE billing_events (
    provider     TEXT        NOT NULL CHECK (provider IN ('apple', 'google', 'web')),
    event_id     TEXT        NOT NULL,
    user_id      UUID,
    payload_ref  TEXT        NOT NULL,
    received_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    applied_at   TIMESTAMPTZ,
    PRIMARY KEY (provider, event_id)
);
CREATE INDEX billing_events_user_idx ON billing_events (user_id) WHERE user_id IS NOT NULL;

-- Append-only audit of console mutations, mirroring `role_grants` /
-- `feature_flag_changes`: no FK (survives account erasure), never consulted for
-- decisions.
CREATE TABLE plan_admin_audit (
    id           BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor        TEXT        NOT NULL,
    action       TEXT        NOT NULL,
    target_user  UUID,
    target_ref   TEXT,
    reason       TEXT        NOT NULL DEFAULT '',
    at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX plan_admin_audit_at_idx ON plan_admin_audit (at DESC);
