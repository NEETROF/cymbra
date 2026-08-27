## MODIFIED Requirements

### Requirement: Module isolation is preserved for application roles

Introducing the ops role SHALL NOT change the per-module isolation invariant
(D0): each application module keeps its confined per-schema role and still cannot
read another module's schema. The ops role is a separate trust tier and MUST NOT
be used by any **request-path** service.

The **worker is an ops-tier actor**, not a request-path service: the jobs it runs —
account erasure, cross-schema sweeps, retention pruning — cross module schemas by
design, and it SHALL be allowed to use the ops connection. This is a standing decision,
not a defect: the role's documentation SHALL name the worker among the ops actors, so
the wiring and the documented intent agree.

The consequence SHALL be stated where the role is defined: inside the worker, no database
privilege confines a module to its own schema, and module boundaries there rest on review
alone.

#### Scenario: Module role still confined

- **WHEN** a module role (e.g. `auth_svc`) queries another module's schema
- **THEN** the database still rejects it (unchanged by the ops role)

#### Scenario: Request-path services do not use the ops role

- **WHEN** the request-path service starts
- **THEN** it is not configured with the `admin_svc` connection — each module is reached
  through its own confined role

#### Scenario: The worker may use the ops role

- **WHEN** the worker runs a job that spans several module schemas
- **THEN** it uses the ops connection, and this is the documented, intended wiring

#### Scenario: The documentation names the worker

- **WHEN** the ops role definition is read
- **THEN** it lists the worker among the permitted ops actors, and states that module
  boundaries inside the worker are not database-enforced
