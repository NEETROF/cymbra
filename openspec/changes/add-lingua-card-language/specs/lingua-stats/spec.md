# lingua-stats — the language of a daily stat is normalised like every other

## ADDED Requirements

### Requirement: The language of a daily stat is normalised on receipt
The language of a daily aggregate SHALL be normalised on receipt with the same rule as statuses and cards (trimmed, lowercased, primary subtag, empty read as `en`), so that one language never splits into several series.

#### Scenario: A regional code on a stat
- **WHEN** a device upserts its daily row with language `es-MX`
- **THEN** the row is stored and consolidated under `es`
