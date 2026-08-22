## MODIFIED Requirements

### Requirement: Restore purchases and refresh are always reachable

Store builds SHALL offer "restore purchases" on the paywall and in the plan status; every build
SHALL refresh the plan on app resume, after a purchase or restore, and on explicit user request.
Purchases and restores on store builds go through the subscription aggregator's SDK behind the
app's store seam; a restore SHALL settle from the SDK's result (subscription found → plan sync
then "restored"; none → "nothing to restore"; owned by another account → the localized
"another account" outcome) without a timed wait. After a purchase or a restore the app SHALL
request a server plan sync and adopt the returned plan view — it MUST NOT grant premium from the
SDK's local customer info. A refresh or sync failure SHALL keep the last-known plan rather than
degrading the UI to free, and SHALL be shown as a localized pending/failed outcome, never a raw
SDK or gRPC string. Builds without a store (Linux, Windows, tests) keep a no-op store client and
the web checkout path.

#### Scenario: Restore on a new device

- **WHEN** a subscriber signs in on a new device and taps restore
- **THEN** the aggregator SDK restores, the app requests a plan sync, and the plan shows premium

#### Scenario: Nothing to restore settles immediately

- **WHEN** a user with no store subscription taps restore
- **THEN** the SDK result carries no subscription and the app shows "nothing to restore" without waiting

#### Scenario: Receipt owned by another account

- **WHEN** a user restores or purchases with a receipt bound to another Cymbra account
- **THEN** the app shows the localized "belongs to another account" outcome and the plan is unchanged

#### Scenario: Offline keeps last-known plan

- **WHEN** the plan refresh or sync fails because the device is offline
- **THEN** the UI keeps the last-known plan and shows no raw error
