# lingua-knowledge-model Specification

## Purpose
TBD - created by archiving change add-lingua-knowledge-model. Update Purpose after archive.
## Requirements
### Requirement: Knowledge keyed by language-lemma pair
Knowledge state SHALL be stored under a `(studied language, lemma)` key — with no part
of speech. Multi-word expressions SHALL be lemmas in their own right (text with
spaces). A token SHALL be considered known when **at least one** of its candidate
lemmas is known.

#### Scenario: Ambiguity resolved in the learner's favour
- **WHEN** the lemma `can` is marked known and the token `cans` is analysed
- **THEN** the token counts as known

### Requirement: Explicit statuses and implicit status by calibration
The model SHALL support the explicit statuses `learning`, `known` and `ignored`; the
absence of an entry means "new". A lemma with no explicit status SHALL be implicitly
known ("presumed") when the pack supplies a CEFR level and that level is below the user's
declared level, or, when no CEFR level is available for the studied language, when its
frequency rank is at or below the user's calibration threshold. Every `known` status
SHALL carry its provenance (`manual`, `calibration`, `srs`, `import`, `exposure`).

#### Scenario: Calibration at startup
- **WHEN** the user sets their calibration to "I know the 3,000 most common words" without having marked any word
- **THEN** every lemma of rank ≤ 3,000 is classified as known by the analysis

#### Scenario: Presumed known below the declared level
- **WHEN** the user declares level B2 and the pack tags a lemma as A2, with no explicit status for it
- **THEN** the analysis classifies it as known with provenance `calibration` (presumed)

#### Scenario: The explicit status wins over calibration
- **WHEN** a lemma of rank 500 is explicitly marked `learning`
- **THEN** the analysis classifies it as learning despite the calibration

### Requirement: L1/L2 profile
The user profile SHALL keep `native_language` (the language of comfort: glosses, future
translations) distinct from the studied languages, and all language-dependent data
(glosses, packs, knowledge state) SHALL be keyed by pair (studied language → native
language). The MVP SHALL ship the (English → French) pair only, and adding a pair SHALL
NOT require a code change.

#### Scenario: Glosses in the native language
- **WHEN** a user whose native language is `fr` opens the popup for an English word
- **THEN** the gloss shown comes from the (en → fr) pack

### Requirement: Exposure counters
The model SHALL maintain, per (studied language, lemma), an exposure counter (occurrences
encountered, the source of the last encounter, a timestamp) and enough state to count the
number of distinct UTC days on which the lemma was encountered. Recording exposure SHALL
NOT modify a lemma's status by itself; a status change driven by exposure SHALL happen
only through the explicit promotion operation (see "Exposure-confirmed known"), never as a
side effect of recording. The counter is input data for that promotion and for future SRS
inference.

#### Scenario: Ingesting an agent session
- **WHEN** a session containing 2 occurrences of a lemma with no status is ingested
- **THEN** that lemma's exposure counter increases by 2 and its status stays "new"

#### Scenario: Distinct days are counted
- **WHEN** a lemma is encountered several times on one UTC day and once on the next
- **THEN** its distinct-day count is 2, independent of the total occurrence count

### Requirement: Jargon-free interface vocabulary
User-facing surfaces SHALL NOT display the term "lemma" (nor its French equivalent
« lemme » in the shipped French UI) — this covers the extension **and** the plugin
(statusline, `/vocab`, MCP responses). Labels SHALL use the wording "dictionary form"
(for the canonical form) and "distinct words" (for counts of unique lemmas).

#### Scenario: Popup for an inflected word
- **WHEN** the user clicks `pitfalls`
- **THEN** the popup is titled `pitfall` and shows the encountered form — in the shipped French UI, « forme vue : “pitfalls” » — with no occurrence of the word "lemma"

### Requirement: Exposure-confirmed known
The model SHALL provide an explicit, caller-driven operation that promotes a lemma to
`known` with provenance `exposure`. The operation SHALL promote a lemma only when ALL of:
it is below the user's declared level (presumed), it has no explicit status, and it has
been read on at least the configured number of distinct days (default 4). It SHALL be a
no-op for any lemma that has an explicit status, so any user interaction (marking,
adding to a deck, ignoring) permanently blocks promotion. A promoted `known` SHALL remain
distinguishable by its `exposure` provenance so promotions are reversible in bulk.

#### Scenario: Promotion after repeated reading
- **WHEN** a below-level lemma with no explicit status has been read on 4 distinct days and the promotion operation runs
- **THEN** it becomes `known` with provenance `exposure`

#### Scenario: An interaction blocks promotion
- **WHEN** a below-level lemma read on 6 distinct days was at any point added to the deck (an explicit status)
- **THEN** the promotion operation leaves it unchanged

#### Scenario: Recording alone never promotes
- **WHEN** exposures are recorded but the promotion operation is not called
- **THEN** no status changes, preserving behaviour for callers (e.g. the agent plugin) that never promote

### Requirement: Per-level knowledge statistics
The model SHALL compute, over the lemmas of a given rank/level band, a breakdown of how
many resolve to confirmed known (an explicit `known`, any provenance except `calibration`),
presumed known (implicit `calibration`), and to-learn (learning or new), so a caller can
render per-level progress. The computation SHALL fold the existing lemma resolution over
the band and SHALL be deterministic and host-testable.

#### Scenario: Band breakdown
- **WHEN** a caller requests statistics for the A2 band
- **THEN** it receives counts of confirmed, presumed, and to-learn lemmas whose total equals the band size

