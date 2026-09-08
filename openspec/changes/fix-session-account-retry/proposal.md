## Why

A macOS user signed in with email + password and landed in a session that was
authenticated but had **no account**: no `@handle` in the account menu, and their
own profile screen showed "This profile isn't available." Signing out and back in
fixed it permanently.

`SessionNotifier._resolveAuthenticated` treats any non-terminal `GetAccount`
failure (deadline exceeded, `UNAVAILABLE`, or the `RefreshTransient` path that
`authedCall` converts into `GrpcError.unavailable`) as "stay signed in with an
unknown account" — `SessionState.authenticated(account: null)`. That is the right
call. What is missing is the other half: **nothing ever re-resolves it.**
`getAccount()` has exactly one call site in the whole app
(`session_notifier.dart:77`), the app-resume handler refreshes only feature flags
and the daily quota, and no screen retries. One flaky RPC at sign-in therefore
strands the session until the user signs out or restarts.

Two shipped requirements are already violated by this gap:
`account-access` promises the app keeps the user signed in "resolving the account
when connectivity returns", and `public-player-profile` promises "Owner always
sees their own profile". Neither holds today.

The damage is far wider than the reported symptom, because everything keyed on
`currentUserIdProvider` silently degrades while the user believes they are signed
in: play sessions are **not captured at all** (`play_sync_notifier.dart:132`
returns early on a null id — no heatmap, no streak, no leaderboard entry), feature
flags lose their user identity so `beta:*` and `premium_only` rollouts stop
matching (`flags_integration.dart:44`), and the favorites index, offline score
cache and plan/entitlement all go quiet. None of it surfaces to the user.

## What Changes

- `SessionNotifier` gains a **`refreshAccount()`** action that re-runs the account
  resolution for a token-bearing session, and an internal **retry loop with
  bounded backoff** that runs only while the session is degraded (authenticated
  with a null account) and stops as soon as the account resolves or the session
  ends.
- The retry loop is **lifecycle-gated**: it is started/reset on
  `AppLifecycleState.resumed` and **cancelled** on `paused`/`hidden`/`detached`,
  so no timer and no RPC survive into the background. This matters on desktop,
  where a backgrounded app keeps running (unlike mobile, where the process is
  frozen) and would otherwise burn battery and RPCs against a backend the user
  is not looking at.
- Resolution is **single-flight**: a resume arriving while a retry is already in
  flight joins it instead of firing a second `GetAccount`, mirroring the existing
  `CoordinatedTokenRefresher` guarantee one layer up.
- The own-profile screen stops claiming the profile does not exist when the
  problem is an unresolved local identity: a degraded self-view offers an
  explicit **retry** instead of "This profile isn't available." (Another player's
  genuinely unavailable profile keeps the existing message.)
- New user-facing strings are added to all four locales (`en`, `fr`, `es`, `it`).

Not in scope: adding a connectivity-listener dependency. Lifecycle + backoff
covers the reported failure without a new package; a connectivity trigger can be
layered on later if it proves necessary.

No breaking changes. No backend, proto, or Rust change — this is entirely
client-side recovery from a failure the backend already reports correctly.

## Capabilities

### New Capabilities

None — this fills gaps in two existing capabilities rather than introducing
behaviour.

### Modified Capabilities

- `account-access`: the existing "Silent token refresh" requirement promises the
  account is resolved "when connectivity returns" but no requirement says *how* or
  *when* the app re-attempts it. Add a requirement covering account re-resolution:
  a degraded session retries with bounded backoff, retries are suspended while the
  app is not in the foreground and resumed on foreground, and concurrent attempts
  are coordinated into one call.
- `public-player-profile`: "Owner always sees their own profile" needs a scenario
  for the case where the *client* has not resolved the owner's identity — the app
  SHALL offer recovery rather than report the profile as unavailable, which is a
  false statement about the account.

## Impact

**Products.** Cymbra Music (`apps/music`) only. It *consumes* Cymbra ID unchanged
— no new RPC, no proto change, no `id-*` backend work. Cymbra Live, the back
office and the public site are untouched.

**Code.**
- `apps/music/lib/state/session_notifier.dart` — `refreshAccount()`, the
  backoff/single-flight retry state, lifecycle entry points, cancellation on
  dispose and on every session teardown path.
- `apps/music/lib/main.dart` — `_AudioLifecycleObserver` gains the
  `paused`/`resumed` wiring. It already refreshes flags and the daily quota, so
  the "audio" name is now plainly wrong; rename it to `_AppLifecycleObserver`.
- `apps/music/lib/screens/profile_screen.dart` — degraded self-view branch.
- `apps/music/lib/l10n/app_{en,fr,es,it}.arb` — retry copy, all four aligned.

**Indirectly repaired** (no edits needed — they simply start receiving a non-null
user id again): play-session capture and sync, feature-flag user targeting, the
favorites index, the offline score cache key, and plan/entitlement resolution.

**Tests.** `test/state/session_notifier_test.dart` (degraded → retry → recovered;
backoff bounded; **no** retry fires while paused; resume restarts it; single-flight
under a concurrent resume; loop stops on sign-out) and
`test/screens/profile_screen_test.dart` (degraded self-view shows retry and drives
the notifier, not the service). Both files already exist. Coverage stays ≥ 80%.

**Risk.** A retry loop against a backend that is down must not become a hot loop —
hence bounded backoff with a cap, foreground-only execution, and a hard stop once
the session is no longer degraded.
