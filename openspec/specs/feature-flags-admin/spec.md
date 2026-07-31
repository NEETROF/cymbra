# feature-flags-admin Specification

## Purpose
TBD - created by archiving change add-runtime-feature-flags. Update Purpose after archive.
## Requirements
### Requirement: Admin flags & config panel in the back office

The back office SHALL provide an **admin-only**, **app-aware** panel that lists the **declared**
flags and config keys (filterable by app) with their current effective values and defaults, and
lets an admin **toggle a flag** or **edit a config value** within its type. A **platform admin**
(global admin) MAY change keys across all apps and the `all`-scoped keys; a **per-app admin** MAY
change only that app's keys and MUST NOT change `all`-scoped or another app's keys. A caller
without the required admin role MUST be rejected. Edits SHALL take effect at runtime (per the
hot-evaluation rule) and SHALL be audited.

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

#### Scenario: Per-app admin cannot change another app's or shared keys

- **WHEN** a per-app admin (e.g. `music/admin`) attempts to change a `live` key or an `all`-scoped key
- **THEN** the request is rejected; only that app's keys are editable by them

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

