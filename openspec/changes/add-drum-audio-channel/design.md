## Context

Three synthesizer sites render notes in this system, and all three hardcode the
piano channel:

- the app engine — `apps/music/rust/src/api/audio_core.rs` (`PIANO_CHANNEL = 0`),
  driven per event over the FFI (`note_on`/`note_off`), score-agnostic;
- the console's offline renderer — `crates/audio-wasm/src/lib.rs`, which parses
  the score itself and carries its own `PIANO_CHANNEL = 0` copy annotated
  "matches the app";
- the backend's headless preview renderer —
  `backend/music/src/soundfont_synth.rs` (`PREVIEW_CHANNEL = 0`), fed by the
  score-teaser and font-preview jobs.

The earlier drum changes speak of the first two; the third exists because
`add-drums-access` had to teach the preview render job to *skip* percussion,
and lifting that skip is this change's obligation — so the discipline below
quantifies over **three** sites, not two.

The pieces this change stands on are already in place. The playback schedule
emits an unpitched note's General MIDI percussion number in the `midi` slot, and
only for percussion-classified scores (`add-unpitched-notation` — the number is
resolved from the part-list's one-based `<midi-unpitched>`: General MIDI number =
element value − 1). The stored `instrument` column and the backend gate exist
(`add-drums-access`). The app player routes a percussion score to the kit view
(`add-drum-kit-view`) — it plays visually and silently. The app swaps SoundFonts
by path at runtime without touching the output stream (`piano-sound-selection`).
And rustysynth follows the SoundFont/MIDI convention: **channel index 9** (MIDI
channel 10 in one-based speak) resolves presets in **bank 128**, where drum kits
live.

One behaviour of rustysynth shapes several decisions below: a missing preset
does not error. A percussion note sent to channel 9 while a piano-only font is
loaded, or a melodic note on channel 0 while a kit-only font is loaded, degrades
to whatever preset the font offers — silence or a wrong timbre, never a failure
the code can catch. Correctness therefore has to come from discipline around the
synth (the right font loaded before the right notes arrive, families that can be
trusted), not from error handling inside it.

## Goals / Non-Goals

**Goals:**

- Sound a percussion score's scheduled playback in the app, its audition in the
  console, and its preview clip on the backend — all on the drum channel, with a
  drum-kit font, keyboard scores byte-for-byte unaffected.
- Ship a bundled kit font so the drum path works out of the box, offline, like
  the bundled piano.
- Make the instrument family a fact (verified against preset banks, one
  vocabulary) rather than an uploader's claim in a private spelling.
- Leave `add-drum-input-mapping` a ready-made one-shot entry point, so wiring
  pad input never has to touch the engine.

**Non-Goals:**

- Sounding pad taps or MIDI drum-pad input — `add-drum-input-mapping` (it needs
  an input event first; the pads stay display-only, as `music-drum-kit-view`
  pins).
- Wait Mode, scoring, rewards for percussion — `add-drum-scoring`.
- Percussion notation drawing — `add-drum-notation-render`, which precedes
  this change in the pipeline; only the console's Play guard is lifted here,
  and the notation preview is governed by `web-notation-render`.
- Multi-instrument synthesis in general (a guitar family, per-part fonts,
  layered channels). Two families, one active font, one channel each.
- `music.courses.instrument`. It shares the `piano` spelling but belongs to the
  courses surface; nothing in this change compares against it, and renaming a
  column this change does not read would be scope creep. Noted so the spelling
  bridge is not thought half-done: the *soundfont* column is bridged here, the
  courses column deliberately is not.

## Decisions

### One channel discipline, defined once — in the crate all three sites already share

`cymbra-musicxml-core` exports the two channel constants (melodic 0, drum 9)
beside `DEFAULT_VELOCITY`, which `audio-wasm` and the backend renderer already
import from it — the app engine holds its own local copy
(`apps/music/rust/src/api/audio_core.rs:36`), and this change unifies that copy
onto the crate's constant (a one-line import; the engine already depends on the
crate), so the single-definition rationale holds without a counterexample
sitting beside the new constants. The wasm file's local channel copy and its
"matches the app" comment are deleted; the backend's `PREVIEW_CHANNEL` alias
likewise.

