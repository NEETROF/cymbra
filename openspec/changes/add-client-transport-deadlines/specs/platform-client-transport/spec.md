## ADDED Requirements

### Requirement: Every backend call is bounded by an application deadline

A Cymbra client MUST NOT issue a backend call whose only bound is the operating
system's transport timeout. Every gRPC call SHALL carry a deadline, and every HTTP
request SHALL carry either a deadline or an explicit, documented no-progress bound.

The deadline SHALL cover connection establishment, not only the time after a
connection is up: a call made while the backend is unreachable SHALL fail within its
own budget regardless of how long the platform would take to give up on the socket.

#### Scenario: Unreachable backend fails within the call's budget

- **WHEN** the user triggers an interactive backend read and the backend is
  unreachable (host down, no route, packets dropped)
- **THEN** the call fails within its category's deadline, and the failure is not
  deferred to the operating system's transport timeout

#### Scenario: No call site can opt out of having a deadline

- **WHEN** a new RPC is added to a client adapter without specifying any options
- **THEN** it still carries the deadline of its category, because the deadline is
  applied by the shared transport layer rather than by each call site

### Requirement: Deadlines are assigned per category, never as one global cap

A single deadline SHALL NOT be applied to all calls. Each call SHALL belong to one
category, and each category SHALL have its own budget:

| Category | Applies to | Budget |
|---|---|---|
| `interactive` | Small request/response calls a user is waiting on: search, listings, account reads and writes, ratings, leaderboards, streaks, flags, unlock/daily access, telemetry | 10 s |
| `transfer` | Unary calls carrying a document or media payload: score bytes, rating-preview bytes, course manifests, preview clips | 30 s |
| `long` | User-initiated submissions the user expects to take a while: score upload | 120 s |
| `bulk` | Multi-hundred-megabyte byte transfers: private SoundFont import and download, catalog SoundFont download | No wall-clock cap (see below) |

Budgets SHALL be defined as named constants in one place per client, so the whole
policy is reviewable without reading call sites.

#### Scenario: An interactive read and a bulk upload do not share a budget

- **WHEN** a catalog search and a 400 MiB SoundFont import are both in flight
- **THEN** the search is bound by the `interactive` budget while the import is not,
  and neither one's budget is derived from the other's

#### Scenario: A slow but healthy upload is not aborted

- **WHEN** a user uploads a large SoundFont over a slow connection and the transfer
  is progressing
- **THEN** the client does not abort it on a wall-clock deadline sized for
  interactive calls

### Requirement: Bulk byte transfers are bounded by progress, not by wall clock

A `bulk` transfer SHALL NOT be bounded by a wall-clock deadline, because the
legitimate duration of a 400 MiB transfer is a function of the user's bandwidth and
cannot be predicted. It SHALL instead be bounded by connection establishment and,
where the transport allows, by a no-progress rule. Where neither is expressible, the
client SHALL leave the transfer uncapped and say so explicitly in the code rather
than picking an arbitrary large number that silently truncates real uploads.

#### Scenario: Bulk transfer to an unreachable host still fails fast

- **WHEN** a bulk transfer is started while the backend is unreachable
- **THEN** it fails at connection establishment rather than hanging, even though no
  wall-clock deadline applies to the transfer itself

### Requirement: Connection establishment is bounded independently

The client channel SHALL set an explicit connect timeout. This is distinct from any
connection-reuse or idle lifetime setting, which govern how long an established
connection is kept and MUST NOT be mistaken for a connect bound.

#### Scenario: Connect bound is set explicitly

- **WHEN** the shared channel is constructed
- **THEN** it carries an explicit connect timeout, and the connection-reuse and idle
  settings are left to their defaults

### Requirement: A deadline overrun is classified as a transient unreachable-backend failure

A call that exceeds its deadline SHALL be classified identically to a backend that
answered `UNAVAILABLE`. Every consumer of the client's error taxonomy — offline cache
fallback, session refresh, and user-facing messaging — SHALL behave the same way for
a timeout as for an unreachable backend. A deadline overrun SHALL NOT fall through to
the generic/unknown error bucket.

#### Scenario: Offline cache fallback survives a timeout

- **WHEN** the app is offline, the user opens a favorited score, and the fetch fails
  by exceeding its deadline rather than by an explicit `UNAVAILABLE` answer
- **THEN** the offline-availability path runs exactly as it does for `UNAVAILABLE`:
  a cached copy is played if present, and otherwise the dedicated "not available
  offline" message is shown — not the generic failure message

#### Scenario: A timed-out session refresh does not sign the user out

- **WHEN** the coordinated session refresh exceeds its deadline
- **THEN** the stored session is left intact and the user stays signed in, exactly as
  for a transient `UNAVAILABLE` refresh failure

#### Scenario: A timeout never surfaces raw transport text

- **WHEN** any call fails by exceeding its deadline
- **THEN** the user is shown a localized message describing an unreachable backend,
  and no gRPC status string, exception text, or duration is surfaced in the UI

### Requirement: A call site may override its category deadline

A single call SHALL be able to specify its own deadline, and that per-call value
SHALL take precedence over the category default. The shared transport layer SHALL
apply its policy as the *base* value so an explicit per-call deadline is never
silently overwritten.

#### Scenario: Explicit per-call deadline wins

