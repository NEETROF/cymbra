# Tasks — add-lingua-decks-review

## 1. lingua-core — decks and review (spec lingua-decks-review)

_Tests on synthetic mini-fixtures (as in the earlier storeys of the core); the real pack arrives with `add-lingua-data-pack`._

- [ ] 1.1 Card schema (lemma, form, sentence, source, gloss, optional `media` with `source`/`sync_policy`, FSRS state), versioned serialisable
- [ ] 1.2 FSRS integration: `again/hard/good/easy` grading, due dates, due count; crate version pinned, parameters stored on the state
- [ ] 1.3 "I know this" during review → `known` status with provenance `srs`, card kept out of the queue; tests
- [ ] 1.4 Backup/restore: full state export (cards, statuses, calibration, FSRS parameters) to a versioned file + identical restore; lossless round-trip tests (field by field)
