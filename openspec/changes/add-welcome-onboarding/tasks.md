## 1. First-run welcome (no account)

- [x] 1.0 A **language step** as the very first launch screen (before the welcome), defaulting to the device locale when supported, applied immediately via `app-localization`; changeable later in settings; no account required.
- [x] 1.1 A welcome flow (Riverpod notifier + Freezed state) shown at first launch **after the language step and before** the entry/handle screens; 2–3 screens (value → core loop → first action); always skippable; a one-time "welcome seen" flag in `shared_preferences`.
- [x] 1.2 Wire ordering: language → welcome → optional sign-in → existing `handle-onboarding` gate → app. Do not require an account to view or skip the welcome.

## 2. No-account "try" of the core loop

- [x] 2.1 A "Try it now" entry from the welcome that plays one of the **already-included bundled scores** (`assets/scores/{beginner,intermediate,advanced}/`) via the existing player + `performance-scoring` seam — **without** sign-in and without opening the authenticated hub. No new asset needed; optionally let the user pick among the included scores.
- [x] 2.2 After the try's end-of-session summary, offer (not force) sign-in with the benefit stated; declining returns to the welcome/app.

## 3. Deferred contextual sign-in

- [x] 3.1 A reusable "sign-in invitation" surface that names the benefit for a gated action (save library / earn points / join leaderboards / go public); shown when a signed-out user triggers such an action.
- [x] 3.2 Declining returns the user to what they were doing (no dead-end); accepting completes sign-in and **resumes the intended action**.

## 4. Unified progressive coaching

- [x] 4.0 A **coaching controller** (`@riverpod` notifier + Freezed state) owning the "seen" set and the guided-sequence step (`markSeen/start/next/skip`), with "seen" persistence behind an **injectable seam** (fake in tests, `shared_preferences` in prod). Logic is unit-testable without native/prefs.
- [x] 4.1 A shared **custom** coach-mark/spotlight overlay (`Overlay` + `CustomPainter` hole + positioned bubble w/ landscape edge-avoidance + Next/Skip): one-time, dismissible, non-blocking; targets located by `GlobalKey`/`CoachTarget` registry after `addPostFrameCallback`; hit-test pass-through in the hole for "do it now" steps, illustrative for passive hints. (No coach-mark dependency; `tutorial_coach_mark` only as a fallback.)
- [x] 4.2 Route the first-run hints from #2 (rating deck), #4 (reward feedback), and #5 (age gate) through this shared mechanism instead of ad-hoc implementations. *(The rating-deck hint is migrated off its own notifier onto the shared one, keeping its legacy prefs key; the rewards and going-public hints are delivered as one-time inline callouts on the profile. The age gate itself stays a mandatory, server-enforced eligibility step — it is a safeguard, not a dismissible hint — so the hint next to it explains going public rather than replacing the gate.)*
- [x] 4.3 A **guided in-context sequence in the player** (directed, one control at a time) highlighting the real controls — piano-sound selection (`piano-sound-selection`), connected-MIDI/device view + manual select (`midi`), hand selection right/left/both (`hand-selection`) — pointing at each control in place. The controller **orchestrates the UI** (open the player settings end-drawer, then spotlight the control inside); runs on first player visit; skippable (never blocks play); replayable from help. Built on 4.0/4.1 so it extends to other controls later. *(The controls live in the pre-play setup surface, which the player already opens on entry — that is the surface the sequence spotlights; replay is armed from help and runs on the next score opened.)*

## 5. Help/tips

- [x] 5.1 A help/tips screen reachable from a stable entry (settings/profile) covering the core loop, ratings, points, shop/badges, profile, leaderboards, and going public.

## 6. Cross-cutting

- [x] 6.1 Localize all welcome/coaching/help copy via `app-localization`.
- [x] 6.2 Accessibility: dismissible without a single required gesture, contrast, screen-reader labels; landscape/`responsive-layout` correct.
- [x] 6.3 Ensure moderation surfaces never appear in user onboarding.

## 7. Tests & verification

- [x] 7.1 Flutter (via fakes): language step precedes the welcome and defaults to device locale; welcome shows pre-account and is skippable; no forced sign-up; no-account demo plays and shows the summary; contextual sign-in invite states a benefit, decline keeps exploring, accept resumes the action; coach-mark shown once + re-findable in help; guided player sequence highlights piano/MIDI/hand controls, is skippable, and is replayable from help. `flutter test --coverage` ≥ 80%.
- [x] 7.2 Golden tests (tagged `golden`) for the spotlight overlay visuals (hole + bubble placement, landscape); excluded from the cross-platform gate.
- [x] 7.3 `melos run analyze` + `dart format` clean; regenerate codegen as needed.
- [x] 7.4 `openspec validate add-welcome-onboarding --strict` passes.
