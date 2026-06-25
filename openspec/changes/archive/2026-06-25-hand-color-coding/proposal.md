## Why

A learner reads two hands at once. With a single feedback colour for every note,
there is no way to tell at a glance which notes belong to the right hand and which
to the left — on the keyboard or on the score. Colour-coding the two hands makes
hands-separate reading and practice immediate across every view.

This change documents behaviour already implemented in the player; it is recorded
here to keep the specs in sync with the code.

## What Changes

- Introduce a **hand colour convention**: right hand (staff 1) = **blue**, left
  hand (staff 2+) = **amber** — distinct from the existing "correct" (green) and
  "extra key pressed" (purple) states.
- **Keyboard**: a key expected at the playhead is tinted by its hand (blue/amber)
  instead of a single expected colour; held-and-correct stays green; an extra
  held key stays purple.
- **Render modes**: notes are coloured by hand everywhere they are drawn —
  Synthesia falling notes, Staff scrolling note heads, and Partition note heads —
  with a brighter tint at the playhead and green once the key is held.

## Capabilities

### New Capabilities
- `hand-color-coding`: the right/left-hand colour convention and its application
  to the keyboard feedback and to the notes in every render mode.

### Modified Capabilities
<!-- None reworded. This refines the "expected" colour of keyboard-display's
     Three-State Key Feedback per hand without changing its three-state structure;
     kept additive to avoid an overlapping delta with the in-flight
     hand-selection-filter change, which restates that requirement. -->

## Impact

- **Theme**: `lib/theme/cymbra_theme.dart` — `handRight` (blue) and `handLeft`
  (amber) colours.
- **Keyboard**: `painters/piano_keyboard_painter.dart` (per-hand expected colour
  via a `leftHandNotes` subset) and `state/player_data.dart`
  (`expectedKeysForHand`).
- **Render modes**: `painters/synthesia_painter.dart`,
  `painters/staff_painter.dart`, `painters/partition_painter.dart` colour notes by
  `TimedNote.staff` / `NoteEvent.staff`.
- **Tests/goldens**: unit test for `expectedKeysForHand`; keyboard + Synthesia +
  Staff + Partition goldens refreshed.
- No Rust/engine or public-API change. Independent of the (visual-only) staff
  layout and overlay work.
