# ops-db-access Specification

## Purpose
TBD - created by archiving change add-ops-db-access. Update Purpose after archive.
## Requirements
### Requirement: Cross-schema operations role

The system SHALL provide a single ops-only login role (`admin_svc`) that can
**read and write data in every module schema** (current and future) through one
connection. The role SHALL be granted read/write via Postgres' predefined
`pg_read_all_data` and `pg_write_all_data` roles, so coverage extends to new
schemas with no per-schema grants. The role MUST NOT own objects or run DDL.

#### Scenario: Reads and writes across all schemas

- **WHEN** an operator connects as `admin_svc`
- **THEN** they can SELECT and INSERT/UPDATE/DELETE in `auth`, `user_account`, and `jobs`

#### Scenario: A newly added schema is covered automatically

- **WHEN** a future module schema and its tables are created by that module's role
- **THEN** `admin_svc` can read and write the new tables without any additional grant

#### Scenario: No DDL or ownership

- **WHEN** `admin_svc` attempts to create, alter, or drop an object in a module schema
- **THEN** the operation is rejected (it holds data privileges only, not ownership/DDL)

### Requirement: Module isolation is preserved for application roles

Introducing the ops role SHALL NOT change the per-module isolation invariant
(D0): each application module keeps its confined per-schema role and still cannot
read another module's schema. The ops role is a separate trust tier and MUST NOT
be used by any application service.

#### Scenario: Module role still confined

- **WHEN** a module role (e.g. `auth_svc`) queries another module's schema
- **THEN** the database still rejects it (unchanged by the ops role)

#### Scenario: Services do not use the ops role

- **WHEN** the application services start
- **THEN** none of them is configured with the `admin_svc` connection (it is ops/psql only)

### Requirement: Environment-driven role bootstrap

The role/schema bootstrap SHALL run across environments from a single source by
taking role **names** and **passwords** from environment variables rather than
hardcoded literals. Each variable SHALL default to its current dev value so local
development and CI work with no extra configuration. The SQL SHALL remain a
secret-free template (credentials injected at run time).

#### Scenario: Dev defaults work out of the box

- **WHEN** the bootstrap runs with no credential environment variables set
- **THEN** the dev roles are created with their default dev passwords and the stack starts

#### Scenario: Credentials sourced from the environment

- **WHEN** credential environment variables are provided (e.g. in staging/prod)
- **THEN** the roles are created/altered with those values and no password literal appears in the committed SQL

#### Scenario: CI bootstrap keeps working

- **WHEN** CI applies the bootstrap
- **THEN** it succeeds via the env-driven path using the dev defaults

### Requirement: Documented production credential strategy

The change SHALL document how production replaces the dev password literals:
sourcing secrets from a secret manager or using IAM database authentication, and
how credentials are rotated. The committed repository MUST NOT contain production
credentials.

#### Scenario: Production secret sourcing is documented

- **WHEN** a reader consults the ops/bootstrap documentation
- **THEN** it describes secret-manager and IAM-auth options and the rotation procedure, with no production secrets committed

