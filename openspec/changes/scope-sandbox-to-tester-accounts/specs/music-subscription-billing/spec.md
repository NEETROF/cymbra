## ADDED Requirements

### Requirement: Sandbox transactions are honoured only for accounts marked as store testers

The billing ingest SHALL apply a store transaction reported in the provider's
**sandbox** environment only when the purchasing account is marked a store tester,
and SHALL drop it for every other account, counting the skip. Production MUST NOT
carry an environment-wide setting that accepts sandbox transactions for all
accounts. The decision SHALL be resolved before the event and customer mappers are
called, so those mappers keep receiving a decided answer rather than a policy to
evaluate.

Transactions in the provider's **production** environment SHALL be unaffected by the
mark: they are applied for every account, tester or not.

#### Scenario: A tester's sandbox purchase grants premium

- **WHEN** a sandbox `INITIAL_PURCHASE` arrives for an account marked a store tester, for a premium product
- **THEN** the entitlement row is written exactly as a production purchase would write it

#### Scenario: A non-tester's sandbox purchase grants nothing

- **WHEN** a sandbox `INITIAL_PURCHASE` arrives for an account that is not marked a store tester
- **THEN** no entitlement row is written and the event is counted as a sandbox skip

#### Scenario: Clearing the mark stops honouring new sandbox transactions

- **WHEN** an account's store-tester mark is cleared and a further sandbox event arrives for it
- **THEN** the event is skipped, and rows already written while the mark was set are left untouched

#### Scenario: Production transactions ignore the mark

- **WHEN** a production `INITIAL_PURCHASE` arrives for an account that is not marked a store tester
- **THEN** the entitlement row is written

#### Scenario: Reconciliation applies the same rule

- **WHEN** the aggregator's customer state is re-read for an account — on `SyncStorePlan` or by the reconciliation sweep — and it holds a sandbox subscription
- **THEN** that subscription is mapped only when the account is marked a store tester

## REMOVED Requirements

### Requirement: Sandbox acceptance is an environment-wide setting

**Reason**: Never specified — the rule lived only in code, as the
`CYMBRA_REVENUECAT_ALLOW_SANDBOX` process flag documented "staging only". No staging
environment exists, so the flag's only real use was being switched on and off by hand
in production, which opens sandbox acceptance for every account while it is on.

**Migration**: Remove `CYMBRA_REVENUECAT_ALLOW_SANDBOX` from every environment file.
Mark the accounts that need sandbox purchases as store testers in the back office
instead.
