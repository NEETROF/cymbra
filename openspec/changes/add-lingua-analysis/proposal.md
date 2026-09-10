# add-lingua-analysis — Cymbra Lingua: monorepo foundations + analysis pipeline

## Why

The existing read-in-a-foreign-language products (LingQ, Readlang, Migaku) count
surface forms instead of lemmas — their "words known" counters lie. Honest
per-lemma counting is Cymbra Lingua's differentiator #1, and it rests entirely on a
deterministic analysis pipeline: tokenisation, cascading lemmatisation, language
detection, percentage of known tokens. This change lays **the first brick** of the
product: the `lingua-core` crate in the monorepo (with its CI/coverage lane and the
`lingua-*` OpenSpec prefix) and the complete analysis pipeline, tested on
mini-fixtures. Everything else in the stack — knowledge model, decks, packs, WASM,
extension, Apple app, agent plugin, backend — consumes this pipeline.

**Position in the stack** (12 changes, implementation order): **1/12 — head of the
stack, no prerequisites.** Then: add-lingua-knowledge-model → add-lingua-decks-review
→ add-lingua-data-pack → add-lingua-wasm → add-lingua-extension-reading →
add-lingua-extension-review → add-lingua-firefox → add-lingua-apple →
add-lingua-agent → add-lingua-backend → add-lingua-connected-clients.

## What Changes

- **New OpenSpec prefix `lingua-*`** added to `openspec/config.yaml`. This change
  touches no existing capability.
- **New crate `crates/lingua-core`** in the root Cargo workspace: UAX #29
  tokenisation with a per-language pre-pass, lemmatisation by FST table +
  morphological fallback + out-of-lexicon plural fallback, per-block language
  detection, known-token percentage, a contractual `analyzer_version`. Pure
  host-testable logic (the `*_core.rs` convention); the module layout (`analysis/`,
  `knowledge/`, `decks/`, `packs/`) is laid out for the changes that follow.
- **CI**: the crate is covered by the existing Rust lane (fmt/clippy/llvm-cov ≥ 80%);
  the exclusion for the future wasm-bindgen glue is added to
  `--ignore-filename-regex`.
- The tests in this change use **synthetic mini-fixtures** (test FST/frequency
  tables); the real pack (AGID/wordfreq/kaikki) arrives with `add-lingua-data-pack`.

## Capabilities

### New Capabilities
- `lingua-analysis`: the text analysis pipeline — tokenisation, per-language pre-pass
  (elisions/clitics), lemmatisation (FST + fallback + out-of-lexicon plural fallback),
  per-block language detection, known-token percentage. Deterministic at a given
  `analyzer_version`.

### Modified Capabilities
_None. This change is local to the new crate: it neither consumes nor modifies
`id-*`/`platform-*`._

## Impact

- **Products**: Lingua (new); **Cymbra ID / Music / Live / back office / site:
  untouched** (no proto, no backend crate, no existing app modified). The platform is
  consumed in exactly one place: the coverage convention and CI.
- **Tree**: `crates/lingua-core` (root Cargo workspace); `openspec/config.yaml` (the
  `lingua-*` domain).
- **CI**: the existing Rust lane (`cargo --workspace`) covers the crate automatically;
  `.github/coverage-ignore-regex.txt` updated for the future wasm-bindgen glue.
- **New dependencies**: `fst`, `unicode-segmentation`, `whichlang`, `serde` (the `fsrs`
  crate arrives with `add-lingua-decks-review`).
- **Out of scope**: statuses/calibration (`add-lingua-knowledge-model`),
  decks/review, the real pack, the WASM target and the native/WASM parity test
  (`add-lingua-wasm`), every user-facing surface.
