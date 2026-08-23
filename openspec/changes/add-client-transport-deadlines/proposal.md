## Why

Opening a score with a degraded connection leaves the user stuck in a **blocking,
non-dismissible modal spinner** — and switching the device to airplane mode does not
end it. Reproduced on device 2026-08-16 (30+ s before the error banner) and again by
the user in an unstable-connection state.

Three independent defects stack to produce that:

1. **No deadline on any RPC.** `bearerOptions`
   ([grpc_client.dart:122](../../../apps/music/lib/services/grpc_client.dart)) builds a
   bare `CallOptions` (auth metadata only) and is the only `CallOptions` constructor in
   `lib/`; `ChannelOptions.connectTimeout` is unset. Failure is bounded only by the OS
   TCP timeout (~75 s on Darwin).
2. **Nothing reacts to losing connectivity mid-flight.** The score-open flow's
   completer listens to `notationProvider` only
   ([open_score.dart:63](../../../apps/music/lib/screens/open_score.dart)), so the
   `ConnectivityService.onlineStatus` transition to `false` — which the OS already
   knows — changes nothing. Hence "airplane mode doesn't stop the loader".
3. **The wait has no exit.** The spinner is `showDialog(barrierDismissible: false)`
   with no cancel affordance ([open_score.dart:77](../../../apps/music/lib/screens/open_score.dart)).
   Even a perfect deadline leaves the user locked out of the app for its duration.

Fixing only (1) would take the loader from ~75 s to 10 s and leave airplane mode
still doing nothing — so this change covers bounding **and** detection, plus the exit.

The same unbounded-call hole exists on the second transport: every `package:http` seam
(soundfont delivery/preview, private soundfont library, score preview) calls
`http.Client` with no timeout at all.

A blanket short cap is wrong — a 400 MiB private-soundfont import is a legitimate
multi-minute transfer — so budgets are **per category**.

## What Changes

**Bounding**

- **A single deadline policy** maps a gRPC method path to a category
  (`interactive` / `transfer` / `long`) and its deadline, installed as one
  `ClientInterceptor` on every generated client. The 60+
  `options: bearerOptions(bearer)` call sites are **untouched** and `bearerOptions`
  keeps its signature. A per-call `CallOptions.timeout` still wins (escape hatch).
- **`ChannelOptions.connectTimeout`** is set on `cymbraChannel`. Note
  `connectionTimeout` (already 50 min by default) is *connection reuse*, not connect —
  it is not the knob.
- **The HTTP seam gets the same categories**: wall-clock timeouts on small
  JSON/control requests, and an explicitly **uncapped** budget on the two bulk-byte
  transfers, since a wall-clock cap on a 400 MiB upload is a bug, not a safeguard.

**Detection — new**

- **Losing connectivity mid-load aborts the load**, instead of waiting out the
  deadline. The in-flight work races the `onlineStatus` transition; whichever resolves
  first decides, and the losing RPC dies on its own deadline in the background.
- **A pre-flight connectivity check** short-circuits a load started while already
  offline, with no socket opened at all. This generalizes the check
  `_decideCachedCatalogOpen` already performs
  ([notation_notifier.dart:164](../../../apps/music/lib/state/notation_notifier.dart))
  to the cache-miss path that lacks it.
- **gRPC keepalive pings are enabled** (off by default in `grpc: 4.2.0`) so a
  half-open connection — the "unstable network" case, where the socket is dead but
  neither end has noticed — is detected by ping timeout rather than by the call
  deadline. Idle pings stay off, so there is no battery cost when nothing is in flight.

**Escapability — new**

- **A blocking wait on a backend call must be cancellable.** The score-open spinner
  gains a cancel affordance; cancelling clears the score selection so the existing
  stale-load guard discards the late result.
- **Leaving the upload screen abandons the upload** and claims nothing. The client
  stops waiting and discards the result; it does not say "cancelled", because the
  request may already have been applied. `MyUploads` reports the truth on its next
  refresh, and a retry cannot duplicate — `(owner_id, sha256)` is UNIQUE over the
  canonical decoded MusicXML — so `AlreadyExists` is reworded as a fact, not an error.
  Deliberately **no** `PopScope`: gating the exit on a "did it land?" request would
  make it depend on the network that just failed.

