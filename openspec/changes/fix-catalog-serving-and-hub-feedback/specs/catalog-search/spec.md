## MODIFIED Requirements

### Requirement: Paginated, attribution-complete results

The search operation SHALL return results in bounded pages (a caller-supplied
limit clamped to a server maximum, plus an offset or cursor) and SHALL report
enough information to page through the full result set. Each returned entry SHALL
carry at least the catalog id, title, composer, difficulty level (when known),
and the licence/source attribution required to lawfully display the score
(licence code and source). Results SHALL NOT include raw score bytes.

Each page SHALL additionally report the **total number of entries matching the
query and filters**, independent of the page's limit/offset, so a caller can show
the full match count without loading every page. The total SHALL reflect the same
filter predicates as the returned entries.

#### Scenario: Results are bounded per page

- **WHEN** a caller requests results with a limit
- **THEN** at most that many entries (clamped to the server maximum) are returned in one page

#### Scenario: Paging reaches later results

- **WHEN** a caller requests the next page using the returned offset/cursor
- **THEN** the following entries are returned without duplicating or skipping items

#### Scenario: Page reports the full match total

- **WHEN** a query matches more entries than fit in one page
- **THEN** the response reports the total count of matching entries (independent of
  the page limit/offset), consistent with the applied filters

#### Scenario: Entry carries attribution

- **WHEN** a search result entry is returned
- **THEN** it includes the catalog id, title, composer, level (when known), and the
  licence code and source needed to attribute the score

#### Scenario: Search never returns bytes

- **WHEN** a search returns entries
- **THEN** no entry includes the score's raw MusicXML/.mxl bytes
