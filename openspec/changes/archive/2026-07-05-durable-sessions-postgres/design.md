## Context

Today `cymbra_auth::session::SessionStore` is a concrete struct over
`Arc<dyn Cache>` (Redis): a session **family** is three keys —
`sess:fam:{fid}` (`user_id`, `audience`, `current_rt`), `sess:rt:{rt}` (rt→fid),
and `sess:userfam:{uid}` (a comma-joined list of the user's fids) — all with a
TTL. `AuthModule` holds the concrete `SessionStore` directly. Because this state
exists only in Redis and cannot be rebuilt from Postgres, a Redis outage logs
everyone out within the ~15-min access-token TTL, which forces Redis to be HA and
durable. Access-token validation is already **offline** (JWKS), so the store is
touched only on sign-in (`create`) and refresh (`rotate`, ~once/15 min per active
session), plus sign-out/revoke — never on the per-request hot path.

## Goals / Non-Goals

**Goals:**
- Make refresh-session state durable (survives a cache-tier restart) by storing
  it in Postgres (`auth` schema).
- Preserve the existing behavioural contract exactly: rotation on use, reuse/theft
  detection (replay revokes the family), audience binding, sliding refresh TTL.
- Close the theft-detection race with an **atomic** check-and-rotate.
- Reclassify Redis as a disposable, non-HA cache (rate-limit + email throttles
  only).
- Keep the store behind an injectable seam so unit tests need no DB.

**Non-Goals:**
- No change to the gRPC auth API, token formats visible to clients beyond an
  internal refresh-token encoding, or the access-token/JWKS flow.
- No migration of existing in-Redis sessions (one-time re-login on cutover).
- No "active devices" UI (the schema enables it; the surface is a later change).
- Moving rate-limit/throttle out of Redis (they stay).

## Decisions

### D1 — Introduce a `SessionStore` trait; provide `PgSessionStore` + `FakeSessionStore`
`SessionStore` becomes a trait (`create`, `rotate`, `revoke`, `revoke_all`,
optionally `list_for_user`). `AuthModule` holds `Arc<dyn SessionStore>`. The
Redis-backed implementation is removed (Redis no longer stores sessions). Tests
use an in-memory `FakeSessionStore`, matching the codebase's provider-override
style (`FakeCredentialRepo`, `FakeCache`).
- *Alternative:* keep the concrete struct and swap only its backend — rejected;
  a trait is the established seam pattern and keeps unit tests DB-free.

### D2 — One row per family; encode the family id in the refresh token
Table `auth.sessions`: `id` (family id, UUIDv7 PK), `user_id`, `audience`,
`current_rt_hash`, `expires_at`, `created_at`. Indexes: unique on
`current_rt_hash` (rotate lookup) and on `user_id` (revoke-all / enumeration).
The refresh token handed to clients encodes its family id plus a random secret
(e.g. `"{fid}.{secret}"`). This lets a **replayed, already-rotated** token still
resolve to its family (for theft detection) without persisting every historical
token — the Redis design needed a per-rt key for this; one row per family
replaces all three Redis keys.
- *Alternative:* a separate `rt → family` table mapping every issued token —
  rejected; unbounded growth + extra cleanup for no benefit.

### D3 — Store a hash of the refresh secret, not the raw token
`current_rt_hash` = SHA-256 of the token secret. Lookups hash the presented
token and compare. Defense-in-depth so a DB dump does not leak usable refresh
tokens. Cost is one hash per refresh — negligible on a cold path.

### D4 — Atomic rotation + reuse detection in one conditional statement
Rotate is a single guarded UPDATE:
`UPDATE auth.sessions SET current_rt_hash=$new, expires_at=$slide
WHERE id=$fid AND current_rt_hash=$old AND expires_at > now() RETURNING user_id, audience`.
- 1 row → rotation wins; return the new token.
- 0 rows → within the same transaction, `SELECT` the family by `id`: if it exists
  and is unexpired, the presented token is a **replay of a rotated token** →
  `DELETE` the family (theft revocation); otherwise reject as invalid/expired.
Two concurrent refreshes of the same token: the conditional UPDATE admits exactly
one; the loser sees 0 rows and is treated as a replay. This closes the race the
Redis get-then-set sequence has.

### D5 — Sliding expiry in Postgres; lazy on read, reap for hygiene
`expires_at = now() + refresh_ttl` at create, and re-extended on each successful
rotate (sliding). All reads filter `expires_at > now()`, so correctness never
depends on cleanup timing. A scheduled **`session_reap`** job (new job kind in
`cymbra_jobs::registry`, seeded as a `session_reap_*` row in `jobs.schedules`,
run by `cymbra-worker`) `DELETE`s expired rows for table hygiene — mirroring the
existing `orphan_reap`.
- *Alternative:* `pg_cron` — rejected; adds a DB extension dependency where the
  worker's scheduler already exists.

### D6 — Worker gets an auth-scoped pool for the reap (per-module isolation kept)
The `session_reap` handler connects with an **`auth_svc`** pool
(`CYMBRA_AUTH_DATABASE_URL`), exactly as the `orphan_reap` handler uses the
`user_svc` pool (`CYMBRA_USER_DATABASE_URL`). The worker process holding several
module-scoped connections — each confined to its own schema — is the established
pattern; `auth_svc` deleting from `auth.sessions` stays within D0. No new
cross-schema grant.

## Risks / Trade-offs

- **Cutover invalidates existing sessions** → on deploy, in-Redis sessions are
  gone and users re-authenticate once. Mitigation: expected and communicated; no
  data loss (accounts/credentials are in Postgres). Schedule at a low-traffic
  time.
- **Token-format change** → old tokens no longer parse and are rejected (→
  re-login), which is the desired cutover behaviour. Mitigation: none needed;
  ensure the parser rejects unknown formats as `UNAUTHENTICATED`.
- **Coverage (Rust ≥80%)** → keep the decision logic (token encode/parse, expiry
  decision, rotate-outcome classification) in a pure, host-tested `session_core`
  module; keep `session_pg` thin I/O exercised by `#[ignore]` live-DB integration
  tests (like `cymbra-auth`/`cymbra-user`). Risk: pg glue is not unit-covered →
  keep it minimal and lean on the integration gate.
- **Extra worker DB pool** → one more connection pool (`auth_svc`) in the worker.
  Mitigation: small pool; the reap runs infrequently.
- **Sliding-expiry write on every refresh** → one UPDATE per refresh; this is the
  write the rotate already performs, so no added round-trip.

## Migration Plan

1. Add `auth/migrations/0002_sessions.sql` (table + indexes); it runs
   automatically on `cymbra-server` boot (embedded `sqlx::migrate!`).
2. Deploy `cymbra-server` with `PgSessionStore` wired on the auth pool; Redis
   stays wired for rate-limit only.
3. Deploy `cymbra-worker` with the `session_reap` handler + `CYMBRA_AUTH_DATABASE_URL`
   pool; add the `session_reap_*` schedule seed.
4. Cutover effect: active users re-login once.
- **Rollback:** redeploy the previous images (Redis-backed sessions). Rows written
  to `auth.sessions` are harmless and reaped by expiry; users re-login once more.

## Open Questions

- Reap cadence + default retention: hourly like `orphan_reap`, or daily? (Lazy
  expiry means cadence only affects table size, not correctness.)
- Do we expose `list_for_user` on the port now (enables "active devices" later) or
  add it when that UI lands? Leaning: add the query + a covered scenario now,
  wire the gRPC surface later.
