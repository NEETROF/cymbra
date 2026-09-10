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
known when its frequency rank is at or below the user's calibration threshold. Every
`known` status SHALL carry its provenance (`manual`, `calibration`, `srs`, `import`).

#### Scenario: Calibration at startup
- **WHEN** the user sets their calibration to "I know the 3,000 most common words" without having marked any word
- **THEN** every lemma of rank ≤ 3,000 is classified as known by the analysis

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
The model SHALL maintain, per (studied language, lemma), an exposure counter
(occurrences encountered, the source of the last encounter, a timestamp). In v1,
exposure SHALL NOT modify a lemma's status — it is input data for the future inference
("known" deduced from the SRS / from exposure).

#### Scenario: Ingesting an agent session
- **WHEN** a session containing 2 occurrences of a lemma with no status is ingested
- **THEN** that lemma's exposure counter increases by 2 and its status stays "new"

### Requirement: Jargon-free interface vocabulary
User-facing surfaces SHALL NOT display the term "lemma" (nor its French equivalent
« lemme » in the shipped French UI) — this covers the extension **and** the plugin
(statusline, `/vocab`, MCP responses). Labels SHALL use the wording "dictionary form"
(for the canonical form) and "distinct words" (for counts of unique lemmas).

#### Scenario: Popup for an inflected word
- **WHEN** the user clicks `pitfalls`
- **THEN** the popup is titled `pitfall` and shows the encountered form — in the shipped French UI, « forme vue : “pitfalls” » — with no occurrence of the word "lemma"