*Rationale:* the comment is a pin by prose — it survived one change unbroken,
and this change would have been the one to break it silently (three sites to
edit, one annotated by a comment pointing at a second). A constant defined once
cannot drift. The crate is admittedly a notation crate, but it already owns the
playback vocabulary these sites consume (`DEFAULT_VELOCITY`, the schedule, the
classification), and the channel is a property of that vocabulary — "where a
scheduled note sounds" — not of any one synth.

*Alternative rejected:* per-site constants pinned by a cross-crate test. Weaker
than a definition (the test must itself name all three sites, recreating the
drift problem one level up) and it buys nothing — no site legitimately wants a
different value.

*Alternative rejected:* making the channel configurable. It is MIDI convention;
there is nothing to configure.

### Routing keys on the score's classification — resolved per site, per event at the FFI

The wasm and backend renderers hold the parsed document, so they pick the channel
once per render from the document's own instrument classification. The app
engine is score-agnostic — events arrive one at a time over the FFI — so the
channel travels **per event**: new entry points `drum_on(key, velocity)` /
`drum_off(key)` beside `note_on`/`note_off`, and the player notifier routes each
scheduled note by the loaded score's family (which `PlayerData` already carries
since the kit view).

The schedule makes this safe by construction: unpitched notes are emitted only
for percussion-classified scores, so a mixed score cannot reach the drum channel
through any renderer, and a percussion score's schedule contains only General
MIDI numbers.

*Alternative rejected:* an engine "percussion mode" set at score open. Stateful —
a mode left set after leaving a drum score would corrupt free play and the next
keyboard score, and clearing it correctly on every exit path is exactly the kind
of obligation that gets missed. Per-event routing is stateless, keeps the piano
entry points untouched, and hands `add-drum-input-mapping` its one-shot verb for
free: sounding a pad tap is `drum_on` with no further engine work.

*Alternative rejected:* widening `note_on` with a channel or percussion
parameter. It breaks every existing call site and the generated bridge for no
gain, and it invites callers to pass a wrong channel; a separate verb keeps the
piano path byte-identical and makes percussion intent explicit at the seam.

The voice bookkeeping (`VoiceTracker`) learns the channel alongside the key, so
a release lands on the channel where the note sounded and `AllOff` covers both
channels. Scheduled percussion notes keep the on/off pairing every other note
has — a kit voice usually decays on its own, but nothing guarantees every preset
in every user-imported kit does, and a paired off is free.

### The active font follows the loaded score's family

One synthesizer holds one font, and rustysynth degrades silently when the font
lacks the bank a channel needs (see Context). So the app keys the loaded font on
the **loaded score's** family: opening a percussion score swaps to the
remembered kit (the bundled kit by default), leaving the player for a keyboard
surface restores the remembered piano. The kit choice is persisted separately
from the piano choice (`add-drums-access` named this deferral: a drum score
after a piano score must not keep the piano font). The home context never
participates — `music-instrument-context` pinned that the score carries its own
instrument and the player reads it from the score.

The swap uses the existing path-swap FFI, which already parses off-thread and
keeps the current font on failure. Two additions make it honest for this use:

