## ADDED Requirements

### Requirement: An inter-module boundary is a Rust trait

The contract between two backend modules SHALL be a Rust trait, resolved in-process at
the composition root. A module MUST NOT depend on another module's concrete
implementation crate, and MUST NOT read another module's database schema directly from
the request path.

The trait SHALL be free of the shape of any `.proto`: it is designed for its in-process
caller, and MAY expose methods that no RPC exposes. Narrow traits declared by the
**consumer** and implemented by the composition root are the default form; a trait
declared by the provider is used when several consumers share the same read. A dedicated
contract crate (a `-port` crate) SHALL be introduced only when it removes a measured
compile-graph dependency, demonstrated with `cargo tree -i`.

#### Scenario: A module reaches another module through a trait

- **WHEN** a module needs data owned by another module
- **THEN** it depends on a trait, and the concrete implementation is supplied by the
  composition root

#### Scenario: A contract crate requires a measured justification

- **WHEN** a new `-port` crate is proposed
- **THEN** it is accepted only if `cargo tree -i` shows a consumer that compiles without
  the implementation crate; otherwise a narrow trait is used instead

#### Scenario: The trait may exceed the served surface

- **WHEN** an in-process caller needs a batched or caller-shaped read
- **THEN** the trait carries that method, whether or not any RPC exposes it

### Requirement: No internal transport is written before a split

The system SHALL NOT carry a gRPC client, an internal `.proto`, or any other
module-to-module transport written in anticipation of a future split. Such a transport
SHALL be designed and written **at the time** a module is actually extracted, against the
contract that extraction requires.

Documentation and design records MUST NOT prescribe a 1:1 mapping between a module's
internal contract and a served gRPC service.

#### Scenario: No anticipatory client

- **WHEN** a module boundary is introduced or revised
- **THEN** no gRPC client adapter and no internal `.proto` are created for it

#### Scenario: Generated client stubs are not emitted without a caller

- **WHEN** a crate compiles a `.proto` it only serves
- **THEN** client stub generation is disabled

### Requirement: The composition root carries no product code

The composition-root crates (`server`, `worker`) SHALL contain wiring only: connecting
resources, constructing modules, installing interceptors, and mounting services. Product
behaviour — request handlers, delivery routes, backfill utilities — SHALL live in the
module that owns the data it operates on.

#### Scenario: A product route lives in its module

- **WHEN** a product exposes an HTTP or gRPC surface
- **THEN** the handler lives in that product's crate, which exposes a router the
  composition root mounts

#### Scenario: Adding a second product does not grow the composition root

- **WHEN** a second product module is added
- **THEN** the composition root gains wiring for it, and no product handler

### Requirement: What prepares a split is data ownership, not transport

A module SHALL be considered separable only on data-ownership grounds: it owns its
Postgres schema and its migrations, it is reached through a least-privilege role from the
request path, and its personal data is erased by a step that names it. The presence or
absence of a transport contract SHALL NOT be treated as evidence of split-readiness.

#### Scenario: A module owns its schema

- **WHEN** a module stores data
- **THEN** the tables live in a schema the module's own migrations create

#### Scenario: Split-readiness is assessed on data, not on protos

- **WHEN** the separability of a module is assessed
- **THEN** the assessment covers schema ownership, role confinement and erasure coverage,
  and does not count served RPCs as progress
