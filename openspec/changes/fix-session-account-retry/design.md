## Context

`SessionNotifier` (`apps/music/lib/state/session_notifier.dart`) is the single
source of truth for the account session. It resolves an authenticated session
through one private method:

```dart
Future<void> _resolveAuthenticated() async {
  try {
    final account = await _account.getAccount();
    state = SessionState.authenticated(account: account);
  } on AuthException catch (e) {
    if (e.error == AuthError.unauthenticated || e.error == AuthError.notFound) {
      await _tokens.clear();
      state = const SessionState.unauthenticated();   // terminal
    } else {
      state = const SessionState.authenticated();     // degraded: account == null
    }
  }
}
```

The degraded branch is deliberate and correct — a flaky network must not sign the
user out. The defect is that it is a **one-way door**. `_resolveAuthenticated` is
reached from exactly two places (`_hydrate` at launch, `onSignedIn` after any
sign-in method), `getAccount()` has exactly one call site in the app, and the
lifecycle observer in `main.dart` refreshes only feature flags and the daily
quota. Nothing re-enters the resolution. The code comment above the degraded
branch already claims "handle onboarding is re-checked when back online" — that
re-check was never implemented.

Constraints this design has to respect:

- **Riverpod 2 layering** (CLAUDE.md): only notifiers call services; a provider
  never imperatively invalidates a sibling; the UI never awaits a notifier
  action's return and branches on it; `build()` never touches `state` before
  returning.
- `SessionNotifier` is `@Riverpod(keepAlive: true)` and lives for the whole app
  session, so anything it schedules must be explicitly cancellable and cancelled
  on dispose.
- Coverage gate ≥ 80%, and the timing behaviour has to be testable without real
  waits. `fake_async` is already a dev dependency (`pubspec.yaml`), used by
  `test/courses/lesson_sounder_test.dart` and
  `test/services/http_seam_deadlines_test.dart`.
- The app runs on desktop (macOS — where this was reported — Windows, Linux) as
  well as mobile. Backgrounding semantics differ between them, which is the whole
  reason the foreground gate is a requirement rather than an optimisation.

## Goals / Non-Goals

**Goals:**

- A session degraded by a transient `GetAccount` failure recovers on its own,
  with no sign-out and no restart.
- Retries never run outside the foreground: no timer, no RPC.
- Retries never hot-loop an unreachable backend, and never double-fire.
- A signed-in user opening their own profile in the degraded state is told the
  truth (temporary, retryable) rather than "This profile isn't available."

**Non-Goals:**

- Adding a connectivity-listener package (`connectivity_plus` or similar). The
  foreground gate plus backoff covers the reported failure; a connectivity
  trigger is a later refinement if telemetry shows it is needed.
- Changing the terminal/transient classification itself. `AuthError.unauthenticated`
  and `AuthError.notFound` stay terminal; everything else stays transient. This
  change adds recovery *after* that classification, it does not revisit it.
- Any backend, proto, or Rust work. The backend already reports these failures
  correctly.
- Surfacing the degraded state anywhere other than the own-profile screen. A
  global banner is a bigger UX question; the profile screen is where the lie is
  currently told.

## Decisions

### D1 — The lifecycle signal comes from the existing observer, not a new provider seam

`main.dart:113` already installs a single `WidgetsBindingObserver` that owns the
app's lifecycle fan-out: it cuts audio on background and refreshes flags + the
daily quota on foreground. The session hooks go there, as two calls on the
notifier:

```dart
} else if (state == AppLifecycleState.resumed) {
  unawaited(container.read(sessionNotifierProvider.notifier).onForeground());
  ...
} else /* paused | hidden | detached */ {
  container.read(sessionNotifierProvider.notifier).onBackground();
  ...
}
```

*Alternative considered:* an injectable `appLifecycleProvider` that
`SessionNotifier` `ref.listen`s. Rejected — it buys no testability (plain notifier
methods are directly callable from a unit test, with no widget binding and no fake
stream to pump), and it would add a cross-provider subscription for a signal that
already has exactly one well-defined publisher.

Consequence: `_AudioLifecycleObserver` is renamed `_AppLifecycleObserver`. The
name was already wrong — it refreshes flags and quota — and adding session
recovery to a class called "audio" would make it actively misleading. It is
private to `main.dart`, so the rename touches nothing else.

`detached` currently falls into the same branch as `paused`/`hidden` for audio;
the session cancellation follows the same grouping. Any state that is not
`resumed` means "do not retry".

### D2 — Cancellable `Timer`, not a `Future.delayed` loop

The pending re-attempt is a single `Timer?` field on the notifier.
`Future.delayed` cannot be cancelled, so a backgrounding app would still fire its
continuation — precisely the behaviour this change exists to prevent. The timer is
cancelled in `onBackground()`, in `ref.onDispose`, and on every session teardown
path (`_endLocalSession`, `onAccountDeleted`, `abandonOnboarding`,
`deleteOrphanForLink`, `continueAsGuest`).

Cancellation is centralised in one private `_stopAccountRetry()` so a future
teardown path added without thinking about the timer is a one-line fix rather
than a leak. The dispose hook is the backstop.

### D3 — Backoff schedule: 2s doubling to a 60s cap, reset on foreground