- **A completion signal.** `audio_load_soundfont` is fire-and-forget today; the
  swap keeps the *old* font until the new one is parsed, so on a slow device a
  drum score could start sounding through the piano font — the exact confident
  wrongness this whole feature series exists to avoid. The load therefore gains
  an awaitable completion (or an equivalent readiness callback), and the player
  treats a percussion score as ready to sound only once the kit is installed.
  Kit fonts are small (a few MB against the piano's ~55 MB), so the wait is
  short where it exists at all.
- **The fallback chain mirrors the piano's.** A missing or unloadable chosen kit
  falls back to the bundled kit, which is always present — so "a percussion
  score is opened and no kit font is available" cannot arise except by the
  bundled asset itself being unreadable, which degrades exactly like the piano
  path does today: silence, no crash.

*Alternative rejected:* two resident synthesizers (piano + kit) routed by
channel, no swapping. Doubles the parsed-font memory — decoded grands run to
hundreds of MB — to optimise a transition that happens at score open, off the
hot path.

*Alternative rejected:* merging fonts at runtime so one synth holds both banks.
SF2 surgery on user-supplied files, for the same memory cost.

*Alternative rejected:* asking the user which font to use per score. Busywork;
the family is derivable and the per-family memory preserves their actual choice.

### The bundled kit: FluidR3's percussion bank, MIT — candidate, not yet committed

The app ships a drum-kit `.sf2` beside the bundled piano, same mechanics: an
asset in `assets/soundfonts/`, resolved to a file by the existing
`SoundFontSource`, recorded in `CREDITS.md` with its licence text vendored
alongside. Constraints already settled by this repo's history and respected
here: **SF2 only** (rustysynth rejects compressed SF3), and the licence must be
**human-verifiable** before the bytes enter the repository — FreePats-style
CC0/CC-BY passed that bar for the pianos; producersbuzz and GeneralUser GS were
rejected and stay rejected.

*Candidate:* the percussion bank of **FluidR3 GM** (Frank Wen, **MIT licence**)
— extract the bank-128 kit presets and their samples into a standalone SF2 (a
few MB; Polyphone does this losslessly), ship the MIT text alongside. MIT is
explicit about redistribution and modification, and the extraction is exactly
the modification it permits.

*Alternative recorded:* the AVL Drumkits samples (Glen MacArthur) — higher
recording quality, but distributed as SFZ (needs conversion) and under a
share-alike licence family, which raises a question CC0/CC-BY/MIT do not.
Held as fallback if FluidR3's kit disappoints the feel pass.

*Not adopted here:* final sign-off. Naming a candidate is design; verifying the
licence text against the actual files fetched is a human step, kept as a manual
task before the asset lands — the same bar the pianos cleared.

The same kit is seeded into the server catalog (admin upload route, family
`percussion`, licence recorded, accepted) under a stable id, so the console's
Play and the backend preview job render with a kit that provably exists — the
bundled piano already follows this pattern (`upright-piano-kw` is both bundled
and served).

### The family vocabulary: migrate the column, normalise the boundary

`music.soundfonts.instrument` holds free text defaulting to `piano`, a fourth
spelling beside the scores' `keyboard`/`percussion`/`unknown`. The column is
migrated to the score vocabulary: `UPDATE … 'piano' → 'keyboard'`, default
`keyboard`, CHECK on the two families (`unknown` stays a score-only value — a
font's family is now verified at the door, so "we could not tell" is not a state
a stored font can be in). The upload boundary normalises legacy `piano` input to
`keyboard` permanently, so shipped app versions and old scripts keep working.

*Rationale:* every consumer this change creates — the app picker filter, the
console picker filter, the preview job's font choice, the verification itself —
compares a score family against a font family. With one vocabulary the
comparison is equality; with two, every site carries the mapping and the one
that forgets it silently filters everything out (an empty picker, a dormant
preview — failures with no error anywhere). `piano` was also simply inaccurate:
the family the scores use is `keyboard`, and a harpsichord font was never a
piano.

*Alternative rejected:* keep `piano` in the database and map at the read
boundary. The mapping multiplies (app, console, worker, server routes) and its
omission is invisible; a migration is one honest step. The wire types
(`SoundFont.instrument`, `AdminSoundFont.instrument`) are already strings, so no
proto change is needed — the *value* changes, and the one shipped consumer that
reads it today (the console) deploys in lockstep with the backend, while the app
ignores the field until this change teaches it to filter by it.

### The declared family is verified against preset banks — asymmetric, refused at the door

An SF2's preset headers name their banks; bank 128 is the percussion bank. The
verification is deliberately asymmetric, because fonts legitimately hold both
banks (a full General MIDI bank is a piano *and* a kit):

- declaring `percussion` requires **at least one bank-128 preset** — without
  one, the drum channel finds nothing and the font is silent-by-construction;
- declaring `keyboard` requires **at least one melodic-bank preset** — a
  kit-only font declared keyboard would be the mirror failure;
- a font holding both passes either declaration, and the declaration decides its
  single recorded family.

A mismatch is **refused** with a typed, localisable reason, at every write path:
the admin upload, the user import sync, and the proposal of a private font.
Existing catalog rows — all uploaded before verification existed — get a
one-shot ops pass that re-reads each stored object and reports (not silently
rewrites) any row whose family its banks cannot support.

*Alternative rejected:* flag mismatches for moderation instead of refusing. The
uploader is present at upload time and can fix the declaration in seconds; a
moderation flag parks a silent-by-construction font in a queue where a human
must rediscover what the machine already knew.

*Alternative rejected:* auto-correcting the declaration from the banks. Sound
for single-bank fonts, ambiguous exactly where it matters — a both-banks font
has no bank-derived answer, so correction would silently override the one case
where the declaration carries information.

App-side imports need no question asked: the importer **detects** the family
with the same rule (only bank-128 presets → `percussion`; otherwise
`keyboard`), via a small engine helper that reads the preset headers. The
detected family rides the private-library sync, where the server re-verifies —
the client is never trusted, per the platform rule. A both-banks *imported* font
therefore lands `keyboard`; see Trade-offs.

### Previews: a per-family configured font; the catch-up is the existing backfill

The score-teaser config gains a second key beside `catalog.preview.soundfont_id`:
`catalog.preview.drum_soundfont_id`. The renderer picks the font by the row's
instrument and renders percussion on the drum channel. While the kit key is
unset (or names a missing/unaccepted/wrong-family font), percussion previews are
**dormant** — the same outcome as today's skip, reached honestly: never a
piano-font clip. Keyboard previews cannot regress: their path reads the same key
it always read.

The catch-up needs no mechanism: the `add-drums-access` skip returned `Dormant`,
which stores nothing and **leaves the row unmarked** — and the existing backfill
enqueues exactly "accepted pieces without a rendered marker". Configure the kit
key, run the backfill, and the accepted percussion corpus renders.

The font-preview clips (`soundfont-preview`) get the analogous fix: the fixed
melodic phrase, played on channel 0, is silent or nonsense through a kit font —
so the sample sequence becomes per-family, with a fixed short groove on the drum
channel for `percussion`-family fonts. Same determinism requirement, second
fixed sequence.

### Scope discipline: playback and previews sound; taps do not — but the verb exists

This change sounds scheduled playback (app), auditions (console) and rendered
clips (backend). A pad tap or a MIDI drum-pad hit produces nothing, exactly as
`music-drum-kit-view` pins — there is no input path yet. The boundary is drawn
so `add-drum-input-mapping` never touches the engine: `drum_on`/`drum_off` and
their `AudioService` seam methods ship here and are exercised by scheduled
playback; input mapping wires input events to the same seam.

## Risks / Trade-offs

**The licence sign-off can invalidate the candidate** → the bundled kit is the
one piece a human must approve before it exists. Mitigation: the candidate's
licence (MIT) is among the most permissive in circulation and the fallback is
named; the code paths are testable with any local kit-shaped SF2 fixture before
the real asset lands, so sign-off gates the asset, not the change.

**A both-banks imported font lands in one family** → a user importing a full
General MIDI bank gets it as `keyboard` and cannot pick it for a drum score,
though its bank 128 would serve. Accepted: the recorded family is single-valued
(a column, a picker bucket), the case is marginal, and widening to a multi-family
facet later is additive. The admin path is unaffected — an admin declares, and a
both-banks font passes either declaration.

**Swap-on-open adds a readiness dependency** → a percussion score must wait for
the kit install before sounding, a state the player never had. Mitigation: the
completion signal makes it explicit rather than raced; kit fonts are small; and
the failure mode (signal never fires) degrades to the visual-only playback the
kit view already ships today.

**The migration renames a value shipped clients still send** → any old client or
script writing `instrument=piano` would violate the CHECK. Mitigation: the
boundary normalisation (`piano` → `keyboard`) is permanent and tested, and the
migration lands in the same deploy as the code that normalises.

**Three sites change in one change** → app engine, wasm, backend renderer.
Mitigation: each site is independently inert — the app without a kit font
behaves as today (visual playback), the console without an accepted kit font
shows the unavailable state, the backend without the config key stays dormant —
so a partial rollout degrades to the status quo, never to piano-rendered drums.

**rustysynth's silent degradation hides wiring mistakes** → a wrong channel or a
wrong font produces sound, or silence, but no error. Mitigation: the
verification makes families trustworthy at the door; per-site tests render a
percussion fixture through a kit-shaped font and assert non-silence on the drum
channel (and, for the app, that the piano path's output is byte-identical to
before).

## Migration Plan

1. **Shared constants + engine.** Export the channel constants from
   `cymbra-musicxml-core`; app engine gains channel-aware events,
   `drum_on`/`drum_off`, the tracker's channel, the swap-completion signal;
   `flutter_rust_bridge_codegen generate`. Wasm and backend renderers switch to
   the shared constants and classification-keyed channel. All inert without a
   kit font.
2. **Family migration + verification.** Migrate `music.soundfonts.instrument`
   (`piano` → `keyboard`, default, CHECK); boundary normalisation; preset-bank
   verification on upload/import/propose; the app maps the wire `instrument`
   field; ops verification pass over existing rows.
3. **Bundled kit.** Human licence sign-off, then the asset + `CREDITS.md` +
   licence file; seed the same kit into the server catalog (family
   `percussion`, accepted).
4. **Consumers.** App: per-family selection, swap-on-open, family-scoped picker,
   import detection, playback routing to `drumOn`/`drumOff`. Console: family
   filter, kit default, Play guard lifted (`yarn gen:wasm`). Backend: per-family
   preview font + drum-channel render + kit groove for font previews; declare
   `catalog.preview.drum_soundfont_id`.
5. **Catch-up.** Configure the kit preview key; run the existing backfill;
   verify the formerly-skipped percussion scores now carry previews.

Steps 1–2 are safe alone and reversible. Step 5 is a flag edit plus an existing
ops command; clearing the key returns percussion previews to dormant (already-
rendered clips remain, correct and harmless).

Rollback of the code is a revert; the migration's value rename is reversed by
the inverse UPDATE (no information was lost — `keyboard` rows were all `piano`).

## Open Questions

- **Which kit ships.** FluidR3's percussion bank is the candidate; the feel pass
  and the licence sign-off have the final word, and the AVL samples are the
  named fallback. Does not block any code path (any kit-shaped SF2 exercises
  them).
- **Velocity shading.** The schedule plays every note at `DEFAULT_VELOCITY`; a
  drum groove's accents (ghost notes, rimshot dynamics) are flattened. Living
  with it here — the same flattening applies to piano playback today — but
  `add-drum-input-mapping` (real pad velocities) and `add-drum-scoring` may
  force the question.
- **Hi-hat choke.** An open hi-hat silenced by a closed one is the font's
  exclusive-class behaviour, not the engine's. Whether the candidate kit's
  exclusive classes are set correctly is a feel-pass question; if not, choking
  in the engine would be a new mechanism, deliberately not designed here.
- **Should the console offer per-moderator kit choice, or just the seeded
  default?** The picker filters by family, so the choice exists whenever the
  catalog holds several accepted kits; whether moderation *needs* more than the
  default kit is a workflow question the moderators will answer by using it.
