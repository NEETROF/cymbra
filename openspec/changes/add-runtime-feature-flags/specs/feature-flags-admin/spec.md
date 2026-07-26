## ADDED Requirements

### Requirement: Admin flags & config panel in the back office

The back office SHALL provide an **admin-only** panel that lists the **declared** flags and config
keys with their current effective values and defaults, and lets an admin **toggle a flag** or
**edit a config value** within its type. The operation SHALL be guarded so only an `admin`
identity may change values; a non-admin MUST be rejected. Edits SHALL take effect at runtime (per
the hot-evaluation rule) and SHALL be audited.

#### Scenario: Admin toggles a feature flag

- **WHEN** an admin toggles a feature flag in the panel
- **THEN** the flag's new state takes effect at runtime and the change is audited

#### Scenario: Admin edits a config value within its type

- **WHEN** an admin edits a typed config value to a valid value
- **THEN** the new value takes effect at runtime and the change is audited

#### Scenario: Invalid or wrong-type edit rejected

- **WHEN** an admin submits a value of the wrong type or outside an allowed range
- **THEN** the edit is rejected and no change is made

#### Scenario: Non-admin cannot change flags

- **WHEN** a non-admin attempts to toggle a flag or edit config
- **THEN** the request is rejected

### Requirement: Sensitive keys are protected from casual change

The panel SHALL protect keys marked **not casually editable** (legal or infrastructure values,
e.g. the minimum public-sharing age or data-retention periods): such keys SHALL be visibly
distinguished, MUST require an explicit, audited confirmation to change, and SHALL NOT be flippable
like an ordinary toggle.

#### Scenario: Sensitive key requires explicit confirmation

- **WHEN** an admin attempts to change a key marked non-casually-editable
- **THEN** an explicit confirmation is required before the change is applied and audited

#### Scenario: Sensitive keys are distinguished in the panel

- **WHEN** the panel lists keys
- **THEN** legal/infrastructure keys are visibly distinguished from ordinary flags/config
