## Why

`piano-sound-output` gives Cymbra one fixed, bundled piano sound. But learners
practice on the instrument they own — a bright upright, a mellow grand, an
electric piano — and a single timbre will feel "wrong" against their own
instrument. Letting the user pick the piano sound (and remember the choice) makes
practice feel faithful to their home setup and is a natural, frequently-requested
extension once audio exists.

## What Changes

- Ship **multiple piano SoundFonts** with **clean, documented licenses and
  provenance** — a small CC0 default bundled, larger CC-BY grands available on
  demand — each a `.sf2`, selectable at runtime. (v1 picks: **Upright Piano KW**
  CC0 bundled default; **YDP Grand** / **Salamander Grand** CC-BY as
  download-on-first-use. Each file is **vendored / self-hosted** — copied once
  under our control, no runtime dependency on the original source; attributions
  recorded for the CC-BY fonts.)
- **Let the user add their own `.sf2`**: import a SoundFont from the device into
  the app, where it joins the catalog as a selectable piano and can be removed.
  Imported files stay on the user's device (we do not redistribute them), which
  also lets a user load a SoundFont of *their own* instrument without any
  licensing concern for the app.
- Add a **persisted user setting** for the selected piano, restored on launch
  (same persistence the app already uses for other player settings).
- **Swap the synthesizer's SoundFont at runtime**: extend the audio FFI so the
  engine can load a different `.sf2` without recreating the audio stream, and
  apply `all-notes-off` across the swap so no voice hangs.
- Add a **piano picker to the settings drawer** (the existing right end-drawer),
  showing the available pianos and the current selection.
- **Optionally** support **download-on-first-use** for larger/realistic
  SoundFonts so the base app bundle stays small (behind the same injectable seam;
  fully degradable — a failed download falls back to the bundled default).
- Keep everything behind the existing injectable `AudioService` seam so the
  picker, persistence and swap are testable with a fake and **degrade gracefully**
  (selection still persists and the UI still works even if audio is unavailable).

## Capabilities

### New Capabilities
- `piano-sound-selection`: choose among multiple piano SoundFonts (bundled,
  download-on-first-use, **and user-imported**), persist the choice across
  launches, and surface the selection in the settings drawer — with graceful
  fallback to a bundled default.

### Modified Capabilities
- `audio-output`: the SoundFont-synthesis requirement is extended so the synth can
  be **(re)initialized with a chosen SoundFont at runtime** (swap the active
  `.sf2`, issuing all-notes-off across the swap), rather than a single fixed
  bundled SoundFont.

## Impact

- **Depends on `piano-sound-output`** landing first (this builds directly on its
  `AudioService` seam, the `audio.rs`/`audio_core.rs` modules and the bundled-asset
  mechanism). Sequence after it.
- **Rust engine** (public API → re-run `flutter_rust_bridge_codegen`):
  - `api/audio.rs` — add a `set_soundfont(sf2_bytes)` (or `audio_load_soundfont`)
    FFI that rebuilds the `rustysynth` synthesizer from new bytes on the audio
    thread, draining/silencing active voices first (coverage-excluded glue).
  - `api/audio_core.rs` — pure logic for the swap (voice clearing, validating the
    selection), host-tested.
- **Flutter**:
  - `services/audio_service.dart` — add `selectPiano(...)` / `loadSoundFont(...)`
    to the `AudioService` seam + `FrbAudioService` impl + fake.
  - new `state/` provider — a `@riverpod` notifier holding the selected piano,
    backed by persisted storage, restored at startup; drives `audioService`.
  - new asset-catalog source listing the available pianos (id, label, asset path /
    download URL / **imported file path**, license/attribution) behind an
    injectable provider.
  - new **import flow**: a file picker (`file_picker`) to choose a `.sf2`,
    validation that it is a loadable SoundFont, a copy into app storage, and a
    persisted registry of imported pianos (with remove).
  - settings drawer widget — add a piano picker row plus an "Add SoundFont…"
    action (compose with `player-settings-drawer`; use a list/radio layout, not a
    flyout dropdown, per the iPad-flicker note).
- **Assets / `pubspec.yaml`**: register the bundled CC0 `.sf2`; record each
  font's source + license + attribution (CC-BY grands credited in an in-app
  acknowledgements/licenses entry). Bundle-size trade-off noted in design
  (download-on-first-use is the mitigation; user imports add nothing to the bundle).
- **CI/coverage**: new `audio.rs` FFI stays in the Rust coverage ignore regex;
  Flutter tests use the fake `AudioService` and a fake catalog/persistence so no
  native lib or real download is touched.
- **Interactions**: composes with `playable-onscreen-keyboard` and `wait-mode`
  (the newly chosen timbre simply sounds for every existing trigger); no change to
  score derivation or visuals.