Delays are `2s, 4s, 8s, 16s, 32s, 60s, 60s, …` while the app stays in the
foreground and the account stays unresolved. At the cap that is one call per
minute, each already bounded by the 10s `kInteractiveDeadline` — negligible load,
and fast enough that a network blip is invisible to the user.

Returning to the foreground **resets the backoff to the first delay and attempts
immediately**. A user who backgrounds the app for an hour and comes back should
not inherit a 60s wait accumulated before they left; a foreground return is also
the single most likely moment for connectivity to have changed.

Retries continue indefinitely while degraded rather than stopping after N
attempts. A bounded attempt count would strand exactly the case that motivated
this change — an app left open on a desktop for hours.

*Jitter deliberately omitted.* Clients desynchronise naturally (the schedule is
anchored on each client's own sign-in or foreground moment), so the
thundering-herd risk on a backend recovery is low, and a random component would
make the backoff tests non-deterministic. Revisit if a real recovery shows a
spike.

Both constants live at the top of the file, named, so tuning them does not mean
reading the loop.

### D4 — Single-flight, mirroring `CoordinatedTokenRefresher`

An `Future<void>? _resolving` field holds the in-flight resolution; a caller that
finds it non-null awaits it instead of starting a second one, and it is cleared
in `whenComplete` guarded by `identical(...)`. This is the exact shape already
proven in `CoordinatedTokenRefresher` (`services/token_refresher.dart`) — matching
it keeps one coordination idiom in the codebase instead of two.

This matters concretely: the user tapping *retry* on the profile screen while a
timer-driven attempt is in flight, or a foreground event arriving during one,
must not issue a second `GetAccount`.

### D5 — "Degraded" is a named predicate on `SessionState`

`SessionState` gains a getter next to the existing `needsHandle`:

```dart
/// Authenticated, but the account could not be fetched (transient failure).
/// The session is live yet carries no identity — see the recovery loop.
bool get accountUnresolved => switch (this) {
  SessionAuthenticated(:final account) => account == null,
  _ => false,
};
```

Naming the condition once keeps the notifier, the profile screen and the tests
from each re-deriving `is SessionAuthenticated && account == null`, and gives the
state a documented name to talk about.

### D6 — The profile screen distinguishes "unresolved identity" from "unavailable profile"

`ProfileScreen.build` currently collapses both into one branch:

```dart
final targetId = userId ?? selfId;
... targetId == null ? Center(child: Text(l10n.profileUnavailable)) : body
```

It becomes a three-way split: a resolved target renders the body; a **self-view
with an unresolved identity** (`userId == null` and the session is
`accountUnresolved`) renders a retry affordance; anything else keeps
`profileUnavailable` unchanged. Another player's private or ineligible profile is
untouched.

The retry button calls `ref.read(sessionNotifierProvider.notifier).refreshAccount()`
and does **not** await it — the screen already watches `currentUserIdProvider`, so
it rebuilds into the profile body when the account resolves. That satisfies the
"never await a notifier action's return in the UI" rule for free, and needs no
listener widget because there is no side effect to isolate (no navigation, no
snackbar) — only a rebuild from watched state.

### D7 — `refreshAccount()` is the public entry point

One public action serves both the UI retry and the lifecycle hooks. It is a no-op
unless the session is `accountUnresolved`, so a caller can invoke it
unconditionally without knowing the session shape — which is what lets
`onForeground()` stay a two-line method.

## Risks / Trade-offs

- **A retry loop that outlives what it is retrying** → the timer is cancelled from
  one central `_stopAccountRetry()`, called from every teardown path and from
  `ref.onDispose`; a test asserts no attempt fires after sign-out.
- **Retrying a session that is actually dead** → a terminal `AuthError`
  (`unauthenticated` / `notFound`) inside a re-attempt goes through the same
  branch as the first attempt: clear the session, route to entry. The loop cannot
  keep a zombie session alive.
- **Load on a recovering backend** → 60s cap, foreground-only, one call per client
  per minute, each with a 10s deadline. Accepted without jitter (D3); revisit if a
  real recovery shows a spike.
- **Timing tests going flaky** → all backoff and lifecycle tests run under
  `fake_async` with no wall-clock waits, following the existing pattern in
  `test/services/http_seam_deadlines_test.dart`.
- **The window before the first successful retry stays degraded** → unchanged
  behaviour for up to a couple of seconds, and identity-keyed features remain off
  during it. Play sessions completed inside that window are still dropped by
  `play_sync_notifier.dart:132`. Durably queueing an unattributed session is a
  separate concern with its own storage and attribution questions; it is out of
  scope here, and the window shrinks from "until sign-out" to "a few seconds".

## Migration Plan

Client-only, no data migration, no backend coordination. The change is additive:
if the retry never fires, behaviour is exactly what ships today. Rollback is a
plain revert of the client commit.

## Open Questions

- Should the account menu also signal the degraded state (a subtle indicator
  rather than a silently missing `@handle`)? Deferred: the profile screen is where
  the app currently states something false, and that is what this change fixes. A
  global signal is a broader UX decision.
- Is a connectivity listener worth its dependency later? Only if reports show
  users sitting degraded in the foreground for long stretches — the foreground
  reset should cover the common case.
