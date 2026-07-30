## Context

`set_local_credential` (backend/auth/src/module.rs, from `add-account-identity-linking`)
lets a signed-in OIDC-only account add an email+password. Today it eagerly:

1. `password::check_policy` + reject if the account already has a `local` identity,
2. `creds.insert(email, hash)` — creates an **unverified** `local_credentials` row,
3. `user.link_identity(user_id, "local", email)` — binds the identity (with a
   compensating `delete_credentials` on link failure),
4. enqueues the verification email (the credential stays unverified until the code
   is confirmed via `verify_email`).

`verify_email(token)` today looks up the `local_credentials` row by token and marks
it verified. `sign_in_local` refuses an unverified credential
(`FailedPrecondition "email not verified"`, module.rs:255), so no access is gained
before verification — but steps 2–3 already **reserve** the email
(`local_credentials.email` unique; `user_identities (provider, subject)` unique) and
nothing cleans it up (the reaper only removes handle-less accounts, `pg.rs:240`).

Constraint: auth already depends on `cymbra_platform::cache` (a Redis/Valkey seam)
for rate-limits and lockouts. CLAUDE.md requires Rust line coverage ≥ 80% and
mockall-generated doubles for trait dependencies (`rust-testing` skill).

## Goals / Non-Goals

**Goals:**
- Bind the email/`local` identity only **after** the emailed code is verified, so an
  email is never reserved without proof of ownership.
- Make an abandoned set-password self-clean (no reaper, no lingering squat).
- Preserve: no takeover, current UX, the 3/h send rate limit, the branded/localized
  verification email.

**Non-Goals:**
- Do **not** rework `sign_up_local`. Its abandoned accounts are handle-less orphans
  the reaper already deletes, so the same persistent squat does not arise there.
- No proto/API/client change; `VerifyEmail`/`SetLocalCredential` signatures are
  unchanged and the app keeps routing unverified sign-ins to the OTP screen.
- No new email type; the verification email content is untouched.

## Decisions

### D1 — Store the pending set-password in the cache with a TTL (no DB, no reaper)
A **pending set-password** record `{ user_id, email, password_hash }` is stored in
`cymbra_platform::cache` keyed by the verification token, with TTL = `verify_ttl`
(24h). Because the cache entry auto-expires, an abandoned set-password simply
vanishes — no `local_credentials`/`user_identities` row is ever created, so there
is nothing to reserve and nothing to reap.
*Alternative considered:* a `pending_local_credentials` table with `expires_at` + a
scheduled reaper — durable across a cache flush, but needs a migration, a reaper
job, and still momentarily reserves nothing extra. Rejected as heavier for no real
gain; a lost pending record on a cache restart just means the user re-submits.
*Note:* only the **argon2 hash** is stored (never plaintext), in a short-lived entry
— acceptable, and the same secret class the DB already holds.

### D2 — `verify_email` handles two flavors, distinguished by token lookup
`verify_email(token)` first checks the pending-set-password store:
- **hit** (pending set-password) → atomically `creds.insert(email, hash)` **as
  verified** + `link_identity(user_id, "local", email)` + delete the pending entry.
  If `insert`/`link` reports `AlreadyExists` (email taken meanwhile), return that
  cleanly and drop the pending entry (nothing was bound).
- **miss** → the existing sign-up path (an unverified `local_credentials` row keyed
  by the token) is marked verified, unchanged.
Tokens are UUIDv4, so the two spaces never collide. A single `VerifyEmail` RPC still
serves both.
*Requires* a credential-store method to insert an **already-verified** credential
(or insert-then-verify in one transaction) for the set-password path, so the code is
usable immediately after confirmation without a second round trip.

### D3 — Request-time validation stays; binding is deferred (TOCTOU accepted)
`set_local_credential` still validates eagerly for good UX: reject a weak password,
and reject if the account already has a `local` identity. It also *may* pre-check
that the email is currently free — but the authoritative check is at verify time.
Two callers could both pass the free-email check and both receive codes; only the
**first to verify** creates the binding, the second gets `AlreadyExists` at verify.
This is acceptable: neither reserved the email before proving ownership, and the
loser sees a clean error. (This is a deliberate weakening of the current
"immediately reserved" behavior — that is the whole point.)

### D4 — Route to the code screen in place (small client change); keep the rate limit
Because binding is deferred, **no unverified credential exists before
verification**, so the old "sign in → `FailedPrecondition` → OTP" route no longer
fires — a pre-verify sign-in now returns `Unauthenticated` ("invalid credentials"),
which the app does not route to the code screen. The set-password flow therefore
**navigates directly to the existing `OtpVerifyScreen` after submit**, where the
user enters the emailed code while staying signed in via their OIDC session
(`OtpVerifyScreen` with no password simply verifies and pops back — no re-login).
This is a small but **necessary** Flutter change: without it, after deferring the
bind there would be no reachable way to enter the code. `set_local_credential`
still sends exactly one verification email per submit under the existing 3/hour
limit.

## Risks / Trade-offs

- **Cache loss drops a pending set-password** → Mitigation: low impact — the user
  simply re-submits; nothing is half-created (no orphaned row, since binding only
  happens on verify).
- **A stored argon2 hash in the cache** → Mitigation: hash only (never plaintext),
  short TTL, same trust boundary as the DB; the cache is already the auth secret
  path (sessions/lockouts).
- **TOCTOU on a contended email** → Mitigation: authoritative check at verify;
  first-verify-wins; loser gets `AlreadyExists`; no reservation occurs pre-verify.
- **Two token-lookup paths in `verify_email`** → Mitigation: clear precedence
  (pending store first, then the credential row) + tests for both.

## Migration Plan

1. Add the pending-set-password store seam (cache-backed) + its double.
2. Rework `set_local_credential` to validate + store pending + send email (no
   insert/link).
3. Extend `verify_email` to consume a pending set-password (create verified
   credential + link) before the existing path; add the "insert as verified"
   credential-store primitive.
4. Tests + coverage, then ship. No migration, no rollback data concerns (additive,
   cache-only state); reverting restores the old eager-bind behavior.

## Open Questions

- Should `set_local_credential` pre-check "email currently free" for a friendlier
  immediate error, or stay silent until verify to minimize any enumeration signal to
  the authenticated caller? Default: keep the existing request-time rejections
  (already-has-password / weak password) and let the email-taken case surface at
  verify.
- Consider applying the same pending-store pattern to `sign_up_local` later for
  consistency (out of scope here; the reaper already covers its abandoned case).
