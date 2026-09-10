# Tasks — add-lingua-data-pack

## 1. Data packs (spec lingua-data-packs)

- [x] 1.1 `scripts/lingua-data/`: reproducible pipeline (dated sources, raw data never committed) — AGID download + wordfreq export + kaikki fr-glosses extract
- [x] 1.2 Build the FST (AGID inverted) + the frequency table (quantised ranks) + the offset-indexed `gloss.zst` (top lemmas, within budget)
- [x] 1.3 `pack.lingua` container format (magic, TOC, meta carrying `pack_version`/`analyzer_version`/licences, NOTICE) + reader in `lingua-core` that refuses incompatible versions
- [x] 1.4 Licence guard-rails: documented GPL/AGPL/NC denylist + NOTICE check at build time; build fails if the pack exceeds 5 MB
- [x] 1.5 Reproducibility test (two builds, identical output); the (en→fr) pack is built in CI and cached (never committed), local build documented
