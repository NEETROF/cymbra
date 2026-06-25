## Why

The on-screen keyboard is display-only — `CustomPaint` with
`PianoKeyboardPainter`, no pointer handling. The only ways to "play" are a real
MIDI device or a 15-key computer-keyboard fallback (QWERTY → C4..D5), which is
cramped and hard to use and cannot reach most of the 88 keys. Users without a
MIDI piano need to play directly on the keys they already see, by mouse or touch.

## What Changes

- Make the displayed keys **playable by mouse and touch**: pressing a key sends a
  note-on, releasing sends a note-off, through the **same path as MIDI input**
  (`Player.noteOn`/`noteOff`), so on-screen play satisfies Wait Mode and the
  expected/correct feedback exactly like a physical key.
- **Multi-touch**: multiple keys can be held at once (chords) where the platform
  supports it; each pointer tracks its own key independently.
- Hit-testing uses the existing `PianoLayout` so taps map to the correct pitch,
  with **black keys taking priority** in their overlap region with white keys.
- Works in all three render modes and at all times (during playback and Wait
  Mode), since the keyboard is in the mode-independent area of the player screen.

## Capabilities

### Modified Capabilities
- `keyboard-display`: add on-screen pointer input — the displayed keys become an
  input surface (mouse/touch, multi-touch) feeding the same note-on/off path as
  MIDI, in addition to their existing display/feedback role.

## Impact

- **UI/input**: `apps/music/lib/screens/player_screen.dart` — wrap the keyboard
  `CustomPaint` in a `Listener`, add pointer-down/move/up handlers and a
  per-pointer pitch map; reuse `_PlayerScreenState`'s existing input wiring that
  already calls `notifier.noteOn/noteOff`.
- **Layout/hit-test**: `apps/music/lib/painters/piano_layout.dart` — add a
  `pitchAt(Offset, height)` (or `pitchAtX`) hit-test using `keyRect` and the
  black-key height band, with black-key priority. (Pure function, host-testable.)
- **Painter**: no change required — pressed keys already render from
  `activeNotes`, which the pointer path updates.
- **Tests**: unit tests for `pitchAt` (white/black boundaries, black-key
  priority, out-of-range → null); widget tests that a pointer-down/up on a key
  region calls `noteOn`/`noteOff` and that two simultaneous pointers hold two
  pitches.
- No Rust/engine, MIDI, or public-API changes. Independent of
  `hand-selection-filter` and `wait-on-keypress`, though it makes the latter
  directly usable without hardware.
