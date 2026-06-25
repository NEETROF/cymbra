## Context

Cymbra renders a selected piece in three modes — Synthesia waterfall, scrolling
Staff, and engraved Partition — all from one parsed `ScoreDocument`. Time-based
modes (Synthesia, Staff) consume a flattened `List<TimedNote>` derived by
`notationToTimedNotes`; Partition consumes the laid-out `systems` directly. Every
note already carries a `staff` field (MusicXML: staff 1 = right hand, staff 2 =
left hand), and a piano part declares `staves` (1 or 2). Player state lives in the
`@riverpod` `Player` notifier over a Freezed `PlayerData`; the same `requiredNotesAt`
gate feeds both Wait Mode and the keyboard's three-state feedback.

There is currently no way to practise one hand: all modes draw both staves and the
gate awaits both hands. This change adds a session-only hand selection that filters
notes out of display and out of the gate, and collapses the unselected staff in the
grand-staff modes.

Key files: `state/player_data.dart`, `state/player_notifier.dart`,
`state/notation_playback.dart`, `state/notation_notifier.dart`,
`painters/{synthesia,staff,partition}_painter.dart`, `screens/player_screen.dart`.

## Goals / Non-Goals

**Goals:**
- A `Hand { left, right, both }` selection in `PlayerData`, default `both`,
  settable via a notifier method; session-only (no persistence).
- A top-bar selector available in all three modes, matching the existing
  `_RangeChooser`/`_ModeToggle` conventions.
- Unselected-hand notes removed from every painter's input and from
  `requiredNotesAt` (so Wait Mode and expected/correct feedback ignore them).
- Unselected hand's staff fully collapsed (lines, clef, signatures, notes) in
  Staff and Partition modes; single-staff layout when one hand is shown.
- Filtering centralised so the three modes and the gate stay consistent.

**Non-Goals:**
- No audio synthesis or auto-accompaniment of the hidden hand (no MIDI output
  exists; derivation is visual only).
- No persistence of the choice across launches.
- No Rust/engine or public-API changes — the `staff` field already exists.
- No per-voice (finger/voice-level) filtering; granularity is per staff/hand.
- No change to how MusicXML maps staves to hands.

## Decisions

### Decision: Model hand as a single 3-value enum, not two booleans
Use `enum Hand { left, right, both }`. **Why:** the requirement is exactly three
mutually exclusive choices and maps cleanly to a `SegmentedButton`/popup. Two
booleans would admit a meaningless "neither" state and complicate the default.
**Alternative considered:** `Set<int> visibleStaves` — more general but
over-engineered for a two-staff piano model and harder to bind to a 3-way control.

### Decision: Filter centrally on `PlayerData`, derive painters/gate from it
Add helpers on `PlayerData`:
- `bool showsStaff(int staff)` — the single source of truth for visibility
  (`both` → always true; `right` → staff == 1; `left` → staff >= 2).
- `List<TimedNote> get visibleNotes` — `notes.where((n) => showsStaff(n.staff))`.
- Restrict `requiredNotesAt(t)` to `visibleNotes`.

Painters receive the already-filtered list (or the `Hand` plus `showsStaff`),
rather than each re-implementing the staff test. **Why:** the spec demands the
display set and the gate set move together; a single predicate guarantees that
and is trivially unit-testable without widgets or the native lib. **Alternative
considered:** filtering inside each painter independently — three copies of the
same rule, easy to drift, and the gate would need its own fourth copy.

### Decision: "Collapse" means recomputing the two-staff flag from visible notes
Both `staff_painter.dart` (`twoStaff = notes.any((n) => n.staff >= 2)`) and
`partition_painter.dart` (`_twoStaff = document.staves >= 2`) decide layout from
"are there two staves". Drive that decision from the *visible* set instead:
- Staff mode: compute `twoStaff` from the filtered notes, so hiding staff 2 (or
  staff 1) collapses to a single staff automatically and the remaining staff
  recentres.
- Partition mode: pass the selected `Hand` so the painter draws only the kept
  staff's lines/clef/armature/time and maps notes for that one staff. When only
  the bass (left) is shown, the single staff uses the bass clef and vertical
  origin.

**Why:** reuses the painters' existing single-vs-grand-staff code paths instead of
adding a new "hidden staff" drawing mode. **Trade-off:** the Partition painter
must parametrise its per-staff Y-origin/clef selection on which staff is kept,
rather than assuming "staff 1 on top, staff 2 below"; this is the main code change.

### Decision: Selector UI — match the existing top-bar pattern
Add a `_HandSelector` to `_TopBar` next to `_RangeChooser`. Use the
`PopupMenuButton` + `_Chip` pattern (Left / Right / Both with radio icons) for
consistency with the range chooser, or a compact `SegmentedButton` like
`_ModeToggle`. Bind to `notifier.setSelectedHands`. **Why:** zero new UI idioms;
the chip stays visible in all three modes since `_TopBar` is mode-independent.

### Decision: Default `both`, session-only
`@Default(Hand.both)` on the field; no storage plumbing. **Why:** matches the
user's choice and the existing behaviour of `mode`/`keyboardRange`, which are also
in-memory only. New pieces keep the current selection within a session (the
selection is not tied to the loaded document), and a relaunch resets to `both`.

## Risks / Trade-offs

- **Hiding a hand desyncs the playhead/scroll mapping** → Keep time derivation
  (`startMs`/`durationMs`) untouched; filtering only removes notes from drawing and
  the gate. The timeline and playhead remain anchored to the full piece so the
  visible hand stays in sync.
- **Auto-fit keyboard range computed from both hands looks empty for one hand** →
  Acceptable for v1 (range stays piece-wide). If it feels wrong, a follow-up can
  base auto-fit on `visibleNotes`; out of scope here to avoid coupling range logic
  to the filter.
- **Partition collapse touches the most complex painter** → Mitigate by
  parametrising the kept-staff origin/clef and adding a golden for each
  single-hand layout; the `both` path is unchanged so existing goldens still guard
  it.
- **Single-staff cross-hand notes** (e.g. a left-hand note written on staff 1) are
  filtered by staff, not by pitch, so an unusually notated note follows its staff,
  not its register → Accept; staff is the score's own hand assignment and matches
  how the engine already routes notes.
- **Coverage gate (≥80%)** → New helpers are pure and unit-tested; selector gets a
  widget test; collapsed layouts get goldens (excluded from the cross-platform
  gate but refreshed on the pinned platform).

## Migration Plan

Purely additive, in-memory, behind a defaulted field — no data migration and no
rollback steps. Shipping with `Hand.both` default reproduces today's behaviour
exactly; the feature is exercised only when the user changes the selector. Revert
is removing the field, helpers, selector and painter parametrisation.

## Open Questions

- Should the hand chip be hidden for single-staff pieces (where Left/Right is
  meaningless)? Lean yes — show the selector only when the document has ≥2 staves —
  but Both-default makes it harmless if always shown. Resolve during UI task.
- Selector affordance: `PopupMenuButton` (matches range chooser) vs
  `SegmentedButton` (matches mode toggle). Pick one in the UI task for top-bar
  space; not behaviour-affecting.
