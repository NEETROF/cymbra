## Context

The Cymbra POC currently renders a hard-coded, time-based score produced by
`apps/music/rust/src/api/score.rs::demo_score()` and consumed by the Riverpod
`Player` notifier as a flat list of `TimedNote`s (waterfall / scrolling-staff
painters). There is no path to load real sheet music, and the existing model is
purely temporal (millisecond timestamps), not notation geometry.

This change adds the ability to load an uncompressed **MusicXML** file, parse it
in Rust, compute engraving geometry (per-measure minimum width, system
breaking), and render it in a new "Partition" mode. The repo's working
agreements constrain the approach:

- **Coverage ≥ 80% both ecosystems.** Pure, host-testable logic must live in a
  `*_core.rs` module (like `api/midi_core.rs`); the thin FFI/IO wrapper is
  coverage-excluded. Flutter native FFI must sit behind an injectable provider
  seam (like `midiServiceProvider`) so state/widgets test without the native lib.
- **State = Riverpod 2 + Freezed (codegen).** No `ChangeNotifier`/`setState`;
  dependencies are providers overridden in tests.
- **Public Rust API changes require bridge regen** (`flutter_rust_bridge_codegen
  generate`).

The user explicitly requires streaming parsing ("ssax") because scores can be
very large.

## Goals / Non-Goals

**Goals:**
- Load an uncompressed MusicXML asset in Flutter and parse it in Rust via the
  bridge, returning typed structs.
- Stream the XML (bounded memory) rather than building a DOM.
- Read a full **piano partition**: metadata; multi-staff structure
  (`staves`/`staff`); per-staff clef, key, time, divisions; measure time
  navigation (`backup`/`forward`); and per-note detail — pitch, duration, dots,
  accidental, voice, staff, chord, ties, tuplets (`time-modification`), stems,
  beams, lyrics — plus directions (words, dynamics, wedge, metronome).
- Compute `min_width` per measure with a non-linear spacing function shared
  across staves, and group measures into systems (grand staff together) for a
  given width.
- Render the result with a new Partition `CustomPainter`, driven by Freezed
  notation state.
- Keep all of it covered ≥ 80% by isolating pure logic.

**Non-Goals:**
- Compressed `.mxl` (zip) support — uncompressed only for this POC.
- **Multi-part / orchestral** scores — a single (piano) part with up to two
  staves is the target; the model allows more staves but layout is tuned for one
  part.
- SMuFL/font glyph engraving fidelity, slurs, articulations, ornaments,
  grace notes, multiple verses of lyrics, repeats/voltas — not modelled here.
- Replacing or unifying with the existing time-based player score; this is
  additive and parallel.
- Audio playback of the parsed score, or deriving MIDI timing from it.
- Page pagination / vertical system stacking beyond returning the ordered systems.

## Decisions

### Decision: `quick-xml` streaming reader over `roxmltree` DOM
The parser uses `quick-xml`'s pull/event `Reader` (SAX-style) so a large
document is consumed as a stream of `Event`s with bounded memory.
- **Why:** The user requires streaming ("ssax"); scores can be multi-MB.
  `roxmltree` builds a full in-memory tree — rejected on memory grounds.
  `xml-rs`/`sax` crates are slower and less maintained than `quick-xml`.
- **Alternative considered:** `roxmltree` (simpler ancestor/descendant queries)
  — rejected: whole-DOM allocation conflicts with the large-file requirement.

### Decision: split `musicxml_core.rs` (pure) vs `musicxml.rs` (FFI seam)
Mirror the `midi_core.rs` / `midi.rs` pattern. `musicxml_core.rs` holds the pure
functions — `parse(bytes) -> Result<ScoreDocument>`, the spacing function, and
the system-layout function — all unit-tested on the host. `musicxml.rs` is the
thin `#[frb]` wrapper exposing `parse_musicxml(bytes) -> ScoreDocument` and
`layout_systems(doc, available_width) -> Vec<System>` to Flutter.
- **Why:** Satisfies the 80% gate (the core is fully testable without FFI; the
  wrapper is excluded via the `frb_generated|/lib\.rs|/midi\.rs`-style regex,
  extended for `musicxml.rs`).
- **Alternative considered:** one module — rejected: couples pure logic to the
  bridge and hurts coverage isolation.

### Decision: new notation data model, separate from the time-based `Score`
Introduce notation structs (full piano fidelity) rather than overloading
`score.rs::Score`:
- `ScoreDocument { meta: ScoreMeta, staves: u32, attributes: Attributes, measures: Vec<Measure> }`
- `ScoreMeta { title: Option<String>, composer: Option<String> }`
- `Attributes { divisions: u32, clefs: Vec<Clef>, key_fifths: i32, time: TimeSignature }`
  — `clefs` is one entry per staff (`Clef { staff: u32, sign: char, line: i32 }`)