**Status mapping**

- **`DEADLINE_EXCEEDED` (code 4) gains an explicit mapping** in `authErrorFromCode`
  to the transient/unreachable bucket. Today code 4 falls through to
  `AuthError.unknown`, which would skip the offline-cache fallback.

Not in scope: retry/backoff/circuit-breaking, the back-office (Vue) grpc-web client,
server-side deadline propagation. No **BREAKING** change — user-visible only as
*faster, better-labelled, escapable* failures.

## Capabilities

### New Capabilities

- `platform-client-transport`: how a Cymbra client behaves while waiting on the
  backend — deadline categories and budgets, connect timeout, how fast a lost
  connection is detected and what aborts in response, which gRPC status codes count as
  transient/unreachable, and the rule that no backend wait may be both blocking and
  inescapable. Placed under `platform-*` because it is socle: Music consumes it today,
  Live (Tauri) will consume the same policy, and the classification rule is already
  relied on by `id-*` (`account-access` session refresh).

### Modified Capabilities

None. The two capabilities that depend on this behaviour already state it in a
transport-agnostic way and stay satisfied:

- `account-access` already requires that a refresh failing on *"`UNAVAILABLE`,
  deadline exceeded, offline"* be treated as transient and never clear the session —
  the existing refresher already classifies code 4 that way.
- `offline-score-cache` (in-flight change `add-offline-score-cache`, implemented, not
  yet archived) requires the dedicated *"not available offline"* message on an offline
  uncached favorite, without naming status codes. This change must **not** regress it
  — see Impact.

## Impact

**Products**: Cymbra Music (`apps/music`) — new. Cymbra ID — consumed only
(`account-access` session-refresh classification is relied on, not redeclared).
Cymbra Live, back-office — untouched; Live will consume the same capability later.

**Regression risk this change must retire** — why the status mapping cannot be a
follow-up:
[`notation_notifier.dart:281`](../../../apps/music/lib/state/notation_notifier.dart)
gates the offline-cache fallback on `AuthError.unavailable`. Because
`authErrorFromCode` has no case for code 4, a deadline added on its own would convert
today's slow failure into a **fast generic failure that skips the cache fallback** —
the offline score cache would stop working exactly when it is needed.

**Code touched**:
- `apps/music/lib/services/grpc_client.dart` — deadline policy + interceptor,
  `connectTimeout`, keepalive options; `bearerOptions` unchanged; `tokenRefresher`'s
  own `AuthServiceClient` wired too (it bypasses the adapters).
- 16 gRPC adapters — one constructor line each to install the interceptor.
- `apps/music/lib/services/auth_service.dart` — `authErrorFromCode` code 4.
- `apps/music/lib/state/notation_notifier.dart` — offline race + pre-flight check.
- `apps/music/lib/screens/open_score.dart` — cancellable wait.
- `apps/music/lib/state/score_upload_notifier.dart` — explicit disposal guard
  (`Ref.mounted` does not exist in riverpod 2.6.1) and the reworded `AlreadyExists`.
- 4 HTTP seams: `soundfont_source.dart`, `soundfont_preview_service.dart`,
  `score_preview_service.dart`, `private_soundfont_service.dart`.

**Dependencies**: none added. `connectivity_plus` and `ConnectivityService` already
exist and already expose `onlineStatus` / `isOnline()`. Relies on `grpc: 4.2.0`
semantics verified in source — interceptors receive the merged `CallOptions` and their
result reaches `createCall`; `ClientCall` arms its deadline at *call creation*, so the
deadline already covers connection establishment; `ClientKeepAliveOptions` is
implemented but disabled by default (`pingInterval: null`).

**External check required**: prod terminates TLS at Caddy, which reverse-proxies to
tonic over h2c ([Caddyfile:31](../../../backend/deploy/Caddyfile)). Two consequences:
the ping interval must be validated against Caddy so frequent pings are not answered
with `GOAWAY`, and — since `PING` is hop-by-hop and stops at the proxy — keepalive
detects a dead link to the **edge**, not a hung backend. See design D9.

**Coverage**: Flutter gate is 80%; the policy table, the interceptor and the race
helper are pure Dart and unit-testable without a channel or a server.
