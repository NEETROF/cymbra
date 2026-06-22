## Context

The on-screen keyboard ([piano_keyboard_painter.dart](apps/music/lib/painters/piano_keyboard_painter.dart))
is a read-only visualizer fed by a single `PianoLayout`
([piano_layout.dart](apps/music/lib/painters/piano_layout.dart)) that
`player_screen` also passes to the Synthesia waterfall — so keyboard range and
falling-note columns are locked together. The range is hard-coded to C4–C6 and
the only highlight is one purple "pressed" color. We add a runtime range chooser
(default auto-fit), force landscape so larger ranges stay legible, and add
expected/correct/pressed feedback. State is Riverpod 2 + Freezed (codegen);
coverage must stay ≥ 80%.

## Goals / Non-Goals

**Goals:**
- Force landscape (Flutter + iOS + Android).
- Runtime keyboard-range chooser: full 88-key default + auto-fit + presets
  25/37/49/61/76.
- Pure, testable auto-fit from the score's pitch range; clamp to A0..C8; always
  cover every note so the waterfall never drops one.
- Three-state key feedback (expected teal / correct green / pressed purple) using
  the existing Wait-Mode required set.

**Non-Goals:**
- Touch input on the on-screen keys (still QWERTY/MIDI only).
- Persisting the chosen range across launches (in-memory only; SharedPreferences
  is a follow-up).
- New glyph/engraving work; this is keyboard + waterfall range and coloring only.

## Decisions

### Decision: compute the range once, feed the single shared `PianoLayout`
`player_screen` builds one `PianoLayout(width, lowPitch, highPitch)` from
`data.keyboardBounds` and passes the same instance to both painters. This is the
only change needed to decouple from the hard-coded range — Synthesia stays
aligned by construction.
- **Alternative:** separate layouts per painter — rejected: risks
  keyboard/waterfall drift.

### Decision: pure `computeKeyboardRange` in a Flutter-free module
`keyboard_range.dart` holds `computeKeyboardRange(mode, notes)` plus consts,
`presetKeyCount`, and the `label` extension — no Flutter imports, so it is fully
host-tested and counts toward coverage. `PlayerData.keyboardBounds` just calls it.
- **Auto:** snap low→C, high→octave-top; ≥ 2-octave span; clamp [21,108];
  re-assert `low ≤ min && high ≥ max`; empty → (60,84).
- **Presets:** anchor window per size, shift to cover the score, widen if the
  piece is wider (correctness > exact preset size), clamp, re-assert coverage.
- **Why:** mirrors the project's pure-core rule (`midi_core.rs`,
  `musicxml_core.rs`); keeps the geometry decision testable and deterministic.

### Decision: three-state coloring by precedence, reusing theme + Wait-Mode gate
Extend `PianoKeyboardPainter` with `requiredNotes`; precedence in `_drawKey`:
`required && active` → `tertiary` (green), `required && !active` →
`secondaryContainer` (teal), `active && !required` → `primaryContainer` (purple),
else key base color. `player_screen` passes `data.requiredNotesAt(data.elapsedMs)`
— the same set Wait Mode gates on — so the "press this" highlight is always
consistent with playback. Black-key highlights get an outline/cap for visibility
at narrow widths.
- **Why:** reuses existing colors ([cymbra_theme.dart](apps/music/lib/theme/cymbra_theme.dart))
  and the existing `requiredNotesAt` ([player_data.dart:83](apps/music/lib/state/player_data.dart)).

### Decision: landscape via Flutter + native config
`SystemChrome.setPreferredOrientations([landscapeLeft, landscapeRight])` in
`main()` (no-op on desktop/web), plus Info.plist (both orientation arrays) and
AndroidManifest `screenOrientation="sensorLandscape"`. The Android activity's
existing `configChanges` already covers orientation so it won't recreate.

### Decision: range chooser as a top-bar PopupMenuButton
Seven options don't fit a segmented control, so use a `PopupMenuButton`
(mirroring the existing `_MidiStatusIndicator` popup) showing `Icons.piano` +
current label, `onSelected: notifier.setKeyboardRange`.

## Risks / Trade-offs

- **88 keys still tight on small landscape phones** → mitigated by auto-fit
  default and the user choosing a smaller preset; legibility is the user's call.
- **Black-key highlight invisibility at small widths** → outline/cap stroke for
  highlighted black keys (covered by a golden).
- **Range recompute each rebuild** → `computeKeyboardRange` is cheap and pure;
  `PianoLayout` is const; `shouldRepaint` already gates the painter.
- **Golden platform sensitivity** → tag `golden`, excluded from the
  cross-platform gate per CLAUDE.md.

## Open Questions

- Exact preset anchor windows (e.g. 61=C2..C7) — final values set in
  `keyboard_range.dart`; tests assert coverage/clamping, not absolute pixels.
- Should a wrong held key (active, not required, Wait Mode blocked) use the
  `error` pink? Out of scope here; the three states cover the user's request.
