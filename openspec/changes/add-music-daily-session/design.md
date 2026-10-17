# Design

## Context

Existing building blocks: `CourseCompletion` (completed ids, and passed/best once
`add-music-piece-lessons` lands), the course listing in path order, the play outbox and
`PlaySync` (a delivered session bumps revisions), `Streak` (server-owned, local-day based,
with a recovery offer and an at-risk nudge), the showing-up practice award (once per local
day). The Learn door reserves a slot for this block. See proposal.md — Why.

Constraints: UI never calls services; a provider never invalidates a sibling; coverage
≥ 80 %; four locales; short landscape viewports.

## Goals / Non-Goals

**Goals:**
- A session the learner can finish in ten minutes, built without a new server table.
- A ring that is honest: it reflects what was actually done today.
- One definition of "today": the local day already used by the streak and the award.

**Non-Goals:**
- No minutes-based goal: the app does not measure practice minutes reliably across modes,
  and a count of finished items is honest where a timer is not.
- No server-side session record or cross-device "goal size" (a device setting; revisit if
  learners ask).
- No new reminder: the streak's evening nudge already exists and now has something to point
  at.
- No celebration visuals here (`add-music-motion-and-celebrations`).

## Decisions

### D1 — Composition is a pure function of existing state
`composeDailySession(courses, completion, passedState, todayLedger, goal)` is a pure Dart
function in a host-testable module, tested exhaustively; the notifier only feeds it. Rules:
the lesson item is the first course neither completed nor passed; the piece item is the
first `piece` step not passed in that course or, if none, a passed piece chosen round-robin
by date; the review item is a `readPlay`/`rhythmTap`/`earChoice` block drawn from the most
recently completed unit, run as a one-block mini-lesson. When the path is finished, the
session is piece + review + a replay suggestion.

### D2 — "Done today" is a small local ledger, keyed by local day
A `TodayLedger` persisted through `preferencesServiceProvider`: `{localDay, lessonIds,
pieceStepKeys, reviewKeys, cueRaised}`. Written at the three places the app already
records the event, right next to the existing call: the lesson player's finish
(`markLesson`), the shared piece-step runner (`markPiece`, each run, passed or not) and a
review's finish (`markReview`). Not diffed off the completion state: a restore or a server
reconciliation grows that state too, and would read as "done today". Any change of local
day — forwards at midnight, or a clock set back — starts a fresh ledger. Cross-device: the
server's first-completion and first-pass moments (`completed_at_ms`, added to
`CourseProgress`; `passed_at_ms`) ride on the completion state after each
reconciliation, and the ledger notifier `ref.listen`s it and marks the lesson and piece
items whose moment falls on today's local day; the review stays device-local (it is a
two-minute drill).

### D3 — Goal size lives in the block, default normal
`DailyGoal { light=1, normal=2, full=3 }` persisted locally; the ring's denominator. The
choice sits in the block itself (three chips under the items) rather than in the player
settings: it is a session setting, read where the session is. The item list always shows
all three so a "light" learner can still do more.

### D4 — The streak is read, never advanced here
The block watches `streakProvider` and shows days + "today covered". It never computes a
streak locally (the one-definition rule in `add-practice-streak`).

### D5 — Completion moment is a state transition, raised once by a listener widget
`DailySession.completedToday` is done ≥ goal; a dedicated listener widget under the door
switcher raises the moment on the shared celebration surface
(`add-music-motion-and-celebrations`, which landed first — no interim toast) and marks
`cueRaised` in the ledger, so it fires once per local day. The moment waits for the home
route to be current again, so it never stacks over the lesson player's own celebration.
The "today" the composition reads is a `localDay` provider the block refreshes at local
midnight, so a door left open overnight recomposes.

## Risks / Trade-offs

- [Review drawn from a block the lesson player needs context for] → the mini-lesson wraps a
  single block in the existing lesson player with a synthetic one-block manifest, reusing
  everything.
- [Ledger and server disagree after a device clock change] → the local day is computed from
  the device clock like the streak's `tz_offset`; a disagreement only affects which items
  look done today, never any recorded data.
- [Learner finishes the path] → the composition has an explicit "path finished" mode tested
  on its own.

## Mockups

Wireframes in the app's palette (landscape phone, French copy as the app's French locale would show it); illustrative, not pixel specs. SVG, editable as text, rendered by GitHub.

![C1](mockups/c1-daily-session.svg)

*C1 — Today's session at the top of the Learn door: lesson · piece · review, the 1/2 ring, the streak read from the server, the goal setting.*

