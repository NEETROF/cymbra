## Why

Beginners can play a piece but can't *read* the staff: they don't know what a sharp, a flat,
a natural, a clef, a key signature, a dotted note or a rest actually mean, and nothing in the
app tells them. `add-welcome-onboarding` teaches the app's *features* (controls, points,
leaderboards); it does not teach *music notation*. New users need to learn the symbols on the
staff **in place, on demand**, without leaving the score — and they need to discover that this
help even exists.

## What Changes

- **Tap-and-hold any staff symbol → a help bubble.** In both the scrolling player staff and the
  static score view, long-pressing **any** rendered symbol — *every glyph the renderers draw*, not a
  subset: notes of any duration, rests, sharp/flat/natural/double accidentals, the G/F/C clefs, the
  key signature, the time signature, augmentation dots, and the structural marks (stems, flags,
  beams, ties/slurs, ledger lines, bar lines, braces, tuplet numbers) — opens a small, dismissible
  **help bubble** explaining what it is, pointing at the symbol in place. Available **at any time**,
  it never blocks or alters playback.
- **A discreet "this help exists" hint.** Because the gesture is not obvious, a **one-time,
  dismissible** hint tells the user they can long-press symbols for help, delivered through the
  existing `feature-discovery` coach-mark + "seen" mechanism — not a new bespoke system. This change
  is therefore **sequenced to land after `add-welcome-onboarding`** (which introduces
  `feature-discovery`); no temporary local "seen" store is added.
- **A browsable notation glossary in help/tips.** The same explanations are reachable from the
  `feature-discovery` help/tips surface as a browsable glossary, so a user can look up a symbol
  even away from a score, and re-read what a one-time bubble showed.
- **Painters publish hit-test geometry.** `StaffPainter` (scrolling player) and `PartitionPainter`
  (static score) expose, per frame, the on-screen region + a **symbol descriptor** (kind and its
  specifics — e.g. note name/octave, accidental type, clef sign) for each glyph they draw, so a
  gesture layer can map a long-press to the symbol under the finger. Render output is unchanged.
- **Localized content.** All help copy is authored through the app's localization (`app_en/fr/es/it.arb`),
  keyed by symbol kind, consistent with `app-localization`.
- **Phase 2 (specified, deferred): scripted beginner reading lessons.** A short, guided
  "reading the staff" course — self-paced **scripted** lessons (explanation + diagram + a light
  quiz step), progress persisted locally, replayable from help/tips. No teacher, no AI.
- **Out of scope: a virtual / AI tutor.** A conversational or LLM-backed teacher persona is a
  future vision only; nothing in this change adds a backend, an LLM call, or TTS.

## Capabilities

### New Capabilities
- `notation-help`: the **v1 deliverable** — contextual, always-available help for the symbols on
  the staff (long-press a rendered symbol → localized bubble), the one-time discovery hint routed
  through `feature-discovery`, the browsable notation glossary in help/tips, and the painter
  hit-test geometry that makes symbol-level long-press possible.
- `notation-lessons`: **phase 2, deferred** — a self-paced set of scripted beginner
  staff-reading lessons (explanation + diagram + light quiz), progress persisted locally and
  replayable from help/tips. Specified here for a coherent single spec; delivered after
  `notation-help`. Excludes any virtual/AI tutor.

### Modified Capabilities
<!-- None. `notation-help` reuses feature-discovery's "seen"/help-surface mechanism without
     changing its requirements; the painters gain a query seam without changing what they render. -->

## Impact

- **App** (`apps/music`) — **no backend change**:
  - `lib/painters/staff_painter.dart` and `lib/painters/partition_painter.dart`: expose per-glyph
    hit regions + a symbol descriptor (render output unchanged).
  - `lib/screens/player_screen.dart` and the static score/partition view: a long-press gesture
    layer over the `CustomPaint` that resolves a hit to a symbol and shows a help bubble.
  - New notation help-content model keyed by symbol kind, backed by the l10n ARBs (new keys in
    `lib/l10n/app_en.arb` + `fr`/`es`/`it`).
  - The one-time discovery hint + browsable glossary wired into the `feature-discovery` help/tips
    surface and its `shared_preferences` "seen" state.
  - Phase 2: a lessons list + a scripted-lesson player (text/diagram/quiz steps) with locally
    persisted progress.
- **Relates to** `welcome-onboarding` / `feature-discovery` (reuses the coach-mark, "seen" state,
  and help/tips surface), `score-notation` / `web-notation-render` (the glyphs being explained),
  `app-localization` (all copy), `responsive-layout` (bubble placement in portrait/landscape),
  and `state-management` (Riverpod for the help/hint/lesson-progress state).
- **Soft integration with** `feature-usage-analytics` (#171): when that taxonomy is available, the
  help actions (help-bubble opened, discovery hint dismissed, glossary opened, lesson completed) are
  emitted as curated actions so we can measure whether the notation help is actually used — a
  best-effort, **optional** hook, never a hard dependency and never blocking the help.
- **Coverage** (Flutter ≥ 80%): painter geometry is unit-tested; the long-press → descriptor →
  bubble mapping and the glossary lookups are widget-tested behind the existing injectable seams;
  no native library required.
