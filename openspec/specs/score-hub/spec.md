# score-hub Specification

## Purpose
TBD - created by archiving change score-hub-search. Update Purpose after archive.
## Requirements
### Requirement: Authenticated-only hub entry point

The app SHALL expose a Score Hub entry point only to a signed-in user. When no
user is authenticated, the hub entry point MUST be unavailable (hidden or
disabled) and the hub screen MUST NOT be reachable. This mirrors the existing
contribution entry point.

#### Scenario: Entry point available when signed in

- **WHEN** a user is authenticated
- **THEN** the hub entry point is available and opens the Score Hub screen

#### Scenario: Entry point hidden when signed out

- **WHEN** no user is authenticated
- **THEN** the hub entry point is not available and the hub screen cannot be opened

### Requirement: Search field over title and author

The Score Hub screen SHALL present a search field that queries the catalog by
title and composer through the backend search operation, showing the returned
results. The search SHALL be driven through injectable state so it can be
exercised in tests without the native library or a live backend. Results SHALL
show each score's title, composer, and difficulty level.

#### Scenario: Typing a query shows matching results

- **WHEN** the user enters a query in the hub search field
- **THEN** the screen shows the catalog scores matching that query with title,
  composer, and level

#### Scenario: Empty query browses the corpus

- **WHEN** the search field is empty
- **THEN** the screen shows browsable catalog results rather than an empty state

#### Scenario: No matches shows an empty state

- **WHEN** a query matches no catalog scores
- **THEN** the screen shows a "no results" state rather than an error

#### Scenario: Search state is injectable in tests

- **WHEN** a test overrides the hub's search provider with in-memory results
- **THEN** the hub renders those results without touching the backend

### Requirement: Difficulty filter control

The Score Hub screen SHALL provide a control to filter results by difficulty
(Beginner / Intermediate / Advanced) with an "all levels" default. Changing the
filter SHALL re-query and compose with the current text query. The active filter
SHALL be visible to the user.

#### Scenario: Selecting a difficulty filters results

- **WHEN** the user selects a difficulty in the hub
- **THEN** the results are limited to that difficulty and combined with any text query

#### Scenario: Clearing the filter restores all levels

- **WHEN** the user clears the difficulty filter
- **THEN** results of every difficulty are shown again

### Requirement: Author filter control

The Score Hub screen SHALL provide a control to filter results by author (composer),
distinct from the text search field. Applying an author filter SHALL re-query and
compose with the current text query and difficulty filter. The active author filter
SHALL be visible to the user and clearable.

#### Scenario: Filtering by author narrows results

- **WHEN** the user applies an author filter in the hub
- **THEN** the results are limited to catalog scores by that author, combined with any
  text query and difficulty filter

#### Scenario: Clearing the author filter restores unfiltered composer results

- **WHEN** the user clears the author filter
- **THEN** results are no longer constrained by composer

### Requirement: "My scores" quick-filter scopes the hub to the user's uploads

The Score Hub SHALL offer a "mes partitions" quick-filter shortcut that scopes the
hub to the signed-in user's own uploaded scores instead of the public catalog. When
active, the hub SHALL show the user's uploaded scores (title, composer, difficulty),
and the author and difficulty filters and text query SHALL still apply to that
scoped list. Because these scores are already the user's own, they SHALL NOT offer an
add-to-library action; selecting one SHALL open it in the player. Deactivating the
shortcut SHALL return the hub to searching the public catalog. The shortcut SHALL be
available only to a signed-in user.

#### Scenario: Activating the shortcut shows the user's uploads

- **WHEN** a signed-in user activates the "mes partitions" quick-filter
- **THEN** the hub shows that user's own uploaded scores rather than public catalog results

#### Scenario: Filters compose within the user's uploads

- **WHEN** the "mes partitions" quick-filter is active and the user applies an author or
  difficulty filter or a text query
- **THEN** the shown uploads are narrowed by those filters

#### Scenario: Uploads do not offer add-to-library

- **WHEN** the hub shows the user's own uploaded scores under the quick-filter
- **THEN** no add-to-library action is offered for them (they are already owned)

#### Scenario: Deactivating returns to the catalog

- **WHEN** the user deactivates the "mes partitions" quick-filter
- **THEN** the hub returns to searching the public catalog

### Requirement: Paged results with load-more

The Score Hub SHALL request results in bounded pages and SHALL let the user load
further results (e.g. by scrolling to the end or a "load more" affordance) until
the result set is exhausted, without re-fetching earlier pages. A result set
larger than one page MUST remain reachable in full.

#### Scenario: Loading more appends the next page

- **WHEN** the user reaches the end of the loaded results and more exist
- **THEN** the next page is fetched and appended without dropping the earlier results

#### Scenario: Exhausted results stop paging

- **WHEN** all matching results have been loaded
- **THEN** no further page is requested and the list end is indicated

### Requirement: Add and remove a catalog score from the library

Each hub result SHALL offer an action to add the score to the user's library, and
a score already in the library SHALL show as saved with an action to remove it.
Adding SHALL persist through the backend save operation and removing through the
backend remove operation; the saved/unsaved state SHALL reflect immediately in the
hub. A saved catalog score SHALL thereafter appear on the home screen.

#### Scenario: Adding a result saves it

- **WHEN** the user taps add on a hub result
- **THEN** the score is saved through the backend and the result shows as saved

#### Scenario: Removing a saved result unsaves it

- **WHEN** the user taps remove on a result already in their library
- **THEN** the score is removed through the backend and the result shows as not saved

#### Scenario: Saved state is reflected in the hub

- **WHEN** the hub shows a result the user has already saved
- **THEN** that result is presented as saved (not offering a duplicate add)

### Requirement: Attribution shown for catalog scores

The Score Hub SHALL display the licence/source attribution for catalog scores as
required for redistributable material, so a user can see under what terms and from
what source a score is provided.

#### Scenario: Result shows its attribution

- **WHEN** a catalog score is shown in the hub
- **THEN** its licence and source attribution are visible to the user

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

