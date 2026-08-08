# Tasks — partition-game-readability

## 1. Rust core — spacing floor + system cap

- [x] 1.1 Add a `MIN_COL` minimum column advance in `crates/musicxml-core`:
      clamp `space()`'s result to it; unit tests for the floor binding on
      16th/32nd columns and not binding on quarter+ columns.
- [x] 1.2 Lower `MAX_MEASURES_PER_SYSTEM` to 3; update layout tests
      (cap scenario, all-fit scenario) accordingly.
- [x] 1.3 `cargo fmt` + `cargo clippy` + `cargo llvm-cov` gate green for the
      workspace.

## 2. Flutter — score size preference

- [x] 2.1 Add `ScoreSize { small, medium, large }` (factors 0.85/1.0/1.2,
      default medium) and a `scoreSize` field on `PlayerPrefs`; encode/decode
      with safe fallback; setter on `PlayerPreferences`; unit tests
      (round-trip, unknown value falls back to medium).
- [x] 2.2 Add the score size chooser to the pre-play/settings modal (both
      column layouts), applied on Validate like the other settings; l10n
      strings (en + fr + other arb files); widget test.

## 3. Flutter — Partition view rework

- [x] 3.1 `PartitionPainter`: make the staff space an instance parameter
      (default 12.0 × size factor); keep every derived metric scaling off it.
- [x] 3.2 `_PartitionView`: request layout at `engraveWidth / factor` and paint
      with the scaled staff space so justification restores exact size; remove
      `_kGaugeGutter` (full-width engraving).
- [x] 3.3 Look-ahead auto-scroll: anchor the current system's top at ~30% of
      the viewport; delete `_NextLineOverlay` + `_buildNextLineOverlay` and
      their tests; adjust/replace widget tests for the new scroll target.
- [x] 3.4 Played-system dimming in `PartitionPainter` (systems before the
      cursor's system at reduced opacity; none when no playhead); tests.
- [x] 3.5 Current-measure wash behind the active measure (subtle accent fill,
      no wash when stopped); tests.

## 4. Flutter — top-bar score chip

- [x] 4.1 New `ScoreChip` widget (tier dot + sync % + combo, tier-up
      shake/firework, hidden when inactive, compact variant on phone); widget
      tests.
- [x] 4.2 Mount the chip in `_TopBar`; remove the floating `ScoringGauge` from
      `ScoringOverlay` in all modes (keep hit sparks); delete/repurpose
      `ScoringGauge` and update its tests.
- [x] 4.3 Portée/Staff view: pass `noteScale` and scaled `lookAheadMs` from the
      score size factor; widget test.

## 5. Gates & docs

- [x] 5.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean;
      `flutter test --coverage --exclude-tags golden` green with coverage ≥ 80%.
- [x] 5.2 Rust gates green (fmt, clippy, llvm-cov ≥ 80%).
- [x] 5.3 `openspec validate partition-game-readability --strict` passes.
- [x] 5.4 Note the back-office stale-wasm gotcha in the PR description
      (`yarn gen:wasm` needed to see the new layout in BO).
