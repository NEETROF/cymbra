## 1. Port shapes

- [ ] 1.1 `src/analyzer/types.ts`: `StudiedLanguage = "en" | "es"`, and `STUDIED_LANGUAGE: StudiedLanguage = "en"`. `src/analyzer/port.ts`: `LanguagePort extends AnalyzerPort` (the 21 language-bound calls, plus `language`), and `LinguaPort` (the whole-reader calls, plus `for(language)` and `languages()`). `yarn typecheck` then lists every call site to move. That list is the work of group 3.

## 2. Implementations

- [ ] 2.1 `src/analyzer/engine.ts`: `WasmEngine` mirrors the engine (`language?: string | null` on bound methods, plus `addPack` and `languages`). `WasmAnalyzerPort.for(language)` returns a view that passes `language` to each engine call. `languages()` parses the engine's JSON. Spec with a fake wasm module: each bound call reaches the engine with its language, and root calls with none.
- [ ] 2.2 `src/analyzer/rpc.ts`: `RpcRequest.language?`. `src/analyzer/messaging-port.ts`: `MessagingLinguaPort.for(language)` returns a view whose every call sends `language`; root calls send none. `src/analyzer/rpc-host.ts`: answer on `port.for(request.language)` when it is set. Specs: the view sends the language, the root sends none, the host dispatches to the view, and an unknown method is reported on both targets.
- [ ] 2.3 `test/helpers.ts`: `makeFakePort` gains `for(language)`, which returns the fake extended with `language` and records it in `calls.languages`; local fakes in specs follow. `yarn test` compiles.

## 3. Surfaces

- [ ] 3.1 Reading: `reading/session.ts`, `reading/selection-card.ts` (its `ports` bundle takes a view), `reading/scan.ts` (takes an `AnalyzerPort` view). Each goes through `port.for(STUDIED_LANGUAGE)`, and the session specs assert the language asked is `"en"`.
- [ ] 3.2 Réglages and onboarding: `reading/settings-view.ts` (`mountSettings`, for every host) and `onboarding/onboarding.ts`. Their specs assert `"en"`.
- [ ] 3.3 Stats, review, state and sync: `stats/view.ts`, `review/review-page.ts` (attributions), `state/storage.ts` (v1 migration), `state/level-choice.ts`, `sync/sync.ts` (calibration after sign-in). Their specs assert `"en"`. `yarn typecheck` is clean, with no language-bound call left on a root port.

## 4. English invariance

- [ ] 4.1 `crates/lingua-wasm/tests/english_baseline.rs` renders every probe with no language and with `"en"`, and both equal `tests/baseline/en-fr.golden`. `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline` is empty, and `cargo test -p lingua-wasm --test english_baseline` passes.
- [ ] 4.2 In `apps/lingua-extension`, all pass: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check` and `yarn build` (the three variants). The Firefox and Safari bundles carry the RPC `language` field (grep of `dist-*/`).

## 5. Gates

- [ ] 5.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-wasm`.
- [ ] 5.2 `openspec validate generalise-lingua-extension-port --strict`; `python3 scripts/check_ci_units.py --list` is unchanged; in `docs/lingua/spanish-programme.md`, change 7 is marked done and change 8 next.
