> Depends on `add-welcome-onboarding` (`feature-discovery`) having landed — the discovery hint and glossary reuse its coach-mark + "seen" + help/tips seam (no local shim). Sequence this change after it.

## 1. Painter geometry seam (`notation-help`)

- [ ] 1.1 Define `SymbolDescriptor` as a Freezed union **exhaustive over every glyph the painters draw**: `note` (pitch name + register), `rest`, `accidental` (which one), `clef` (sign), `keySignature`, `timeSignature`, `augmentationDot`, plus structural marks `stem`, `flag`, `beam`, `tieOrSlur`, `ledgerLine`, `barLine`, `brace`, `tupletNumber`
- [ ] 1.2 Define a per-frame `StaffHitIndex` value object holding `(Rect region, SymbolDescriptor)` entries plus a nearest-hit query (contains, else nearest within a small radius, else none)
- [ ] 1.3 In `StaffPainter` (`lib/painters/staff_painter.dart`), record each drawn glyph's region + descriptor into a `StaffHitIndex` and expose it (via `ValueNotifier`/provider), without changing draw order or output
- [ ] 1.4 Do the same in `PartitionPainter` (`lib/painters/partition_painter.dart`) for the static score view
- [ ] 1.5 Unit-test the hit index (containment, nearest-when-dense, empty-area→none) and add a geometry/golden test asserting rendered output is byte-identical with the seam present
- [ ] 1.6 Add a **totality test**: every glyph the painters emit maps to a `SymbolDescriptor` that has localized content — fails if a painter draws a glyph with no descriptor/help (guards future glyphs)

## 2. Long-press gesture + help bubble (`notation-help`)

- [ ] 2.1 Add a Riverpod notifier holding the active help bubble state (visible `SymbolDescriptor` + anchor rect), with show/dismiss actions
- [ ] 2.2 Add a dedicated staff-overlay/listener widget wrapping the `CustomPaint` in `player_screen.dart`: `LongPressGestureRecognizer` → query hit index → resolve descriptor → drive the bubble notifier (scoped so it does not steal play/scrub taps/drags)
- [ ] 2.3 Wire the same overlay onto the static score/partition view
- [ ] 2.4 Build the help-bubble overlay widget: dismissible, points at the symbol, clamps to safe area and flips anchor side in portrait/landscape; screen-reader friendly and dismissible without a single specific gesture
- [ ] 2.5 Verify (widget test) that showing/dismissing a bubble never pauses, blocks, or alters playback and that play/scrub still work with help enabled

## 3. Localized help content (`notation-help`)

- [ ] 3.1 Add a content lookup `SymbolDescriptor → localized strings` backed by the generated l10n; localize note names per the app's existing do-ré-mi / C-D-E convention in one helper
- [ ] 3.2 Add the new keys to `lib/l10n/app_en.arb` and translate in `app_fr.arb`, `app_es.arb`, `app_it.arb`; run l10n codegen
- [ ] 3.3 Ensure the same content function feeds both the on-staff bubble and the glossary; unit-test lookups for each symbol kind across locales

## 4. Discovery hint + glossary via `feature-discovery` (`notation-help`)

- [ ] 4.1 Register the "you can long-press symbols for help" hint through the shared `feature-discovery` coach-mark + persisted "seen" state (one-time, dismissible, non-blocking) — reuse its seam directly, no local shim
- [ ] 4.2 Add a browsable notation glossary screen (same content as the bubbles) reachable from the `feature-discovery` help/tips surface
- [ ] 4.3 Widget-test: hint shows once then not again; glossary lists and reads all covered symbol kinds and matches the on-staff copy
- [ ] 4.4 (Optional, only if `feature-usage-analytics` #171 has landed) Emit curated actions — help-bubble opened, hint dismissed, glossary opened, lesson completed — via its taxonomy, fire-and-forget and guarded so its absence never blocks the help; `variant` at most the symbol kind, no score content

## 5. Quality gate (v1)

- [ ] 5.1 `melos run analyze` + `dart format` clean; `dart run custom_lint` passes
- [ ] 5.2 Flutter line coverage ≥ 80% for the new code (`flutter test --coverage`, excluding goldens); native lib not required
- [ ] 5.3 `openspec validate add-notation-help --strict` passes

## 6. Phase 2 — scripted staff-reading lessons (`notation-lessons`, deferred)

- [ ] 6.1 Model a lesson as an ordered list of scripted steps (`explanation | diagram | quiz`), diagrams reusing existing painters/glyphs where possible; all copy localized (en/fr/es/it)
- [ ] 6.2 Build the lessons list + scripted-lesson player UI (skippable, never a prerequisite to play); quiz step gives immediate feedback but does not block continuing
- [ ] 6.3 Persist lesson completion locally (`shared_preferences`); mark completed lessons across launches; make lessons reachable and replayable from the help/tips surface
- [ ] 6.4 Widget-test lesson flow (step-through, skip, quiz feedback, completion persistence, replay) and keep Flutter coverage ≥ 80%
