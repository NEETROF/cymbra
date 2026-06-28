## Context

The player already derives a single tempo (BPM) and time signature
(`beats`/`beatType`) from the MusicXML and exposes them on `PlayerData`, and it
already builds a `measureStartMs` table when notation is applied
(`notation_playback.dart` → `notationToTimedNotes`). Playback is **UI-driven**: a
`Ticker` in `player_screen.dart` calls `Player.advance(dtMs)` each frame, which
moves `elapsedMs` forward by `dt * speed`, sounds score notes across the crossed
span via `_applyScoreAudio`, and silences everything on a loop wrap. Audio goes
through the injectable `audioServiceProvider` seam (`AudioService` →
`FrbAudioService` → Rust `note_on/note_off/all_notes_off`). The bundled SoundFont
is a piano only — there is no percussion bank for a click.

The header `_TopBar` shows a read-only `_Chip(icon: Icons.speed, label: 'Tempo:
$bpm')`. Because the chip is in the shared header, anything we render on it is
visible in all three view modes (synthesia/staff/partition) for free.

This change adds a metronome toggled from that chip: beat events derived from the
playhead, expressed as a synthesized audio click plus a visual pulse on the chip,
silent while paused.

## Goals / Non-Goals

**Goals:**
- Toggle the metronome by tapping the existing Tempo chip; reflect on/off on the chip.
- Fire exactly one beat event per beat of the measure, accenting the downbeat.
- Drive beats from the **same playhead/measure timing** as note playback (no drift,
  honours the speed multiplier), staying in sync with the score.
- Express each beat both audibly (synthesized click, accent distinct) and visually
  (chip pulse), so it works identically across all three view modes.
- Be silent while paused/stopped; resume cleanly; no spurious tick on seek/loop.
- Keep beat detection and click logic host-testable behind the existing seams so
  both ecosystems stay ≥ 80% coverage.

**Non-Goals:**
- No standalone/independent metronome (not tied to a loaded score) and no
  count-in.
- No per-measure tempo map or tempo changes mid-piece (playback already assumes a
  single derived BPM; we follow the existing `measureStartMs` model).
- No user-configurable click sound, subdivisions, or volume in this change.
- No change to how notes are sounded or to Wait Mode behaviour.

## Decisions

### Beat timing: derive beat boundaries from the existing measure table

Beats are computed from `measureStartMs` and the time signature rather than a
free-running timer, so they inherit the playhead's sync (and the speed multiplier,
which scales `dtMs` upstream). Within measure `i`, the `beats` beats are evenly
spaced between `measureStartMs[i]` and the next measure start (or
`songEndMs`/derived measure length for the last). Detection mirrors
`_applyScoreAudio`: on each `advance`, examine the half-open span
`[elapsedMs, next)` and fire a beat event for every beat boundary that falls in
it; mark a boundary as an accent when it coincides with a measure start (beat
index 0).

- *Why:* reuses the one source of truth that already keeps audio in step with the
  notes; "synchronised with the score" falls out for free.
- *Alternative — independent timer at 60/BPM:* simpler but drifts from the playhead
  (especially under speed changes, pause, seek, loop) and would need its own
  re-sync logic. Rejected.

Edge cases handled like the existing audio path: on a **loop wrap** or **seek**, do
not emit a tick across the seam — reset the beat tracker so the next genuine
boundary fires (matches `_silenceAll` on loop). Use a half-open span so a frozen
Wait Mode onset does not double-fire.

### State: a `metronomeEnabled` flag plus a beat-pulse signal on `PlayerData`

Add `metronomeEnabled` (bool, default off, persisted with the other player
settings) and a beat-pulse signal the chip can watch — a monotonically increasing
`beatCount` plus a `lastBeatAccent` bool (or a small `beatTick` value object).
`advance()` increments the pulse and calls the audio seam when a boundary is
crossed while enabled and playing. A `toggleMetronome()` action flips the flag.

