## Context

The staff is drawn on a Flutter `Canvas` by two `CustomPainter`s — `StaffPainter`
(`lib/painters/staff_painter.dart`, the scrolling, time-synced player staff) and
`PartitionPainter` (`lib/painters/partition_painter.dart`, the static score view). Both consume the
`TimedNote`/`TimedRest` model (`lib/state/player_data.dart`, which already carries `pitch`, spelled
step, `staff`, `clefSign`/`clefLine`, `dots`) and draw SMuFL glyphs via `lib/painters/smufl.dart`
(distinct codepoints already exist for each notehead, accidental, clef, rest, and dot).

Because the staff is a single canvas, **there are no per-glyph widgets to hit-test** — a long-press
gives a raw offset, and nothing today maps that offset to "the F# in measure 3". The painters
compute every glyph's position while drawing but discard it. This change's core technical work is to
**capture that geometry** and expose a query seam, then layer a gesture + bubble on top.

`add-welcome-onboarding` already introduces `feature-discovery`: a shared, one-time, dismissible
coach-mark mechanism with `shared_preferences`-backed "seen" state and a re-findable help/tips
surface. This change **reuses** that mechanism for its discovery hint and hangs its glossary (and,
in phase 2, its lessons) off that help surface — it does not build a parallel system.

App state is Riverpod 2 + Freezed; UI never calls services directly; localization is via
`app_en/fr/es/it.arb`. Coverage gate is ≥ 80% Flutter, with the native FFI kept behind seams so
widgets/state are testable without the native library.

## Goals / Non-Goals

**Goals:**
- Long-press **any** rendered staff symbol — every glyph the renderers draw, not a subset — → a
  localized, dismissible help bubble pointing at it, in both the player staff and the static score,
  without touching playback.
- A single geometry seam on the painters that both renderers populate and the gesture layer queries,
  with **byte-identical render output**.
- One-time "you can long-press for help" discovery hint via `feature-discovery`; a browsable
  glossary in help/tips sharing the same localized content.
- Phase 2: scripted, offline, self-paced staff-reading lessons with locally persisted progress,
  reachable/replayable from help/tips.

**Non-Goals:**
- No virtual/AI tutor, no LLM, no TTS, no conversational teacher — future vision only.
- No backend change: no new RPC, no server-side content, no cross-device sync of "seen"/progress.
- No change to *what* the staff renders (no new glyphs, no re-engraving).
- Not re-implementing `feature-discovery`'s coach-mark/help-surface plumbing here.

## Decisions

### 1. Capture glyph geometry in the painter, return it via a shared hit-test index
Rather than re-deriving glyph positions in a second pass (fragile, duplicates engraving math), the
painters **record each glyph as they draw it** into a per-frame list of
`(Rect region, SymbolDescriptor)` entries, exposed through a small shared value object (e.g. a
`StaffHitIndex`) surfaced on a `ValueNotifier`/provider the gesture layer reads. `SymbolDescriptor`
is a Freezed union over the symbol kinds (`note` with pitch name + register, `rest`, `accidental`
with which one, `clef` with sign, `keySignature`, `timeSignature`, `augmentationDot`).
- *Why:* positions are already computed during `paint`; capturing them is close to free and
  guaranteed consistent with what's on screen. A separate geometry model would drift from the
  engraving code.
- *Alternative rejected:* converting each glyph into its own positioned widget — kills the
  canvas performance model and complicates the scrolling/time-sync in `StaffPainter`.
- *Guardrail:* the capture must be inert when unused and must not alter draw order/output — covered
  by a golden/geometry test asserting identical rendering.

### 2. Resolve a press to the nearest symbol, not strict containment
Glyphs are small and dense; requiring the press to land exactly inside a notehead frustrates
beginners (the exact audience). The gesture layer picks the entry whose region contains the point,
else the nearest region within a small radius; a press in genuinely empty staff area resolves to
nothing.
- *Why:* forgiving hit-testing matches the spec's "closest symbol when dense" and "empty area does
  nothing" scenarios.

### 3. Long-press gesture layered above the `CustomPaint`, isolated per architecture rules
A dedicated gesture/overlay widget wraps the existing `CustomPaint` in `player_screen.dart` and the
static score view. Long-press → query the hit index → resolve descriptor → show an overlay bubble.
Per the Flutter architecture skill, this side-effect (showing/dismissing the bubble) lives in a
**dedicated listener/overlay widget near the feature subtree**, not scattered in build methods; the
UI reacts to notifier state, and never calls a service directly. Bubble state (visible descriptor +
anchor) is a small Riverpod notifier.
- *Why:* keeps the long-press from competing with play/scrub gestures and keeps the staff painters
  presentation-only.

### 4. Content model keyed by symbol kind, backed by the ARBs
Help copy is a lookup from a `SymbolDescriptor` (kind + specifics) to localized strings via the
generated l10n. Note names are localized (e.g. do/ré/mi vs C/D/E) using the app's existing
convention. The **same** content function feeds both the on-staff bubble and the glossary list, so
they can't diverge (spec: "glossary and on-staff help stay consistent").
- *Alternative rejected:* a JSON/asset content bundle — bypasses the enforced l10n pipeline and the
  4-language coverage the ARBs already give.

