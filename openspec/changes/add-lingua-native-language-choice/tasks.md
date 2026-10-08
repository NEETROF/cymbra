# Tasks

## 1. The engine and the ports (crates/lingua-wasm, apps/lingua-extension)

- [x] 1.1 `crates/lingua-wasm`: `reprofileBackup(backup, native, studied)`, pure, validating as `Profile::set` does, written at the version `LinguaState::to_backup` writes (1 for the default profile with English-only records, 2 otherwise) (D2); tests: a backup reprofiled restores on an engine of the new native; a refused choice returns an error.
- [x] 1.2 `src/analyzer/pairs.ts` `shippedNatives()` (D1); tests: one native today, two with a test pair list.
- [x] 1.3 `src/analyzer/engine.ts` `WasmAnalyzerPort.restore` rebuilds the engine when the backup's native differs from the engine's (D3); test: a restore under another native rebuilds and keeps the state.
- [x] 1.4 `src/background.ts` serves `lingua-native-language`: refusal, the studied-languages rule (a pure function, tested), `reprofileBackup`, the backup saved and the interface-language key written from the new profile before the announcement, the store change announced with its reason, the two ports' memoised engines and the `hydrated` memo cleared (D2); `onInstalled` sets `cymbra-lingua-native-chosen` on `update` (D4).

## 2. The choice (apps/lingua-extension)

- [x] 2.1 `src/reading/native-language-view.ts`: the natives of `shippedNatives()` in their own names, the current one selected, the consequence stated, hidden under two, the message sent and the host's port rebuilt (D4); its copy in the catalogue (D6). Tests: *Every reader today*, *A native language that was the only studied one*, *A native language with no pair*.
- [x] 2.2 Réglages « Langue »: the block above « Langues étudiées », its title from the catalogue; the onboarding's first question with the preset applied before painting on a new install; the popup's first-run call to action when the marker is unset (D4). Tests: *A new install when two native languages ship*, *A browser in another language*, *An installed extension is not asked*, *Safari without the onboarding*.
- [x] 2.3 On a native change: every extension page reloads itself on the announced change, and `content.ts` tears the `ReadingSession` down and builds a new one, reading the key first (D3). Tests: the content script's rebuild on a fake store change; a page's reload hook.

## 3. What follows (apps/lingua-extension)

- [x] 3.1 Sync `widen`, the translation controller's reconcile, the review's language and the level prompts follow the change (D5); a full reset keeps the native (*A full reset keeps the native language*). Test: *Changing the native language* on the measured modules (accepted languages, pairs, review language) with a two-native pair list.

## 4. Gates and docs

- [x] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-wasm`; in `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; the choice hidden in every built target (`SHIPPED_PAIRS` French-native only).
- [x] 4.2 `openspec validate add-lingua-native-language-choice --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-choice` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 20 is marked done in `docs/lingua/language-matrix-programme.md`.
