## MODIFIED Requirements

### Requirement: Hub constrains to piano scores

The Score Hub SHALL constrain catalog results by **instrument** rather than by the
former staff-count piano proxy. The corpus is no longer keyboard-only, so the hub
SHALL NOT pin a piano constraint; it SHALL offer the instrument as a filter the
user can set, listing the drum option only when the drum feature is visible to
them. When the user sets no instrument, the hub SHALL NOT constrain by instrument —
the backend already withholds what they may not see.

#### Scenario: Catalog results are not pinned to keyboard

- **WHEN** the user browses or searches the catalog in the hub without choosing an
  instrument
- **THEN** results are not constrained by instrument, and the hub does not impose a
  keyboard filter of its own

#### Scenario: The drum option appears only for the drum audience

- **WHEN** a user for whom the drum feature is not visible opens the instrument
  filter
- **THEN** no drum option is offered

#### Scenario: A drummer can filter to drum scores

- **WHEN** a user for whom the drum feature is visible sets the instrument filter to
  drums
- **THEN** only percussion scores are returned
