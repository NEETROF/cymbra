# Tasks — add-lingua-decks-review

## 1. lingua-core — decks and review (spec lingua-decks-review)

_Tests on synthetic mini-fixtures (as in the earlier storeys of the core); the real pack arrives with `add-lingua-data-pack`._

- [x] 1.1 Card schema (lemma, form, sentence, source, gloss, optional `media` with `source`/`sync_policy`, FSRS state), versioned serialisable
- [x] 1.2 FSRS-5 scheduling in the core (no new deps): `again/hard/good/easy` grading, due dates, due count; the 19-weight parameter vector + request-retention + max-interval stored on the state (optimiser out of scope — design D2)
- [x] 1.3 "I know this" during review → `known` status with provenance `srs`, card kept out of the queue; tests
- [x] 1.4 Backup/restore: full state export (cards, statuses, calibration, FSRS parameters) to a versioned file + identical restore; lossless round-trip tests (field by field)