- `Measure { index: u32, notes: Vec<NoteEvent>, directions: Vec<Direction>, min_width: f64 }`
- `NoteEvent { staff: u32, voice: u32, position_divisions: u32, pitch: Option<Pitch>, is_rest: bool, is_chord: bool, duration_divisions: u32, note_type: Option<String>, dots: u32, accidental: Option<String>, tie_start: bool, tie_stop: bool, tuplet: Option<Tuplet>, stem: Option<StemDir>, beams: Vec<BeamState>, lyric: Option<Lyric> }`
  — `position_divisions` is the running time position (set via backup/forward) so
  staves stay aligned.
- `Pitch { step: char, octave: i32, alter: i32 }`
- `Tuplet { actual: u32, normal: u32 }`; `StemDir { Up, Down }`;
  `BeamState { Begin, Continue, End }`; `Lyric { syllabic: Option<String>, text: String }`
- `Direction { staff: u32, position_divisions: u32, kind: DirectionKind }` where
  `DirectionKind` ∈ `Words(String)`, `Dynamics(String)`, `Wedge { crescendo: bool, stop: bool }`, `Metronome(...)`
- `System { measures: Vec<u32>, staves: u32 }` (measure indices in order; carries
  the staff count so a grand staff lays out together)
- **Why:** The temporal `Score` (ms-based, BPM-driven) and engraving notation are
  genuinely different concerns; keeping them separate avoids breaking the player.
- **Alternative considered:** extend `Score`/`Note` with geometry — rejected:
  conflates time and layout, risks regressing the working player.

### Decision: explicit measure time cursor honouring `backup`/`forward`
The parser keeps a `position_divisions` cursor per measure. A normal note
advances it by its `duration`; a `<chord/>` note does not advance it; `<backup>`
subtracts and `<forward>` adds their `duration`. Each note/direction records the
cursor value at emission.
- **Why:** A piano part interleaves staves/voices by writing one staff, then
  `backup`-ing to the bar start to write the other. Without honouring backup,
  staff-2 notes would be appended after staff-1 instead of aligned under it —
  every bass note would be at the wrong time. This is the single most important
  correctness point for multi-staff scores.
- **Alternative considered:** infer staff timing from order alone — rejected:
  MusicXML explicitly models it with backup/forward; ignoring them is wrong.

### Decision: spacing keyed on the union of time positions across staves
`min_width` sums the non-linear spacing over the *set* of distinct
`position_divisions` in the measure (across both staves and all voices), not per
note. Notes sharing a position (chords, or a treble+bass note on the same beat)
collapse to one spacing column, so the grand staff stays vertically aligned.
- **Why:** Engraving aligns simultaneous events vertically; spacing is a property
  of time columns, not individual notes.

### Decision: non-linear spacing = power-law per note, summed, with a floor
`min_width(measure) = max(FLOOR, LEFT_PAD + Σ_over_time_positions space(d))` where
`space(d) = UNIT * (d / divisions) ^ K` with `K ≈ 0.6` (sub-linear, à la
standard engraving / Gould). Chord members share a time position and add no
width. This makes denser measures wider while longer durations grow sub-linearly.
- **Why:** Matches the requested "fonction d'espacement non linéaire"; the
  exponent < 1 yields the perceptually correct compression of long notes.
- **Alternative considered:** linear `space(d) ∝ d` — rejected by the spec
  (sub-linear required); logarithmic — harder to tune for a POC.

### Decision: Rust owns geometry; Flutter passes available width
`min_width` is computed at parse time; `layout_systems` is a separate call taking
the viewport's available width (from Flutter `LayoutBuilder`/`size`). Flutter
re-requests layout when the width changes meaningfully.
- **Why:** The prompt lists `System` as a Rust-returned struct; geometry belongs
  in the engine. Separating parse from layout avoids re-parsing on resize.

### Decision: Flutter seam = `scoreAssetSource` provider + Freezed `NotationData`
A `@riverpod` `scoreAssetSource` exposes `Future<Uint8List> load(String assetPath)`
(default impl uses `rootBundle`; overridden with in-memory bytes in tests). A
`@riverpod NotationNotifier` calls the source, invokes the bridge, stores a
Freezed `NotationData { document, systems, availableWidth, error }`. A new
`PartitionPainter` consumes it.
- **Why:** Same injectable-seam pattern as `midiServiceProvider`; keeps widgets
  and state testable without `rootBundle` or the native lib.

### Decision: app boots into a library screen; selection drives the partition screen
`main.dart` `home` changes from `PlayerScreen` to a new `LibraryScreen`. The
library lists the catalog grouped by practice level; tapping an entry sets a
`selectedScore` provider and `Navigator.push`es the partition screen.
- Catalog model: `CatalogEntry { id, title, composer, assetPath, level }` with
  `enum PracticeLevel { beginner, intermediate, advanced }`.
- `@riverpod scoreCatalog` returns the curated `List<CatalogEntry>` (const for the
  POC), overridable in tests.
