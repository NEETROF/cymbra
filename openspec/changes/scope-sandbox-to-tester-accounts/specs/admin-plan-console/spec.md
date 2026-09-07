## ADDED Requirements

### Requirement: An admin can mark an account a store tester

The plan console SHALL let an authorized admin set and clear a **store tester** mark
on an account, and SHALL show the account's current mark. Setting or clearing it
SHALL be recorded in the same audit trail as the console's other plan changes,
naming the actor and the new value. A caller without admin authority over the
account's scope SHALL be refused, and the account left unchanged.

The mark SHALL NOT grant any entitlement by itself: it only decides whether the
account's **sandbox** store transactions are honoured.

#### Scenario: An admin marks an account and the change is audited

- **WHEN** an authorized admin sets the store-tester mark on an account
- **THEN** the account reads as a store tester, and the audit trail carries the change with the acting admin

#### Scenario: Clearing the mark is audited too

- **WHEN** an authorized admin clears the mark
- **THEN** the account no longer reads as a store tester, and the audit trail carries that change

#### Scenario: A non-admin is refused

- **WHEN** a caller without admin authority over the account attempts to set the mark
- **THEN** the request is refused and the account's mark is unchanged

#### Scenario: The mark alone grants nothing

- **WHEN** an account is marked a store tester and has no entitlement row
- **THEN** its effective plan stays free
