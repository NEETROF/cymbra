## ADDED Requirements

### Requirement: Facet filters on catalog search

The search operation SHALL accept optional musical-facet filters in addition to the text
query, author, and difficulty filters: a maximum note-value granularity (the fastest note
value allowed), presence of chords, presence of tuplets, presence of dotted rhythms, a maximum
ambitus span, a staff count, and a tempo range (BPM). Every supplied facet filter SHALL
compose conjunctively with all other filters — a result MUST satisfy all of them. When a facet
filter is active, a score whose corresponding facet is unknown (null) SHALL NOT match (an
unknown trait cannot be asserted to satisfy the filter). When no facet filter is supplied, that
facet imposes no constraint.

#### Scenario: Rhythmic granularity filter excludes faster notes

- **WHEN** a caller searches with a maximum granularity of an eighth note
- **THEN** only scores whose fastest note is an eighth or slower are returned, and scores with
  sixteenths or faster are excluded

#### Scenario: Trait filters narrow to scores with the trait

- **WHEN** a caller filters for scores containing chords
- **THEN** only scores whose chord facet is true are returned

#### Scenario: Ambitus filter narrows by hand span

- **WHEN** a caller filters by a maximum ambitus span
- **THEN** only scores whose lowest-to-highest pitch span is within that range are returned

#### Scenario: Tempo range filter narrows by BPM

- **WHEN** a caller filters by a tempo range (e.g. 76–120 BPM)
- **THEN** only scores whose recorded tempo falls within that range are returned, and scores
  with no recorded tempo are excluded

#### Scenario: Facet filters compose with text, author, and level

- **WHEN** a caller supplies a text query, an author, a difficulty, and one or more facet filters
- **THEN** results satisfy the query AND the author AND the difficulty AND every facet filter

#### Scenario: Unknown facet excluded under an active filter

- **WHEN** a facet filter is active and a score's corresponding facet is null
- **THEN** that score is not returned

#### Scenario: Invalid facet value rejected

- **WHEN** a caller supplies a granularity or staff-count value outside the allowed set
- **THEN** the request is rejected with a typed error

### Requirement: Piano-only filter parameter

The search operation SHALL accept an optional `is_piano` filter; when set true, only scores
whose piano (grand-staff) facet is true SHALL be returned. When unset, scores are not
constrained by that facet. This parameter is independent of the caller (the app may always
supply it) and composes conjunctively with the other filters.

#### Scenario: Piano filter narrows to keyboard scores

- **WHEN** a caller searches with the piano filter set true
- **THEN** only scores flagged as piano (grand-staff) are returned

#### Scenario: No piano filter includes all instrumentation

- **WHEN** a caller searches without the piano filter
- **THEN** results are not constrained by the piano facet
