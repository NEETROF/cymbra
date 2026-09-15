# Tasks — fix-auth-lockout-dos

## 1. Client address

- [x] 1.1 `cymbra_platform::client_addr`: first non-empty `X-Forwarded-For` hop, else `X-Real-IP`, else the peer address, else `unknown`, over a header getter; unit tests; `server/src/web_plans.rs` uses it instead of its private copy
- [x] 1.2 `cymbra-auth-port`: `ClientAddr` newtype; `sign_up_local`, `resend_verification`, `sign_in_local` and `request_password_reset` take `&ClientAddr`
- [x] 1.3 Callers: the gRPC adapter resolves the address from tonic metadata and `remote_addr()`; `server/src/web_auth.rs` from its `HeaderMap`; update `MockAuthPort` expectations, `FakeAuth` and the `auth_flow` integration test

## 2. Limits

- [x] 2.1 Pure, host-tested limit logic in `backend/auth` (email normalization, key builders, allow/refuse decisions)
- [x] 2.2 Configuration: `CYMBRA_SIGNIN_ADDR_MAX_FAILURES`, `CYMBRA_SIGNIN_ACCOUNT_FAILURE_RATE`, `CYMBRA_EMAIL_ADDR_SEND_RATE`, `CYMBRA_EMAIL_ACCOUNT_SEND_RATE` parsed with defaults (30, 200/1h, 20/1h, 10/1h); `AuthConfig` carries them in an `AuthLimits` value; both env examples updated
- [x] 2.3 `sign_in_local`: lockout per (email, address), per-address failures, per-account ceiling — read before the password check, all incremented on a wrong password, lockout and per-account counter cleared on success; tests: victim not locked by another address, per-address limit across emails, per-account ceiling, case/space variants share counters
- [x] 2.4 `resend_verification` / `request_password_reset`: per (email, address), per address, per email, stopping at the first refusal; `sign_up_local`: per address; tests: another address still gets the email, per-address budget across endpoints, per-email ceiling, uniform reset answer kept

## 3. Docs and gates

- [x] 3.1 `backend/deploy/DEPLOY.md`: the limits, the reverse-proxy requirement for trusting the forwarded headers, and the manual unlock (Valkey keys)
- [x] 3.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, tests, and `cargo llvm-cov --workspace --fail-under-lines 80` with the shared ignore regex
- [x] 3.3 `openspec validate fix-auth-lockout-dos --strict`
- [ ] 3.4 [manual] Production: failures from one network do not lock the same account signing in from another network; the documented unlock works
