## 1. Rust scaffolding & dependencies

- [x] 1.1 Add `quick-xml` to `apps/music/rust/Cargo.toml` dependencies
- [x] 1.2 Create `apps/music/rust/src/api/musicxml_core.rs` and
  `apps/music/rust/src/api/musicxml.rs`; register both in `api/mod.rs`
  (`musicxml_core` private, `musicxml` public) with the standard license header
- [x] 1.3 Define the notation data model in `musicxml_core.rs`: `ScoreDocument`
  (with `staves`), `ScoreMeta`, `Attributes` (with `clefs: Vec<Clef>`), `Clef`,
  `TimeSignature`, `Measure` (with `directions`), `NoteEvent` (staff, voice,
  `position_divisions`, dots, tie flags, `tuplet`, `stem`, `beams`, `lyric`),
  `Pitch`, `Tuplet`, `StemDir`, `BeamState`, `Lyric`, `Direction`,
  `DirectionKind`, `System` — using `u32`/`i32`/`f64` (no `u64`) to avoid Dart
  `BigInt`

## 2. Streaming MusicXML parser (pure core)

- [x] 2.1 Implement `parse(input: &[u8]) -> Result<ScoreDocument>` using a
  `quick-xml` event `Reader` (streaming, no DOM)
- [x] 2.2 Extract metadata: work title and `composer` creator into `ScoreMeta`
  (absent → `None`, never a parse failure)
- [x] 2.3 Extract multi-staff structure: `attributes/staves`; route each note /
  clef / direction to its `staff`/`number` (default staff 1 when absent)
- [x] 2.4 Extract starting attributes: `divisions`, one `Clef` per staff (sign +
  line by clef number), key `fifths`, time (beats / beat-type); apply
  most-recent attributes to later notes
- [x] 2.5 Maintain a per-measure `position_divisions` cursor honouring `backup`
  (subtract), `forward` (add), normal notes (advance), and `<chord/>` (no
  advance); stamp each note/direction with its position
- [x] 2.6 Extract note events in document order: pitch (step/octave/alter) or
  rest, duration, note-type, `dot` count, accidental, voice, staff; flag chord
  members
- [x] 2.7 Extract `tie` (start/stop) flags and `time-modification`
  (actual/normal) tuplet ratio per note
- [x] 2.8 Extract `stem` direction, `beam` states, and `lyric` (syllabic + text)
  per note
- [x] 2.9 Extract `direction` elements → `words`, `dynamics`, `wedge`
  (crescendo/diminuendo + stop), `metronome`, each with staff + position;
  ignore unknown direction types
- [x] 2.10 Handle malformed XML and non-MusicXML input as a recoverable
  `Result::Err` (no panic)

## 3. Geometry engine (pure core)

- [x] 3.1 Implement the non-linear spacing function `space(d) = UNIT *
  (d/divisions)^K` (K ≈ 0.6) with named constants (`UNIT`, `K`, `FLOOR`,
  `LEFT_PAD`)
- [x] 3.2 Implement `min_width(measure)`: sum spacing over the *union* of
  distinct `position_divisions` across all staves/voices (shared columns, chord
  members add no width), clamp to `FLOOR`; store on `Measure`
- [x] 3.3 Implement `layout_systems(doc, available_width) -> Vec<System>`:
  greedily pack measures by `min_width`, wrap on overflow, oversized measure
  alone, preserve order; each `System` carries the staff count (grand staff
  together)

## 4. Rust tests (≥ 80% on core)

- [x] 4.1 Add MusicXML fixtures (minimal score, two-staff grand-staff with
  backup, chord, rest, tie, triplet, lyric, direction, malformed, non-MusicXML)
  and unit-test metadata, multi-staff routing, per-staff clefs, backup/forward
  positioning, notes/dots/accidentals, ties, tuplets, stems, beams, lyrics,
  directions, and bytes==string equivalence
- [x] 4.2 Unit-test geometry: denser measure wider, sub-linear duration growth,
  shared columns across staves (no double-count), `FLOOR` respected, and system
  wrapping (overflow, oversized-alone, all-fit, grand staff together)