- **WHEN** a call is issued with an explicit deadline that differs from its
  category's budget
- **THEN** the explicit deadline is the one sent to the server

### Requirement: Losing connectivity aborts in-flight work immediately

A client SHALL NOT wait out a deadline for work it already knows cannot succeed. When
the device reports a transition to offline while a backend call is in flight, the
operation waiting on that call SHALL resolve immediately — to its offline outcome
(cached content, or the dedicated offline message) — rather than waiting for the call
to exhaust its deadline.

The abandoned call MAY be left to expire on its own deadline; what MUST NOT happen is
the user-visible operation staying pending after the device knows it is offline.

#### Scenario: Airplane mode ends a pending load at once

- **WHEN** a score is loading over the network and the user enables airplane mode
- **THEN** the load resolves as soon as the connectivity transition is reported —
  playing a cached copy if one exists, otherwise showing the dedicated offline message
  — and the blocking wait ends without running to the call's deadline

#### Scenario: A completed load is unaffected by a later transition

- **WHEN** a load completes successfully and connectivity is lost afterwards
- **THEN** the loaded content stays available and no failure is surfaced for it

#### Scenario: Watching connectivity leaks nothing

- **WHEN** a load completes normally, without any connectivity transition
- **THEN** any subscription opened to observe connectivity for that load is released

### Requirement: A call known to be pointless is never issued

Before opening a connection for a user-facing read, a client SHALL consult the
device's reported reachability, and when the device is offline SHALL resolve the
operation from local state (cache, or the dedicated offline message) without issuing a
network call.

Reported reachability is evidence of a *negative* only: a device reporting a usable
interface MAY still be unable to reach the backend (captive portal, backend down), so
a positive reading SHALL NOT be treated as proof and the call proceeds under its
normal deadline.

#### Scenario: Offline at tap opens no socket

- **WHEN** the user opens a score while the device reports no connectivity
- **THEN** the outcome is decided from the local cache or the offline message with no
  network call attempted, and the result is immediate

#### Scenario: Connected but unreachable still fails on its deadline

- **WHEN** the device reports a usable interface but the backend cannot be reached
- **THEN** the call is issued normally and fails on its deadline, classified as an
  unreachable backend

### Requirement: A dead connection is detected by keepalive, not by call deadlines

The client channel SHALL send keepalive pings while calls are in flight and SHALL tear
down a connection whose ping goes unanswered within a bounded time, so that a
connection that has died without either endpoint noticing (half-open socket, network
switch, dropped NAT mapping) is discovered independently of any individual call's
deadline, and in-flight calls on it fail as soon as it is torn down.

Pings SHALL NOT be sent while no call is in flight, so an idle app does not wake the
radio. The ping interval SHALL be validated against the deployed edge so that a
conforming client is not penalized for its ping rate.

#### Scenario: Half-open connection fails fast

- **WHEN** a call is in flight on a connection that has silently died
- **THEN** the unanswered ping tears the connection down and the call fails at that
  point rather than at its own deadline

#### Scenario: Idle app sends no pings

- **WHEN** the app has no call in flight
- **THEN** no keepalive ping is sent

### Requirement: No backend wait is both blocking and inescapable

A UI that blocks interaction while waiting on a backend call SHALL offer the user a
way out at all times. Cancelling SHALL return the user to where they were, SHALL NOT
present the cancellation as an error, and SHALL ensure a late result from the
abandoned call cannot alter the screen the user has returned to.

#### Scenario: The user leaves a slow load

- **WHEN** a blocking load is taking too long and the user cancels it
- **THEN** the blocking UI closes, the user is back on the previous screen, no error
  message is shown, and the load's late result — success or failure — changes nothing

#### Scenario: No blocking wait lacks an exit

- **WHEN** any modal blocking UI is shown for the duration of a backend call
- **THEN** it exposes a cancel affordance for the whole time it is displayed

### Requirement: Abandoning a wait never claims an outcome the client cannot know

When the user leaves a screen while a mutating call is in flight, the client SHALL stop
waiting and discard the result, and SHALL NOT present the operation as cancelled,
failed, or succeeded — it cannot know which, because the request may already have been
applied by the server.

The client SHALL instead let the next successful read of the affected collection report
the truth, and the operation SHALL be safe to retry: a repeated attempt MUST NOT create
a duplicate. Where the server rejects a repeat as already-existing, the client SHALL
present that as a statement of fact, not as an error.

#### Scenario: Leaving mid-upload makes no claim

- **WHEN** the user leaves the upload screen while the submission is in flight
- **THEN** the wait is abandoned with no "cancelled", "failed" or "sent" message, and
  the user's list of contributions reflects the real outcome on its next refresh

#### Scenario: An abandoned upload that landed shows up as a normal contribution

- **WHEN** an abandoned upload in fact completed on the server
- **THEN** it appears in the user's contributions on the next refresh, with no error
  and no duplicate

#### Scenario: Retrying an upload that already landed is not an error

- **WHEN** the user re-submits a score whose content the server already holds for them
- **THEN** the app tells them the score is already in their library, worded as a fact
  rather than as a failure, and no duplicate is created

#### Scenario: The exit is never gated on a network answer

- **WHEN** the user leaves a screen with a call in flight
- **THEN** leaving does not wait on, and is not blocked by, any additional request to
  confirm what happened
