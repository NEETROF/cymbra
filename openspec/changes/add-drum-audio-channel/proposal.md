## Why

A percussion score can be parsed (`add-unpitched-notation`), admitted and gated
(`add-drums-access`) and read in the kit view (`add-drum-kit-view`) — but it still
makes no sound. Every synthesizer site in the system plays on one hardcoded piano
channel: the app engine (`audio_core.rs`, `PIANO_CHANNEL = 0`), the console's wasm
renderer (`crates/audio-wasm/src/lib.rs`, its own `PIANO_CHANNEL = 0` copy
annotated "matches the app"), and — a third site the earlier changes did not have
to name — the backend's headless preview renderer
(`backend/music/src/soundfont_synth.rs`, `PREVIEW_CHANNEL = 0`). A drum part's
General MIDI numbers fed to channel 0 sound as piano notes, which is why
`add-drums-access` installed three guards instead: the console's Play control
refuses percussion, the preview render job skips it, and the app never enters the
player (since lifted by the kit view, which plays visually but silently).

This change makes percussion sound. In the SoundFont/MIDI convention the drum kit
lives on **MIDI channel 10 — index 9 as rustysynth counts channels** — where
preset lookup goes to the font's **bank 128**, the bank that holds drum kits. So
sounding a drum score takes three things: route percussion notes to that channel
in every synth site under one shared constant discipline; have a **drum-kit
SoundFont** loaded when they arrive — bundled in the app like the bundled piano,
configured for the preview job, selectable in the console; and stop trusting the
uploader-declared instrument family that decides which fonts are offered where,
by verifying it against the file's actual preset banks.

The drum feature is being built across eight changes; this is the sixth. Of the
four that remained after `add-instrument-context`, the delivery order is
`add-drum-notation-render` → **`add-drum-audio-channel`** →
`add-drum-input-mapping` → `add-drum-scoring`. This change depends on the four
implemented ones — the schedule's General MIDI numbers, the stored instrument and
its gate, and the kit view (the only route by which the app player reaches a
percussion score). It needs no **code** from `add-drum-notation-render` — audio
needs no drawing — but the two changes modify the same `moderation-console`
requirement, so their order is not free: this change's delta builds on
`add-drum-notation-render`'s and archives after it, per the delivery order
above. `add-drum-input-mapping` and `add-drum-scoring` build on the sounding
path this change creates.

## What Changes

**One drum channel, one constant discipline, three synth sites**