- [x] 4.3 Run `cargo llvm-cov` and confirm `musicxml_core` keeps coverage ≥ 80%
  (extend the ignore regex to exclude the thin `musicxml.rs` wrapper)

## 5. FFI bridge

- [x] 5.1 Implement `musicxml.rs` thin `#[frb]` wrappers:
  `parse_musicxml(bytes) -> ScoreDocument` and
  `layout_systems(doc, available_width) -> Vec<System>` delegating to the core
- [x] 5.2 Run `flutter_rust_bridge_codegen generate`; commit generated
  `lib/src/rust/api/musicxml.dart` and bindings

## 6. Bundled score catalog (free / public-domain)

- [x] 6.1 Acquire several uncompressed public-domain `.musicxml` files spanning
  levels and place them under `apps/music/assets/scores/<level>/` (≥1 per level:
  Beginner / Intermediate / Advanced); convert any `.mxl` to uncompressed
- [x] 6.2 Add `apps/music/assets/scores/CREDITS.md` recording each file's source
  + license; register the `assets/scores/` tree (and `assets:` block) in
  `apps/music/pubspec.yaml`
- [x] 6.3 Define `CatalogEntry { id, title, composer, assetPath, level }` and
  `enum PracticeLevel { beginner, intermediate, advanced }`; add an injectable
  `@riverpod scoreCatalog` returning the curated list (overridable in tests)

## 7. Flutter asset loading & notation state

- [x] 7.1 Add an injectable `@riverpod scoreAssetSource` exposing
  `Future<Uint8List> load(String assetPath)` (default uses `rootBundle`),
  overridable in tests
- [x] 7.2 Add a `@riverpod selectedScore` holding the chosen `CatalogEntry?`
- [x] 7.3 Add a Freezed `NotationData { document, systems, availableWidth,
  error }` and a `@riverpod NotationNotifier` that watches `selectedScore`, loads
  its asset, calls the bridge, and computes systems; run `build_runner`

## 8. Library screen, navigation & partition rendering

- [x] 8.1 Implement `apps/music/lib/screens/library_screen.dart` listing the
  catalog grouped by practice level (title + composer); make it the app `home`
  in `main.dart` (replacing the direct `PlayerScreen`)
- [x] 8.2 On entry tap: set `selectedScore` and `Navigator.push` the
  piano/partition screen; back returns to the library
- [x] 8.3 Implement `apps/music/lib/painters/partition_painter.dart`
  (`CustomPainter`) drawing the computed systems with both staves of the grand
  staff (treble + bass), measures, notes (with dots/accidentals), and at least
  lyrics + dynamics/words directions
- [x] 8.4 Wire the Partition screen to watch `NotationNotifier`, supply available
  width (`LayoutBuilder`), and trigger re-layout on width change

## 9. Flutter tests (≥ 80%)

- [x] 9.1 Unit-test `scoreCatalog` (≥1 entry per level; entry fields present) and
  `scoreAssetSource` override; unit-test `NotationNotifier` (selecting an entry →
  loads fake bytes → populates `NotationData`; error path sets `error`)
- [x] 9.2 Widget test the library screen (entries grouped by level) and
  navigation (tapping an entry sets `selectedScore` and pushes the partition
  screen; back returns to the library)
- [x] 9.3 Widget test the state→painter wiring (two-staff notation state renders
  both staves; new document re-renders) and add a `golden`-tagged Partition
  painter golden
- [x] 9.4 Run `flutter test --coverage` and confirm line coverage ≥ 80%

## 10. Wrap-up