- `@riverpod selectedScore` holds the chosen `CatalogEntry?`; the partition
  screen's `NotationNotifier` watches it to know which `assetPath` to load via
  `scoreAssetSource`.
- **Why:** Mirrors the project's provider-seam pattern (catalog and selection are
  injectable, so the library and partition screens are testable without the
  bundle or native lib). Navigator keeps a normal back-stack to the library.
- **Alternatives considered:** (a) a manifest JSON asset parsed at runtime —
  more flexible but adds a parse/IO path to test; deferred. (b) passing the entry
  as a route argument instead of a provider — works, but a `selectedScore`
  provider keeps the notifier pure-Riverpod and easy to override in tests.

### Decision: curate free / public-domain MusicXML across three levels
Bundle several uncompressed `.musicxml` files under
`apps/music/assets/scores/<level>/`, at least one per level. Candidate
public-domain / openly-licensed sources (verify license per file at apply time):
- **Beginner:** Bach/Petzold *Minuet in G* (BWV Anh.114), Beethoven *Ode to Joy*
  theme, *Twinkle, Twinkle*.
- **Intermediate:** Satie *Gymnopédie No.1*, Schumann *The Wild Horseman*
  (Op.68), Clementi *Sonatina Op.36 No.1*.
- **Advanced:** Chopin *Prelude Op.28 No.7/No.4*, Debussy *Clair de Lune*
  excerpt.
- Sources: OpenScore / MuseScore public-domain (CC0), the Mutopia Project
  (public domain). Each bundled file must be uncompressed MusicXML (convert
  `.mxl` → `.musicxml` if needed) and small enough to ship.
- **Why:** "free real partitions with different practice levels" — real pieces,
  legally redistributable, spanning difficulty.
- **Risk:** licensing must be confirmed per file before committing (see Risks).

## Risks / Trade-offs

- **MusicXML is large and irregular** → For the POC, parse only the subset in the
  spec (single part, the listed elements); ignore unknown elements rather than
  failing. Document the supported subset.
- **`quick-xml` event handling is verbose/error-prone** → Centralize parsing in
  one well-tested `musicxml_core` state machine; cover with fixtures (minimal
  valid score, chord, rest, malformed, non-MusicXML).
- **Spacing constants are aesthetic guesses** → Encode `K`, `UNIT`, `FLOOR`,
  `LEFT_PAD` as named constants; assert *relative* properties in tests (denser >
  sparser, sub-linear, floor respected) rather than brittle absolute pixels.
- **Bridge regen churn / `BigInt` mapping** → Use `u32`/`i32`/`f64` (not `u64`)
  for notation fields to avoid Dart `BigInt`, matching the `TimedNote` lesson in
  `player_data.dart`. Regen the bridge and commit generated Dart.
- **Painter golden tests are platform-sensitive** → Tag goldens `golden`
  (excluded from the cross-platform gate per CLAUDE.md); keep a non-golden widget
  test for the state→painter wiring.
- **Bundled-score licensing** → Only ship files confirmed public-domain or
  openly licensed (CC0); record each file's source + license in a short
  `assets/scores/CREDITS.md`. When in doubt, drop the file rather than ship it.
- **Real scores stress the parser** → Real public-domain files exercise far more
  MusicXML than hand-made fixtures; treat the first failing real file as a parser
  bug to fix (or an explicitly-ignored unsupported element), not a reason to
  widen scope silently.
- **Resize thrashing layout calls** → Debounce / only recompute `layout_systems`
  when available width changes beyond a threshold.

## Migration Plan

Additive, no data migration. Steps:
1. Add `quick-xml` to `apps/music/rust/Cargo.toml`; create `musicxml_core.rs` +
   `musicxml.rs`; register in `api/mod.rs`.
2. Implement + unit-test pure parsing and geometry in `musicxml_core`.
3. Regenerate the bridge; add the `musicxml.rs` exclusion to the Rust coverage
   regex.
4. Add the curated `.musicxml` files under `assets/scores/<level>/`, a
   `CREDITS.md`, and register the `assets/scores/` tree in `pubspec.yaml`.
5. Add the Dart seam, Freezed `NotationData`, notifier, and `PartitionPainter`;
   run `build_runner`.
6. Add `scoreCatalog` + `selectedScore` providers and `LibraryScreen`; switch
   `main.dart` `home` to `LibraryScreen`; wire selection → partition screen.
   Add Flutter unit/widget tests (catalog, library, navigation, notifier) and a
   tagged golden.
7. Verify `cargo llvm-cov` and `flutter test --coverage` both ≥ 80%.

**Rollback:** the feature is isolated behind a new render mode and new modules;
reverting the commit removes it with no impact on the existing player.

## Open Questions

- Final spacing exponent `K` and unit constants — tune against the sample asset;
  start at `K = 0.6`.
- Should `layout_systems` justify (stretch measures to fill the line) for the
  POC, or left-align with `min_width`? Proposed: left-align first, justify later.
- Which sample score to bundle (public-domain) — pick a short single-part piece.
