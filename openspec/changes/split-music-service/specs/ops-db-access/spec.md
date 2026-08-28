## ADDED Requirements

### Requirement: An extracted module keeps its own least-privilege role

A module served by its own process SHALL reach the shared database through **its own
least-privilege role**, exactly as it did in-process. Extraction SHALL NOT be an occasion
to widen its privileges, and a separate process MUST NOT be wired to the ops role to make
its queries succeed.

The module's schema ownership and migrations SHALL be unchanged by extraction: the module
continues to own and migrate its schema, and no other component migrates it.

#### Scenario: The extracted process is confined

- **WHEN** the extracted module's process queries another module's schema
- **THEN** the database rejects it, exactly as before extraction

#### Scenario: Extraction does not promote the module to the ops role

- **WHEN** the extracted process starts
- **THEN** it is configured with its own module role, not the ops connection

#### Scenario: Migrations stay with the module

- **WHEN** the extracted module's schema is migrated
- **THEN** the migration is run by the module, not by another component

### Requirement: A cross-schema read grant is named and bounded

An extracted module reading another module's data directly SHALL do so through a narrow,
explicitly granted read — named tables or columns, read-only — recorded where the roles are
defined, together with the reason a served call was not used. This applies only on a path
that must not become a network call.

Such a grant SHALL be the exception, SHALL never be a blanket privilege, and SHALL NOT be
used on a path that could reasonably call the owning module instead.

#### Scenario: A granted read is documented

- **WHEN** an extracted module reads another module's table directly
- **THEN** the grant names the tables it covers, is read-only, and records why a served
  call was not used

#### Scenario: No blanket cross-schema privilege

- **WHEN** an extracted module's role is provisioned
- **THEN** it holds no database-wide read or write privilege — only its own schema plus any
  named exception

#### Scenario: An ungranted cross-schema read fails

- **WHEN** the extracted module reads a table outside its schema and outside its named
  grants
- **THEN** the database rejects the query
