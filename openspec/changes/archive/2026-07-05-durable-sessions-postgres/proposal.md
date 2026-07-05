## Why

Refresh-token session state (families: `user_id`, `audience`, `current_rt`,
rotation + reuse detection) lives **only in Redis** today, and it is not
reconstructable from Postgres. A Redis restart or blip therefore logs every user
out within the access-token TTL (~15 min). That single fact forces Redis to be
highly available and durable — the most expensive, most complex part of the
production story (managed Valkey on OVH/Scaleway/DigitalOcean is cache-only; the
self-hosted alternative is a Sentinel fleet). Moving sessions to Postgres — the
system we already back up and must make HA anyway — removes that constraint:
Redis becomes a disposable cache, and only one stateful system needs HA.

## What Changes

- Introduce a **storage-agnostic session-store seam** (a trait) with a
  **`PgSessionStore`** implementation, wired in the composition root in place of
  the Redis-backed store. The gRPC auth API and refresh-token semantics are
  unchanged.
- Add an **`auth.sessions`** table (session family id, `user_id`, `audience`,
  `current_rt`, `expires_at`, `created_at`) with indexes for the refresh-token
  lookup and for `user_id` (revoke-all / session enumeration).
- Make **rotation + reuse detection atomic**: a single conditional `UPDATE …
  WHERE current_rt = $old RETURNING …` closes the race window that the current
  multi-op Redis sequence has; a replayed (already-rotated) token revokes the
  whole family.
- Handle **expiry in Postgres**: `expires_at` filtered on read (lazy expiry keeps
  correctness independent of cleanup timing) plus a scheduled **reap job** on the
  existing worker + `jobs.schedules` substrate (analogous to `orphan_reap`) for
  table hygiene.
- **Rate-limit and email-throttle counters stay in Redis** (`INCR`+TTL,
  high-frequency, disposable). Redis is reclassified as a **pure, non-HA cache**.
- Keep the injectable seam so unit tests use an in-memory fake session store (no
  DB), consistent with the codebase's provider-override testing style.
- Enable **`revoke_all` and active-session enumeration** as clean indexed
  queries (replacing the comma-joined `sess:userfam` string).
- **Migration effect (not a breaking API change):** existing in-Redis sessions
  are not carried over; on cutover, currently-signed-in users re-authenticate
  once. No data loss (accounts/credentials are in Postgres).

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `backend-auth`: the internal-session / refresh-token requirements gain a
  **durability guarantee** (refresh-session state MUST survive a cache-tier
  outage) and an **atomic rotation/reuse-detection** guarantee; sign-out /
  revoke-all is served from the durable store. Behavioural contract (rotation,
  reuse detection, audience binding, offline access-token validation) is
  otherwise unchanged.

## Impact

- **`backend/auth`**: `session.rs` becomes a trait + `FakeSessionStore`;
  new `session_pg.rs` (`PgSessionStore`); `AuthModule` holds `Arc<dyn
  SessionStore>` instead of the concrete struct.
- **`backend/auth/migrations`**: new `0002_sessions.sql` (table + indexes).
- **`backend/server`** (`main.rs`): wire `PgSessionStore` on the auth pool;
  comment/reclassify Redis as cache-only.
- **`backend/worker`** + **`backend/jobs`**: new `session_reap` job kind +
  handler (reads an auth-scoped pool, like the orphan reaper reads the user
  pool) + a `session_reap_*` row in the `jobs.schedules` seed.
- **`backend/platform`**: Redis (`cache.rs`) is now documented as a disposable
  cache; no HA requirement.
- **Prod / ops**: Redis no longer needs HA or durable persistence — simplifies
  every hosting option discussed.
- **Tests / CI**: Rust line coverage MUST stay ≥ 80%; keep pure logic
  host-testable; add integration coverage for the Postgres store behind the
  `#[ignore]` live-DB gate, mirroring `cymbra-auth`/`cymbra-user`.
