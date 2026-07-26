## Why

The app has grown a lot of surfaces — swipe rating, a points economy + shop + badges, a
curator profile with a play heatmap, public profiles, and per-piece + global leaderboards —
none of which are self-evident. New users need to grasp the **core loop** first and discover
the rest **gradually**, not through a wall of tours. Crucially, **a user has no account at
first launch and MUST NOT be forced to create one**: the welcome and first experience happen
without sign-up, and account creation is invited only when a feature actually needs it.

## What Changes

- **Lean first-run welcome (no account required)** — a short (2–3 screen) welcome, shown at
  first launch **before any sign-in**, that states the value and routes the user to a first
  action. It is **always skippable** and **never forces sign-up**.
- **Value before sign-up** — the user SHALL be able to experience the **core loop** (play a
  piece, see the live sync gauge and end-of-session summary) **without an account** — via a
  no-account "try" path (e.g. a bundled demo piece). Sign-in is **deferred**.
- **Deferred, contextual sign-in** — when the user reaches a feature that genuinely needs an
  account (saving a library, rating to earn points, appearing on leaderboards, going public),
  the app **invites** sign-in with the benefit stated, and lets them **back out** to keep
  exploring. Exploration is never gated behind a forced wall.
- **Unified progressive discovery** — one consistent, one-time, dismissible coach-mark system
  that introduces each feature **at first relevant use**, consolidating the ad-hoc first-run
  hints already specified for the rating deck (#2), reward feedback (#4), and the age gate (#5)
  under a single mechanism, with "seen" state persisted.
- **Re-findable help/tips** — because hints show once, a **help/tips** surface lets users
  re-read how things work (points, ratings, leaderboards, going public).
- **Cross-cutting** — everything is **localized** (`app-localization`), accessible, skippable,
  and never blocks the core experience. Moderation is **not** surfaced to normal users.

Out of scope: a "what's new" feature-highlights sheet for existing users and server-side
persistence of "seen" across devices (scope C, deferred); rearchitecting auth or opening the
whole hub to anonymous browsing (only a minimal no-account "try" is introduced).

## Capabilities

### New Capabilities
- `welcome-onboarding`: the first-run welcome that runs **without an account** and never forces
  sign-up, the **no-account way to experience the core loop**, and the **deferred, contextual**
  sign-in invitation offered (never forced) at the point a feature needs an account.
- `feature-discovery`: the unified progressive coaching system — one-time, dismissible,
  re-findable hints introduced at first use (consolidating #2/#4/#5), plus a help/tips surface.

### Modified Capabilities
<!-- None. Onboarding wraps the existing entry/handle flow and the per-feature hints without
     changing their requirements; the no-account "try" is an additive path, not a change to the
     authenticated hub. -->

## Impact

- **App** (`apps/music`): a welcome flow before the entry/handle screens; a no-account "try"
  path (a bundled demo piece playable without sign-in) exercising the existing player/scoring
  seam; a shared coach-mark/spotlight widget + "seen" state (`shared_preferences`); a help/tips
  screen; contextual sign-in prompts at gated actions.
- **Relates to** `handle-onboarding` (welcome precedes the post-auth handle gate; ordering:
  welcome → optional sign-in → handle gate), `state-management`, `app-localization`,
  `responsive-layout`, and the per-feature hints in #2/#4/#5 it consolidates.
- **Likely no backend change** (scope B): "seen" state is local; the demo piece is bundled. A
  backend "seen"/what's-new surface is scope C, deferred.
- **Coverage**: Flutter ≥ 80% for the welcome/skip/deferred-sign-in flow and the coach-mark
  system via fakes; the demo-try exercised behind the existing injectable player seam.
