## 1. Rust scaffolding & dependencies

- [ ] 1.1 Add `quick-xml` to `apps/music/rust/Cargo.toml` dependencies
- [ ] 1.2 Create `apps/music/rust/src/api/musicxml_core.rs` and
  `apps/music/rust/src/api/musicxml.rs`; register both in `api/mod.rs`
  (`musicxml_core` private, `musicxml` public) with the standard license header
- [ ] 1.3 Define the notation data model in `musicxml_core.rs`: `ScoreDocument`
  (with `staves`), `ScoreMeta`, `Attributes` (with `clefs: Vec<Clef>`), `Clef`,
  `TimeSignature`, `Measure` (with `directions`), `NoteEvent` (staff, voice,
  `position_divisions`, dots, tie flags, `tuplet`, `stem`, `beams`, `lyric`),
  `Pitch`, `Tuplet`, `StemDir`, `BeamState`, `Lyric`, `Direction`,
  `DirectionKind`, `System` — using `u32`/`i32`/`f64` (no `u64`) to avoid Dart
  `BigInt`

## 2. Streaming MusicXML parser (pure core)

- [ ] 2.1 Implement `parse(input: &[u8]) -> Result<ScoreDocument>` using a
  `quick-xml` event `Reader` (streaming, no DOM)
- [ ] 2.2 Extract metadata: work title and `composer` creator into `ScoreMeta`
  (absent → `None`, never a parse failure)
- [ ] 2.3 Extract multi-staff structure: `attributes/staves`; route each note /
  clef / direction to its `staff`/`number` (default staff 1 when absent)
- [ ] 2.4 Extract starting attributes: `divisions`, one `Clef` per staff (sign +
  line by clef number), key `fifths`, time (beats / beat-type); apply
  most-recent attributes to later notes
- [ ] 2.5 Maintain a per-measure `position_divisions` cursor honouring `backup`
  (subtract), `forward` (add), normal notes (advance), and `<chord/>` (no
  advance); stamp each note/direction with its position
- [ ] 2.6 Extract note events in document order: pitch (step/octave/alter) or
  rest, duration, note-type, `dot` count, accidental, voice, staff; flag chord
  members
- [ ] 2.7 Extract `tie` (start/stop) flags and `time-modification`
  (actual/normal) tuplet ratio per note
- [ ] 2.8 Extract `stem` direction, `beam` states, and `lyric` (syllabic + text)
  per note
- [ ] 2.9 Extract `direction` elements → `words`, `dynamics`, `wedge`
  (crescendo/diminuendo + stop), `metronome`, each with staff + position;
  ignore unknown direction types
- [ ] 2.10 Handle malformed XML and non-MusicXML input as a recoverable
  `Result::Err` (no panic)

## 3. Geometry engine (pure core)

- [ ] 3.1 Implement the non-linear spacing function `space(d) = UNIT *
  (d/divisions)^K` (K ≈ 0.6) with named constants (`UNIT`, `K`, `FLOOR`,
  `LEFT_PAD`)
- [ ] 3.2 Implement `min_width(measure)`: sum spacing over the *union* of
  distinct `position_divisions` across all staves/voices (shared columns, chord
  members add no width), clamp to `FLOOR`; store on `Measure`
- [ ] 3.3 Implement `layout_systems(doc, available_width) -> Vec<System>`:
  greedily pack measures by `min_width`, wrap on overflow, oversized measure
  alone, preserve order; each `System` carries the staff count (grand staff
  together)

## 4. Rust tests (≥ 80% on core)

- [ ] 4.1 Add MusicXML fixtures (minimal score, two-staff grand-staff with
  backup, chord, rest, tie, triplet, lyric, direction, malformed, non-MusicXML)
  and unit-test metadata, multi-staff routing, per-staff clefs, backup/forward
  positioning, notes/dots/accidentals, ties, tuplets, stems, beams, lyrics,
  directions, and bytes==string equivalence
