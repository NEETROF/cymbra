# catalog-search Specification

## Purpose
TBD - created by archiving change score-hub-search. Update Purpose after archive.
## Requirements
### Requirement: Full-text search over title and composer

The backend SHALL expose an authenticated operation that searches the public
`catalog_scores` corpus by a free-text query, matching against both the score
title and the composer (author). Matching SHALL be case- and accent-insensitive
and SHALL tolerate partial words and minor misspellings (trigram similarity), so
a search-as-you-type query returns relevant results without an exact match. An
empty query SHALL return the corpus browsable (unfiltered by text), ordered
deterministically. Unauthenticated requests MUST be rejected.

For a normal (non-admin, non-moderator) caller the operation SHALL return only
scores whose moderation status is `accepted`; `pending` and `rejected` scores MUST
NOT appear in results. This visibility gate SHALL be enforced server-side and SHALL
compose with the text query and every other filter (a result must be `accepted` AND
satisfy the query AND all filters).

#### Scenario: Query matches title

- **WHEN** an authenticated caller searches for a term contained in a score's title
- **THEN** that score is returned in the results

#### Scenario: Query matches composer

- **WHEN** an authenticated caller searches for a term contained in a score's composer
- **THEN** that score is returned in the results

#### Scenario: Matching ignores case and accents

- **WHEN** a caller searches with different case or without accents than the stored value
- **THEN** the score still matches (e.g. "faure" matches "Fauré")

#### Scenario: Empty query browses the corpus

- **WHEN** a caller searches with an empty query
- **THEN** results are returned across the corpus in a deterministic order, not an error

#### Scenario: Unauthenticated search rejected

- **WHEN** a search request arrives without a valid authenticated identity
- **THEN** the request is rejected and no results are returned

#### Scenario: Only accepted scores returned to a normal caller

- **WHEN** a normal caller runs any search (including an empty browse query)
- **THEN** only scores whose moderation status is `accepted` are returned, and `pending`
  or `rejected` scores are absent regardless of the query or filters

### Requirement: Difficulty filter

The search operation SHALL accept an optional difficulty filter constrained to
the fixed set Beginner / Intermediate / Advanced. When a difficulty is supplied,
only catalog scores whose level equals it SHALL be returned; the filter SHALL
compose with the text query (both constraints apply). When no difficulty is
supplied, scores of every difficulty — including scores with no recorded level —
SHALL be eligible.

#### Scenario: Filter narrows to one difficulty

- **WHEN** a caller searches with difficulty = Intermediate
- **THEN** only catalog scores whose level is Intermediate are returned

#### Scenario: Difficulty composes with text query

- **WHEN** a caller supplies both a text query and a difficulty
- **THEN** results match the text query AND have the given difficulty

#### Scenario: No filter includes all levels

- **WHEN** a caller searches without a difficulty filter
- **THEN** results may include Beginner, Intermediate, Advanced, and unleveled scores

#### Scenario: Invalid difficulty rejected

- **WHEN** a caller supplies a difficulty outside Beginner / Intermediate / Advanced
- **THEN** the request is rejected with a typed error

### Requirement: Author filter

The search operation SHALL accept an optional author (composer) filter, distinct
from the free-text query. When an author value is supplied, only catalog scores
whose composer matches it (case- and accent-insensitive, partial-match tolerant)
SHALL be returned. The author filter SHALL compose conjunctively with the text
query and the difficulty filter — when several are supplied, a result MUST satisfy
all of them. When no author is supplied, composer imposes no additional constraint.

#### Scenario: Author filter narrows to a composer

- **WHEN** a caller searches with author = "Chopin"
- **THEN** only catalog scores whose composer matches Chopin are returned

#### Scenario: Author filter composes with query and difficulty

- **WHEN** a caller supplies a text query, an author, and a difficulty
- **THEN** results match the text query AND the author AND the difficulty

#### Scenario: Author filter ignores case and accents

- **WHEN** a caller filters by an author written with different case or without accents
- **THEN** matching catalog scores are still returned (e.g. "faure" matches "Fauré")

#### Scenario: No author filter imposes no composer constraint

- **WHEN** a caller searches without an author filter
- **THEN** results are not constrained by composer

### Requirement: Paginated, attribution-complete results

