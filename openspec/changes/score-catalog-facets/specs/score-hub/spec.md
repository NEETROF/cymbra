## ADDED Requirements

### Requirement: Advanced facet filters in the hub

The Score Hub screen SHALL provide advanced musical-facet filter controls in addition to the
existing text/author/difficulty controls: a rhythmic-granularity selector (fastest allowed note
value), toggles for chords / tuplets / dotted rhythms, an ambitus (hand-span) selector, and a
tempo band selector (BPM range). These controls SHALL be presented behind an expandable
"advanced filters" affordance so the default view stays simple. Changing any advanced filter SHALL
re-query and compose with the current text/author/difficulty filters. Active advanced filters
SHALL be visible and clearable. The advanced filters SHALL be driven through injectable state so
they are testable without a live backend.

#### Scenario: Applying a granularity filter narrows results

- **WHEN** the user selects "no faster than an eighth" in the advanced filters
- **THEN** the results are limited to scores whose fastest note is an eighth or slower

#### Scenario: Advanced filters compose with the basic filters

- **WHEN** the user has a text query and a difficulty set and then applies a chords toggle
- **THEN** the results satisfy the query, the difficulty, and the chords filter together

#### Scenario: Clearing advanced filters restores the broader results

- **WHEN** the user clears an advanced filter
- **THEN** results are no longer constrained by that facet

#### Scenario: Advanced filters are collapsed by default

- **WHEN** the user opens the hub
- **THEN** the advanced filter controls are collapsed behind their affordance, leaving the basic
  controls visible

### Requirement: Hub constrains to piano scores

While the corpus is piano-only, the Score Hub SHALL constrain catalog results to piano scores
by always applying the piano filter, without requiring the user to set it. The underlying search
parameter SHALL remain available so the constraint can later be relaxed or made user-controlled.

#### Scenario: Catalog results are piano-only

- **WHEN** the user browses or searches the catalog in the hub
- **THEN** only piano (grand-staff) scores are returned, because the hub applies the piano filter
