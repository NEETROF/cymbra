## 1. The vote (lingua-core)

- [x] 1.1 `analysis/language.rs`: `detect_document_language(blocks, candidates, hint)` (design D1). Tests:
  - Spanish blocks against English and Spanish;
  - an English page with a short Spanish quotation;
  - blocks too short, with and without a hint;
  - a tie broken by the hint, then by the order;
  - a single candidate.

## 2. The engine and the port

- [x] 2.1 `lingua-wasm`: `detectLanguage(blocks, candidates, hint)` (design D2). Native tests for the choice. `wasm-bindgen` tests for the refusals: no candidate, an unknown candidate.
- [x] 2.2 `LinguaPort.detectLanguage`, in the engine mirror, `WasmAnalyzerPort` (no pack loaded), `MessagingLinguaPort` and the fake port. Specs: forwarded as a whole-reader call, loading no pack.

## 3. The reading session

- [x] 3.1 `session.ts`: `languages` from `acceptedLanguages`; with several, `repaint()` asks for the document's language with its `lang` hint, then reads in it (design D3).
- [x] 3.2 Specs, with en-fr and es-fr shipped:
  - a Spanish page is analysed in Spanish, and a word marked there is recorded in Spanish;
  - the hint is passed;
  - with one accepted language, no detection is asked.

## 4. Gates

- [x] 4.1 `cargo test -p lingua-core -p lingua-wasm`, the English baseline unchanged, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`. In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`, `yarn build`, `yarn check:variants`.
- [x] 4.2 `openspec validate add-lingua-language-routing --strict` passes. In `docs/lingua/spanish-programme.md`, change 12 is marked done.
