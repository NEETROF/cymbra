# Design — fix-auth-lockout-dos

## Context

- `backend/auth/src/module.rs::sign_in_local` reads `signin:{email}` from the shared cache
  (Valkey), refuses with `RESOURCE_EXHAUSTED` once it reaches `CYMBRA_SIGNIN_MAX_ATTEMPTS`
  (5), and on a wrong password increments it with `incr_with_ttl(…, CYMBRA_SIGNIN_LOCKOUT)`
  (15 min, TTL set on the first failure). A success deletes the key.
- `resend_verification` and `request_password_reset` call
  `ratelimit::check(cache, "verify_email" | "reset_email", email, email_max, email_window)`
  (`CYMBRA_EMAIL_SEND_RATE` = 3/1h). `sign_up_local` sends its verification email through a
  job and is not throttled.
- No client address reaches `AuthModule`. `backend/server/src/web_plans.rs::client_addr`
  already reads the first `X-Forwarded-For` hop, else `X-Real-IP`, else `unknown`, for the
  code-redemption throttle.
- Production: Caddy is the only public entry (`reverse_proxy` for both the Axum routes and
  tonic over h2c); the server container only `expose`s its ports. Caddy v2 ignores
  client-supplied `X-Forwarded-*` headers unless `trusted_proxies` is configured, and sets
  `X-Forwarded-For` to the real client address.
- Callers of the affected `AuthPort` methods: the gRPC adapter (`auth/src/grpc.rs`), the
  browser cookie sign-in (`server/src/web_auth.rs`), their test doubles (`MockAuthPort`,
  `FakeAuth`) and the `auth_flow` integration test.

## Goals / Non-Goals

**Goals**
- A third party cannot lock a user out of password sign-in or exhaust their reset emails
  from their own address.
- Brute force and credential stuffing stay bounded, from one address and from many.
- One normalized key per email.

**Non-Goals**
- CAPTCHA, proof of work, progressive delays.
- Rate-limiting OIDC sign-in (no password to guess; the provider limits it).
- Changing error codes or client copy.

## Decisions

### D1 — The client address is an explicit port argument
`cymbra-auth-port` gains `ClientAddr` (a newtype over the address string, `unknown` when
none could be read). `sign_up_local`, `resend_verification`, `sign_in_local` and
`request_password_reset` take `&ClientAddr`. The newtype keeps an address from being passed
where an email or password is expected — all three are `&str` today.

Rejected: a task-local or request extension read inside the module — it hides an input the
limits depend on and makes the tests order-dependent. Rejected: rate-limiting in the
adapters — the gRPC and cookie surfaces would each need their own copy of the rules.

### D2 — One address helper, the deployment's view of the client
`cymbra_platform::client_addr` resolves the address from a header getter plus an optional peer
address: first non-empty `X-Forwarded-For` hop, else `X-Real-IP`, else the peer address,
else `unknown`. The gRPC adapter feeds it tonic metadata and `Request::remote_addr()`; the Axum
routes feed it their `HeaderMap`. `web_plans.rs` drops its private copy.

Trusting the forwarded headers is sound in this deployment only because Caddy is the sole
entry and rewrites them (Context). A backend exposed directly would let a client choose its
address — the per-email ceilings (D3, D4) are what still bound it; `DEPLOY.md` states the
requirement.

### D3 — Password sign-in: three counters
All keys use the normalized email `e = email.trim().to_lowercase()` and the address `a`.

| Counter | Key | Limit (default) | Window | On success |
|---|---|---|---|---|
| Lockout | `signin:{e}:{a}` | `CYMBRA_SIGNIN_MAX_ATTEMPTS` (5) | `CYMBRA_SIGNIN_LOCKOUT` (15m) | cleared |
| Per address | `signin-addr:{a}` | `CYMBRA_SIGNIN_ADDR_MAX_FAILURES` (30) | `CYMBRA_SIGNIN_LOCKOUT` | kept |
| Per account | `signin-acct:{e}` | `CYMBRA_SIGNIN_ACCOUNT_FAILURE_RATE` (200/1h) | from the rate | cleared |

The three are read before the password is checked; any one at its limit refuses with
`RESOURCE_EXHAUSTED` and increments nothing. A wrong password increments all three (each
TTL set on its first increment, as today). A success clears the lockout and the per-account
counter — only the owner can produce one — and leaves the per-address counter, which also
reflects other accounts.

Effect: an attacker at one address can spend at most 5 failures per 15 minutes on a given
account (their own lockout), and 30 across accounts. Reaching the per-account ceiling takes
at least 10 addresses sustained for an hour; the owner then still has OIDC sign-in and the
password reset (whose limits are separate, D4). The ceiling is configurable, so an operator
under a distributed attack can tune it.

Rejected: keeping a pure per-email lockout with a smaller window — still a lock anyone can
trigger. Rejected: a per-address lockout alone — it does nothing against a brute force spread
over many addresses.

### D4 — Verification and reset emails: the same shape
| Counter | Key | Limit (default) |
|---|---|---|
| Per (email, address) | `rl:verify_email:{e}:{a}` / `rl:reset_email:{e}:{a}` | `CYMBRA_EMAIL_SEND_RATE` (3/1h) |
| Per address, all email endpoints | `rl:email_addr:{a}` | `CYMBRA_EMAIL_ADDR_SEND_RATE` (20/1h) |
| Per email, across addresses | `rl:email_acct:{e}` | `CYMBRA_EMAIL_ACCOUNT_SEND_RATE` (10/1h) |

`resend_verification` and `request_password_reset` check the three in that order, stopping at
the first refusal (so a refused request does not spend the later budgets). `sign_up_local`
checks the per-address budget only: it emails an address with no account yet, so a per-email
counter would protect nobody. The keys never depend on whether the account exists, so the
reset request keeps its uniform answer.

### D5 — Configuration
`AuthConfig` groups the limits in one `AuthLimits` value instead of more positional arguments.
The rates reuse the existing `N/<duration>` syntax; every new variable is optional with the
defaults above, and both `backend/.env.example` and `backend/deploy/.env.prod.example` list
them.

## Risks / Trade-offs

- **Many users behind one address** (campus, CGNAT) share the per-address budgets → the
  defaults (30 failures / 15 min, 20 emails / hour) sit well above one person's use; both are
  configurable.
- **Headers trusted without a proxy** → documented deployment requirement (D2); the
  per-email ceilings still cap a spoofing client.
- **Unreadable address** (`unknown`) → every such request shares one address bucket; in
  production Caddy always sets the header, and the peer address is the fallback before
  `unknown`.
- **The per-account ceiling is still a lock** → it takes a sustained, distributed effort,
  leaves OIDC and reset available, and is tunable; accepted as the price of bounding
  distributed brute force.
- **Counters reset at deploy** (new key shapes) → at most one lockout window of forgiveness.

## Migration Plan

1. Deploy the backend; unset variables take the defaults, no migration.
2. Manual unlock, documented in `DEPLOY.md`: delete the relevant keys in Valkey
   (`signin:{e}:{a}`, `signin-acct:{e}`, `signin-addr:{a}`).
3. Rollback: redeploy the previous image; old and new keys do not collide.

## Open Questions

- None blocking. The defaults (30, 200/1h, 20/1h, 10/1h) are a first estimate, to revisit with
  production metrics.
