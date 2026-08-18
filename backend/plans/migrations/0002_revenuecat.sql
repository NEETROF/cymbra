-- Store channels through the aggregator (change: swap-store-billing-to-revenuecat,
-- design D1): `billing_events` gains the `revenuecat` provider — the idempotency
-- scope of its webhook / customer API. Ledger rows keep the STORE as `source`
-- (`plan_entitlements.source` is unchanged); the aggregator only shows up here.
-- The old `apple` / `google` providers stay accepted: nothing to migrate (the
-- channels were dark), nothing to reject.
ALTER TABLE billing_events DROP CONSTRAINT IF EXISTS billing_events_provider_check;
ALTER TABLE billing_events
    ADD CONSTRAINT billing_events_provider_check
    CHECK (provider IN ('apple', 'google', 'web', 'revenuecat'));
