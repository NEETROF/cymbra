## Context

The Flutter app (`apps/music`) hydrates its account session at launch in
`SessionNotifier._hydrate` → `_resolveAuthenticated`, which calls
`getAccount()`. Every authenticated gRPC adapter wraps its calls in
`authedCall` (`services/grpc_client.dart`): run with the current access token,
and on `UNAUTHENTICATED` call a `refreshAccessToken` callback, then retry once.

Two structural problems in the refresh path cause random silent sign-outs:

1. `_refreshAccess` is **duplicated in six adapters** (grpc auth + account,
   play-sync, profile, catalog, score-upload, rating). Each independently reads
   the stored refresh token and calls `Refresh`. There is no coordination, so
   concurrent authenticated calls at launch replay the same refresh token. The
   backend (`backend/auth/src/session.rs`) rotates refresh tokens with reuse
   detection that **revokes the whole family** on replay — killing even the
   winning refresh.
2. Every copy's `catch (_) { await _tokenStore.clear(); return null; }` treats
   *any* exception — including a transient `UNAVAILABLE`/timeout on weak Wi-Fi —
   as terminal and wipes the session. Because a launch refresh is triggered by
   the original `UNAUTHENTICATED`, the failure resurfaces as `UNAUTHENTICATED`
   and defeats the "offline stays signed in" guard in `_resolveAuthenticated`.

Both surface as: entry screen at launch, no message, must re-sign-in with Apple.

Constraints: Riverpod 2 + Freezed, provider-injected dependencies, mockito
doubles, ≥80% coverage (CLAUDE.md). Errors must never leak raw gRPC strings to
the UI. No backend or wire-protocol change is in scope.

## Goals / Non-Goals

**Goals:**
- One in-flight `Refresh` per stored session; concurrent callers share its result.
- Clear the local session only on a genuine refresh-token rejection
  (`UNAUTHENTICATED` / `INVALID_ARGUMENT`); keep it on transient/offline failures.
- Preserve the existing `_resolveAuthenticated` "offline ⇒ stay signed in
  (unknown account)" behavior for a refresh that was *needed* but failed
  transiently.
- Remove the six duplicated `_refreshAccess` bodies in favor of one seam.

**Non-Goals:**
- No backend changes (rotation/reuse semantics stay as-is).
- No new user-facing UI or error surface (still silent recovery; entry only on a
  real rejection).
- No change to storage keys, sign-in flows, or the `authedCall` retry-once shape.
- Not adding proactive/pre-emptive refresh (refresh stays reactive to
  `UNAUTHENTICATED`).

## Decisions

### D1 — A single shared `TokenRefresher` seam (single-flight)

Introduce `TokenRefresher` (abstract) with one production implementation, exposed
as a `@Riverpod(keepAlive: true)` provider and injected into every authenticated
adapter (replacing their private `_refreshAccess`). It owns the stored refresh
token read, the `Refresh` call, the rotated-pair write, and the outcome
classification.

Single-flight: the refresher holds a nullable `Future<RefreshOutcome>?
_inFlight`. A caller that finds `_inFlight != null` awaits it; otherwise it
starts one and stores the future, clearing it in a `whenComplete`. This
guarantees N concurrent callers ⇒ one `Refresh`, so the stored refresh token is
never replayed by a sibling. `keepAlive` makes the single instance the shared
coordination point across all adapters.

*Alternative considered:* a coalescing lock inside `TokenStore`. Rejected —
`TokenStore` is a dumb secure-storage seam; refresh coordination is auth policy
and belongs with the refresher, keeping storage trivially fakeable in tests.

### D2 — Classify the outcome; return it, don't just null-or-token

The refresher returns a small result instead of `String?`:

```
sealed RefreshOutcome:
  Refreshed(String accessToken)   // rotated pair stored
  Rejected                        // refresh token expired/revoked -> session cleared
  Transient                       // offline/timeout -> session left intact
```

- `Refreshed` ⇒ store the rotated pair, return the new access token.
- `Rejected` (map from `AuthError.unauthenticated` / `invalidArgument`) ⇒ clear
  the store.
- `Transient` (everything else: `unavailable`, deadline exceeded, `unknown`,
  offline) ⇒ **do not** touch the store.

`authedCall` maps the outcome back to its callback contract: `Refreshed` ⇒ retry
with the new token; `Rejected` ⇒ propagate `UNAUTHENTICATED` (terminal, as
today); `Transient` ⇒ throw a **non-`UNAUTHENTICATED`** error (surface the
transient gRPC status, e.g. `UNAVAILABLE`) so `_resolveAuthenticated`'s `else`
branch keeps the user signed in. This is the crux that reconnects the existing
offline guard.

*Alternative considered:* keep the `Future<String?>` signature and infer
transient-vs-terminal from a side flag. Rejected — an explicit sealed outcome is
self-documenting and directly testable.

### D3 — `authedCall` signature change is internal-only

`authedCall<T>` swaps its `Future<String?> Function() refreshAccessToken` +
`void Function() onExpired` for a single `Future<RefreshOutcome> Function()
refresh`. All call sites are the six adapters we are already touching; the
retry-once behavior and the `isUnauthenticated` hook are unchanged. `onExpired`
was a no-op at every call site, so it is dropped.

### D4 — `_resolveAuthenticated` stays as the routing authority

No behavior change beyond what D2 enables: it still clears on `unauthenticated`
/ `notFound` and stays authenticated (unknown account) otherwise. With D2 a
transient launch refresh now arrives here as a transient error (not
`UNAUTHENTICATED`), so the user is kept signed in exactly as the code comment
already promises. No new clearing paths are added.

## Risks / Trade-offs

- **A rotated pair is written while other callers await the same future** → all
  waiters receive the same `Refreshed` token; the write happens once inside the
  single flight before completion, so no waiter reads a half-written store.
- **Misclassifying a terminal error as transient would keep a dead session** →
  the user would see repeated failed calls but stay on the library instead of
  being routed to entry. Mitigation: classification keys only on the two
  unambiguous terminal codes (`UNAUTHENTICATED`, `INVALID_ARGUMENT`); everything
  else is transient (fail-open to "stay signed in"), which is the safer default
  for this bug and recovers on the next successful call.
- **Genuine revocation now needs one real `UNAUTHENTICATED` from `Refresh`** to
  sign out (a transient error no longer does) → acceptable and intended; a
  revoked refresh token returns `UNAUTHENTICATED`, so real sign-out still works.
- **Single-flight instance lifetime** → provider is `keepAlive`; if it were ever
  rebuilt mid-flight the in-flight future would be lost, but nothing invalidates
  it, and a rebuild would at worst allow one extra refresh (degrades to today's
  behavior, not worse).

## Migration Plan

Pure client-side refactor, no data or schema migration. Land behind the normal
review; no flag needed. Rollback is a straight revert (no persisted state shape
changes — same secure-storage keys, same token pair).

## Open Questions

- Should a `Transient` refresh at launch schedule an automatic re-hydrate when
  connectivity returns, or is next-launch/next-call recovery enough? Proposed:
  next-call recovery for this change (the app already re-checks the account when
  online); a connectivity-triggered re-hydrate can be a follow-up.
