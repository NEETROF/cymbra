## MODIFIED Requirements

### Requirement: Two plans with a fixed premium unlock set

The system SHALL define the plans `free` and `premium`. `premium` SHALL map to a set of named
**unlocks** fixed in code (`catalog.unlimited`, `soundfonts.library`,
`soundfont_library.extended`, `scores.extended_quotas`, `offline.cache`); `free` SHALL grant
no unlock. Consumers
SHALL ask "does the effective plan grant unlock X" and MUST NOT compare plan names. Security
guardrails (catalog download/enumeration limits, auth throttles) MUST NOT depend on the plan or
on any beta membership.

Every unlock SHALL belong to exactly one **product**, and a `premium` entitlement SHALL
grant only the unlocks of the product it was purchased for. The product SHALL be carried
on the entitlement and on the beta campaign, and SHALL be the value consulted when
resolving a plan — never inferred from the caller, the audience, or a default. An
entitlement written before products were distinguished SHALL resolve as `music`.

A consumer asking "does the effective plan grant unlock X" SHALL therefore be answered
for the product that owns X, so a subscriber to one product never receives another
product's unlocks.

#### Scenario: Premium unlock set is not editable at runtime

- **WHEN** an operator edits any runtime configuration
- **THEN** the set of unlocks granted by `premium` is unchanged

#### Scenario: Guardrails ignore plan and betas

- **WHEN** a premium user or a beta member exceeds the catalog download burst or enumeration cap
- **THEN** the request is refused exactly as for a free user

#### Scenario: A subscriber to one product receives only its unlocks

- **WHEN** an account holds an active `premium` entitlement for a product other than
  `music`, and no music entitlement
- **THEN** every music unlock is denied, and the account is treated as `free` by music

#### Scenario: An account may hold entitlements for several products

- **WHEN** an account holds active `premium` entitlements for two products
- **THEN** each product's unlocks are granted, independently of the other

#### Scenario: Pre-existing entitlements stay music

- **WHEN** an entitlement recorded before products were distinguished is resolved
- **THEN** it grants the music unlocks, exactly as before this change
