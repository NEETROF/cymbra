# Tasks — add-lingua-knowledge-model

## 1. lingua-core — knowledge model (spec lingua-knowledge-model)

- [ ] 1.1 Status types (`learning/known/ignored` + provenance `manual/calibration/srs/import`) and implicit-known resolution by rank ≤ calibration; the explicit status wins
- [ ] 1.2 Multi-candidate resolution (known if any candidate is) with tests
- [ ] 1.3 L1/L2 profile: `native_language` + studied languages; every API keyed by pair; tests with a dummy pair
- [ ] 1.4 LingQ import (CSV) → `known` statuses with provenance `import`, entries lemmatised; test on a real anonymised sample
- [ ] 1.5 Exposure counters per (language, lemma): increments on ingestion, source + timestamp, with no effect on statuses; tests
