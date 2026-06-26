## Context

`piano-sound-output` adds a polyphonic SoundFont synthesizer to the Rust engine:
hardware/thread glue in `api/audio.rs` (coverage-excluded) over pure logic in
`api/audio_core.rs`, exposed to Flutter through an injectable `AudioService`
(`audioServiceProvider`) with a `FrbAudioService` production impl and a fake for
tests — mirroring the MIDI pattern. It bundles a **single** piano `.sf2` and calls
`audio_init(sf2_bytes)` once at startup.

This change makes the piano **chooseable**: ship a few SoundFonts, let the user
pick one (persisted), and swap the synth's active SoundFont at runtime so the new
timbre sounds for every existing trigger (on-screen, computer keyboard, MIDI, and
score playback). It depends on `piano-sound-output` having landed.

The app already has a settings drawer (a right end-drawer; dropdown menus flicker
on iPad — see `player-settings-drawer`) and persists player settings, so the
picker and its persistence follow those established patterns.

## Goals / Non-Goals

**Goals:**
- A small curated set of cleanly-licensed piano SoundFonts, selectable at runtime.
- **Let the user import their own `.sf2`** into the catalog (and remove it).
- Persist the selected piano and restore it on launch.
- Runtime SoundFont swap with no hanging voices and no audio-stream teardown.
- A picker in the settings drawer, consistent with existing settings UI.
- Everything behind the injectable seam; testable with fakes; graceful fallback.

**Non-Goals:**
- No per-note effects, EQ, reverb, or instrument families beyond piano-like `.sf2`s
  (still a future enhancement).
- No editing/repacking of SoundFonts in-app — import accepts a `.sf2` as-is.
- **No SFZ format** in v1. `rustysynth` is SF2-only; SFZ is a text manifest plus a
  separate sample folder and has no mature pure-Rust engine (reference is `sfizz`,
  C++), which breaks both the "pure Rust, no C build" decision and the single-file
  import flow. All chosen pianos exist as SF2. (SF3 — compressed SF2 — may be a
  cheap win to verify; SFZ is a future enhancement requiring a second engine.)
- No cloud sync of imported SoundFonts across devices (local to the device).
- No master-volume/mute control (tracked as an open question in `piano-sound-output`).
- No change to score derivation, visuals, Wait Mode, or hand filtering.

### Chosen SoundFonts (v1)
Sourced once from the **FreePats** project (documented provenance) but **vendored
into our own repo/infra — no runtime dependency on FreePats**:
- **Upright Piano KW** — **CC0** (public domain, no attribution), ~27 MiB →
  **bundled default** (vendored by `piano-sound-output`; always present, no network).
- **YDP Grand Piano** — **CC-BY 3.0** (credit Roberto / Zenph Studios), ~36 MiB →
  download-on-first-use from our **self-hosted** copy.
- **Salamander Grand Piano** — **CC-BY 3.0** (credit Alexander Holm), ~296 MiB →
  download-on-first-use from our **self-hosted** copy.

The licenses (CC0 / CC-BY) explicitly permit redistribution, so we copy each file
once and host it ourselves; the original source is just provenance, not a
dependency. CC-BY fonts require visible attribution → add an in-app
acknowledgements/licenses entry. (Deliberately **not** using GeneralUser GS: its
own license warns sample provenance is uncertain for commercial software
redistribution — the same concern that rules out the producersbuzz pack.)

## Decisions

### Decision: Runtime SoundFont swap via a new FFI, not stream teardown
Add `audio_load_soundfont(sf2_bytes)` to the audio FFI. It pushes a "replace
synth" event onto the existing lock-free queue; the audio callback issues
all-notes-off, drops the old `rustysynth` synthesizer, and installs one built from
the new bytes — **without** stopping/recreating the `cpal` stream. **Why:** keeps
one audio path alive (no device re-acquisition glitch), reuses the established
queue hand-off, and confines real-time concerns to the callback. **Alternatives:**
tear down and re-`audio_init` (rejected: device re-acquisition latency/clicks,
risk of losing the stream); hold multiple synths loaded and switch a pointer
(rejected for v1: N× memory for little benefit — swaps are rare/manual).

### Decision: Selection state is a Riverpod notifier over persisted storage
A `@riverpod` notifier holds the selected piano id, reads the persisted value at
startup (defaulting to the bundled default), and on change persists it and calls
`audioService.loadSoundFont(...)`. **Why:** matches the mandated Riverpod 2 +
persisted-settings pattern already used by player settings; the UI just watches
the provider. **Trade-off:** startup must load the *persisted* SoundFont, not
always the default — sequence the load after the catalog/persistence resolve, and
fall back to the default if the stored id is unknown (e.g. an asset was removed).

### Decision: A catalog provider describes available pianos
An injectable provider returns the list of pianos — `{id, label, source, license}`
where `source` is a bundled asset path, a download URL, **or an imported-file
path** (`kind: bundled | download | user`). The catalog is the **union** of a
static built-in list and the persisted imported registry. The selection notifier
and the picker both read it. **Why:** decouples "what pianos exist" from "which is
selected" and from "how bytes are obtained", so tests inject a fake catalog and a
fake byte-loader without touching assets, network, or the filesystem.

