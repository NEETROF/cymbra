# add-lingua-language-stats-review — review and statistics, language by language

## Why

A reader of English and Spanish holds cards and statistics in both, but nothing tells them apart:
- **The review queue.** It takes every due card, and a card never says its language.
- **The daily statistics.** Words read, new words met, words learned and reviews are counted per
  day only, and the sync sends them as English (`pushStats`).
- **The statistics page.** It shows the reader's first language and offers no other.

The product owner decided the review (D8 in `docs/lingua/spanish-programme.md`): one queue ordered by
due date across languages, with a language filter, not one session per language.

This is change 14 of the programme, in R3, a silent English release. Every reader studies English
alone, so the filter, the card's language and the statistics' selector stay hidden. The statistics
are those of English, as today.

## What Changes

- **The engine reviews across languages, or within some.**
  - `startReview(now, languages?)` and `dueCount(now, languages?)` take an optional list: absent
    or empty means every language, ordered by due date as today.
  - The review card names its language (`reviewCurrentLanguage`, apart from the card's view,
    which the English baseline pins).
- **The review page filters by language** when the reader accepts several: « Toutes », then one
  choice per language.
  - The summary and the session follow the filter.
  - The card shows its language.
  - Each review and each word learned is counted in the card's language.
- **Daily statistics are kept per language.** `cymbra-lingua-daily-v3` holds day → language →
  counts. The day-only counts of `-v2` are read as English, so nothing is lost. Reading, learning
  and reviewing record in their language. The sync sends one statistic per day and language, with
  that language.
- **The statistics page has a language selector** when the reader accepts several: the ladder, the
  estimate, the seed, the marked words and the daily figures are the selected language's. The
  reader's first language is the default.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-decks-review`: ADDED — *One review queue across languages, with a language filter* and *A
  review card says its language*. The open changes on this capability (`add-lingua-card-language`,
  `add-lingua-reader`) hold requirements under other names.
- `lingua-browser-extension`: ADDED — *Statistics per studied language*.
- `lingua-stats`: ADDED — *A device's daily statistics carry their language*. The capability is
  created by open changes that only add requirements. This change archives after
  `add-lingua-card-language`.

## Impact

- **Products.** Cymbra Lingua's browser extension and its engine:
  - `crates/lingua-core`: a review session over some languages;
  - `crates/lingua-wasm`: optional languages on two bindings, and the card's language;
  - the extension's review page, statistics page, daily statistics, session and sync.

  No server or proto change: `DailyStat.language` exists, and the server normalises it.
- **Release.** R3, silent: one language, the controls hidden. The day-only statistics become English
  ones, which they were.