- [ ] 4.2 Unit-test geometry: denser measure wider, sub-linear duration growth,
  shared columns across staves (no double-count), `FLOOR` respected, and system
  wrapping (overflow, oversized-alone, all-fit, grand staff together)
- [ ] 4.3 Run `cargo llvm-cov` and confirm `musicxml_core` keeps coverage ≥ 80%
  (extend the ignore regex to exclude the thin `musicxml.rs` wrapper)

## 5. FFI bridge

- [ ] 5.1 Implement `musicxml.rs` thin `#[frb]` wrappers:
  `parse_musicxml(bytes) -> ScoreDocument` and
  `layout_systems(doc, available_width) -> Vec<System>` delegating to the core
- [ ] 5.2 Run `flutter_rust_bridge_codegen generate`; commit generated
  `lib/src/rust/api/musicxml.dart` and bindings

## 6. Bundled score catalog (free / public-domain)

- [ ] 6.1 Acquire several uncompressed public-domain `.musicxml` files spanning
  levels and place them under `apps/music/assets/scores/<level>/` (≥1 per level:
  Beginner / Intermediate / Advanced); convert any `.mxl` to uncompressed
- [ ] 6.2 Add `apps/music/assets/scores/CREDITS.md` recording each file's source
  + license; register the `assets/scores/` tree (and `assets:` block) in
  `apps/music/pubspec.yaml`
- [ ] 6.3 Define `CatalogEntry { id, title, composer, assetPath, level }` and
  `enum PracticeLevel { beginner, intermediate, advanced }`; add an injectable
  `@riverpod scoreCatalog` returning the curated list (overridable in tests)

## 7. Flutter asset loading & notation state

- [ ] 7.1 Add an injectable `@riverpod scoreAssetSource` exposing
  `Future<Uint8List> load(String assetPath)` (default uses `rootBundle`),
  overridable in tests
- [ ] 7.2 Add a `@riverpod selectedScore` holding the chosen `CatalogEntry?`
- [ ] 7.3 Add a Freezed `NotationData { document, systems, availableWidth,
  error }` and a `@riverpod NotationNotifier` that watches `selectedScore`, loads
  its asset, calls the bridge, and computes systems; run `build_runner`

## 8. Library screen, navigation & partition rendering

- [ ] 8.1 Implement `apps/music/lib/screens/library_screen.dart` listing the
  catalog grouped by practice level (title + composer); make it the app `home`
  in `main.dart` (replacing the direct `PlayerScreen`)
- [ ] 8.2 On entry tap: set `selectedScore` and `Navigator.push` the
  piano/partition screen; back returns to the library
- [ ] 8.3 Implement `apps/music/lib/painters/partition_painter.dart`
  (`CustomPainter`) drawing the computed systems with both staves of the grand
  staff (treble + bass), measures, notes (with dots/accidentals), and at least
  lyrics + dynamics/words directions
- [ ] 8.4 Wire the Partition screen to watch `NotationNotifier`, supply available
  width (`LayoutBuilder`), and trigger re-layout on width change

## 9. Flutter tests (≥ 80%)

- [ ] 9.1 Unit-test `scoreCatalog` (≥1 entry per level; entry fields present) and
  `scoreAssetSource` override; unit-test `NotationNotifier` (selecting an entry →
  loads fake bytes → populates `NotationData`; error path sets `error`)
- [ ] 9.2 Widget test the library screen (entries grouped by level) and
  navigation (tapping an entry sets `selectedScore` and pushes the partition
  screen; back returns to the library)
- [ ] 9.3 Widget test the state→painter wiring (two-staff notation state renders
  both staves; new document re-renders) and add a `golden`-tagged Partition
  painter golden
- [ ] 9.4 Run `flutter test --coverage` and confirm line coverage ≥ 80%

## 10. Wrap-up

- [ ] 10.1 `melos run analyze`, `dart format`, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` all clean
- [ ] 10.2 `openspec validate musicxml-parsing-and-geometry --strict` passes
