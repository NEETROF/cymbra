# add-lingua-wasm — Cymbra Lingua: the WASM target and native/WASM parity

## Why

Lingua's "one brain" principle demands that the same analysis (same `analyzer_version`)
produce the same results in the extension (WASM) and in the plugin (native). The core and
the pack exist (`add-lingua-analysis`, `add-lingua-data-pack`) but only compile natively;
this change delivers the **WASM target** of `lingua-core` — wasm-bindgen bindings, a
`wasm-pack --target web` build — and makes **native/WASM parity a contract tested in CI**
before a single browser surface depends on it. The extension changes
(`add-lingua-extension-reading` and its successors) consume this module as-is.

**Position in the stack (5/12)**: add-lingua-analysis → add-lingua-knowledge-model →
add-lingua-decks-review → add-lingua-data-pack → **add-lingua-wasm** →
add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox →
add-lingua-apple → add-lingua-agent → add-lingua-backend →
add-lingua-connected-clients. **Explicit prerequisites: `add-lingua-analysis`** (the
analyser, its `analyzer_version` and its fixture corpus) **and `add-lingua-data-pack`**
(the pack both targets load — parity is only meaningful at equal pack).

## What Changes

- **`lingua-wasm` crate/feature** — wasm-bindgen bindings (batch-of-blocks analysis →
  classified tokens, statuses, percentage, glosses); a `wasm-pack --target web` build.
- **Native/WASM parity tests** over the fixtures: same `analyzer_version` (and same pack)
  ⇒ byte-for-byte identical output.
- **CI lane** — build the wasm module and run the parity tests.
- The logic stays host-tested in `lingua-core`; only the wasm-bindgen glue is excluded
  from coverage (the `--ignore-filename-regex` convention, the same treatment as music's
  frb glue).

## Capabilities

### New Capabilities
_None._

### Modified Capabilities
- `lingua-analysis`: gains the "Native/WASM parity" requirement — the analysis pipeline
  becomes compilable as a WASM module, with parity as a contract verified in CI. No
  existing requirement moves.

## Impact

- **Products**: Lingua only (a new compilation target for the core — nothing consumed
  outside the Lingua stack); **Cymbra ID / Music / Live / back office / site: untouched**
  (no proto, no backend crate, no existing app modified).
- **Tree**: `crates/lingua-core` (a `lingua-wasm` companion crate/feature) — the WASM
  module gets embedded by `apps/lingua-extension` in the next change. If `lingua-wasm` is
  a separate workspace crate, it is added to the `ci-units` filter alongside the wasm lane
  that watches it.
- **CI**: a new wasm-pack build + parity-test lane; the existing Rust lane keeps covering
  the logic (llvm-cov ≥ 80%, wasm-bindgen glue excluded through the shared
  `--ignore-filename-regex`).
- **New dependencies**: `wasm-bindgen` / wasm-pack.
- **Out of scope**: where the WASM lives inside the extension (content script vs event
  page — the `AnalyzerPort` seam of `add-lingua-extension-reading`), the Firefox CSP spike
  (`add-lingua-firefox`), and any UI.
