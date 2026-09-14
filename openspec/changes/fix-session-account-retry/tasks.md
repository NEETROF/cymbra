## 1. Session state

- [x] 1.1 Add `accountUnresolved` to `SessionState` (`lib/state/session_state.dart`), next to `needsHandle`: true only for `SessionAuthenticated` with a null account, with a doc comment naming it as the degraded-but-live session (design D5).
- [x] 1.2 Unit-test `accountUnresolved` across all four states in `test/state/session_state_test.dart` (create if absent): authenticated-with-account → false, authenticated-without → true, guest/unauthenticated/unknown → false.

## 2. Retry loop in SessionNotifier

- [x] 2.1 Add the tunable backoff constants at the top of `lib/state/session_notifier.dart`: first delay 2s, factor 2, cap 60s — named and documented (design D3).
- [x] 2.2 Add the single-flight guard `Future<void>? _resolving` and route `_resolveAuthenticated` through it, clearing it in `whenComplete` guarded by `identical(...)`, mirroring `CoordinatedTokenRefresher` (design D4). A caller finding it non-null awaits the in-flight resolution instead of starting a second `GetAccount`.
- [x] 2.3 Add `Timer? _accountRetry` plus `_scheduleAccountRetry()` and `_stopAccountRetry()`. Schedule from the degraded branch of `_resolveAuthenticated`; cancel and reset the backoff on success (design D2).
- [x] 2.4 Add the public `refreshAccount()`: a no-op unless the session is `accountUnresolved`, otherwise re-run the (single-flight) resolution (design D7).
- [x] 2.5 Call `_stopAccountRetry()` from every teardown path — `_endLocalSession`, `onAccountDeleted`, `abandonOnboarding`, `deleteOrphanForLink`, `continueAsGuest` — and register it in `ref.onDispose` as the backstop.
- [x] 2.6 Verify a terminal `AuthError` (`unauthenticated` / `notFound`) inside a retry still clears the session and routes to entry, and that the loop stops — the existing branch must not be bypassed by the retry path.
- [x] 2.7 Delete the stale "handle onboarding is re-checked when back online" claim in `_resolveAuthenticated`'s doc comment and describe what the code now actually does.

## 3. Foreground gating

- [x] 3.1 Add `onForeground()` (attempt immediately, reset the backoff) and `onBackground()` (cancel the pending timer, leave the session untouched) to `SessionNotifier` (design D1).
- [x] 3.2 Rename `_AudioLifecycleObserver` to `_AppLifecycleObserver` in `lib/main.dart` — it already refreshes flags and the daily quota, so the audio-only name is wrong before this change and misleading after it.
- [x] 3.3 Wire the observer: `onBackground()` on `paused`/`hidden`/`detached` (the existing audio-cut branch), `onForeground()` on `resumed` alongside the existing flag and quota refreshes.

## 4. Own-profile recovery UI

- [x] 4.1 Split the `targetId == null` branch in `lib/screens/profile_screen.dart` three ways: resolved target → body; self-view (`userId == null`) with an `accountUnresolved` session → retry affordance; anything else → the unchanged `profileUnavailable` message (design D6).
- [x] 4.2 Wire the retry button to `ref.read(sessionNotifierProvider.notifier).refreshAccount()` **without awaiting it** — the screen already watches `currentUserIdProvider` and rebuilds into the profile body when the account resolves.
- [x] 4.3 Add the new strings to `lib/l10n/app_en.arb` with a description, then translate into `app_fr.arb`, `app_es.arb` and `app_it.arb` — all four locales aligned, no drift.

## 5. Tests

- [x] 5.1 `test/state/session_notifier_test.dart`: a transient `GetAccount` failure at `onSignedIn` leaves the session degraded, then a later attempt succeeds and the account, handle and user id become available.
- [x] 5.2 Same for the launch path (`_hydrate` with stored tokens).
- [x] 5.3 Backoff under `fake_async`: successive failures are spaced 2s, 4s, 8s… capped at 60s, and no attempt fires early (follow the pattern in `test/services/http_seam_deadlines_test.dart`).
- [x] 5.4 **No retry while backgrounded**: after `onBackground()`, advancing `fake_async` well past the cap issues zero `getAccount` calls.
- [x] 5.5 `onForeground()` on a degraded session attempts immediately and resets the backoff; on an already-resolved session it issues nothing.
- [x] 5.6 Single-flight: `refreshAccount()` (or `onForeground()`) called while a resolution is in flight issues exactly one `getAccount`.
- [x] 5.7 Sign-out during a scheduled retry cancels it — no `getAccount` after the teardown, verified by advancing `fake_async`.
- [x] 5.8 A terminal failure inside a retry clears the session, routes to entry, and schedules no further attempt.
- [x] 5.9 `test/screens/profile_screen_test.dart`: a degraded self-view shows the retry affordance and not `profileUnavailable`; tapping it drives the notifier; the profile body appears once the account resolves.
- [x] 5.10 `test/screens/profile_screen_test.dart`: another player's unavailable profile still shows `profileUnavailable` with no retry offered.
- [x] 5.11 **Amended during apply.** Reuse and extend the existing hand fakes in `test/support/auth_fakes.dart` (added `FakeAccountService.getErrors`, a scripted per-call error list mirroring the `FakeAuthService.linkErrors` idiom) instead of introducing mockito. `session_notifier_test.dart` and the whole auth suite are already built on these shared fakes; mixing a second double style into the same container overrides would be worse than the `flutter-testing` default it would satisfy. Injection still goes through `ProviderContainer` overrides.

## 6. Verification

- [x] 6.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`.
- [x] 6.2 `melos run analyze`, `dart format` from the repo root, and `cd apps/music && dart run custom_lint` all clean.
- [x] 6.3 `flutter test --coverage --exclude-tags golden` green with line coverage ≥ 80%.
- [x] 6.4 `openspec validate fix-session-account-retry --strict` passes.
- [ ] 6.5 Manual check on macOS: sign in with email + password while the backend is unreachable (degraded session), restore it, confirm the `@handle` and the profile come back with no sign-out; then background the app while degraded and confirm no `GetAccount` is issued until it is foregrounded again.
