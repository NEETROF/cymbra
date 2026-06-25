## 1. Layout: pointer hit-testing

- [x] 1.1 Add `int? pitchAt(Offset p, double height)` (or `pitchAtX` + height band) to `PianoLayout`: scan black keys first within the black-height band, then white keys, using `keyRect`; return null when out of range.
- [x] 1.2 Ensure black-key priority in the overlap region and exact bounds at white/black boundaries.

## 2. Input wiring: Listener + per-pointer tracking

- [x] 2.1 Wrap the keyboard `CustomPaint` in `screens/player_screen.dart` with a `Listener` (`onPointerDown`/`onPointerUp`/`onPointerCancel`, optional `onPointerMove`).
- [x] 2.2 Add a `Map<int, int>` pointerId→pitch in `_PlayerScreenState`; on down hit-test and `noteOn`, storing the pointer's pitch; on up/cancel look up, `noteOff`, and remove.
- [x] 2.3 Pass the live `PianoLayout` and keyboard height into the handlers so hit-testing matches what is drawn.
- [x] 2.4 Decide same-pitch-from-multiple-sources handling (last-release-wins vs ref-count) and implement; document the choice. → last-release-wins (documented on `_keyboardPointers`); chords use distinct pitches.

## 3. Tests

- [x] 3.1 Unit-test `pitchAt`: white-key hit, black-key priority in overlap, boundary pixels, and out-of-range → null.
- [x] 3.2 Widget-test: a pointer-down then pointer-up over a key region calls `noteOn`/`noteOff` for the right pitch and toggles the pressed visual.
- [x] 3.3 Widget-test multi-touch: two simultaneous pointers hold two pitches; releasing one note-offs only its pitch.
- [x] 3.4 Widget-test: pressing the awaited note on-screen satisfies Wait Mode (same path as MIDI).
- [x] 3.5 Widget-test: keys respond in all three render modes (top-bar/keyboard is mode-independent).

## 4. Verify & gate

- [x] 4.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [x] 4.2 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80% (piano_layout 100%, player_screen 91.7%).
- [x] 4.3 Manually confirm in-app on desktop (mouse) and a touch device: tap plays/holds/releases keys, chords via multi-touch, and on-screen play drives Wait Mode.
- [x] 4.4 `openspec validate playable-onscreen-keyboard --strict` passes.
