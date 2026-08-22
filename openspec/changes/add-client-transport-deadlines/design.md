## Context

Two transports reach the backend from `apps/music`, and neither is bounded.

**gRPC** (`grpc: 4.2.0`). `bearerOptions(String? token)`
([grpc_client.dart:122](../../../apps/music/lib/services/grpc_client.dart)) is the
only `CallOptions` constructor in `lib/`; it carries auth metadata and nothing else.
It is passed at **60+ call sites across 16 adapters**
(`GrpcCatalogService`, `GrpcScoreUploadService`, `GrpcAccountService`,
`GrpcCourseCatalogService`, `GrpcRatingService`, `GrpcLeaderboardService`,
`GrpcGlobalLeaderboardService`, `GrpcPlaySyncService`, `GrpcStreakService`,
`GrpcProfileService`, `GrpcNotificationRegistryService`, `GrpcAchievementsService`,
`GrpcCuratorRewardsService`, `GrpcSoundFontCatalogService`, `GrpcUsageTrackingService`,
`GrpcCourseProgressService`), plus the `AuthServiceClient` that `tokenRefresher`
builds directly at `grpc_client.dart:398` — that one bypasses every adapter and is
easy to miss.

**HTTP** (`package:http`). `soundfont_source.dart`,
`soundfont_preview_service.dart`, `score_preview_service.dart` and
`private_soundfont_service.dart` call `_client.get/post/delete` with no timeout.
The 400 MiB private-SoundFont import is **HTTP, not gRPC** —
`HttpPrivateSoundFontService.import` — so the "some RPCs are legitimately long"
constraint mostly lands on the transport that is *not* the one being capped.

Three facts verified in the `grpc-4.2.0` source, because the design depends on them:

1. `ClientCall` arms its deadline with `Timer(options.timeout!, _onTimedOut)` **in its
   constructor** (`call.dart:226`), i.e. at call creation, before a connection exists.
   A per-call deadline therefore already bounds connect time; `connectTimeout` is
   belt-and-braces, not the fix.
2. `Client.$createUnaryCall` builds the interceptor chain and invokes it with
   `_options.mergedWith(options)` (`client.dart:65`) — an interceptor sees the merged
   client+call options and **its** returned options are what reach `createCall`.
3. `CallOptions.mergedWith(other)` resolves timeouts as `other.timeout ?? timeout`
   (`call.dart:95`) — the argument wins. Merge direction is load-bearing (see D3).

One correction to the obvious reading of `ChannelOptions`: `connectionTimeout`
(default 50 min) is *"the maximum time a single connection will be used for new
requests"* — connection reuse. The connect bound is `connectTimeout`, which defaults
to `null` → the OS timeout. Setting `connectionTimeout` would do nothing for this bug.

## Goals / Non-Goals

**Goals:**

- An unreachable backend surfaces an error in seconds, not tens of seconds.
- A connection lost *during* a load ends that load at once, rather than after its
  deadline — the device already knows, so the app should too.
- A load that cannot possibly succeed opens no socket at all.
- A silently-dead connection is discovered without waiting on a call deadline.
- The user is never trapped in a blocking wait, whatever bound applies to it.
- Long operations (score upload, 400 MiB SoundFont import) keep working unchanged.
- A timeout is indistinguishable from `UNAVAILABLE` to every downstream consumer, so
  the offline score cache, session refresh and UI messaging keep their behaviour.
- Adding a new RPC later gives it a sane deadline **by default**, with no ceremony.
- The whole policy is readable in one file and testable without a server.

**Non-Goals:**

- Retries, backoff, hedging, or circuit breaking. Out of scope; `authedCall`'s
  existing refresh-once-on-`UNAUTHENTICATED` retry is untouched. In particular, a
  sticky "backend is down" state that would let calls 2..N fail instantly after the
  first failure is deliberately deferred — it is a behaviour change with its own
  staleness questions, not a bound.
- Reachability probing beyond what the OS reports (no health-check RPC, no captive-
  portal detection). The pre-flight check trusts the OS negatively only (D8).
- Server-side deadline propagation / `tonic` honouring `grpc-timeout`.
- The back-office (Vue) grpc-web client, and Cymbra Live.
- Streaming RPCs — the app currently issues none (the only `Stream`s in `services/`
  are local `StreamController`s in `audio_routing_service.dart`).
- Per-user or remotely-tunable budgets. Constants; revisit if field data justifies it.

## Decisions

