# Tasks

## 1. Pairs and ports (apps/lingua-extension)

- [ ] 1.1 `src/analyzer/pairs.ts` `shippedNatives()` (D1); tests: one native today, two with a test pair list.
- [ ] 1.2 `src/analyzer/engine.ts` `WasmAnalyzerPort.restore` rebuilds the engine when the backup's native differs from the engine's (D3); `port.ts`, `messaging-port.ts` `setNativeLanguage(native)`; tests: a restore under another native rebuilds and keeps the state.
- [ ] 1.3 `src/background.ts` serves `setNativeLanguage`: refusal, the studied languages' rule, the fresh engine, the backup saved, the ports replaced, the interface-language key written, the store change announced (D2). Tests on the pieces that are measured (the studied-languages rule as a pure function; the key writer).

## 2. The choice (apps/lingua-extension)

- [ ] 2.1 `src/reading/native-language-view.ts`: the natives of `shippedNatives()` in their own names, the current one selected, the studied-languages consequence stated, hidden under two (D4); its French, English and Spanish copy in the catalogue (D6). Tests: *Every reader today*, *A native language that was the only studied one*, *A native language with no pair*.
- [ ] 2.2 Réglages « Langue »: `settingBlock("Langue maternelle")` above « Langues étudiées » (`lint-settings-hosts` follows); the onboarding's first question, preset from the browser's language (D4); the popup's first-run call to action when the native was never chosen (`cymbra-lingua-native-chosen`). Tests: *A new install when two native languages ship*, *A browser in another language*, *Safari without the onboarding*.

## 3. What follows (apps/lingua-extension)

- [ ] 3.1 Sync `widen`, the translation controller's reconcile, the review's language and the level prompts follow the change (D5); a full reset keeps the native (*A full reset keeps the native language*). Test: *Changing the native language* on the measured modules (accepted languages, pairs, review language) with a two-native pair list.

## 4. Gates and docs

- [ ] 4.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; the choice hidden in every built target (`SHIPPED_PAIRS` French-native only).
- [ ] 4.2 `openspec validate add-lingua-native-language-choice --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-choice` exits 0; change 20 is marked done in `docs/lingua/language-matrix-programme.md`.
