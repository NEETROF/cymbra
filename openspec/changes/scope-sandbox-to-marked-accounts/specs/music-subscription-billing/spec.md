## ADDED Requirements

### Requirement: Sandbox transactions are honoured only for accounts marked as sandbox accounts

The billing ingest SHALL apply a store transaction reported in the provider's
**sandbox** environment only when the purchasing account is marked as accepting sandbox purchases,
and SHALL drop it for every other account, counting the skip. Production MUST NOT
carry an environment-wide setting that accepts sandbox transactions for all
accounts. The decision SHALL be resolved before the event and customer mappers are
called, so those mappers keep receiving a decided answer rather than a policy to
evaluate.

Transactions in the provider's **production** environment SHALL be unaffected by the
mark: they are applied for every account, marked or not.

#### Scenario: A marked account's sandbox purchase grants premium

- **WHEN** a sandbox `INITIAL_PURCHASE` arrives for an account marked as accepting sandbox purchases, for a premium product
- **THEN** the entitlement row is written exactly as a production purchase would write it

#### Scenario: An unmarked account's sandbox purchase grants nothing

- **WHEN** a sandbox `INITIAL_PURCHASE` arrives for an account that is not marked as accepting sandbox purchases
- **THEN** no entitlement row is written and the event is counted as a sandbox skip

#### Scenario: Clearing the mark stops honouring new sandbox transactions

- **WHEN** an account's sandbox-account mark is cleared and a further sandbox event arrives for it
- **THEN** the event is skipped, and rows already written while the mark was set are left untouched

#### Scenario: Production transactions ignore the mark

- **WHEN** a production `INITIAL_PURCHASE` arrives for an account that is not marked as accepting sandbox purchases
- **THEN** the entitlement row is written

#### Scenario: Reconciliation applies the same rule

- **WHEN** the aggregator's customer state is re-read for an account — on `SyncStorePlan` or by the reconciliation sweep — and it holds a sandbox subscription
- **THEN** that subscription is mapped only when the account is marked as accepting sandbox purchases

### Requirement: A sandbox transfer is honoured only when every account involved is marked

The billing ingest SHALL apply a **sandbox** `TRANSFER` only when every account it
names — source and destination alike — is marked as accepting sandbox purchases, and SHALL drop it
otherwise. A sandbox entitlement MUST NOT reach an account that is not itself a
marked by being transferred onto it.

#### Scenario: Transfer between marked accounts is applied

- **WHEN** a sandbox `TRANSFER` names only accounts marked as sandbox accounts
- **THEN** it is applied as a production transfer would be

#### Scenario: Transfer onto an unmarked account is refused

- **WHEN** a sandbox `TRANSFER` moves an entitlement from a marked account to one that is not
- **THEN** the event is skipped and no row moves

#### Scenario: Production transfers are unaffected

- **WHEN** a production `TRANSFER` names accounts that are not marked
- **THEN** it is applied
