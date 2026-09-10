# Tasks — add-lingua-knowledge-model

## 1. lingua-core — knowledge model (spec lingua-knowledge-model)

- [x] 1.1 Status types (`learning/known/ignored` + provenance `manual/calibration/srs/import`) and implicit-known resolution by rank ≤ calibration; the explicit status wins
- [x] 1.2 Multi-candidate resolution (known if any candidate is) with tests
- [x] 1.3 L1/L2 profile: `native_language` + studied languages; every API keyed by pair; tests with a dummy pair
- [x] 1.4 Exposure counters per (language, lemma): increments on ingestion, source + timestamp, with no effect on statuses; tests
