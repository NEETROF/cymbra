# Design — add-lingua-wasm

## Context

The fifth storey of the Lingua stack. The core (`add-lingua-analysis`) is deterministic at
a given `analyzer_version` and carries a regression corpus of fixtures; the pack
(`add-lingua-data-pack`) is versioned and compatibility-gated. This change adds the second
compilation target for the same brain. Inherited and not re-litigated here: contractual
determinism and the fixtures (`add-lingua-analysis`), and the pack format and its 5 MB
budget (`add-lingua-data-pack`).

## Decisions

### D1 — WASM target: wasm-pack `--target web`, thin bindings, logic in the core
`lingua-core` compiles to WASM through wasm-pack `--target web` — the format a content
script or an extension event page consumes as-is, with no dedicated bundler. The
wasm-bindgen bindings (the `lingua-wasm` crate/feature) are a thin layer:
**batch-of-blocks** analysis → classified tokens, statuses, percentage, glosses — no logic
inside. The logic stays in `lingua-core`, host-tested (the monorepo convention: binding
glue is excluded from coverage, like music's frb glue; the logic never is). One WASM
artefact only: what varies per browser (where it is instantiated) is confined behind the
`AnalyzerPort`, a decision owned by `add-lingua-extension-reading`.

### D2 — Parity is a contract tested in CI, not a promise
Contractual determinism, inherited from `add-lingua-analysis`, extended across targets: at
equal `analyzer_version` and equal pack, output is byte-for-byte identical between native
and WASM, tested in CI over the fixture corpus (fixtures crossed native/WASM). The parity
lane fails on any divergence — a silent divergence between targets would corrupt the
percentage the extension shows without a single native test noticing. This is the "one
brain" guarantee every surface change downstream rests on.

## Risks / Trade-offs

- [Native/wasm32 divergence (floats, integer widths, iteration order)] → canonicalised
  output (deterministic representations, ordered collections); the parity lane is the net
  — it turns a class of undetectable bugs into a CI failure.
- [Module size (~1 MB of wasm code + a ≤ 5 MB pack)] → the pack budget is set by
  `add-lingua-data-pack`; lazy instantiation, memoisation and per-tab cost belong to
  `add-lingua-extension-reading` (risk managed there, target < 50 ms init).
- [WASM inside Firefox content scripts (CSP)] → out of scope here: it is the day-one spike
  of the Firefox port (`add-lingua-firefox`); the `--target web` build holds in either
  location (content script or event page).
