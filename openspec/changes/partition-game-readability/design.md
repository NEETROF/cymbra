# Design — partition-game-readability

## Context

The Partition (vertical) render mode engraves the score with
`PartitionPainter` over systems computed by `cymbra_musicxml_core::layout_systems`.
Current behaviour under playback:

- `_followCursor` centres the current system in the viewport; the next line
  therefore often falls below the fold, which motivated the `_NextLineOverlay`
  "NEXT" box (top-left, full system height) with its visibility heuristics.
- `ScoringOverlay` floats an 88×120 `ScoringGauge` at the viewport top-right;
  `_PartitionView` reserves a 104 px right gutter (`_kGaugeGutter`) so the
  gauge never paints over notes — at the cost of 104 px of engraving width on
  every line.
- `layout_systems` packs greedily up to `MAX_MEASURES_PER_SYSTEM = 4`, with
  per-column spacing `UNIT * (d/div)^0.6` (UNIT = 30). A 16th-note column gets
  ~13 px, so dense measures render nearly side by side.
- The horizontal Portée view (`StaffPainter`) already exposes two scaling
  seams: `noteScale` (staff/glyph size) and `lookAheadMs` (time window across
  the width — i.e. horizontal spacing).

Constraints: no public FFI signature change (avoid frb regen); Riverpod 2 +
Freezed conventions; coverage ≥ 80% both ecosystems; the back-office wasm
shares `musicxml-core` so layout changes propagate there by design.

## Goals / Non-Goals

**Goals:**

- Uncluttered Partition game view: nothing floating over the engraved score.
- Next line always readable in place (no preview box).
- Dense scores legible: bounded minimum spacing per note column, fewer
  measures per line.
- One user setting scaling notation size in both notation views, persisted.

**Non-Goals:**

- No change to scoring semantics, Wait Mode, timing, or per-note hit sparks in
  Synthesia/Staff.
- No re-engraving engine work beyond the spacing floor (no optical kerning,
  collision avoidance, etc.).
- No back-office UI change (it picks up the core layout change via wasm
  rebuild, nothing else).
- No phone-specific Partition work (mode stays tablet/desktop-only).

## Decisions

### D1 — Look-ahead scroll replaces the NEXT overlay

`_followCursor` anchors the top of the current system at 30% of the viewport
height (clamped to scroll extents) instead of centring it. With systems ~340 px
tall, any viewport ≥ ~500 px shows the full current line plus the next line
below it. `_NextLineOverlay`, `_buildNextLineOverlay`, and their heuristics are
deleted. Rationale: the overlay was compensating for the scroll policy; fixing
the policy removes an entire widget + heuristic surface and the occlusion it
caused. Alternative considered: keep the overlay but reposition/shrink it —
rejected, still occludes and still duplicates content.

### D2 — Played-system dimming in the painter

`PartitionPainter` gains the cursor's system index and draws systems whose
index is lower (fully played) at reduced opacity (~0.45 on the ink/staff
colours). Only when a playhead exists (`elapsedMs` within range); paused or
stopped views render normally. Implemented as a per-system alpha applied via
`canvas.saveLayer` over the system's rect (cheap: 2–3 systems visible thanks to
viewport culling). Alternative: dim per-measure — rejected, per-line matches
the reading unit and the auto-scroll granularity.

### D3 — Score chip in the top bar, gauge removed everywhere

A new `ScoreChip` widget (compact horizontal pill: tier-coloured dot, live sync
%, combo) renders in `_TopBar` between the MIDI status and the tempo chip,
watching the same `performanceScorerProvider` slices as the gauge. Hidden when
no scored run is active. The tier-up shake + firework animate on/inside the
chip. `ScoringGauge` and the `Positioned` gauge inside `ScoringOverlay` are
removed in **all** modes (one HUD, one place); `ScoringOverlay` keeps only the
hit-sparks layer for Synthesia/Staff. `_kGaugeGutter` is deleted and
`_PartitionView` engraves at full width. Rationale: a single always-visible
top-bar location can never occlude any play surface, satisfying the
learning-safe constraint by construction. Alternative: keep the vertical gauge
in Synthesia only — rejected, two HUD locations for one metric is confusing.

### D4 — Spacing floor in `musicxml-core`, cap 4 → 3

`space()` clamps its result to a new `MIN_COL: f64 = 22.0` (px, ≈1.8 staff
spaces at the app's 12 px staff space): `UNIT * (d/div)^K` keeps governing
quarter-and-longer columns (30 px+) but 16th/32nd columns now get 22 px instead
of ~13/8 px. `MAX_MEASURES_PER_SYSTEM` drops to 3. Effects: dense measures get
a larger `min_width`, so the greedy packer fits fewer per line — density
self-regulates by content; simple scores still show 3 per line. `min_width`
is computed at parse time and stored on the measure, so no API change. Existing
sub-linearity tests keep passing (the floor only binds below a quarter note).
Alternative: raise UNIT — rejected, it widens everything including sparse
scores, wasting width where legibility is fine.

### D5 — One `scoreSize` preference scaling both views

`PlayerPrefs` gains `scoreSize` (enum `ScoreSize { small, medium, large }`,
default `medium`; factors 0.85 / 1.0 / 1.2). Persisted in the existing JSON
record; unknown/absent decodes to `medium`.

- **Partition**: `PartitionPainter`'s staff space `_s` becomes an instance
  field `s = 12.0 * factor` (all derived paddings already scale off it), and
  `_PartitionView` requests layout with
  `setAvailableWidth(engraveWidth / factor)` — the painter's justification then
  stretches each system by exactly `factor`, so glyph size and horizontal
  spacing scale together and lines re-wrap. No Rust change needed for sizing.
- **Portée/Staff**: `player_screen` passes `noteScale: factor` and
  `lookAheadMs: default / factor` to `StaffPainter` — both seams already exist
  (the in-card preview uses them). Notes grow and horizontal spacing keeps
  pace.
- **Control**: a three-segment control in the pre-play/settings modal (both
  the wide two-column and narrow one-column layouts), applied on Validate like
  the other settings.

Alternative: separate size and spacing settings — rejected per UX (one knob
that keeps the engraving coherent beats two that can fight each other).

### D6 — Current-measure wash

`_paintMeasure` already knows the active measure (it computes the cursor x).
Before glyphs, the painter fills the active measure's rect (system top to
bottom) with `CymbraColors.secondary` at ~8–10% alpha. Dimming (D2) never
applies to the current system, so the wash never fights the dim layer.

## Risks / Trade-offs

- [Anchor at 30% leaves less context above the current line] → acceptable: the
  played lines matter less than the upcoming ones; dimming makes that explicit.
- [saveLayer per dimmed system costs a compositing pass] → bounded by viewport
  culling (~2–3 systems painted); measured as negligible next to glyph drawing.
- [Spacing floor reflows every score's line breaks] → intended; goldens for the
  partition (if any) must be refreshed, and BO wasm must be rebuilt to stay in
  sync (documented gotcha).
- [Removing the vertical gauge changes a shipped visual] → the chip keeps the
  same data, colours and celebrations; session summary is untouched.
- [Chip crowds the phone top bar] → the phone top bar already compacts; the
  chip ellipsizes to dot + % only on `isPhoneLayout`.
- [scoreSize=large on small tablets could make lines very short] → cap the
  effective factor so at least one measure plus header fits (`layout_systems`
  already gives an oversized lone measure its own system — degraded but
  correct).

## Migration Plan

Pure client + core-crate change; ships with the app build. No data migration —
the prefs JSON gains an optional key with a safe default. Rollback = revert.

## Open Questions

(none — all decisions taken above)
