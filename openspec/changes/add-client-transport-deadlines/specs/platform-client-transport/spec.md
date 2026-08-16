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