- *Why a counter, not a transient bool:* Riverpod/Freezed state is rebuilt by
  value; a monotonic counter lets the chip widget animate a pulse on change
  without races, and is trivial to assert in tests. The accent flag rides
  alongside it.
- *Alternative — drive the chip animation straight from audio callbacks:* breaks
  the "state is the source of truth" rule and is hard to test without native audio.
  Rejected.

### Audio: synthesized click in the Rust engine, new FFI entry point

Add `metronome_click(accent: bool)` to the Rust audio API and a matching
`metronomeClick(accent)` on the `AudioService` seam. The audio thread mixes a
short synthesized tick (a brief enveloped tone / filtered noise burst) into its
output buffer alongside the synth — accent = higher pitch/level, normal = lower —
independent of the loaded `.sf2`. The pure click sample/envelope generation lives
in `audio_core.rs` (host-testable); `audio.rs` owns only the cpal/thread glue.

- *Why synthesize, not reuse the SoundFont:* the bundled font is piano-only, and
  the piano-sound-selection feature swaps fonts at runtime — a synthesized click
  stays constant and unmistakably distinct from the music. No new asset to license
  or bundle.
- *Alternative — bundle a click `.wav`:* an extra asset + decoder path and another
  licensing item; the click is trivially synthesizable. Rejected.
- *Alternative — percussion channel on a GM SoundFont:* the bundled font has no
  drum bank; would force a second font. Rejected.

### UI: make the existing chip tappable with active + pulse styling

Wrap the Tempo `_Chip` in a tap target that calls `toggleMetronome()`. When
enabled, the chip shows an active style (e.g. filled/coloured). It watches the
beat-pulse signal and runs a short scale/opacity pulse animation per beat, with a
stronger pulse on the accent. Keep the existing `Tempo: $bpm` label.

- *Why the chip, not a new button:* the request is explicit ("sur le bouton
  Tempo"); it is already present in every mode's header. No layout change.

## Risks / Trade-offs

- **Frame granularity vs. beat precision** → at high BPM/speed a frame may span a
  beat; the half-open-span scan fires the boundary on the frame that crosses it
  (same precision as note onsets today). Acceptable — the click aligns with the
  notes, which is the user-visible requirement. If two boundaries fall in one
  span, fire each once.
- **Click during the audio thread mix could glitch** → keep the click generator
  allocation-free and bounded (precomputed short envelope); push only a
  lightweight "start click(accent)" event to the thread, mirroring the existing
  note-event channel.
- **Drift suspicion under pause/seek/loop** → explicitly reset the beat tracker on
  stop/seek/loop (no tick at the seam) and recompute the next boundary from
  `measureStartMs`; covered by scenarios.
- **Graceful degradation** → if audio is unavailable the visual pulse must still
  work (the seam no-ops the click), preserving the existing "visuals unaffected by
  audio" guarantee.
- **Coverage** → native cpal/thread glue (`audio.rs`) is excluded from Rust
  coverage, so the click's testable logic must live in `audio_core.rs`; beat
  detection lives in `player_notifier`/`player_data` and is unit-tested with a fake
  audio service.

## Resolved Questions

- **Beat model:** one tick per beat; the number of ticks per measure follows the
  time signature (4/4 → 4, 3/4 → 3, 2/4 → 2, …) — not a fixed two-sound "tic-tac"
  alternation. The first beat of each measure is **accented** (distinct
  pitch/level) so the start of the measure is audible; the other beats are uniform
  normal ticks.
- **Persistence:** `metronomeEnabled` is **global** — a single app-wide preference
  stored with the existing player settings, restored on launch and shared across
  all scores (not per-score).

## Open Questions

- Exact click timbre/pitches for accent vs. normal beat (tune by ear during
  implementation; the spec only requires "short", self-terminating, and an
  accent that is "distinct" from a normal beat).
