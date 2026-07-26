## 1. First-run welcome (no account)

- [ ] 1.1 A welcome flow (Riverpod notifier + Freezed state) shown at first launch **before** the entry/handle screens; 2–3 screens (value → core loop → first action); always skippable; a one-time "welcome seen" flag in `shared_preferences`.
- [ ] 1.2 Wire ordering: welcome → optional sign-in → existing `handle-onboarding` gate → app. Do not require an account to view or skip the welcome.

## 2. No-account "try" of the core loop

- [ ] 2.1 Add a license-clean **bundled demo piece** (reuse a fixture/public-domain asset) and a "Try it now" entry from the welcome that plays it via the existing player + `performance-scoring` seam — **without** sign-in and without opening the authenticated hub.
- [ ] 2.2 After the demo's end-of-session summary, offer (not force) sign-in with the benefit stated; declining returns to the welcome/app.

## 3. Deferred contextual sign-in

- [ ] 3.1 A reusable "sign-in invitation" surface that names the benefit for a gated action (save library / earn points / join leaderboards / go public); shown when a signed-out user triggers such an action.
- [ ] 3.2 Declining returns the user to what they were doing (no dead-end); accepting completes sign-in and **resumes the intended action**.

## 4. Unified progressive coaching

- [ ] 4.1 A shared coach-mark/spotlight widget: one-time, dismissible, non-blocking, with per-hint "seen" state (`shared_preferences`).
- [ ] 4.2 Route the first-run hints from #2 (rating deck), #4 (reward feedback), and #5 (age gate) through this shared mechanism instead of ad-hoc implementations.

## 5. Help/tips

- [ ] 5.1 A help/tips screen reachable from a stable entry (settings/profile) covering the core loop, ratings, points, shop/badges, profile, leaderboards, and going public.

## 6. Cross-cutting

- [ ] 6.1 Localize all welcome/coaching/help copy via `app-localization`.
- [ ] 6.2 Accessibility: dismissible without a single required gesture, contrast, screen-reader labels; landscape/`responsive-layout` correct.
- [ ] 6.3 Ensure moderation surfaces never appear in user onboarding.

## 7. Tests & verification

- [ ] 7.1 Flutter (via fakes): welcome shows pre-account and is skippable; no forced sign-up; no-account demo plays and shows the summary; contextual sign-in invite states a benefit, decline keeps exploring, accept resumes the action; coach-mark shown once + re-findable in help. `flutter test --coverage` ≥ 80%.
- [ ] 7.2 `melos run analyze` + `dart format` clean; regenerate codegen as needed.
- [ ] 7.3 `openspec validate add-welcome-onboarding --strict` passes.