The search operation SHALL return results in bounded pages (a caller-supplied
limit clamped to a server maximum, plus an offset or cursor) and SHALL report
enough information to page through the full result set. Each returned entry SHALL
carry at least the catalog id, title, composer, difficulty level (when known),
and the licence/source attribution required to lawfully display the score
(licence code and source). Results SHALL NOT include raw score bytes.

#### Scenario: Results are bounded per page

- **WHEN** a caller requests results with a limit
- **THEN** at most that many entries (clamped to the server maximum) are returned in one page

#### Scenario: Paging reaches later results

- **WHEN** a caller requests the next page using the returned offset/cursor
- **THEN** the following entries are returned without duplicating or skipping items

#### Scenario: Entry carries attribution

- **WHEN** a search result entry is returned
- **THEN** it includes the catalog id, title, composer, level (when known), and the
  licence code and source needed to attribute the score

#### Scenario: Search never returns bytes

- **WHEN** a search returns entries
- **THEN** no entry includes the score's raw MusicXML/.mxl bytes

### Requirement: Trigram index backs the search

The system SHALL enable the `pg_trgm` extension (from an admin/ops migration, not
the least-privilege module role) and SHALL create a trigram GIN index covering
the normalised title and composer so the full-text query is index-backed rather
than a full-table scan. The search SHALL use the persisted normalised search
columns captured at ingest, requiring no per-query re-parse.

#### Scenario: Extension and index provisioned

- **WHEN** the migrations are applied
- **THEN** `pg_trgm` is enabled and a trigram GIN index exists over the catalog's
  normalised title and composer

#### Scenario: Search uses persisted normalised columns

- **WHEN** a search runs
- **THEN** it matches against the ingest-time normalised search columns without
  re-parsing any score

### Requirement: Fetch catalog score bytes for playback

The backend SHALL expose an authenticated operation that returns the canonical
(decoded) bytes of a catalog score by its id, read from the object store under
the public-corpus prefix, so the app can open it in the player. A request for a
non-existent catalog id MUST be rejected with a typed not-found error.
Unauthenticated requests MUST be rejected. For a normal (non-admin, non-moderator)
caller, a score whose moderation status is not `accepted` SHALL be treated as not
found: its bytes MUST NOT be served, so an unvalidated score cannot be opened by id.
A caller authorized as a moderator or admin SHALL be able to fetch the bytes of a
score in **any** moderation status (`pending` / `accepted` / `rejected`), so a reviewer
can open an unvalidated score to evaluate it. Until a dedicated moderator role exists,
"authorized" means an `admin` identity.

#### Scenario: Bytes returned for a known catalog score

- **WHEN** an authenticated caller requests the bytes of an existing `accepted` catalog id
- **THEN** the canonical score bytes are returned

#### Scenario: Unknown catalog id rejected

- **WHEN** a caller requests the bytes of a catalog id that does not exist
- **THEN** a typed not-found error is returned and no bytes are served

#### Scenario: Unauthenticated bytes request rejected

- **WHEN** a bytes request arrives without a valid authenticated identity
- **THEN** the request is rejected

#### Scenario: Bytes of an unvalidated score refused to a normal caller

- **WHEN** a normal caller requests the bytes of a `pending` or `rejected` catalog id
- **THEN** the request is refused as not found and no bytes are served

#### Scenario: Authorized reviewer fetches an unvalidated score's bytes

- **WHEN** a moderator/admin caller requests the bytes of a `pending` or `rejected` catalog id
- **THEN** the canonical score bytes are returned so the reviewer can evaluate it

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

### Requirement: Privileged moderation-status filter

The search operation SHALL accept an optional moderation-status filter that selects
scores by their moderation status (`pending` / `accepted` / `rejected`). This filter
is **privileged and back-office-only**: the server SHALL honor it only for a caller
authorized as a moderator or admin, and SHALL reject the request with a
permission-denied error when an unauthorized caller supplies it — the query MUST NOT
run and no results (of any status) are returned. When the filter is absent, the
operation behaves as for a normal caller (accepted-only visibility). The Flutter app
SHALL NOT send this filter; it exists for the moderation back office. Until a
dedicated moderator role exists, "authorized" means an `admin` identity.

#### Scenario: Authorized caller filters by status

