# fix-auth-lockout-dos — Cymbra ID: brute-force limits that cannot be turned against a user

## Why

Cymbra ID's protections against password guessing are keyed on the **email alone**, so they
double as a targeted denial of service:

- `sign_in_local` counts failures under `signin:{email}` — 5 wrong passwords lock the
  account for 15 minutes, and while locked even the right password is refused. Anyone who
  knows an address can keep its owner out of password sign-in indefinitely, one burst of
  five attempts every quarter of an hour.
- Verification and reset emails are throttled under `rl:verify_email:{email}` /
  `rl:reset_email:{email}` (3 per hour), so the same person can burn a victim's reset emails
  for an hour — right when the victim needs one.
- The key is the email as typed: letter case or surrounding spaces start a fresh counter.

The `backend-auth` spec already says the lockout applies "for an account/IP", but no client
address reaches the auth module: only `web_plans.rs` reads one, for code redemption. Found on
2026-09-15 while testing the Lingua extension's sign-in against production.

## What Changes

- **The client address reaches the auth module.** One shared helper reads it the way the
  deployment presents it — the first `X-Forwarded-For` hop set by Caddy, else `X-Real-IP`,
  else the peer address — for the gRPC `AuthService` and the browser `/web/auth/*` routes
  alike; `web_plans.rs` uses the same helper. The `AuthPort` methods that are rate-limited
  receive it explicitly.
- **Password sign-in is limited by three counters instead of one**, all keyed on the
  normalized email (trimmed, lower-cased):
  - per **(email, address)**: the existing hard lockout (5 failures / 15 min) — an attacker
    locks only their own address out of that account;
  - per **address**, across emails: a failure limit against credential stuffing;
  - per **email**, across addresses: a much higher ceiling, the last resort against a
    brute force spread over many addresses.
- **Verification and reset emails get the same shape**: per (email, address) at the existing
  rate, per address across emails and endpoints (sign-up included — it emails any free
  address), and a higher per-email ceiling.
- New configuration knobs with safe defaults, documented in both env examples and
  `DEPLOY.md` (including how to lift a lock by hand).
- Responses do not change: a refusal is still `RESOURCE_EXHAUSTED`, and the reset request
  stays identical whether or not the account exists. **No `.proto` change.**

## Capabilities

### New Capabilities
_None._

### Modified Capabilities
- `backend-auth` (legacy name, `id-*` domain): the "Local credential hardening" requirement
  specifies what each limit is keyed on — (email, client address) for the lockout and the
  email throttles, per-address and per-email ceilings, email normalization — so that one
  party cannot lock another out.

## Impact

- **Products**:
  - **Cymbra ID** (modified): `backend/auth` (limits, `AuthConfig`), `backend/auth-port`
    (the `AuthPort` signatures gain the client address), `backend/platform` (address helper,
    configuration), `backend/server` (`web_auth.rs` and `web_plans.rs` pass the address).
  - **Music / Lingua / back office / site**: consumers, untouched — same RPCs, same errors.
    Their users stop being lockable by someone else.
- **Env/deploy**: new optional variables (defaults apply when unset); no migration. Valkey keys
  change shape, so counters in flight at deploy time are simply forgotten.
- **CI**: no new unit; Rust tests extended (auth module, platform helper, adapters).
- **Out of scope**: CAPTCHA or progressive delays, sign-up abuse beyond the per-address email
  limit, OIDC sign-in (not rate-limited today and not attackable this way).
