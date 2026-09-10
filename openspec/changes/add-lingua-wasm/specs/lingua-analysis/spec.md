# lingua-analysis — WASM target and parity

## ADDED Requirements

### Requirement: Native/WASM parity
The core SHALL be compilable as a WASM module (a `wasm-pack --target web` build) exposing batch-of-blocks analysis (classified tokens, statuses, percentage, glosses), and SHALL produce, at equal `analyzer_version` and equal pack, byte-for-byte identical output between the native target and the WASM target over the fixture corpus. A CI lane SHALL build the WASM module and run the parity tests.

#### Scenario: Parity over the fixture corpus
- **WHEN** the same fixture text is analysed by the native binary and by the WASM module at the same `analyzer_version` and with the same pack
- **THEN** the outputs (token, lemma, classification, percentage) are byte-for-byte identical

#### Scenario: Divergence blocked in CI
- **WHEN** a change to the core makes the WASM output diverge from the native output on a fixture
- **THEN** the CI parity lane fails
