## MODIFIED Requirements

### Requirement: Lingua usage aggregates
The "Lingua" screen SHALL present, over a filterable date window (30 days by default), aggregates served by the Lingua backend's admin RPCs: tiles (active synced accounts, words learned, reviews, words read, new words seen, reading comprehension), per-day time series (words learned, reviews, words read, new words seen), and a breakdown by studied language carrying the same totals and comprehension. The aggregates are computed server-side over the existing synchronisation data — the synced daily statistics, no other telemetry — and the screen states that only accounts that sync are counted. Labels are plain-language: no UI string contains the word "lemma" (say "words learned", "distinct words"), and the words-read figures are labelled "Words read" / « Mots lus », never "exposures".

**Reading comprehension** SHALL be the share of words read that were not new — `1 − new words seen ÷ words read`. When no word was read in scope, it SHALL be shown as unavailable, never as 0 % or 100 %.

#### Scenario: Consulting a window
- **WHEN** a lingua admin opens the screen with the default window
- **THEN** the tiles, the per-day series and the breakdown by studied language for that window are displayed, together with the "synced accounts" note

#### Scenario: Plain-language vocabulary
- **WHEN** the screen's UI strings (en and fr) are run through the vocabulary lint
- **THEN** no occurrence of "lemma" is found

#### Scenario: Comprehension
- **WHEN** the window holds 1 000 words read with 50 new words seen
- **THEN** the comprehension shows 95 %

#### Scenario: Nothing read
- **WHEN** no word was read in the window
- **THEN** the comprehension is shown as unavailable

#### Scenario: Per-language comprehension
- **WHEN** the breakdown lists `en`
- **THEN** its row shows the words read, new words seen and comprehension of `en` alone

## ADDED Requirements

### Requirement: Only up-to-date extensions are counted
The Lingua backend SHALL store a pushed daily statistic only when it carries the new-words-seen counter; a daily statistic pushed without it — by an extension that predates the viewport-gated counting — SHALL be acknowledged and not stored, so that its whole-document figures never reach the aggregates.

#### Scenario: An outdated extension pushes
- **WHEN** an extension that predates the new counter pushes a day of statistics
- **THEN** the push succeeds for the client and no daily statistic is stored for it, while its word statuses and cards sync as before

#### Scenario: An up-to-date extension pushes a quiet day
- **WHEN** an updated extension pushes a day with words learned but zero new words seen
- **THEN** the statistic is stored, the zero being a reported value