### 5. Reuse `feature-discovery` for the hint and the help/tips home — sequenced after it
The discovery hint is registered as one more `feature-discovery` coach-mark keyed for "seen"; the
glossary (and phase-2 lessons) are entries on the existing help/tips surface. **This change is
sequenced to land after `add-welcome-onboarding`** so it consumes the shared coach-mark + "seen" +
help/tips mechanism directly; there is **no temporary local "seen" shim**.
- *Why:* the proposal and `feature-discovery` spec explicitly consolidate one-time hints under one
  mechanism; a second bespoke "seen" store would violate that. Waiting removes the shim and the risk
  of two "seen" stores to reconcile later.

### 7. Cover every symbol the renderers can draw
v1 help covers **every symbol kind the staff renderers actually draw** — not a minimum subset. The
`SymbolDescriptor` union is exhaustive over the painters' glyph vocabulary (noteheads of every
duration, rests, accidentals incl. double, the G/F/C clefs, key signature, time signature,
augmentation dots, **and** the relational/structural marks the painters produce: stems, flags,
beams, ties/slurs, ledger lines, bar/measure lines, braces, tuplet numbers). A test asserts the seam
is **total**: every glyph the painters emit maps to a descriptor that has localized content, so no
rendered symbol is a dead long-press.
- *Why:* the audience is beginners, who will long-press whatever they don't recognise; "nothing
  happens" on a real symbol is the worst outcome.
- *Guardrail against drift:* the totality test fails if a painter later draws a glyph with no
  descriptor/content, forcing new glyphs to ship with help.

### 6. Phase 2 lessons are data-driven and offline
Lessons are an ordered list of scripted steps (`explanation | diagram | quiz`) described in-app
(localized copy + references to existing painters/glyphs for diagrams), progressed via a Riverpod
notifier, with completion persisted in `shared_preferences`. Diagrams reuse the existing painters
where possible rather than shipping images.
- *Why:* keeps lessons offline, testable, and localizable, and avoids an asset/content pipeline.

### 8. Emit help usage through `feature-usage-analytics` when present — optional
If `add-feature-usage-analytics` (#171) has landed, the notation-help actions (help-bubble opened,
discovery hint dismissed, glossary opened, lesson completed) are emitted as **curated actions** in
its client-owned taxonomy, so we can tell whether beginners actually use the help. This is
**best-effort and optional**: the emit is fire-and-forget, guarded so its absence never blocks or
alters the help, and no new PII is sent (an `action` name, plus at most a low-cardinality `variant`
such as the symbol kind — never score content).
- *Why:* the help exists to move a metric (beginner comprehension/retention); measuring its use is
  cheap once the taxonomy exists, and analytics explicitly wants purely client-side actions.
- *Not a dependency:* if analytics is absent, the help ships unchanged with no telemetry.

## Risks / Trade-offs

- **Dependency on `feature-discovery` (a separate in-flight change).** → **Accepted and sequenced:**
  `notation-help` lands after `add-welcome-onboarding`, consuming its coach-mark + "seen" + help/tips
  seam directly. No temporary local shim. (The on-staff bubble alone has no such dependency, but we
  wait rather than ship the hint/glossary twice.)
- **Geometry capture regressing render output or performance.** → A geometry/golden test asserts
  identical rendering; capture is a plain list append per glyph (O(glyphs already drawn)), and the
  scrolling staff already iterates those glyphs each frame.
- **Long-press colliding with existing player gestures (scrub/seek).** → Use a dedicated
  `LongPressGestureRecognizer` scoped to the staff overlay so it doesn't steal taps/drags; verify in
  a widget test that play/scrub still work with help enabled.
- **Bubble overflow in landscape / small screens.** → Clamp the bubble to the safe area and flip its
  anchor side; covered by the responsive scenario.
- **Localizing pitch names across do-ré-mi vs C-D-E systems.** → Reuse the app's existing note-name
  localization convention rather than inventing one; keep the mapping in one localized helper.
- **Scope creep from lessons.** → Lessons are explicitly phase 2 and deferred in tasks; v1 is the
  bubbles + hint + glossary. Virtual/AI tutor stays out entirely.

## Migration Plan

Additive, local-only, no data migration. Rollout order:
1. Painter geometry seam + `SymbolDescriptor` (+ geometry/golden test proving unchanged render).
2. Gesture overlay + bubble + localized content (v1 on-staff help), in player and static score.
3. Discovery hint + glossary wired into `feature-discovery`'s hint/help surface.
4. Phase 2 (separate slice): scripted lessons + local progress.
No rollback concerns beyond reverting the additive widgets/keys; nothing persisted is destructive.

## Open Questions

- **Resolved — sequencing:** `notation-help` waits for `feature-discovery` (`add-welcome-onboarding`)
  and reuses its mechanism; no local "seen" shim.
- **Resolved — symbol coverage:** v1 covers **every symbol the user can encounter** — i.e. every
  glyph the staff renderers draw (see Decision 7), enforced by the totality test.
- Lesson catalogue for phase 2: how many lessons and their exact ordering (out of scope for this
  change's implementation, to be decided when the phase-2 slice starts).