- `cymbra-musicxml-core` exports the channel constants beside `DEFAULT_VELOCITY`
  (which `audio-wasm` and the backend renderer already import from it; the app
  engine holds a local copy in `audio_core.rs` that this change unifies onto
  the crate's): the melodic channel 0 and the drum channel 9. The wasm copy's
  "matches the app" comment — a pin by prose — is replaced by a pin by
  definition.
- **App engine** (`apps/music/rust/src/api/`): the FFI gains percussion note
  entry points (`drum_on` / `drum_off`) that route to the drum channel; the voice
  bookkeeping learns the channel so releases land where the note sounded. The
  piano entry points are byte-for-byte untouched. This is a public-API change:
  `flutter_rust_bridge_codegen generate` runs.
- **Console renderer** (`crates/audio-wasm`): the render picks its channel from
  the document's own instrument classification — a percussion-classified score
  renders on the drum channel, everything else exactly as today. (The schedule
  only emits unpitched notes for percussion-classified scores, so mixed scores
  cannot reach the drum channel by construction.)
- **Backend preview renderer** (`backend/music/src/soundfont_synth.rs`): same
  rule, same shared constants.
- The metronome click and the WAV preview-clip player are synthesized outside
  the SoundFont and are untouched.

**The active font follows the loaded score's family**

- The app already swaps SoundFonts by path at runtime (`piano-sound-selection`).
  It now keys the loaded font on the **loaded score's** instrument family: opening
  a percussion score loads the remembered kit (bundled kit by default), returning
  to a keyboard surface restores the remembered piano. The home context never
  decides — `music-instrument-context` pinned that the score carries its own
  instrument.
- The kit choice is remembered **separately** from the piano choice, so loading a
  drum score after a piano score never keeps a piano font and plays nonsense —
  the deferral `add-drums-access` named.

**A bundled drum kit, like the bundled piano**

- The app ships a drum-kit `.sf2` in `assets/soundfonts/` beside
  `UprightPianoKW`, recorded in `CREDITS.md` with its licence file — SF2 only
  (rustysynth rejects compressed SF3), licence human-verifiable before bundling
  (the settled bar: FreePats-style CC0/CC-BY acceptable; producersbuzz and
  GeneralUser GS stay rejected). The concrete candidate and its licence are named
  in `design.md`; final sign-off is a manual task.
- The same kit is seeded into the server SoundFont catalog (family `percussion`,
  accepted) so the console's Play and the backend preview job have a kit to
  render with.

**Previews stop skipping percussion**

- The render-job skip installed by `add-drums-access` is lifted: the job renders
  a percussion score with a **configured kit font on the drum channel** (a second
  config key beside `catalog.preview.soundfont_id`); while no kit font is
  configured, percussion previews stay dormant exactly as today — never a
  piano-font clip.
- The catch-up for accepted percussion scores left without previews is the
  **existing** backfill: the skip left their rows unmarked, so "accepted pieces
  without a rendered marker" is precisely the work list. Configure the kit font,
  run the backfill, done — no new mechanism.
- The **console's Play guard is lifted**: a percussion row auditions through the
  wasm renderer on the drum channel with a percussion-family font; the
  `SoundFontPicker` filters by the **score's** family (a keyboard score under a
  drums home context still lists piano fonts). With no accepted kit font in the
  catalog, Play shows a localised "no drum kit available" state — not an error.
  The notation preview is untouched here: it is governed by
  `web-notation-render`, which `add-drum-notation-render` owns.

**The instrument family becomes trustworthy**

- **Vocabulary bridge:** `music.soundfonts.instrument` speaks a fourth spelling
  (`piano`, `DEFAULT 'piano'`) while scores speak
  `keyboard`/`percussion`/`unknown`. The column is **migrated** to the score
  vocabulary — values `piano` → `keyboard`, default `keyboard`, CHECK on the two
  families — and the upload boundary normalises legacy `piano` input forever
  (shipped clients keep working). One vocabulary end-to-end beats a mapping
  re-implemented at every comparison site. `music.courses.instrument` is **not**
  touched — it is not this change's column, and nothing here compares against it.
- **Verification:** a font's declared family is checked against its preset banks
  at every write path (admin upload, user import sync, proposal): declaring
  `percussion` requires at least one **bank-128** preset, declaring `keyboard` at
  least one melodic-bank preset; a font holding both passes either declaration.
  A mismatch is refused with a typed reason — the one facet that picks the synth
  channel is no longer merely trusted. Existing rows get a one-shot ops
  verification pass.
- **Pickers filter by family:** the app's sound picker offers the loaded score's
  family; a user-imported font's family is **detected** from its preset banks,
  never asked; the console picker and the back-office font drawer carry the
  two-family vocabulary.

**Explicitly in scope / out of scope**

- This change sounds **scheduled playback and preview clips**. Pad-tap and MIDI
  drum-pad sounds are `add-drum-input-mapping`'s — they need an input event
  first — but the one-shot "sound this General MIDI number now" entry point they
  will call ships **here** (`drum_on`/`drum_off` through the audio-service seam),
  so that change wires input to an existing verb rather than growing the engine.
- Wait Mode, scoring, and the pad strip's display-only status are untouched
  (`add-drum-scoring`, `add-drum-input-mapping`).

## Capabilities

### New Capabilities

- `music-drum-audio`: how percussion sounds — the drum channel and the shared
  constant discipline across the three synth sites, the font-follows-score rule
  and the per-family memory, the bundled kit, the one-shot entry point for input
  mapping, and the preset-bank verification of a declared family.

### Modified Capabilities

- `audio-output`: the engine's control surface gains percussion note entry
  points; the synthesis requirement is renamed off "piano" (it now synthesizes
  two instrument families); score playback sounds a percussion score's General
  MIDI numbers as kit voices.