- [x] 10.1 `melos run analyze`, `dart format`, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` all clean
- [x] 10.2 `openspec validate musicxml-parsing-and-geometry --strict` passes

## 11. Host the loaded partition in the existing player screen

> Revision: the first pass routed selection to a standalone notation-only
> `PartitionScreen`, which orphaned the existing player (keyboard / waterfall /
> MIDI). This section reuses `PlayerScreen` instead and derives playback timing
> so all render modes show the selected piece. Tasks 8.2/8.4 and 9.2/9.3 are
> superseded by the items below; 8.3 (the `PartitionPainter`) is retained.

- [x] 11.1 Add a pure, host-tested helper `notationToTimedNotes(ScoreDocument)`
  → `(notes, songEndMs, bpm)`: MIDI pitch from step/octave/alter, start/duration
  in ms from running division position × `ms_per_division`, BPM from a
  `metronome` direction (else default), chords share onset, rests skipped
- [x] 11.2 Add `RenderMode.partition`; have the `Player` notifier load the
  selected score's notation (via the asset source + notation engine seams),
  derive its `TimedNote`s, and fall back to the demo score when nothing is
  selected
- [x] 11.3 Render the Partition mode inside `PlayerScreen` (scrollable
  `PartitionPainter` from the notation state), add it to the mode toggle, and
  keep the keyboard / MIDI / transport in all modes
- [x] 11.4 Route the library tap to `PlayerScreen` (not a standalone screen);
  remove the orphaned `PartitionScreen`; keep `LibraryScreen` as `home`
- [x] 11.5 Update tests: timing-derivation unit tests; player loads the selected
  MusicXML and exposes all three modes; library navigation lands on the player;
  refresh/replace the partition widget/golden tests; coverage stays ≥ 80%
- [x] 11.6 Re-run wrap-up gates (analyze, format, clippy, fmt, custom_lint,
  coverage) and `openspec validate --strict`

## 12. SMuFL/Bravura engraving glyphs

> Revision: hand-drawn note heads/flags read poorly (mis-rendered eighth notes,
> "tent" beams). Adopt SMuFL with the Bravura reference font for real engraving.

- [x] 12.1 Bundle `assets/fonts/Bravura.otf` (SIL OFL, + `OFL.txt`) and register
  it as a `Bravura` font family in `pubspec.yaml`
- [x] 12.2 Add a `Smufl` helper (`lib/painters/smufl.dart`): glyph codepoints
  (heads, clefs, flags, accidentals, rests, dynamics, dots), Bravura engraving
  metrics/anchors, and a baseline-anchored glyph-draw helper (staff-space units)
- [x] 12.3 Rewrite `PartitionPainter` to use glyphs for heads/clefs/flags/
  accidentals/rests/dynamics, with stems/beams/ledgers stroked at Bravura
  thicknesses and stem-anchor attachment; beamed groups → one straight beam with
  varying stem lengths
- [x] 12.4 Use SMuFL glyphs in the grand-staff `StaffPainter` (heads + clefs),
  with eighth/sixteenth flags derived from note duration
- [x] 12.5 Load Bravura in golden tests (FontLoader) and refresh goldens; add a
  beamed-group golden to verify beaming
- [x] 12.6 Re-run gates (analyze, format, custom_lint, coverage) and
  `openspec validate --strict`

## 13. Engraving fidelity (review feedback)

> Iterative fixes from comparing the rendered Arabesque against an engraved
> edition: signatures, clef changes, tuplet/tie/slur drawing, layout density.

- [x] 13.1 Draw the **key signature** (armature) on every system and the **time
  signature** on the first, as SMuFL glyphs, in both Partition and Staff modes
- [x] 13.2 **Clef changes**: keep the initial clef per staff and record
  per-measure clef changes on `NotationMeasure.clefs` (bridge regenerated); the
  Partition draws the clef in effect per measure (header + mid-system change) and
  positions notes from it
- [x] 13.3 **Slur extraction**: parse `<slur>` into `NoteEvent.slur_start/stop`
  (bridge regenerated)
- [x] 13.4 Render **tuplet numbers**, **ties** (short belly-down arc) and
  **slurs** (phrase arc clearing the highest note) in the Partition; tune tuplet
  number / dynamics glyph sizes
- [x] 13.5 Cap measures per system (`MAX_MEASURES_PER_SYSTEM`) so dense scores
  stay legible on a wide viewport
- [x] 13.6 **Staff mode**: carry staff + beam states + clef in effect on
  `TimedNote`; lay out a grand staff with beamed groups and clef-aware note
  positions (fix an octave-off bug in the clef reference scale)
- [x] 13.7 Re-run gates (analyze, format, clippy, fmt, custom_lint, coverage) and
  `openspec validate --strict`
