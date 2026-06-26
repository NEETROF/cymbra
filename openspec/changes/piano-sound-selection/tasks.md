## 1. Assets & catalog

- [ ] 1.1 Reference the already-bundled **Upright Piano KW** (CC0, vendored by `piano-sound-output`) as the catalog's default piano — no re-bundling.
- [ ] 1.2 Register **YDP Grand** and **Salamander Grand** (CC-BY 3.0) as download-on-first-use entries pointing at our **self-hosted** copies (no runtime dependency on the original source); record attribution (Roberto/Zenph, Alexander Holm) and add an in-app acknowledgements/licenses entry crediting the CC-BY fonts.

## 2. Rust engine: runtime SoundFont swap

- [ ] 2.1 `api/audio_core.rs`: add pure logic for the swap (a "replace synth" event, voice-clearing on swap, validating an incoming selection), host-tested.
- [ ] 2.2 `api/audio.rs` (coverage-excluded glue): add `audio_load_soundfont(sf2_bytes)` FFI — push a replace-synth event onto the existing queue; the callback issues all-notes-off and installs a synth built from the new bytes, **without** recreating the `cpal` stream. Prepare/build the new synth off the real-time callback where needed.
- [ ] 2.3 Register the new FFI in `api/mod.rs`; run `flutter_rust_bridge_codegen generate` (public API changed) and ensure the bridge builds.
- [ ] 2.4 Unit-test `audio_core.rs` swap logic: all-notes-off clears voices on swap; an unknown/invalid selection is rejected/falls back.

## 3. Flutter seam & sources

- [ ] 3.1 Extend `services/audio_service.dart`: add `loadSoundFont(bytes)` (or `selectPiano(id)`) to the abstract `AudioService`, the `FrbAudioService` impl forwarding to the bridge, and the fake.
- [ ] 3.2 Add an injectable `SoundFontSource` seam: bundled assets load from the asset bundle; download sources fetch-once + cache + load-from-cache; a fake returns fixed bytes in tests.
- [ ] 3.3 Add an injectable **catalog provider** returning the union of built-in pianos and the imported registry (`id`, `label`, `kind` bundled/download/user, `source`, `license`); a fake catalog for tests.

## 3b. User import

- [ ] 3b.1 Add an `SoundFontImporter` seam: pick a `.sf2` (`file_picker`, `.sf2` filter), validate the `sfbk`/RIFF header (reject non-SoundFonts non-fatally), copy into app storage, return an entry `{id, label, path, kind: user}`; a fake for tests.
- [ ] 3b.2 Persist an **imported registry** (list of user entries) alongside the selection; default the label to the SoundFont `INAM`/filename (user-editable).
- [ ] 3b.3 Support **remove**: delete the entry + copied file; if the removed piano was selected, fall back to the default and re-persist. Extend `SoundFontSource` with the `user` kind (load from the copied path; missing file → fall back to default).

## 4. Selection state & persistence

- [ ] 4.1 Add a `@riverpod` notifier holding the selected piano id, backed by persisted storage (same mechanism as existing player settings); default to the bundled default.
- [ ] 4.2 On startup, restore the persisted selection; validate it against the catalog and fall back to the default (re-persisting) if unknown; load its SoundFont via the source + `audioService`.
- [ ] 4.3 On selection change, persist the new id and call `audioService.loadSoundFont(...)`; fall back to the default piano (non-fatal) if its bytes cannot be obtained.

## 5. UI

- [ ] 5.1 Add a piano picker row to the settings drawer (compose with `player-settings-drawer`) as a selectable list / radio layout — not a flyout dropdown (iPad flicker).
- [ ] 5.2 Wire the picker to the selection notifier: list catalog pianos (grouped/labeled by kind), mark the active one, update on tap.
- [ ] 5.3 Add an "Add SoundFont…" action that runs the import flow, and a remove affordance on user-imported pianos.

## 6. Tests

- [ ] 6.1 Selecting a piano calls `audioService.loadSoundFont` with the right bytes (fake source) and updates the selection provider.
- [ ] 6.2 Persistence: a stored selection is restored on launch; an unknown stored id falls back to the default and is re-persisted.
- [ ] 6.3 Graceful fallback: a failing `SoundFontSource` (e.g. download error or missing imported file) falls back to the default and does not crash; selecting while audio is unavailable still persists and applies later.
- [ ] 6.4 Import: a valid `.sf2` is accepted, copied, added to the catalog and survives a relaunch (fake importer/persistence); an invalid file is rejected non-fatally and leaves the catalog unchanged; removing a selected imported piano falls back to the default.
- [ ] 6.5 Widget test: the drawer picker lists pianos, marks the active one, changes selection on tap, and exposes Add/Remove for imported pianos.

## 7. Verify & gate

- [ ] 7.1 Keep the new `audio.rs` FFI in the Rust coverage ignore regex; `cargo llvm-cov --workspace --fail-under-lines 80` passes; `cargo fmt`/`clippy` clean.
- [ ] 7.2 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [ ] 7.3 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%; refresh goldens if the drawer layout changed.
- [ ] 7.4 Manually confirm on macOS + one mobile device: switching pianos changes the timbre live for keys and score playback; a held note doesn't hang across a swap; the choice survives a relaunch.
- [ ] 7.5 `openspec validate piano-sound-selection --strict` passes.
