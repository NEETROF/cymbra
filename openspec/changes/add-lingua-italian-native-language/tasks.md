# Tasks

Implemented after changes 5, 15 and 20.

## 1. The core

- [ ] 1.1 `crates/lingua-core/src/knowledge/profile.rs`: `NativeLanguage::Italian` after Spanish, `ALL`, `tag` `it`; its round-trip test's unknown tags without `it` (`pt` instead); the tag pin with `it`; `decks/backup.rs`'s name pin with `Italian`; a test that an Italian native's backup is version 2 studying English or Spanish (D1).
- [ ] 1.2 Change 5's tests rewritten: Italian answers `studied()` as Italian; an `it` pack glossed in `it` refused as its own studied language (D1).
- [ ] 1.3 `crates/lingua-pack/src/lib.rs`: `spec_scenario_a_native_language_the_core_does_not_know_is_refused_at_build` without `it`; `apps/lingua-agent/rust/src/vocab.rs` `native_name`: `Italian => "italien"`.

## 2. The extension

- [ ] 2.1 `src/analyzer/types.ts`: `NativeLanguage` gains `"it"`; `src/i18n/language.ts`: `InterfaceLanguage = Exclude<NativeLanguage, "it">`, `INTERFACE_LANGUAGES` unchanged; the store's mirror of the native as the interface language maps `it` to English, a test holding it unreachable while no pair is glossed in Italian (D2).
- [ ] 2.2 `NATIVE_LANGUAGES` (fr, en, es, it) read by `isNativeLanguage`, `recordedNative`, `reasonOf` and the Safari app's day counts; `state/profile.ts` `NATIVE_NAMES` with `Italian`; `ownNames.italian` « Italiano » in `src/i18n/{fr,en,es}/languages.ts` and `language-labels.ts`; « Italiano » in `test/i18n.spec.ts`'s `SAME_EVERYWHERE` (D3).
- [ ] 2.3 Tests: the offer and the preset with no pair glossed in Italian (French, English, Spanish; an Italian browser preset to English); an `it` day label kept; `profile.spec.ts` reading `Italian`; the casts of `apple-activation-page.spec.ts`; `yarn typecheck`, `yarn lint`, `yarn test`, `yarn build`, `yarn check:variants`.

## 3. Checks

- [ ] 3.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` — every golden and `backup_format.rs` unchanged; `yarn gen:wasm` and `wasm-pack test --node crates/lingua-wasm`.
- [ ] 3.2 `openspec validate add-lingua-italian-native-language --strict`; the archive order; `docs/lingua/italian-programme.md` row 32's status.

## 4. Release (owner)

- [ ] 4.1 [manual] Released with the extension's next release, live on every store before change 45's release (D4).
