## Why

The Flutter app sets **no deadline on any RPC**: `bearerOptions`
([grpc_client.dart:122](../../../apps/music/lib/services/grpc_client.dart)) builds a
bare `CallOptions` (auth metadata only), and it is the only place in `lib/` that
constructs one. `cymbraChannel` leaves `ChannelOptions.connectTimeout` unset. Failure
is therefore bounded only by the OS TCP timeout (~75 s on Darwin): reproduced on
device 2026-08-16, tapping a catalog score with the backend unreachable spins for
30+ s before the error banner appears.

The same hole exists on the second transport: every `package:http` seam
(soundfont delivery/preview, private soundfont library, score preview) calls
`http.Client` with no timeout at all.

A blanket short cap is wrong — a 400 MiB private-soundfont upload is a legitimate
multi-minute transfer — so the policy has to be **per category**, and it has to land
together with the status mapping (below) or it silently breaks the offline cache.

## What Changes

- **A single deadline policy object** maps a gRPC method path to a category
  (`interactive` / `transfer` / `long`) and its deadline, installed as one
  `ClientInterceptor` on every generated client at construction. The 60+
  `options: bearerOptions(bearer)` call sites are **untouched**; `bearerOptions` keeps
  its current signature. A per-call `CallOptions.timeout` still wins over the policy
  (escape hatch), because the interceptor merges policy-as-base.
- **`ChannelOptions.connectTimeout`** is set on `cymbraChannel` as defence in depth.
  Note `connectionTimeout` (already defaulted to 50 min) is *connection reuse*, not
  connect — it is not the knob and is left alone.
- **`DEADLINE_EXCEEDED` (gRPC code 4) gains an explicit mapping** in
  `authErrorFromCode` to the transient/unreachable bucket. Today code 4 falls through
  to `AuthError.unknown`.
- **The HTTP seam gets the same categories**: wall-clock timeouts on the small
  JSON/control requests, and an explicitly **uncapped** (or very generous) budget on
  the two bulk-byte transfers, since a wall-clock cap on a 400 MiB upload is a bug,
  not a safeguard.
- Deadlines are **constants in one file**, not per-call literals scattered across
  adapters, so the policy is reviewable in one place and testable as a table.

Not in scope: retry/backoff policy, the back-office (Vue) grpc-web client, and any
server-side deadline propagation. No **BREAKING** change — this is additive and
user-visible only as *faster, better-labelled* failures.

## Capabilities

### New Capabilities

- `platform-client-transport`: how a Cymbra client bounds a backend call — deadline
  categories and their budgets, connect timeout, which gRPC status codes are
  classified as transient/unreachable, and the rule that a timeout must be
  indistinguishable from an unreachable backend to every downstream consumer
  (offline fallback, session refresh, UI messaging). Placed under `platform-*`
  because it is socle: Music consumes it today, Live (Tauri) will consume the same
  policy, and the classification rule is already relied on by `id-*`
  (`account-access` session refresh).

### Modified Capabilities

None. The two capabilities that depend on this behaviour already state it in a
transport-agnostic way and stay satisfied:

- `account-access` already requires that a refresh failing on *"`UNAVAILABLE`,
  deadline exceeded, offline"* be treated as transient and never clear the session —
  the existing refresher already classifies code 4 that way, so no requirement
  changes.
- `offline-score-cache` (in-flight change `add-offline-score-cache`, implemented, not
  yet archived) requires the dedicated *"not available offline"* message on an
  offline uncached favorite, without naming status codes. This change must **not**
  regress it — see Impact.

## Impact

**Products**: Cymbra Music (`apps/music`) — new. Cymbra ID — consumed only
(`account-access` session-refresh classification is relied on, not redeclared).
Cymbra Live, back-office — untouched; Live will consume the same capability later.

**Regression risk this change must retire** — the reason the status mapping is not a
separate change:
[`notation_notifier.dart:281`](../../../apps/music/lib/state/notation_notifier.dart)
gates the offline-cache fallback on `AuthError.unavailable`. Because
`authErrorFromCode` has no case for code 4, a deadline added on its own would convert
today's slow failure into a **fast generic failure that skips the cache fallback** —
the offline score cache would stop working exactly when it is needed. The mapping
must ship in the same change, and the offline path must be re-verified.

**Code touched**:
- `apps/music/lib/services/grpc_client.dart` — deadline policy + interceptor,
  `cymbraChannel` connect timeout, `bearerOptions` unchanged, `tokenRefresher`'s own
  `AuthServiceClient` wired with the interceptor too (it bypasses the adapters).
- 16 gRPC adapters (`GrpcCatalogService`, `GrpcScoreUploadService`,
  `GrpcAccountService`, …) — one constructor line each to install the interceptor.
- `apps/music/lib/services/auth_service.dart` — `authErrorFromCode` code 4.
- 4 HTTP seams: `soundfont_source.dart`, `soundfont_preview_service.dart`,
  `score_preview_service.dart`, `private_soundfont_service.dart`.

**Dependencies**: none added. Relies on `grpc: 4.2.0` semantics verified in source —
interceptors receive the client+call merged `CallOptions` and their result reaches
`createCall`; `ClientCall` arms its deadline timer at *call creation*, so the deadline
already covers connection establishment.

**Coverage**: Flutter gate is 80%; the policy table and the interceptor are pure Dart
and fully unit-testable without a channel or a server.