- **WHEN** a moderator/admin caller searches with the moderation-status filter set to `pending`
- **THEN** only `pending` scores are returned, composing with the text query and other filters

#### Scenario: Unauthorized caller supplying the filter is rejected

- **WHEN** a normal caller supplies the moderation-status filter
- **THEN** the request is rejected with a permission-denied error and no results are returned

#### Scenario: Absent filter keeps accepted-only visibility

- **WHEN** any caller searches without supplying the moderation-status filter
- **THEN** a normal caller sees only `accepted` scores, unchanged from the default behavior

#### Scenario: Authorized caller can still request accepted only

- **WHEN** a moderator/admin caller sets the moderation-status filter to `accepted`
- **THEN** only `accepted` scores are returned

### Requirement: Optional structured result sort

The catalog search operation SHALL accept an **optional** sort argument expressed as an
**ordered list of `{ field, direction }` keys** (not an opaque encoded string), where
`direction` is ascending or descending and the list expresses multi-key ordering (the
first key is primary, the next breaks ties, and so on). Each `field` SHALL be validated
against a server **allow-list** of sortable keys; an unknown or non-allow-listed field
MUST be rejected with a typed error and the query MUST NOT run.

When **no** sort argument is supplied, the operation SHALL apply its existing default
ordering unchanged. The app hub sends no sort argument, so its result ordering MUST be
identical to before this change — the hub is unaffected.

Sort keys that concern moderation (the moderation status ordering and the re-review flag,
and any composite "review priority" key built from them) SHALL be **privileged**: only a
`moderator`/`admin` identity may sort by them, rejected otherwise, consistent with the
privileged moderation-status filter. Substance/facet sort keys (e.g. measure count, staff
count) MAY be offered to any caller. Sorting SHALL be applied server-side so the order is
correct across the entire paginated result set, and paging with the same sort SHALL be
stable.

#### Scenario: App hub ordering is unchanged

- **WHEN** the app hub searches without supplying a sort argument
- **THEN** results come back in the same default order as before this change

#### Scenario: Structured multi-key sort is applied

- **WHEN** a caller supplies a sort list such as `[{re-review flag, desc}, {measure count, desc}]`
- **THEN** results are ordered by the first key, then the second as a tie-breaker, across the whole result set

#### Scenario: Unknown sort field is rejected

- **WHEN** a caller supplies a sort field not in the allow-list
- **THEN** the request is rejected with a typed error and no results are returned

#### Scenario: Moderation sort key is privileged

- **WHEN** a normal caller supplies a moderation-oriented sort key (status or re-review flag)
- **THEN** the request is rejected (permission denied), while a moderator/admin may use it

#### Scenario: Paging keeps the sort stable

- **WHEN** a caller pages through results supplying the same sort argument on each page
- **THEN** the ordering is consistent across pages with no duplicates or skips

### Requirement: Instrument filter parameter

The search operation SHALL accept an optional **instrument** filter naming an
instrument family (`keyboard` or `percussion`); when set, only scores recorded with
that instrument SHALL be returned. When unset, scores are not constrained by
instrument. This parameter is independent of the caller (the app may always supply
it) and composes conjunctively with the other filters. It replaces the former
boolean piano filter, which rested on a staff-count proxy rather than on the
score's actual instrument.

Independently of the filter the caller supplies, results SHALL be constrained by
the caller's drum eligibility (see `music-drums-visibility`): a caller without the
drum feature never receives a percussion score, whether they filtered for one,
filtered for keyboard, or filtered for nothing at all. The filter narrows; it never
widens.

#### Scenario: Instrument filter narrows to that family

- **WHEN** an eligible caller searches with the instrument filter set to keyboard
- **THEN** only scores recorded as keyboard are returned

#### Scenario: No instrument filter is not constrained by instrument

- **WHEN** an eligible caller searches without the instrument filter
- **THEN** results are not constrained by instrument

#### Scenario: Filtering for percussion without the feature returns nothing

- **WHEN** an ineligible caller searches with the instrument filter set to
  percussion
- **THEN** no scores are returned, rather than an error revealing the constraint

#### Scenario: An unfiltered search still withholds percussion

- **WHEN** an ineligible caller searches without any instrument filter
- **THEN** percussion scores are absent from the results

