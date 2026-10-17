# Design

## Context

The home (`LibraryScreen`) is one `CustomScrollView` whose slivers are: a rating nudge, the
daily-access chip, the `CoursesSection` (omitted under drums), then the favorites grouped by
level. The header already hosts a sticky `InstrumentSwitcher` (keyboard / drums) when the
drum feature is visible. The learning path is a pushed route (`LearningPathScreen`) and the
lesson player another. Nothing animates between them: all 34 navigations use the default
`MaterialPageRoute`. See proposal.md — Why.

Constraints: landscape-only app, phone viewports are short; the UI never calls services;
listeners live in dedicated widgets; coverage ≥ 80 %; strings in all four ARB locales.

## Goals / Non-Goals

**Goals:**
- One sticky, obvious control that answers "play or learn?" and remembers the answer.
- The Learn door is a real home for the path, not a pushed route the player has to find.
- Transitions that make the two doors feel like one surface.
- Revertible from the server.

**Non-Goals:**
- No new learning content and no change to how a lesson runs (`add-music-piece-lessons`).
- No celebration, counter or path-drawing motion (`add-music-motion-and-celebrations`).
- No today's-session block (`add-music-daily-session`), though the Learn door reserves its
  slot above the continue card.
- No drum learning path.

## Decisions

### D1 — A segmented switcher in the header, not bottom tabs
Alternatives: a bottom navigation bar; two top-level tabs with their own navigators.
Chosen: a `SegmentedButton` in the app bar's bottom slot, exactly where the instrument
switcher sits. In landscape on a phone a bottom bar eats the short axis the keyboard needs;
tabs with separate navigators would split the player stack in two and complicate the
"resume after sign-in" flows that already push routes from the home. The switcher is cheap,
already a familiar shape in this app, and keeps one navigator.

When both switchers are visible (drums enabled), they stack in one row: instrument on the
left, door on the right; the door switcher collapses to *Play* alone under drums.

### D2 — The door is a persisted preference, defaulting to Play
Alternatives: default to Learn for new users; infer from last activity.
Chosen: a `homeDoorProvider` (Riverpod notifier, Freezed state) persisted through the
existing `preferencesServiceProvider`, default **Play** so the released app looks exactly
like today on first launch. `add-music-placement-test` may later set Learn after a
placement; this change only persists what the player tapped.

### D3 — Learn door body = today's `LearningPathScreen` content, not a new screen
The path screen's unit sections and lesson nodes become a reusable body widget; the pushed
`LearningPathScreen` keeps using it (deep links from the continue card still work). The
Learn door shows: a slot reserved for today's session (empty widget until
`add-music-daily-session`), the continue card, then the path body. No duplicate layout.

### D4 — Shared-element transitions with `Hero` and a short page transition
Alternatives: `PageRouteBuilder` everywhere; an animation package.
Chosen: wrap the tapped card (continue card, lesson node, score card) and its target's
header in `Hero` with a 280 ms `easeOutCubic`; the body under the switcher swaps through an
`AnimatedSwitcher` (fade + 24 px slide). No dependency added. `MediaQuery.disableAnimations`
(reduce motion) shortens every duration to zero — the same rule the motion change will
generalise.

### D5 — `music.learn_door.enabled`, default on, read like `drums.enabled`
Flag off ⇒ the switcher is absent and the home is today's single body with the courses card
in its current place (the `notation-courses` MODIFIED requirement keeps that fallback
explicit). The flag is a kill-switch, not a rollout gate: it defaults on in the app so a
missing flag server-side never hides the doors.

## Risks / Trade-offs

- [Two switchers in one header on a phone] → the door switcher uses icon-only segments
  below 600 px of width; verified in the responsive widget tests.
- [Players who liked courses above favorites lose that glance] → the Play door keeps a
  one-line "Continue learning" chip in the header area when a lesson is in progress, linking
  to the Learn door; it is a link, not a card.
- [Hero across differently sized cards can jump] → the Hero child is the card's title row
  only; the rest fades.
- [Flag flipped off mid-session] → the switcher disappears on the next flag refresh and the
  body falls back to Play; the persisted preference is kept for when it returns.