### Decision: User import via file picker → validate → copy → persist registry
"Add SoundFont…" opens the OS file picker (`file_picker`, restricted to `.sf2`).
The chosen file is **validated** (it must be a loadable SoundFont — verify the
RIFF/`sfbk` header in pure Dart and/or have the engine attempt a load) **before**
it is accepted; invalid files are rejected with a non-fatal message. Accepted
files are **copied into app storage** (so the catalog never depends on a transient
external URI/permission), and an entry `{id, label, path, kind: user}` is appended
to a persisted **imported registry**; the label defaults to the SoundFont's `INAM`
name or the filename and is user-editable. Imported pianos can be **removed**
(deletes the entry and the copied file; if the removed piano was selected, fall
back to the default). **Why:** copying makes imports durable and self-contained;
validating up front prevents a bad file from breaking the synth at swap time.
**Trade-off:** copies use device storage — acceptable for a handful of pianos;
removal reclaims it. **Licensing:** imported files are the **user's own**, stored
locally and never redistributed by the app — so a user may load a SoundFont of
their actual instrument (or any `.sf2` they hold) without any app-side licensing
concern. Reuse the same `set_soundfont` swap path as bundled/download pianos.

### Decision: Optional download-on-first-use behind the same byte-loader seam
Bytes for a chosen piano come from a `SoundFontSource` seam keyed by `kind`:
bundled assets load from the asset bundle; download sources fetch once, cache to
app storage, and load from cache thereafter; **user** sources load from the copied
imported file. A failed/absent download — or a now-missing imported file — **falls
back to the bundled default** and surfaces a non-fatal message. **Why:** keeps the
base bundle small while staying fully degradable; tests use a fake source returning
fixed bytes.
**Trade-off:** download adds first-use latency and a cache to manage — keep it
optional and out of the critical path (default piano is always bundled).

### Decision: List/radio picker in the settings drawer, not a dropdown
Render the pianos as a selectable list (radio-style) in the existing right
end-drawer. **Why:** the project already learned dropdown menus flicker on iPad
(`player-settings-drawer`); a list avoids that and reads better for a handful of
named timbres.

### Decision: Graceful degradation keeps selection working without audio
If audio is unavailable (init failed in `piano-sound-output`), the picker and
persistence still function — the choice is stored and will apply once audio works;
`loadSoundFont` on a no-op audio service simply does nothing. **Why:** the seam's
degrade-don't-crash contract; the user's preference must never be lost just because
a device lacks audio.

## Risks / Trade-offs

- **Voice continuity across a swap** → if a key is held while the user switches
  pianos, the old voice must not hang. Mitigation: all-notes-off as the first step
  of the swap event; test the held-note-during-swap path.
- **Bundle size from multiple `.sf2`s** → several piano SoundFonts inflate the app.
  Mitigation: keep bundled ones small/piano-only; push larger/realistic ones to
  download-on-first-use; document each size + license.
- **Unknown persisted id after an update** → a stored piano id may no longer exist
  if assets change. Mitigation: validate against the catalog at load; fall back to
  the default and re-persist.
- **Download failures / offline** → a download source may be unreachable.
  Mitigation: always fall back to the bundled default; cache after first success;
  never block playback on a download.
- **Invalid / huge imported file** → a user may pick a non-SoundFont or a corrupt/
  enormous `.sf2`. Mitigation: validate the `sfbk` header before accepting; reject
  invalid files non-fatally; never let a bad import become the active synth without
  a successful load; consider a size warning.
- **Missing imported file at launch** → an imported `.sf2` may be deleted by the OS
  or the user out-of-band. Mitigation: on load failure, fall back to the default and
  mark/clean the stale registry entry.
- **Real-time safety of the swap** → rebuilding the synth in the callback must not
  block audio for long. Mitigation: build the new synth off the callback if needed
  (prepare bytes/synth on the FFI side, hand a ready synth over the queue), keeping
  the callback to a pointer swap + all-notes-off.
- **Public API change** → re-run `flutter_rust_bridge_codegen`; the new FFI stays
  in the coverage ignore regex; pure swap/validation logic lives in `audio_core.rs`.

## Migration Plan

Additive on top of `piano-sound-output`: new FFI method, new providers
(catalog + selection + byte-source), new drawer row, extra bundled assets, plus
one reworded `audio-output` requirement. Default behavior is unchanged for a fresh
user (the bundled default plays); a user who picks a piano gets it persisted.
Public Rust API change → run `flutter_rust_bridge_codegen generate`. Rollback =
remove the swap FFI and providers, drop the extra assets, restore the single-
SoundFont `audio-output` wording, and the app reverts to one fixed piano.

This change MODIFIES the same `audio-output` capability that `piano-sound-output`
ADDS; it must be sequenced and archived **after** `piano-sound-output` so the base
requirement exists to modify.

## Open Questions

- **Download hosting** — decided: **self-host** our own copies of the CC-BY grands
  (no runtime dependency on FreePats). Remaining detail: which bucket/CDN.
- **Imported-file size cap** — clamp very large `.sf2` imports (e.g. multi-hundred-MB
  banks) or warn only? Decide during implementation.
- **Per-piano gain normalization** — different SoundFonts have different loudness;
  do we normalize? Ties into the deferred master-volume control.
- **SFZ support (future)** — worth it only with a second synth engine and a
  multi-file/folder import. Revisit if users ask; verify SF3 support in `rustysynth`
  first as the cheaper size win.
