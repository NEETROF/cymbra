## Context

Change #8, an onboarding/discovery layer over the whole feature set (#1–#7). The existing
`handle-onboarding` only covers the post-auth **handle-selection gate** (escape + orphan
cleanup); there is no feature-orientation onboarding, and the hub is authenticated-only today.
Per-feature first-run hints were specified piecemeal (#2 rating-deck coach-mark, #4 reward
feedback/celebrations, #5 age-gate prompt). The new constraint from the stakeholder: **the
user has no account at first launch and must not be forced to create one** — value first,
account later.

## Goals / Non-Goals

**Goals:**
- A lean first-run welcome that runs **without an account** and is always skippable.
- Deliver the **core-loop "aha" without sign-up**, then invite sign-in **contextually**.
- One **unified, progressive** discovery system (consolidating #2/#4/#5), shown once and
  **re-findable** via help/tips.
- Localized, accessible, non-blocking.

**Non-Goals (scope B):**
- A "what's new" sheet for existing users; server-side "seen" sync (scope C, deferred).
- Rearchitecting auth or opening the whole hub to anonymous browsing.
- Surfacing moderation to normal users.

## Decisions

### D1 — Welcome runs pre-account and never forces sign-up
The welcome is the **first** surface at first launch, **before** the entry/handle screens, and
requires no account. It is 2–3 screens (value prop → core loop → a first action), **always
skippable**, and offers sign-in only as an **option**. Ordering becomes: **welcome → (optional)
sign-in → handle gate (`handle-onboarding`) → app**.
- **Why**: the stakeholder is explicit — no forced account. A forced sign-up wall at launch is
  the highest-drop-off point; deferring it lifts activation.

### D2 — Value before sign-up via the already-included scores (decided)
The core loop (play → live sync gauge → end-of-session summary) MUST be reachable **without an
account**. The app **already ships bundled scores** as local assets
(`assets/scores/{beginner,intermediate,advanced}/`), playable without the backend — so the
no-account "try" simply routes the welcome to **those included scores**. No new demo asset and
no anonymous access to the authenticated catalog/hub is needed.
- **Why reuse the bundled scores**: they already exist and already work offline/without an
  account; it's the smallest possible change and delivers the "aha" immediately. The hub stays
  authenticated-only; only the local included scores are used for the no-account experience.
- **Alternative**: make the whole catalog/hub browsable anonymously. Rejected for scope B —
  large auth/permission change; the bundled scores already cover the try.

### D3 — Deferred, contextual sign-in with the benefit stated
When the user hits an account-gated action (save to library, rate to earn points, appear on a
leaderboard, go public, sync progress), the app shows a **contextual invitation** naming the
**benefit** ("Sign in to save your library / earn points / join the leaderboards") and lets the
user **decline and keep exploring**. Nothing about exploration/try is blocked behind a forced
wall.
- **Why contextual**: users say yes to sign-in when they understand the immediate payoff, not
  at a cold upfront wall.
- **Note**: the gated features already require auth by their own specs; #8 only standardizes
  *how* the sign-in is invited (benefit-first, decline-able), it does not loosen those gates.

### D4 — One progressive coaching system, consolidating #2/#4/#5
A single shared **coach-mark/spotlight** mechanism introduces each feature at **first relevant
use** — one-time, dismissible, non-blocking — with per-hint "seen" state persisted
(`shared_preferences`). The ad-hoc first-run hints in #2 (swipe deck), #4 (reward feedback), and
#5 (age gate) are expressed through this one system rather than each rolling its own.
- **Why unify**: consistent look/behaviour, one "seen" store, no divergent implementations; the
  earlier changes named the *need*, #8 provides the *mechanism*.

### D5 — Discovery is re-findable via a help/tips surface
Because hints show once, a **help/tips** surface lets a user re-read how each system works
(core loop, points, ratings, shop/badges, profile, leaderboards, going public). Reachable from
a stable entry (e.g. settings/profile).
- **Why**: one-time hints + a re-findable reference is the pattern that respects both new and
  returning users without nagging.

### D6 — Localized, accessible, non-blocking, no moderation
All copy is localized (`app-localization`); coach-marks are dismissible and never trap focus or
block the underlying action; onboarding surfaces adapt to the landscape/`responsive-layout`.
Moderation is backstage and never appears in user onboarding.

### D7 — Language is chosen first, before the welcome
The very first step at first launch is a **language choice**, defaulting to the device locale
when supported, applied immediately to the welcome and everything after, and changeable later in
settings. Flow becomes: **language → welcome → optional sign-in → handle gate → app**. No account
required.
- **Why first**: the welcome copy itself must be in the user's language to land; asking language
  after showing English/French copy is backwards. It is one tap (locale-defaulted), so it does
  not add friction.

### D8 — Guided in-context coaching for the player controls (Clash-of-Clans style)
Beyond passive one-time hints (D4), the player gets a **directed** guided sequence that
highlights the real controls one at a time — **piano-sound selection**, **connected-MIDI/device
view + manual selection**, **hand selection (right/left/both)** — the first time the user reaches
the player, pointing at each control in place (e.g. inside the player settings drawer). It is
**skippable**, never permanently blocks play, and is **replayable from help**.
- **Why directed, not just passive**: these controls are not obvious and matter for a good first
  session; a "show me where to tap" walk-through (like Clash of Clans' first-build guidance) is
  the right level — but kept **skippable**, consistent with the "don't force" ethos (we guide,
  we don't gate).
- **Reuses existing specs/surfaces**: it points at the controls defined by `piano-sound-selection`,
  the `midi` device selection, and `hand-selection`, rendered where they already live (the player
  settings end-drawer). #8 provides the guidance layer, not new controls.
- **Extensible**: the same guided mechanism can later cover other controls (wait-mode, metronome,
  tempo) without new machinery.

### D9 — Help overlays: a thin custom overlay driven by a Riverpod coaching controller
The coach-marks and the guided player sequence are built **in-house** (no coach-mark
dependency), because the key case — *open the settings drawer, then spotlight a control inside
it, interactively* — is too specific to fit a package's controller model cleanly.
- **Coaching controller** (`@riverpod` notifier + Freezed state): owns the "seen" set, the current
  guided-sequence step, and `markSeen / start / next / skip`. All logic lives here so it is
  **unit-testable**.
- **Seen persistence behind an injectable seam** (like `midiService`): a fake in tests, real
  `shared_preferences` in production — no native/prefs in widget tests, keeps the 80% gate.
- **Target discovery**: each coachable control carries a `GlobalKey` (or registers in a
  `CoachTarget` registry); the overlay reads its `RenderBox` (`localToGlobal` + size) to get the
  spotlight rect, after `addPostFrameCallback` (rects exist only post-layout).
- **The overlay** (`Overlay`/`OverlayEntry`): a `CustomPainter` scrim with a cut-out hole over
  the target (`Path.combine(difference, …)`), a bubble positioned near the target with landscape
  edge-avoidance, and Next/Skip. Hit-test **pass-through** in the hole for the guided "do it now"
  steps (tap the real control to advance); illustrative-only for passive hints.
- **Drawer orchestration**: the controller drives the UI between steps — e.g. open the player
  settings end-drawer, then spotlight the piano / MIDI / hand control inside — since those
  controls aren't on screen until the drawer opens.
- **Testing split**: step/seen logic → unit tests on the notifier; the spotlight visuals → golden
  tests (tagged `golden`, already excluded from the cross-platform gate).
- **Alternative**: `tutorial_coach_mark` / `flutter_showcaseview` (MIT). Rejected as the default —
  they'd have to be bent to the drawer-open + Riverpod + pass-through flow and add a dependency;
  a ~1-file custom overlay is cleaner here. (Fallback if the custom placement proves fiddly.)

## Risks / Trade-offs

- **"Don't force account" vs auth-only hub** → resolved by D2's no-account demo (aha without
  sign-up) + D3's deferred contextual sign-in; the hub stays auth-only, so no broad auth change.
- **Onboarding overload** → lean welcome (D1) + progressive disclosure (D4); never a big upfront
  tour. Skippable throughout.
- **Coach-mark nagging** → one-time + dismissible + "seen" persisted (D4); help/tips (D5) covers
  re-discovery instead of re-showing.
- **Demo piece licensing/size** → pick a bundled asset that is license-clean (e.g. an existing
  fixture/public-domain piece) and small.
- **Coverage** → Flutter tests for welcome/skip, no-forced-sign-up, deferred contextual prompt
  (decline keeps exploring), coach-mark shown-once + re-findable; demo-try via the injectable
  player seam.

## Migration Plan

1. App: welcome flow before entry/handle; a bundled demo piece + a no-account "try" entry;
   contextual sign-in prompts at gated actions; the shared coach-mark system + "seen" store;
   help/tips screen; localized copy.
2. Consolidate #2/#4/#5 first-run hints onto the shared coach-mark system as those land.
3. **Rollback**: additive UI; removing the welcome/coach-marks/help leaves the app and its
   auth flow unchanged.

## Open Questions

- **No-account "try" mechanism (resolved, D2)**: reuse the **already-included bundled scores**
  (`assets/scores/**`), already playable without an account; no new demo asset, no anonymous hub.
- **Help/tips home** — settings, profile, or a dedicated entry; and its depth (short tips vs a
  small FAQ).
- **Deferred-sign-in copy per feature** — the exact benefit lines for each gated action
  (library, points, leaderboards, public profile), localized.
