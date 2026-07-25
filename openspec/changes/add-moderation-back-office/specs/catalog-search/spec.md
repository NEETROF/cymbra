## ADDED Requirements

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
