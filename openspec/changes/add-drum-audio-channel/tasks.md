## 1. Shared constants and helpers (`crates/musicxml-core`)

- [ ] 1.1 Export the two channel constants (melodic 0, drum 9) beside
  `DEFAULT_VELOCITY`, documented as the SoundFont/MIDI convention (MIDI channel
  10, index 9 as rustysynth counts channels; presets resolve in bank 128);
  unify the app engine's local `DEFAULT_VELOCITY` copy
  (`apps/music/rust/src/api/audio_core.rs:36`) onto the crate's constant, so
  every synth site imports the playback constants from the one shared
  definition
- [ ] 1.2 Test that pins the values (0 and 9) so a refactor cannot silently move
  them — the constants are wire-like: three synth sites and every stored kit
  font depend on them

## 2. App engine (`apps/music/rust/src/api/`)

- [ ] 2.1 `audio_core.rs`: `AudioEvent::NoteOn`/`NoteOff` gain the channel
  (melodic/drum), built by `note_on`/`note_off` (melodic, unchanged semantics)
  and new `drum_on`/`drum_off` constructors; `VoiceTracker` keys on
  (channel, key) so a release lands on its channel and `AllOff` covers both —
  host-tested like the existing bookkeeping
- [ ] 2.2 `renderer.rs`: route each event to its channel via the shared
  constants; `ReplaceSynth` keeps silencing both channels across the swap
- [ ] 2.3 `audio.rs`: FFI entry points `drum_on(key, velocity)` /
  `drum_off(key)` beside the melodic pair; `note_on`/`note_off` byte-for-byte
  untouched
- [ ] 2.4 `audio.rs`: the SoundFont swap gains an awaitable completion (resolves
  when the incoming font is installed, or when the swap failed and the previous
  font was kept) — the player's percussion-readiness gate depends on it
- [ ] 2.5 Engine helper exposing a local `.sf2`'s family evidence from its
  preset headers (has bank-128 presets / has melodic presets), for the app's
  import detection — pure parse, host-testable
- [ ] 2.6 `flutter_rust_bridge_codegen generate` (public API change)

## 3. Console renderer and back office

- [ ] 3.1 `crates/audio-wasm/src/lib.rs`: delete the local `PIANO_CHANNEL` copy
  and its "matches the app" comment; pick the render channel from the document's
  instrument classification via the shared constants; test that a percussion
  fixture through a kit-shaped font renders non-silence and a keyboard fixture
  renders byte-identically to before
- [ ] 3.2 `yarn gen:wasm` and commit nothing stale — the console must run the
  rebuilt wasm (see the back-office wasm-rebuild gotcha)
- [ ] 3.3 `useSoundFontChoice.ts`: carry each option's family from the listing;
  filter the offered fonts by the **score's** family; default a percussion
  score's choice to the seeded kit; expose an "empty family" state
- [ ] 3.4 `ScoreDetailView.vue`: lift the Play guard — Play for a percussion row
  renders on the drum channel with the picked kit font; leave the notation
  preview untouched (it is `web-notation-render`'s, owned by
  `add-drum-notation-render`)
- [ ] 3.5 Localised "no drum kit available" state (fr/en) when the catalog holds
  no accepted percussion-family font — distinct from error states
- [ ] 3.6 `SoundFontDrawer.vue`: the create drawer offers both families
  (`keyboard` default), localised labels; surface the typed
  family-mismatch refusal as user-facing state, never a raw string

## 4. Backend — family vocabulary and verification

- [ ] 4.1 Migration on `music.soundfonts`: `UPDATE instrument 'piano' →
  'keyboard'`, default `keyboard`, CHECK on (`keyboard`,`percussion`); nothing
  else in the migration
- [ ] 4.2 `backend/server/src/soundfont.rs`: normalise legacy `piano` (and
  empty) declared family to `keyboard` at every upload boundary — admin upload,
  private-library import, seeded scripts — permanently, with tests
- [ ] 4.3 Preset-bank verification helper (pure, host-testable): parse the
  uploaded bytes' preset headers; `percussion` requires ≥1 bank-128 preset,
  `keyboard` ≥1 melodic-bank preset; both-banks fonts pass either declaration
- [ ] 4.4 Wire the verification into admin upload, private-library import sync,
  and the propose path; a mismatch is a typed, localisable refusal — nothing
  stored, no row written; one refused-case test per path
- [ ] 4.5 One-shot ops pass over existing `music.soundfonts` rows: re-read each
  stored object, check its banks against the recorded family, **report**
  mismatches (never rewrite silently)

## 5. Backend — previews on the drum channel

- [ ] 5.1 `soundfont_synth.rs`: the render takes the channel (or family) instead
  of hardcoding `PREVIEW_CHANNEL = 0`; consume the shared constants
- [ ] 5.2 Registry: declare `catalog.preview.drum_soundfont_id` beside
  `catalog.preview.soundfont_id` (empty = percussion previews dormant); extend
  `ScorePreviewConfig` and both config sources (server and worker)
- [ ] 5.3 `score_preview_module.rs`: remove the percussion skip; choose the font
  by the row's instrument — keyboard rows read the existing key unchanged,
  percussion rows read the kit key and render on the drum channel; percussion
  with an unset/unknown/unaccepted/wrong-family kit font returns `Dormant`
  (nothing stored, row unmarked); tests for each outcome, plus one asserting a
  keyboard render is byte-identical to before
- [ ] 5.4 Confirm the acceptance path enqueues percussion renders again (the
  skip covered acceptance, backfill and regenerate through this one module —
  lifting it here lifts all three) and that the existing backfill picks up the
  formerly skipped, unmarked percussion rows with no new code
