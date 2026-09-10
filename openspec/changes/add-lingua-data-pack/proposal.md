# add-lingua-data-pack — Cymbra Lingua: the English → French data pack

## Why

The analysis core (`add-lingua-analysis`) and every stage above it are tested against
tiny synthetic fixtures; analysing real text needs real data — the form→lemma FST, the
frequency ranks and the glosses. This change delivers the **English → French data pack**
built offline (AGID as an FST, wordfreq frequencies, kaikki glosses): a versioned pack
format, keyed by pair (studied language → native language), with the full stack of
licence notices embedded. It is also the change that fixes the product's licence hygiene
(commercial-use data only, GPL/NC denylist) and the size budget that makes the pack
shippable inside an extension.

**Position in the stack (4/12)**: add-lingua-analysis → add-lingua-knowledge-model →
add-lingua-decks-review → **add-lingua-data-pack** → add-lingua-wasm →
add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox →
add-lingua-apple → add-lingua-agent → add-lingua-backend →
add-lingua-connected-clients. **Explicit prerequisite: `add-lingua-analysis`** — the core
is what reads the format: the lemmatisation cascade consumes the FST, the percentage
consumes the ranks, and the `analyzer_version` the core exposes gates pack
compatibility.

## What Changes

- **`scripts/lingua-data/` pipeline** — a reproducible offline build (dated sources, raw
  data never committed): AGID download, wordfreq export, kaikki fr-glosses extract; then
  the FST (AGID inverted), the frequency table (quantised ranks) and the offset-indexed
  `gloss.zst` (top lemmas, within budget).
- **`pack.lingua` container format** — magic + TOC: `meta` (pair, `pack_version`,
  compatible `analyzer_version`, licences), `forms.fst`, `lemmas.bin`, `freq.bin`,
  `gloss.zst`, `NOTICE` — plus a **reader in `lingua-core`** that refuses incompatible
  versions.
- **Licence guard-rails** — a GPL/AGPL/NC denylist documented in `scripts/lingua-data`,
  and a NOTICE check at build time.
- **Size budget** — the build fails if the pack exceeds 5 MB; the remedy shrinks gloss
  coverage, never the FST or the frequencies.
- **The pack is never committed** — it is rebuilt in CI (determinism tested) and cached;
  a local dev builds it once via the script.
- MVP ships a single pack (EN→FR), but **all the code is pair-keyed** — adding (ES→FR) is
  data, not code.

## Capabilities

### New Capabilities
- `lingua-data-packs`: the per-pair (L2→L1) data-pack format — form→lemma FST,
  frequencies, glosses — versioned, with licence attributions (AGID / wordfreq CC BY-SA /
  kaikki CC BY-SA), plus the reproducible offline build pipeline.

### Modified Capabilities
<!-- None. `lingua-analysis` consumes the pack through its existing APIs (FST lookup,
     frequency ranks) — the analysis contract does not move. The "Attributions" page the
     spec requires is a contract of this capability, realised on the extension side by
     `add-lingua-extension-reading`. -->

## Impact

- **Products**: Lingua only (data + pack reader — nothing consumed outside the Lingua
  stack); **Cymbra ID / Music / Live / back office / site: untouched** (no proto, no
  backend crate, no existing app modified).
- **Tree**: `scripts/lingua-data/` (new pipeline), `crates/lingua-core` (a `packs/`
  module: the container reader). No new `apps/*`/`packages/*`/`crates/*` unit — nothing to
  add to `ci-units`.
- **CI**: the existing Rust lane covers the reader (llvm-cov ≥ 80%); the (en→fr) pack is
  built in CI and cached (never committed), with a reproducibility test.
- **New dependencies**: AGID + wordfreq + kaikki data (built offline, raw sources not
  committed); zstd compression for the glosses.
- **Out of scope**: the "Attributions" page in the extension
  (`add-lingua-extension-reading`), embedding the pack in the WASM module
  (`add-lingua-wasm`) and in the Apple bundle (`add-lingua-apple`), and the Romance-language
  packs (fr/it/es/pt — the pair-keyed format is waiting for them).