- `piano-sound-selection`: the catalog and picker become family-scoped with a
  per-family remembered selection; an imported font's family is detected from
  its preset banks.
- `music-drums-visibility`: the render-job skip is lifted — a percussion score's
  preview is rendered with a kit font, or left absent while none is configured;
  never a piano clip. The vocabulary note that delegated the spelling bridge to
  this change is settled. (Builds on `add-drums-access`'s not-yet-archived
  version of both requirements.)
- `moderation-console`: the Play guard is lifted — percussion rows audition on
  the drum channel with a family-filtered font; the badge stays, and the
  notation preview is governed by `web-notation-render`, untouched here.
  (Builds on `add-drum-notation-render`'s not-yet-archived version of the
  requirement — the two changes modify the same requirement and this one
  archives second; renamed, since the retained title promised the refusal this
  change removes.)
- `music-score-audio-preview`: the render job's configured font becomes
  per-family; percussion renders with the kit font on the drum channel, and
  stays dormant while no kit font is configured.
- `soundfont-catalog-admin`: the instrument family speaks the score vocabulary,
  offers both families, and is verified against the file's preset banks instead
  of recorded on trust.
- `soundfont-preview`: the fixed sample sequence becomes per-family — a
  percussion font's preview clip is a short groove on the drum channel, not a
  silent melodic phrase.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the stored instrument, the kit view's player routing, the existing font-swap FFI | drum channel + `drum_on`/`drum_off` in the engine, bundled kit, family-scoped picker and per-family memory, font-follows-score swap |
| **Back-office** (`apps/back-office`) | the same wasm renderer, the font catalog listing | percussion Play on the drum channel, family-filtered picker, two-family font drawer |
| **Backend** (`backend/`) | the flag-read config shape of the preview job, the existing backfill | per-family preview font, drum-channel render, family migration + preset-bank verification, kit groove for font previews |
| **Platform** (feature flags) | the existing declared-key registry | one config key (`catalog.preview.drum_soundfont_id`) |
| **ID / Live / Site** | — | untouched |

**Code**

- `crates/musicxml-core/src/`: the channel constants exported beside
  `DEFAULT_VELOCITY`; the instrument classification is already public.
- `apps/music/rust/src/api/`: `audio_core.rs` (channel-aware events + voice
  bookkeeping), `audio.rs` (`drum_on`/`drum_off`, swap-completion signal),
  `renderer.rs` (route by event channel); then
  `flutter_rust_bridge_codegen generate`.
- `crates/audio-wasm/src/lib.rs`: channel from the document classification;
  the local constant copy deleted.
- `backend/music/src/`: `soundfont_synth.rs` (channel parameter),
  `score_preview.rs` + `score_preview_module.rs` (per-family font, skip lifted),
  `soundfont_preview.rs` (kit groove sequence), `soundfont.rs` (family
  vocabulary); `backend/server/src/soundfont.rs` (boundary normalisation +
  preset-bank verification on upload/import/propose);
  `backend/feature-flags/src/registry.rs` (the new config key); one migration
  on `music.soundfonts`.
- `apps/music/lib`: `services/soundfont_catalog_service.dart` (map the wire
  `instrument`), `state/piano_catalog.dart` / `selected_piano.dart` (family
  scoping + per-family selection), `services/soundfont_importer.dart` (family
  detection), `state/player_notifier.dart` (route percussion notes to
  `drumOn`/`drumOff`, swap on open), `widgets/sound_selector_field.dart`,
  `screens/soundfonts_screen.dart`, `assets/soundfonts/` + `CREDITS.md`.
- `apps/back-office/src`: `composables/useSoundFontChoice.ts` (family filter +
  kit default), `views/ScoreDetailView.vue` (Play guard lifted),
  `components/SoundFontDrawer.vue` (two families), locales fr/en; `yarn
  gen:wasm` after the wasm change.

**Operational prerequisite.** The bundled kit's licence must be human-verified
before its bytes enter the repository, and the kit must be seeded into the server
catalog (admin upload, family `percussion`) before the console Play lift and the
preview backfill do anything. Both are named manual tasks.