- [ ] 5.5 `soundfont_preview.rs`: a second fixed sample sequence — a short
  groove of General MIDI numbers on the drum channel — selected by the font's
  family; deterministic, host-tested; keyboard fonts keep the existing phrase

## 6. Bundled kit and seeding

- [ ] 6.1 Prepare the candidate kit `.sf2` (design: FluidR3 GM's bank-128
  presets extracted to a standalone SF2, MIT) — do NOT commit the asset before
  the licence sign-off in 9.1
- [ ] 6.2 Add the asset under `assets/soundfonts/` with its licence text file;
  record source, author and licence in `CREDITS.md`; register in `pubspec.yaml`
- [ ] 6.3 App catalog: a bundled-kit entry beside the bundled piano (stable id,
  family `percussion`), the percussion family's default and fallback
- [ ] 6.4 Seed the same kit into the server catalog via the admin upload route
  (family `percussion`, licence recorded, accepted) under the same stable id —
  operational step, after 9.1

## 7. App — family-scoped selection and playback routing

- [ ] 7.1 `soundfont_catalog_service.dart`: map the wire `SoundFont.instrument`
  onto the catalog entries (the field exists; the app drops it today)
- [ ] 7.2 `piano_catalog.dart`: entries carry the family; imported entries
  persist the detected family in the registry JSON
- [ ] 7.3 Per-family persisted selection: the existing `selected_piano` key
  remains the keyboard memory; add the kit selection (own prefs key, default =
  bundled kit, unknown id self-heals to the bundled kit) — same
  restore/fallback shape as `SelectedPiano`
- [ ] 7.4 Font-follows-score: on player open, resolve the loaded score's family
  and apply that family's remembered font through the existing resolve+load
  path; restore the keyboard font when a keyboard surface returns; reaction via
  `ref.listen` in a dedicated listener/controller — no provider imperatively
  pokes a sibling, UI never calls the service directly
- [ ] 7.5 Percussion readiness: the player treats a percussion score as ready to
  sound only after the swap completion (2.4) resolves; before that, playback is
  visual-only — widget-tested (a notifier test cannot see this seam)
- [ ] 7.6 `player_notifier.dart`: route scheduled notes by the score's family —
  percussion notes through `AudioService.drumOn`/`drumOff`, keyboard unchanged;
  stop/seek/restart still issue all-off; tests with a mocked `AudioService`
  asserting drum events for a percussion score and byte-identical melodic calls
  for a keyboard score
- [ ] 7.7 `sound_selector_field.dart` + the player settings drawer: offer the
  loaded score's family only; label the percussion picker as the kit picker
  (fr/en)
- [ ] 7.8 `soundfonts_screen.dart`: show every font with a family badge
  (no filtering — the screen is not scoped to a score), localised fr/en
- [ ] 7.9 `soundfont_importer.dart`: detect the family via the engine helper
  (2.5) — kit-only → `percussion`, otherwise `keyboard` — record it, and send it
  on the private-library sync; the pads stay display-only (assert unchanged)

## 8. Gates

- [ ] 8.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings`
- [ ] 8.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's
  usual ignore regex (the new engine glue stays in the excluded files; the
  channel bookkeeping, verification and sequence helpers are host-tested)
- [ ] 8.3 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 8.4 `cd apps/music && flutter test --coverage --exclude-tags golden`,
  coverage ≥ 80%
- [ ] 8.5 BO — `yarn test` and the Playwright e2e (pass `BO_E2E_PORT` to avoid
  colliding with another worktree's dev server)
- [ ] 8.6 `flutter_rust_bridge_codegen generate` ran (2.6) and the generated
  bridge is in sync
- [ ] 8.7 `openspec validate add-drum-audio-channel --strict`

## 9. Manual verification and operations

- [ ] 9.1 **Licence sign-off** for the bundled kit: fetch the candidate, read
  its licence text against the actual files, confirm redistribution and
  modification are permitted; record the verdict in `CREDITS.md`. If the
  candidate fails, evaluate the named fallback — no asset lands before this
  passes
- [ ] 9.2 On-device: open a percussion score — the kit loads, playback sounds as
  drums (kick/snare/hi-hat recognisably), the metronome click is unchanged;
  open a keyboard score next — the chosen piano is restored and sounds as before
- [ ] 9.3 On-device: pick a different kit for a drum score, relaunch, confirm
  the kit is remembered while the piano choice is untouched; delete an imported
  kit and confirm the fallback to the bundled kit
- [ ] 9.4 On-device: confirm a pad tap still produces nothing (display-only
  until `add-drum-input-mapping`), and Wait Mode is still not offered
- [ ] 9.5 Console: audition a percussion proposal — the picker offers kits only,
  Play sounds drums; a keyboard score's picker offers pianos only; with the kit
  font unseeded (staging before 6.4), the localised "no drum kit available"
  state shows
- [ ] 9.6 Staging: apply the migration (4.1) and run the ops verification pass
  (4.5) — confirm every pre-existing row reads `keyboard` and no mismatch is
  reported
- [ ] 9.7 Staging: set `catalog.preview.drum_soundfont_id` to the seeded kit,
  accept a percussion score → its preview renders as drums; run the backfill →
  the formerly skipped percussion scores gain previews; keyboard previews
  unchanged; a percussion piece's card audition in the app plays the drum clip
- [ ] 9.8 Upload verification: try declaring `percussion` on a piano `.sf2` and
  `keyboard` on a kit-only `.sf2` — both refused with the localised reason in
  the BO drawer; a full-GM font passes either declaration
- [ ] 9.9 Confirm a keyboard score is unaffected end to end — app playback,
  picker, console Play, previews — and record in `design.md` anything the
  listening pass changed (kit choice, groove sequence)
