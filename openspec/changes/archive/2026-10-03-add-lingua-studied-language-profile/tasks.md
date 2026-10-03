## 1. The list and the backup version (lingua-core)

- [x] 1.1 `LinguaState` gains `profile: Profile`, with `set_studied_languages` and the defaults of D1. `KnowledgeState`, `ExposureCounters` and `Deck` expose the languages they hold.
- [x] 1.2 `to_backup` computes the version (D2) and `from_backup` reads it first (D3), with `UnsupportedVersion` naming what it found. Host tests cover:
  - an English state written as version 1 with no profile, equal to the output of the previous build;
  - version 2 for a non-default profile, and for Spanish records under the default profile;
  - round trips of both versions;
  - an empty or duplicated list refused, the previous one kept;
  - `reset` restoring the default profile and `resetStatuses` keeping it;
  - version 3 refused as unsupported, and a file without a version as malformed.

## 2. The engine (lingua-wasm)

- [x] 2.1 `studiedLanguages()` and `setStudiedLanguages(tags)` (D5). Native tests cover the round trip through `backup`/`restore`. `wasm-bindgen` tests cover the refusals: an unknown tag, an empty list and a duplicate.
- [x] 2.2 `cargo test -p lingua-wasm --test english_baseline` passes without re-blessing the golden.

## 3. The extension

- [x] 3.1 `LinguaPort` gains `studiedLanguages()` and `setStudiedLanguages()`, in the engine mirror, `WasmAnalyzerPort`, `MessagingLinguaPort` and the fake port. Specs cover:
  - the RPC forwards both as whole-reader calls;
  - neither loads a pack.
- [x] 3.2 `readingLanguage(port, pairs)` in `src/analyzer/pairs.ts` (D6). Its spec covers: the first shipped language of the list, the default pair's when none is shipped, and the list left unchanged.
- [x] 3.3 Every surface reads in `readingLanguage`, and `STUDIED_LANGUAGE` is removed (D6). The speaker takes its language as a getter, and `hydrateFromV1` migrates in English. Specs (`test/reading-language.spec.ts`), on a fake port whose list is Spanish then English:
  - with en-fr alone, the session, settings, statistics and review ask in English, and with en-fr and es-fr they ask in Spanish;
  - the list stays as it was;
  - an open session switches language after an external restore changes the list;
  - the sync's post-erasure calibration asks in English, the reset profile's language, even with es-fr shipped.

  Onboarding is DOM wiring outside the coverage gate; its two calls take the same rule.

## 4. Gates and docs

- [x] 4.1 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check`, then `yarn build` (three variants) and `yarn check:variants`. At the root:
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo llvm-cov` at or above 80 %.
- [x] 4.2 `openspec validate add-lingua-studied-language-profile --strict` passes. In `docs/lingua/spanish-programme.md`, change 10 is marked done, and the R2 dogfood pass is noted as next.
