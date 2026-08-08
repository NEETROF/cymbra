> Depends on `add-welcome-onboarding` (`feature-discovery`) having landed — the discovery hint and glossary reuse its coach-mark + "seen" + help/tips seam (no local shim). Sequence this change after it.

## 1. Painter geometry seam (`notation-help`)

- [x] 1.1 Define `SymbolDescriptor` as a Freezed union **exhaustive over every glyph the painters draw**: `note` (pitch name + register), `rest`, `accidental` (which one), `clef` (sign), `keySignature`, `timeSignature`, `augmentationDot`, plus structural marks `stem`, `flag`, `beam`, `tie`, `slur`, `ledgerLine`, `barLine`, `brace`, `tuplet`, `dynamics` (`lib/painters/staff_hit_index.dart`)
- [x] 1.2 Define a per-frame `StaffHitIndex` value object holding `(Rect region, SymbolDescriptor)` entries plus a nearest-hit query (contains → closest centre, else nearest within a small radius, else none)
- [x] 1.3 In `StaffPainter` (`lib/painters/staff_painter.dart`), record each drawn glyph's region + descriptor into an optional `StaffHitIndex` side channel, cleared/refilled per paint, without changing draw order or output
- [x] 1.4 Do the same in `PartitionPainter` (`lib/painters/partition_painter.dart`) for the static score view (notes, accidentals, dots, rests, clefs, key/time sigs, bar lines, ledger lines, ties, slurs, beams, tuplets, dynamics, brace); byte-identical render verified (`test/painters/partition_hit_geometry_test.dart`)
- [x] 1.5 Unit-test the hit index (containment, closest-centre-when-overlapping, nearest-when-dense, empty-area→none) and add a geometry test asserting rendered output is **byte-identical** with the seam present (`test/painters/staff_hit_index_test.dart`, `staff_hit_geometry_test.dart`)
- [x] 1.6 Add a **totality test**: every glyph the fixture score draws is recorded as a `SymbolDescriptor` (guards dead long-presses). Content-side totality (every recorded kind has localized help) is asserted in task 3.3 once the content lookup exists.

## 2. Long-press gesture + help bubble (`notation-help`)

- [x] 2.1 Add a Riverpod notifier holding the active help bubble state (`SymbolDescriptor` + anchor + area size), with show/dismiss actions (`lib/state/notation_help_notifier.dart`, autoDispose)
- [x] 2.2 Add a dedicated staff-overlay widget (`lib/widgets/notation_help_area.dart`) wrapping the `CustomPaint` in `player_screen.dart` (staff view): `onLongPressStart` → query hit index → resolve descriptor → drive the bubble notifier; long-press only, so it never steals play/scrub taps/drags
- [x] 2.3 Wire the same overlay onto the static score/partition view (inside the scroll content so press coords match painter coords)
- [x] 2.4 Build the help-bubble overlay widget (`lib/widgets/notation_help_bubble.dart`): dismissible, points near the symbol, clamps to the area and flips anchor side (above/below); one Semantics node + explicit close button (dismissible without the long-press)
- [x] 2.5 Widget test: long-press opens the bubble, close button + outside tap dismiss it, a plain tap passes through to the child (play/scrub unaffected), disabled area shows nothing (`test/widgets/notation_help_area_test.dart`)

## 3. Localized help content (`notation-help`)

- [x] 3.1 Add a content lookup `SymbolDescriptor → localized strings` backed by the generated l10n (`lib/notation/notation_help_content.dart`), exhaustive over the sealed union so **content-side totality is compile-enforced**; note names + key tonic localized per the app's do-ré-mi / C-D-E convention in one helper
- [x] 3.2 Add the ~28 entries to `lib/l10n/app_en.arb` and translate in `app_fr.arb`, `app_es.arb`, `app_it.arb`; `flutter gen-l10n` clean (0 untranslated)
- [x] 3.3 Same content function feeds both the on-staff bubble and the glossary (`notationGlossarySamples`); unit-tested for every symbol kind across all 4 locales (`test/notation/notation_help_content_test.dart`)

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
