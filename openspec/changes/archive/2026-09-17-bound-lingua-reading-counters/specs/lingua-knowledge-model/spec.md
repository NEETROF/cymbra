## MODIFIED Requirements

### Requirement: Exposure counters
The model SHALL maintain, per (studied language, lemma), an exposure counter (occurrences encountered, the source of the last encounter, a timestamp) and enough state to count the number of distinct UTC days on which the lemma was encountered. Recording exposure SHALL NOT modify a lemma's status by itself; a status change driven by exposure SHALL happen only through the explicit promotion operation (see "Exposure-confirmed known"), never as a side effect of recording. The counter is input data for that promotion and for future SRS inference.

A client that reads continuously SHALL record a counter only for a lemma that promotion could still confirm — a declared level, and a lemma presumed known under it with neither an explicit nor a withdrawn status — and SHALL bound what it keeps: counters not encountered again within a retention window are dropped, and the kept set is capped, the most recently encountered winning. Loading a stored model SHALL apply the same bound, so a model saved without one shrinks instead of growing. A store the browser refuses to write SHALL be reported as such to the reader, never as a network or server failure.

#### Scenario: Ingesting an agent session
- **WHEN** a session containing 2 occurrences of a lemma with no status is ingested
- **THEN** that lemma's exposure counter increases by 2 and its status stays "new"

#### Scenario: Distinct days are counted
- **WHEN** a lemma is encountered several times on one UTC day and once on the next
- **THEN** its distinct-day count is 2, independent of the total occurrence count

#### Scenario: Reading without a declared level
- **WHEN** a reader with no declared level reads pages containing thousands of distinct words
- **THEN** no exposure counter is kept, because no amount of reading could promote anything

#### Scenario: A word the reader has decided about
- **WHEN** a lemma that was being counted receives an explicit status, or has one withdrawn
- **THEN** its counter is dropped: reading it again changes nothing

#### Scenario: A model saved without a bound
- **WHEN** a model carrying counters for tens of thousands of lemmas is loaded
- **THEN** it is pruned to the bound as part of loading

#### Scenario: The browser refuses the write
- **WHEN** storing the model fails because the browser's quota is exhausted
- **THEN** the reader is told the extension's memory is full and what to do about it
