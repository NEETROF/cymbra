## Context

The keyboard is a bare `CustomPaint(painter: PianoKeyboardPainter(...))` in
`screens/player_screen.dart` (~lines 127–139) with no `GestureDetector`,
`Listener`, or `MouseRegion` — taps do nothing. Pressed-key visuals already derive
from `PlayerData.activeNotes`, which `Player.noteOn/noteOff`
(`player_notifier.dart:173`) mutate. Those two methods are the single input choke
point shared by the MIDI stream (`_onMidi`) and the computer-keyboard fallback
(`_PlayerScreenState._onKey`, QWERTY → pitch).

`PianoLayout` (`painters/piano_layout.dart`) maps pitch → geometry:
`keyRect(pitch) → ({left, width})`, `centerX`, `contains`, static `isBlack`,
`whiteWidth`, `blackWidth`. It does not yet map a pointer position back to a pitch.
The painter draws black keys over the upper ~62% height band on top of the whites.

So the work is purely additive: a hit-test on `PianoLayout` plus a `Listener`
wrapping the existing `CustomPaint`, calling the same `noteOn/noteOff`.

## Goals / Non-Goals

**Goals:**
- Mouse/touch press → `noteOn`, release → `noteOff`, via the existing path.
- Multi-touch: independent per-pointer pitch tracking.
- Correct hit-testing through `PianoLayout`, black keys prioritized in the overlap
  band; out-of-range → no note.
- No new visual code — reuse `activeNotes`-driven rendering.

**Non-Goals:**
- No audio synthesis (there is none; this only feeds the same event path).
- No velocity from touch (MIDI events already normalize; on-screen play has no
  velocity sensing — emit the same way the keyboard fallback does).
- No drag-glissando requirement (may be added later; see Open Questions).
- No engine/MIDI/public-API change.

## Decisions

### Decision: `Listener` (raw pointer events), not `GestureDetector`
Wrap the keyboard `CustomPaint` in a `Listener` and handle
`onPointerDown/onPointerUp/onPointerCancel` (and optionally `onPointerMove`).
**Why:** `Listener` exposes per-pointer `event.pointer` ids and fires for every
concurrent contact, which is exactly what multi-touch polyphony needs.
`GestureDetector` arena/recognizer semantics collapse simultaneous touches and add
tap/long-press disambiguation we do not want. **Alternative considered:**
`GestureDetector` — rejected for multi-touch.

### Decision: Hit-test lives on `PianoLayout` as a pure function
Add `int? pitchAt(Offset p, double height)` (or `pitchAtX` + a black/white height
check) to `PianoLayout`: scan black keys first within the black-height band using
`keyRect`, then white keys across full height; return null when outside range or
no key matches. **Why:** keeps the geometry math in the same const, host-testable
class the painter already uses (no widget/native needed for tests) and guarantees
hit regions match drawn regions. **Trade-off:** a linear scan over the displayed
pitches per event — trivial for ≤88 keys.

### Decision: Per-pointer pitch map in `_PlayerScreenState`
Maintain `Map<int pointerId, int pitch>`. On down: hit-test, if a pitch and not
already pressed by another pointer, store `pointerId→pitch` and `noteOn`. On
up/cancel: look up and remove the pointer's pitch, `noteOff` it. **Why:** matches
the MIDI/computer-keyboard model where each source calls `noteOn/noteOff`;
per-pointer tracking gives independent release. **Note:** `noteOn/noteOff` already
guard against duplicate add/remove on the shared `activeNotes` set, so overlapping
sources (two pointers, or pointer + MIDI on the same pitch) won't double-fire
state changes; releasing one source while another still holds the pitch keeps it
sounding only if we reference-count — see Risks.

### Decision: Reuse existing wiring, no painter change
`_PlayerScreenState` already reads `playerProvider.notifier` and calls
`noteOn/noteOff` for the QWERTY fallback; the pointer handlers sit next to
`_onKey` and call the same methods. The painter repaints from `activeNotes`
unchanged. **Why:** smallest possible surface; consistent with house style.

## Risks / Trade-offs

- **Same pitch from two sources / two pointers, shared `activeNotes` set** →
  `activeNotes` is a plain set, so the first release would `noteOff` the pitch even
  if another pointer still holds it. Mitigate by either (a) accepting last-release
  wins (simplest, fine for v1 since same-pitch double-press is rare on screen) or
  (b) ref-counting held pitches. Decide in implementation; default to (a) with a
  test documenting the behavior.
- **Pointer leaves the key / keyboard while down** → handle `onPointerCancel` and
  treat leaving as release (or, if `onPointerMove` is implemented, retarget). v1:
  release on up/cancel; no slide-retarget.
- **Black/white overlap correctness** → covered by `pitchAt` unit tests at the
  band boundary; black-first scan guarantees priority.
- **Coverage (≥80%)** → `pitchAt` is pure and fully unit-testable; widget tests
  drive synthetic pointer events to assert `noteOn/noteOff` and multi-touch.

## Migration Plan

Purely additive UI/input; no data, no rollback steps beyond removing the
`Listener` and `pitchAt`. Default behavior with no pointer input is identical to
today. Ships independently of the other in-flight changes.

## Open Questions

- **Slide/glissando**: should dragging across keys retarget the held note
  (note-off old, note-on new)? Useful but optional — defer unless requested; if
  added, implement in `onPointerMove`.
- **Ref-count shared pitches** vs last-release-wins for the same pitch from
  multiple sources — pick during implementation; v1 leans last-release-wins.
- **Visual press affordance for pointer** — pressed state already shows via
  `activeNotes`; no extra hover/cursor styling planned for touch-first use.
