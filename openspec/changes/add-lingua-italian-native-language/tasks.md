# Tasks

Implemented after changes 5, 15 and 20.

## 1. The core

- [ ] 1.1 `crates/lingua-core/src/knowledge/profile.rs`: `NativeLanguage::Italian` after Spanish, `ALL`, `tag` `it`; its round-trip test's unknown tags without `it` (`pt` is already there); the tag pin with `it`; `decks/backup.rs`'s name pin with `Italian`; a test that an Italian native's backup is version 2 studying English or Spanish (D1).
- [ ] 1.2 Change 5's tests rewritten: Italian answers `studied()` as Italian; an `it` pack glossed in `it` refused as its own studied language (D1).
- [ ] 1.3 `crates/lingua-pack/src/lib.rs`: `spec_scenario_a_native_language_the_core_does_not_know_is_refused_at_build` with `pt` for `it`; `apps/lingua-agent/rust/src/vocab.rs` `native_name`: `Italian => "italien"`.

## 2. The extension

- [ ] 2.1 `src/analyzer/types.ts`: `NativeLanguage` gains `"it"`; `src/i18n/language.ts`: `InterfaceLanguage = Exclude<NativeLanguage, "it">`, `INTERFACE_LANGUAGES` unchanged; one `interfaceOf(native)` for the store's two typed mirrors and `rememberInterfaceLanguage`, mapping `it` to English, a test holding it unreachable while no pair is glossed in Italian (D2).
- [ ] 2.2 `NATIVE_LANGUAGES` (fr, en, es, it) read by `isNativeLanguage`, `recordedNative`, `reasonOf` and the Safari app's day counts; `state/profile.ts` `NATIVE_NAMES` with `Italian`; `ownNames.italian` « Italiano » in `src/i18n/{fr,en,es}/languages.ts` and `language-labels.ts`; « Italiano » in `test/i18n.spec.ts`'s `SAME_EVERYWHERE` (D3).
- [ ] 2.3 Tests: the offer and the preset with no pair glossed in Italian (French, English, Spanish; an Italian browser preset to English); the native-language message with `it` refused `no-pair`; `reasonOf` and `recordedNative` accepting `it`, `recordedNative("it")` reading `fr` with no pair; `nativeLanguageName(fr|en|es, "it")` « Italiano »; an `it` day label from the app kept; a backup naming an Italian native restored and kept; `profile.spec.ts` reading `Italian`; the casts of `apple-activation-page.spec.ts`; `yarn format:check`; `yarn typecheck`, `yarn lint`, `yarn test`, `yarn build`, `yarn check:variants`.

## 3. Checks

- [ ] 3.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` — every golden and `backup_format.rs` unchanged; `yarn gen:wasm` and `wasm-pack test --node crates/lingua-wasm`.
- [ ] 3.2 `openspec validate add-lingua-italian-native-language --strict`; the archive order; `docs/lingua/italian-programme.md` row 32's status.

## 4. Release (owner)

- [ ] 4.1 [manual] An extension release and a host-app release carrying this change (change 40's at the latest) live on the Chrome Web Store, addons.mozilla.org and the App Store before change 45's release (D4).
