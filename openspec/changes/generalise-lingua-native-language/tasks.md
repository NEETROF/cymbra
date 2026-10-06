# Tasks

## 1. Native languages, packs and the profile (lingua-core)

- [ ] 1.1 `NativeLanguage` keeps French, English and Spanish under their names and in their order. It gains `ALL`, `from_tag` and the mapping to `StudiedLanguage` by tag (D2). Host tests:
  - `from_tag` round-trips every variant;
  - `de`, `it` and `FR` are unknown;
  - English and Spanish map to their studied language, and French maps to none.
- [ ] 1.2 `Pack` reads its native language at load (D1), with `PackError::UnknownNative` and `PackError::NativeStudied`, and gains `native()`, `pair()` and `Pack::pair_in(bytes)`. `LanguagePair::key` and `PackMeta::pair_key` read `<studied>-<native>`. `Profile::pair_for` and `pairs`, now unused, go. Host tests:
  - the en-fr sample reports French and `en-fr`;
  - `pair_in` reads a bare container's metadata;
  - `de` and `en`-on-English are refused with no analysis;
  - the existing errors keep their precedence;
  - the `pair_key` assertions in `lingua-core` and `lingua-pack` read `en-fr`.
- [ ] 1.3 `PackSet` takes its native language from its first pack and refuses another with `OtherNative` (D3). Host tests:
  - `native()` is the first pack's;
  - a pack of another native language is refused before `AlreadyHeld`, and the set is unchanged;
  - a second language of the same native language is still added;
  - the existing messages are unchanged.
- [ ] 1.4 `Profile` refuses the native language among the studied ones, and gains `set(native, studied)` (D4). Host tests:
  - `set_studied_languages` refuses a list holding the native language;
  - `set` refuses a native language that would be studied, and keeps the profile whole;
  - the existing Display strings are unchanged;
  - `english_for_french` and `is_default` are unchanged.
- [ ] 1.5 `backup.rs` tests:
  - a profile studying Spanish with English native is written as version 2 and round-trips;
  - a version 2 backup of the previous build restores with French native;
  - the serde names the extension reads are pinned (`"native_language": "English"`), beside the test that pins `studied_languages`;
  - `tests/backup_format.rs` passes without re-blessing.
- [ ] 1.6 `ENGLISH_TYPICAL_VOCABULARY` in `knowledge/vocabulary.rs`, with its provenance (D6), and a test that pins its six values.

## 2. The engine (lingua-wasm)

- [ ] 2.1 `new` and `reset` set the profile to the pack set's native language, studying its default language (D5). Host tests:
  - an en-fr engine's profile is the default, and its reset backup is version 1;
  - an engine whose first pack studies Spanish glossed in English (built in the test, metadata rewritten) keeps English native and studies Spanish alone after a reset.
- [ ] 2.2 Bindings `nativeLanguage()`, `profileNativeLanguage()` and `setProfile(native, languages)` (D4). Tests:
  - on host: an en-fr engine answers `fr` twice, and `setProfile("fr", ["es","en"])` round-trips through the backup;
  - with `wasm-bindgen`, these are refused and leave the profile and the packs as they were:
    - `addPack` of a pack glossed in another language;
    - `setProfile` with a native language no held pack serves;
    - `setStudiedLanguages` naming the native language.
- [ ] 2.3 `levelLadder` uses `ENGLISH_TYPICAL_VOCABULARY` for an estimated pack of another language, with `typicalFrom: "en"`, and English's own ladder keeps its live figures (D6). `languages.rs` tests:
  - the Spanish rows equal the constant, borrowed;
  - an engine with no English pack gives the same figures;
  - the English rows keep the testdata pack's own figures.
- [ ] 2.4 `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline` passes without `LINGUA_BLESS`, and `git diff origin/main -- crates/lingua-wasm/tests/baseline` is empty.

## 3. The pack builder (lingua-pack)

- [ ] 3.1 `build_pack` refuses an unknown native language and a native language equal to the studied one, with a `BuildError` for each, as it refuses an unknown studied language (D1). Tests for both, and the en-fr and es-fr pins are unchanged.

## 4. The extension

- [ ] 4.1 `pairs.ts` gains `DEFAULT_NATIVE`, `nativeOf`, `pairsOf`, `defaultPair` and `pairFor(language, native)`. `readingLanguage` and `acceptedLanguages` read the port's native language (D7). `test/pairs.spec.ts`:
  - the choice is made by both sides;
  - es-en is never chosen for a French reader;
  - es-en is the default for an English reader;
  - with today's list, every result is unchanged.
- [ ] 4.2 `nativeLanguageOf(backup)` in `state/profile.ts` and `storedNativeLanguage(area)` in `state/storage.ts`. Specs cover:
  - French for no profile, for unknown or unparseable input, and for a native language no listed pair serves;
  - `en` and `es` read from their names.
- [ ] 4.3 `LinguaPort.nativeLanguage()` (whole-reader, forwarded by the RPC) and `WasmAnalyzerPort`'s native-language resolver, wired in `create-port.ts` and `background.ts`. `WasmEngine` declares the new bindings, after `yarn gen:wasm`. `engine.spec.ts`:
  - a French resolver over en-fr, es-fr and es-en fetches en-fr, then es-fr for Spanish, and never es-en;
  - the resolver is called once;
  - `nativeLanguage()` fetches no pack;
  - a native language no pair serves fails before any fetch;
  - the French refusal message is unchanged.
- [ ] 4.4 « Langues étudiées » offers the studied languages of the reader's native language's pairs. The comments about the reset in `sync.ts` and `settings-view.ts` describe D5. Specs:
  - `studied-languages-view.spec.ts` with a mixed list for a French and an English reader;
  - `sync.spec.ts` keeps calibrating English after an erasure.
- [ ] 4.5 `packMeta` reads the container's metadata exactly. `assertPacksMatchEngine` refuses a pack whose native language is not its pair's, and `shippedPairs` refuses `fr-fr` (D8). `test/packs.spec.ts` covers both, and both shipped pairs pass.

## 5. The agent (lingua-agent)

- [ ] 5.1 `Library` reads files with `Pack::pair_in`, anchors on `pack.lingua`'s native language, skips packs of another, and exposes the skipped files for `lingua vocab` (D9). `tests/pipeline.rs`:
  - `es-en.lingua` beside `pack.lingua` and `es-fr.lingua` is skipped and named, with English and Spanish still followed;
  - `$LINGUA_PACK` alone is followed;
  - today's installs show no notice.

## 6. Gates and docs

- [ ] 6.1 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check`, then `yarn build` (three variants) and `yarn check:variants`. At the root:
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo llvm-cov` at or above 80 %;
  - the agent's tests.
- [ ] 6.2 `openspec validate generalise-lingua-native-language --strict` passes, and `python3 scripts/openspec_archive_order.py generalise-lingua-native-language` exits 0. In `docs/lingua/language-matrix-programme.md`, change 4 is marked done.
