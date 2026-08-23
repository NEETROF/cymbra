# Tasks — add-repeat-unrolling

## 1. Model + parser (crates/musicxml-core)

- [x] 1.1 Extend the model: `RepeatMark` (forward / backward+times) on the
      measure's left/right barline, `Volta { numbers, kind }`,
      `measure_repeat_of: Option<u32>` (+ slash count), and jump markers
      (segno/coda/to-coda, D.C./D.S./Fine words with their `<sound>`
      semantics) — written order untouched.
- [x] 1.2 Parse `<barline>`/`<repeat>`, `<ending>`,
      `<measure-style>/<measure-repeat>` (transitive `%` resolution with a
      depth cap), `<segno>`/`<coda>` and `<sound>` jump attributes.
- [x] 1.3 Unit tests: each construct extracted; chained/malformed `%` bounded;
      scores without repeats parse byte-identically to today (model defaults).

## 2. Playback order (crates/musicxml-core)

- [x] 2.1 Implement `play_order(&ScoreDocument) -> Vec<PlayedMeasure>`
      (written_index + pass): repeats with `times`, volta selection per pass
      (passes beyond the listed brackets replay the last), one D.C./D.S. jump
      (repeats not re-taken unless `<sound>` says so, end at Fine/coda).
- [x] 2.2 Safety caps (≤ 8× written measures, absolute ceiling) and 1:1
      written-order fallback on any inconsistency.
- [x] 2.3 Unit + property tests: simple repeat, times=3, voltas (incl. 1.–3.
      lists and discontinue), D.C. al Fine, D.S. al Coda, mismatched repeats →
      fallback, cap enforcement; fuzz random repeat marks never loop or panic.

## 3. Rust derivation + server surfaces

- [x] 3.1 `playback::schedule()` iterates `play_order`, accumulates time per
      played slot, replays `%` content, merges ties on played adjacency only.
- [x] 3.2 Expose the played-slot table + written-measure mapping in
      `PlaybackSchedule`; keep the no-repeat path identical (regression tests).
- [ ] 3.3 Verify the backend audio-preview render job picks up the unrolled
      schedule (render a repeat fixture; clip duration covers the unroll).
- [ ] 3.4 Coverage: keep `cargo llvm-cov --workspace --fail-under-lines 80`
      green (logic in host-testable modules).

## 4. Bridge + app derivation (apps/music)

- [x] 4.1 Mirror the new model/play-order types in
      `apps/music/rust/src/api/musicxml.rs` and regenerate the bridge
      (`flutter_rust_bridge_codegen generate`).
- [x] 4.2 `notationToTimedNotes` walks the play order: repeated passes,
      volta selection, `%` replay, tie merge on played adjacency; tie
      continuations (render channel) follow the played slots.
- [x] 4.3 `measureStartMs` becomes the played-slot table + `writtenMeasureOf`
      mapping in `DerivedPlayback`/`PlayerData`; armure table aligned to
      played slots; `songEndMs`/start-trim/rewind operate on played slots.
- [x] 4.4 Practice: range selection stays written-measure-based; a selective
      run plays the chosen written measures linearly once (no unroll), `%`
      content audible; update `practice_range` logic + tests.
- [x] 4.5 Unit tests: unrolled timeline (repeat ×2, voltas, `%`), mapping
      tables, Wait-Mode gate across a backward repeat (re-attack expected),
      no-repeat scores byte-identical.

## 5. App engraving (apps/music)

- [x] 5.1 Partition painter: repeat barlines (thick/thin + dots), volta
      brackets + numbers, `%` sign, segno/coda/D.C./D.S./Fine glyphs at
      written positions.
- [x] 5.2 Scrolling staff: bar lines from played slots (a repeated measure
      scrolls past once per pass, repeat barlines drawn each pass, only the
      played volta shown); Partition cursor highlights the written measure of
      the current played slot (backward jump covered by a widget test).
- [x] 5.3 Painter tests via the hit index + goldens refresh on the pinned
      platform.

## 6. Notation help (apps/music)

- [x] 6.1 New `SymbolDescriptor` kinds (repeat barline, volta, measure-repeat,
      segno, coda, jump words); both painters record their regions.
- [x] 6.2 Help sheet + glossary entries for each family; copy added to
      **all four** ARB locales in the same commit (run the drift check from
      the flutter skill).
- [x] 6.3 Widget tests: long-press each new symbol resolves to its help; ARB
      alignment test/check green.

## 7. Back office (apps/back-office)

- [x] 7.1 Rebuild the wasm (`yarn gen:wasm`) and surface the new model in
      `lib/notation` types.
- [x] 7.2 Painter: draw repeat barlines, volta brackets, `%`, segno/coda/words
      in the rendered score (written order).
- [x] 7.3 Play preview: confirm the worker consumes the unrolled schedule
      (duration + repeat audible on a fixture); Vitest on the schedule
      mapping; e2e spot-check with a repeat fixture in the seam.

## 8. Validation + release

- [ ] 8.1 `openspec validate add-repeat-unrolling --strict` passes.
- [ ] 8.2 Full gates: melos analyze/format, custom_lint, cargo fmt/clippy,
      Rust + Flutter coverage ≥ 80 %, BO typecheck/test.
- [ ] 8.3 Manual pass on device: a real repeat-carrying score (waterfall,
      Portée, Partition, Wait Mode, practice range, help long-press) + BO
      render/Play preview side-by-side.
- [ ] 8.4 Regenerate the audio preview of one repeat-carrying catalog piece
      via the back-office and verify the clip.