### D1 — One `ClientInterceptor` per client, not a parameter on `bearerOptions`

The obvious move is `bearerOptions(bearer, deadline: …)`. Rejected:

- It edits 60+ call sites, and every one is a place to get it wrong or forget.
- It is opt-in: a new RPC written without the argument is silently unbounded again —
  exactly the state we are fixing.
- It conflates two concerns in one helper (who you are; how long you'll wait).

Instead, a `RpcDeadlines implements ClientInterceptor` is installed once per generated
client — `ScoreServiceClient(channel, interceptors: [deadlines])` — which the
generated constructors already accept (`ScoreServiceClient(super.channel, {super.options,
super.interceptors})`). That is **one line per adapter**, `bearerOptions` keeps its
signature, and the default is fail-safe: an RPC added tomorrow is bounded whether or
not its author thought about it.

Alternative considered and rejected: client-level `options: CallOptions(timeout: …)`
at construction. It is equally central but only expresses **one** budget per client,
and `ScoreServiceClient` is shared by catalog reads *and* `UploadScore` — the exact
pair that needs different budgets. The interceptor sees `method.path`, so it can
discriminate.

### D2 — Category resolved from `method.path`, in one table

The interceptor receives `ClientMethod.path` (e.g.
`/cymbra.score.v1.ScoreService/UploadScore`). The policy is a `Map<String, Duration>`
of explicit overrides plus a default of `interactive`:

```
transfer (30s): GetCatalogScoreBytes, GetScoreBytes, GetRatingPreviewBytes,
                GetCourseManifest
long   (120s):  UploadScore
default (10s):  everything else
```

Listing only the exceptions keeps the table short and makes the default the safe one.
The cost is that a *new* long RPC inherits 10 s and breaks loudly on first use —
acceptable, and far better than the reverse failure mode (a new interactive RPC
silently inheriting 120 s). A task adds a comment at the table pointing this out.

Alternative rejected: keying the table on a category enum threaded through each
adapter method. More code, more plumbing, same result, and it drifts from the
generated names.

### D3 — Merge direction: policy is the base, call options win

The interceptor MUST do `CallOptions(timeout: policy).mergedWith(options)` and not the
reverse. With `mergedWith` resolving `other.timeout ?? timeout`, base-then-override
means an explicit per-call `timeout` survives, while the reverse would silently
overwrite it. This is the single most invertible line in the change, so it gets its
own test.

### D4 — `DEADLINE_EXCEEDED` joins the `unavailable` bucket, and ships here

`authErrorFromCode` ([auth_service.dart:65](../../../apps/music/lib/services/auth_service.dart))
maps 3/5/6/8/9/10/14/16 and defaults to `AuthError.unknown`. Code 4 is absent.

This is not a cosmetic gap.
[`notation_notifier.dart:281`](../../../apps/music/lib/state/notation_notifier.dart)
gates the offline-cache fallback on `e.error == AuthError.unavailable`, and `_classify`
maps `unavailable → ScoreLoadFailure.unavailable`, everything else → `generic`. Adding
deadlines **without** this mapping would turn a slow-but-correct failure into a fast
failure that skips the cache and shows the wrong message — a regression in the exact
feature the deadline was supposed to help. The two changes are one change.

Chosen: reuse `AuthError.unavailable` rather than introduce `AuthError.timedOut`.
A new enum value would force every `switch` over `AuthError` to grow an arm, and no
consumer wants to distinguish the two — the spec requires they behave identically. If
a consumer ever needs the distinction, the `GrpcError` message is still carried on
`AuthException`.

### D5 — HTTP: wall-clock on control calls, no wall-clock on bulk

`package:http`'s only bound is `Future.timeout`, which is wall-clock over the whole
request — response body included. That is correct for the small JSON/control calls
(`list`, `delete`, `propose`: `interactive`, 10 s) and for the bounded media fetches
(preview clips, score preview: `transfer`, 30 s).

It is **wrong** for `import`/`download` of SoundFont bytes: at 400 MiB, any number
large enough to be safe on a slow connection is too large to be a useful bound, and
any number small enough to be useful truncates real uploads. So bulk transfers get
**no wall-clock timeout**, with a comment saying so, and are bounded at connection
setup only. Picking an arbitrary "generous" number here would be a latent bug that
only fires for users on poor connections with big files — the worst possible
population to fail on silently.

Alternative for later, deliberately not taken now: move the bulk seams to
`HttpClient` (`dart:io`) which exposes `connectionTimeout` and `idleTimeout`
(no-progress) separately. That is the right long-term shape but rewrites four working
seams and their tests; it is noted as a follow-up rather than smuggled in here.

### D6 — Set `connectTimeout` anyway

Per fact (1) the per-call deadline already covers connect, so `connectTimeout: 10s` on
`cymbraChannel` is redundant for gRPC calls — but it is one line, it bounds channel
setup for any future path that doesn't go through a deadline-carrying call, and it
documents intent next to the credentials. Cheap defence in depth.

### D7 — Abort on the connectivity transition by **racing**, not by cancelling

The user-visible bug is that airplane mode does not stop the loader. The device knows
it is offline within milliseconds; the app finds out only when the call gives up.

The direct fix would be to cancel the in-flight call — grpc-dart exposes
`ResponseFuture.cancel()` (`common.dart:41`). Rejected: the adapters return plain
`Future<T>`, not `ResponseFuture<T>` (e.g. `CatalogService.fetchScoreBytes`), so the
cancel handle is **erased at the seam**. Recovering it means changing the return type
of every adapter method the notifiers await, and threading a cancellation token
through 16 adapters — a large, invasive diff for an abort we can get for free.

Chosen: **race** the awaited work against the connectivity transition inside the
notifier. First to resolve wins; if the offline signal wins, the notifier goes
straight to its offline outcome and the orphaned RPC dies on its own deadline (D1) in
the background, harmlessly. No adapter signature changes.

Two implementation constraints that are easy to get wrong:

- **The subscription must be released when the work wins.** `onlineStatus` is a
  broadcast stream from `connectivity_plus`; a `firstWhere` that is never satisfied
  keeps its subscription alive, so every score open would leak one. The helper owns an
  explicit `StreamSubscription` and cancels it in a `finally`.
- **Only the *transition* counts, not the current value.** The race listens for a
  `false` **event**; it must not re-read the current state, or a load starting while
  `onlineStatus` has already emitted `false` would be decided by the pre-flight check
  (D8) rather than by the race, and the two would double-fire.

A cancellation token remains the better long-term shape if a second consumer ever
needs to abort for a non-connectivity reason. Noted, not built.

### D8 — Pre-flight check generalizes an existing pattern

`_decideCachedCatalogOpen` already gates on
`await ref.read(connectivityServiceProvider).isOnline()`
([notation_notifier.dart:164](../../../apps/music/lib/state/notation_notifier.dart))
before hitting the network for a *cached* catalog piece. The cache-**miss** path
(`_fetchScoreBytes`, line 112) has no such gate — so a load that cannot possibly
succeed still opens a socket and waits. Extending the existing check there is a small,
consistent edit, not a new mechanism.

The reading is trusted **negatively only**. `connectivity_plus` reports interface
state, not reachability: a captive portal or a down backend both report "online". So
`false` short-circuits, `true` proves nothing and the call proceeds under its deadline.
`ConnectivityPlusService.isOnline()` already fails closed (`catch (_) → false`), which
is the right bias here too — worst case we serve the cache.

### D9 — Enable gRPC keepalive, with idle pings off

`ClientKeepAliveOptions` is fully implemented in grpc 4.2.0 (`client_keepalive.dart`,
wired at `http2_connection.dart:113`) but **disabled by default**: `pingInterval` is
`null`, so `shouldSendPings` is false and no ping is ever sent. On ping timeout the
transport calls `transport.finish()`, which tears the connection down and fails its
in-flight calls immediately.

This is the only mechanism here that addresses an *unstable* connection rather than a
cleanly lost one: a socket the NAT dropped or that died in a network switch looks alive
to both endpoints, so neither the OS nor `connectivity_plus` reports anything, and
today only the (absent) call deadline would ever end it.

`permitWithoutCalls: false` — the default — is deliberately kept: pings flow only while
a call is in flight, which is exactly when we care, and an idle app never wakes the
radio. So the battery objection to keepalive does not apply to this configuration.

Interval and timeout are proposed at 5 s / 5 s (detection in ~10 s worst case) but must
be validated against the edge before merge: prod terminates TLS at **Caddy** in front
of tonic, and the backend sets no keepalive policy of its own
([main.rs:557](../../../backend/server/src/main.rs)). A too-aggressive ping rate can be
answered with `GOAWAY`, which would be worse than the bug. Verify, then pick.

### D10 — The blocking wait gets an exit, independent of how fast detection is

`open_score.dart:77` shows `showDialog(barrierDismissible: false)` with no cancel
affordance, popped only when the completer resolves
([open_score.dart:63](../../../apps/music/lib/screens/open_score.dart)). Every bound in
this design is a bound on *how long the user is trapped*, not on *whether* they are —
and no bound we pick is short enough to justify an inescapable modal. This is a
separate defect from the transport ones and is fixed on its own terms.

Cancelling clears `selectedScoreProvider`. That is not incidental: `_load` already
guards every state write with `if (ref.read(selectedScoreProvider) != entry) return`
(three times, including after the `catch`), so clearing the selection makes the
existing guard discard the late result. No new "was this cancelled?" flag, no race
between the cancel and the load's completion.

Cancelling is a user decision, not a failure: the dialog closes, the user is back on
the list, and **no** error banner is shown. Surfacing "load failed" for something the
user deliberately stopped is the kind of dishonest message the offline-cache work
already fought.

## Risks / Trade-offs

- **A 10 s default is too tight on a bad mobile link** → The categories exist precisely
  so this is tunable in one table, and the failure mode is a retryable error message,
  not data loss. Budgets are constants in one file; adjust after field data. The device
  repro (30+ s of spinner) is the baseline to beat.
- **A future long RPC inherits 10 s and times out in production** → It fails loudly and
  immediately on first use, not subtly; the table carries a comment naming the trap,
  and the task list adds the check to the RPC-adding path.
- **Merge direction inverted, silently clobbering explicit per-call deadlines** →
  Dedicated test (D3); it is the one line whose inversion is invisible at runtime.
- **`tokenRefresher`'s own `AuthServiceClient` is missed** → It is constructed outside
  every adapter (`grpc_client.dart:398`); it gets its own explicit task and its own
  wiring test, since a missed deadline there means a hung *sign-in* — the most visible
  possible instance of this bug.
- **Deadlines land but the offline path silently regresses** → D4 ships in the same
  change, plus a test that drives the offline fallback with a `DEADLINE_EXCEEDED`
  rather than an `UNAVAILABLE`. The in-flight `add-offline-score-cache` change is
  implemented but not archived; its scenarios are the regression suite here.
- **Bulk HTTP stays unbounded** → Accepted and explicit (D5). An unreachable host still
  fails at connect; the uncapped part is the transfer of a healthy, progressing
  connection. Follow-up noted (`dart:io HttpClient` idle timeout).

- **The race leaks a connectivity subscription per load** → The helper owns the
  subscription and cancels it in a `finally`; a dedicated test asserts release on the
  normal (work-wins) path, since a leak here is invisible until it is a lot of leaks.
- **Keepalive pings draw a `GOAWAY` from Caddy** → Validated against the deployed edge
  before merge (D9); `permitWithoutCalls: false` keeps the ping rate proportional to
  real traffic. If the edge is unhappy, the interval grows or keepalive ships disabled
  — the other four mechanisms stand on their own.
- **Pre-flight check trusts a false "online"** → Only the negative reading
  short-circuits (D8); a positive reading changes nothing about the call path.
- **Cancel leaves the notifier writing state for an abandoned load** → Cancel clears
  `selectedScoreProvider`, which the existing stale-load guards already honour (D10);
  the test drives a cancel followed by a late completion and asserts no state change.

## Migration Plan

Additive and self-contained; no schema, no wire, no server change, so no rollout
sequencing and no backfill. Reverting is `git revert` of one commit — the interceptor
and the status-code arm are independent of any stored state.

The one ordering constraint is internal: **D4's status mapping must be in the same
commit as the first deadline**, never a follow-up, or the window between them is a
live offline-cache regression.

## Open Questions

- Are the budgets (10 / 30 / 120 s) right for the worst network Cymbra actually sees?
  Proposed as informed defaults; the device repro validates the order of magnitude but
  not the exact values. Worth one pass on a throttled connection before merge.
- Should the bulk HTTP seams move to `dart:io HttpClient` for a real no-progress bound
  (D5 alternative)? Deferred — it is a separate refactor of four working seams.
- What keepalive interval does Caddy tolerate (D9)? Blocking for the keepalive task
  only; the rest of the change does not depend on the answer.
- Should the race + pre-flight live in a reusable helper for every notifier that awaits
  the network, or stay local to the notation load? Built local first — the score open is
  the one place with a blocking modal. Generalize when a second caller needs it, rather
  than designing an abstraction for one user.
