## Why

Today the score the app renders is hard-coded in Rust (`api/score.rs::demo_score`)
as a time-based melody — there is no way to load real sheet music. To move the
POC toward a usable score reader, we need to ingest standard **MusicXML**
(`.musicxml` / uncompressed `.xml`) files, extract their musical content, and
compute the geometry needed to draw them as engraved notation ("Partition"
mode). Scores can be very large, so parsing must stream the document rather than
hold a full DOM in memory.

## What Changes

- Add a **MusicXML ingestion path**: Flutter loads a raw `.musicxml`/`.xml`
  asset as bytes and hands them to Rust over the FRB bridge; Rust does the heavy
  parsing and geometry.
- Parse MusicXML in Rust with a **streaming (SAX-style) reader** (`quick-xml`,
  not a DOM parser like `roxmltree`) so multi-megabyte scores parse with bounded
  memory.
- Extract the full musical content needed to engrave a **piano partition**:
  **metadata** (title, composer); **multi-staff structure** (`staves`, per-note
  `staff`) so the grand staff's treble/bass are split correctly; **starting
  attributes** (per-staff clef, key signature, time signature, divisions);
  **measure time navigation** (`backup`/`forward`) so interleaved voices/staves
  land at the right time positions; and per-note detail — pitch, duration,
  note-type, dots, accidental, voice, staff, chord membership, **ties**,
  **tuplets** (`time-modification`), **stems**, **beams**, and **lyrics**.
- Extract **directions**: tempo/expression `words` (Andantino, dolce),
  `dynamics` (pp), `wedge` hairpins (crescendo/diminuendo), and `metronome`.
- Implement a **musical-geometry engine**: for each measure compute a minimum
  width (`min_width`) from its note density using a **non-linear spacing
  function**, then group measures into **systems** (justified staff lines) for a
  given available width.
- Expose clean, bridge-friendly Rust structs (`ScoreDocument`, `System`,
  `Measure`, `NoteEvent`, …) to Flutter.
- Add a **Partition render path** on the Flutter side: a new injectable
  asset-loading seam, Freezed notation state, and a `CustomPainter` that draws
  the computed systems/measures dynamically.
- Make the app **boot into a partition-library screen**: a catalog of several
  bundled, free/public-domain MusicXML scores grouped by **practice level**
  (Beginner / Intermediate / Advanced). Selecting a partition navigates to the
  piano/partition screen and loads that score.

This change is **additive**: the existing time-based player score (waterfall /
scrolling staff) is left untouched. Notation parsing/geometry is a parallel
capability.

## Capabilities

### New Capabilities
- `score-notation`: load and parse uncompressed MusicXML for a multi-staff piano
  partition into a structured score document (metadata, multi-staff attributes,
  measure time navigation via backup/forward, and notes with pitch, duration,
  dots, accidental, voice, staff, chord, ties, tuplets, stems, beams, lyrics, and
  directions), compute per-measure geometry (`min_width`) via a non-linear
  spacing function shared across staves, lay measures out into systems (grand
  staff kept together) for a given width, and surface the result to the Flutter
  Partition renderer.

- `score-library`: a catalog of bundled free/public-domain MusicXML scores
  tagged with a practice level; the app's start screen lists them grouped by
  level, and selecting one navigates to the piano/partition screen and loads
  that score.

### Modified Capabilities
<!-- None: the existing time-based `score` model and `midi` capability are unchanged. -->

## Impact

- **Rust**: new `apps/music/rust/src/api/musicxml.rs` (thin FFI seam: accepts
  bytes, returns typed structs) + `apps/music/rust/src/api/musicxml_core.rs`
  (pure, host-testable parsing + geometry, per the 80% coverage rule); new
  `quick-xml` dependency in `apps/music/rust/Cargo.toml`; `api/mod.rs` registers
  the module. Public FFI surface changes → **bridge regen required**
  (`flutter_rust_bridge_codegen generate`).
- **Dart**: new asset-loading seam (`scoreAssetSource` provider) and notation
  state (Freezed `NotationData` + `@riverpod` notifier); a new
  `partition_painter.dart`. A new **library screen** (`library_screen.dart`)
  becomes the app `home` (replacing the direct `PlayerScreen` entry in
  `main.dart`), backed by a `scoreCatalog` provider and a `selectedScore`
  provider; selecting an entry navigates to the partition screen. Several
  bundled `.musicxml` files under `apps/music/assets/scores/` (grouped by level)
  registered in `apps/music/pubspec.yaml`. Generated `*.g.dart`/`*.freezed.dart`
  produced by `build_runner`.
- **Tests/CI**: Rust unit tests on `musicxml_core` (parsing + spacing/layout);
  Flutter unit tests for the source seam, notifier, and a painter golden — both
  ecosystems keep line coverage ≥ 80%.
- **Users**: a real MusicXML sample renders in Partition mode; no change to the
  existing MIDI/player behavior.
