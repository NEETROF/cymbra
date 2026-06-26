## 1. Workspace & modular layout

- [ ] 1.1 Create the backend directory with workspace members: `platform`, a `server` binary, and two module crate pairs — `auth-port`/`auth` and `user-port`/`user`
- [ ] 1.2 Establish the dependency rule in the manifests: consumers depend on `<module>-port` only (`auth` depends on `user-port`); impl crates depend on `platform` + their own `-port`; `server` depends on all; no impl crate depends on another impl crate
- [ ] 1.3 Add shared dependencies: `tonic`, `prost`, `tokio`, `tower`, `axum` (JWKS + health HTTP endpoint), `sqlx` (postgres, runtime-tokio, migrate), `redis`/`deadpool-redis` (sessions, rate-limit, throttles), `jsonwebtoken` (EdDSA/RS256 for internal tokens + provider verification), `argon2`, `config`, `tracing`/`tracing-subscriber`, `tracing-opentelemetry`, `opentelemetry`, `opentelemetry-otlp`, `opentelemetry-appender-tracing`, `tokio-metrics` + `sysinfo`, an SMTP/email crate (e.g. `lettre`), `thiserror`/`anyhow`, `uuid` (`v7` feature — internal ids are UUID v7 via `Uuid::now_v7()`), `async-trait`
- [ ] 1.4 Set up `tonic-build` so each `<module>-port` owns its `proto/` and generated client/server stubs (packages `cymbra.auth.v1`, `cymbra.user.v1`)
- [ ] 1.5 Add docker-compose for local dev: Postgres + **Redis** + a mock OIDC issuer + Mailpit (SMTP sink)
- [ ] 1.6 Add a DB bootstrap that provisions schemas `auth` and `user_account`, each with a per-module Postgres role granted privileges **only** on its own schema (`search_path` pinned)
- [ ] 1.7 Add `.env.example` / config schema: DB roles, Redis URL, Google/Apple `iss`/`aud`, Apple `.p8` client-secret key, the **app-audience allow-list** (`music`/`live`), **asymmetric internal-token signing keypair** (+ `kid`) and TTLs, password-policy + rate-limit params, SMTP settings, OTLP endpoint + toggle

## 2. Platform crate (cross-cutting)

- [ ] 2.1 Typed configuration loading (env + optional file) with fail-fast validation and a unit-tested `config_core` parser
- [ ] 2.2 Structured logging via `tracing` with request correlation ids; redact tokens/secrets
- [ ] 2.3 SQLx pool factory building a pool per module using that module's own DB role, plus a per-module migration runner
- [ ] 2.4 Shared `AuthIdentity { user_id, roles }` context type and gRPC error/status mapping helpers
- [ ] 2.5 Internal-token **JWT codec** in a host-testable `token_core` — sign with the **asymmetric private key** (EdDSA/RS256, `kid` in header), verify with the public key; the **internal-token interceptor** validates the access token on protected methods and injects `AuthIdentity`; reject missing/invalid/expired with `UNAUTHENTICATED`
- [ ] 2.6 **JWKS publishing**: expose the internal-token public key(s) at `/.well-known/jwks.json` over an **Axum** HTTP surface mounted alongside tonic (also `/healthz`), supporting multiple active `kid`s for rotation — so Music/Live verify Cymbra ID tokens offline
- [ ] 2.7 Provide a **Redis client/port** (fake for tests) for sessions/refresh state, rate-limit counters, and email throttles
- [ ] 2.8 Rate-limiter helper over Redis (fixed-window or token-bucket) in a host-testable `ratelimit_core`, for sign-in attempts and email sends
- [ ] 2.9 OIDC verification helper: JWKS fetch+cache and signature/`iss`/`aud`/`exp` checks in a host-testable `oidc_core`
- [ ] 2.10 argon2id password-hash helper (+ password-policy check) and an **email-sender port** (fake for tests, SMTP impl for runtime)
- [ ] 2.11 Role-based guard `require_role(r)` / `is_admin` reading the role set from `AuthIdentity`; returns `PERMISSION_DENIED` when the role is absent
- [ ] 2.12 Tests: config validation; asymmetric token sign/verify (incl. roles claim) + interceptor; JWKS output shape; OIDC claim checks; rate-limiter window; guard allow/deny

## 3. User module (user-account)

- [ ] 3.1 In `user-port`: define the **port trait** (resolve-or-provision by `(provider, subject)`, link identity, **unlink identity**, list identities, get account, update account, **delete account**, read **effective roles for a scope** / `has_role(scope, role)`), DTOs, and `.proto`
- [ ] 3.2 In `user`: own schema `user_account`; migrations for `users` (`id` UUID v7, profile, preferences, `version`, timestamps), `user_identities` (`id`, `user_id`, `provider`, `subject`, `linked_at`, `UNIQUE(provider, subject)`), and `user_roles` (`user_id`, `scope`, `role`, `UNIQUE(user_id, scope, role)`)
- [ ] 3.3 In `user`: repositories scoped to the caller's `user_id`; resolve-or-provision (seed default role `(global, user)`) and link enforcing uniqueness (reject identity bound elsewhere); effective-roles query returns `global` + requested scope
- [ ] 3.4 Implement the **direct adapter**: resolve/provision, link, **unlink (reject removing the last identity)**, list identities, read roles, get account, update account (optimistic concurrency via `version_core`), **delete account** (erase users + identities + roles for the `user_id`)
- [ ] 3.5 Implement the **gRPC server adapter** (in `user`) and **gRPC client adapter** (in `user-port`)
- [ ] 3.6 Tests: provision defaults to `(global, user)`; resolve reuses; link attaches; already-linked rejected; **unlink ok / unlinking last identity rejected**; **delete erases account + identities + roles**; effective roles scoped per app; same set across providers; update version + stale rejected; contract test across both adapters

## 4. Auth module (backend-auth)

- [ ] 4.1 In `auth-port`: define the `AuthService` `.proto` + port — `SignUpLocal`, `VerifyEmail`, `ResendVerification`, `SignInLocal`, `SignInOidc`, `Refresh`, `Logout`, `RequestPasswordReset`, `ResetPassword`, `LinkIdentity`, `UnlinkIdentity`; **sign-in carries a target app audience; refresh derives it from the session** (one login per app, audience-bound sessions); depend on `user-port`
- [ ] 4.2 In `auth`: own schema `auth`; migration for `local_credentials` (email, argon2id hash, `email_verified`, verification token + expiry, password-reset token + expiry). Session/refresh state lives in **Redis** (TTL), not Postgres
- [ ] 4.3 Define the `IdentityVerifier` port; implement `OidcJwtVerifier` (Google + Apple, multi-issuer by `iss`) and `LocalCredentialVerifier` (email + argon2id). Generate the **Apple client secret** as a JWT signed with the `.p8` key (≤6-month expiry, regenerated)
- [ ] 4.4 Implement `SignUpLocal` (enforce password policy, argon2id hash, email unverified, send verification token; reject duplicate `ALREADY_EXISTS`), `VerifyEmail` (single-use, expiring), and `ResendVerification` (rate-limited)
- [ ] 4.5 Implement `SignInLocal` (verify password + `email_verified`; **rate-limit + temporary lockout** via Redis; wrong password `UNAUTHENTICATED`, unverified `FAILED_PRECONDITION`) and `SignInOidc` (verify → resolve/provision via `user` port); both validate the audience against the allow-list (`INVALID_ARGUMENT` if unknown)
- [ ] 4.6 Issue audience-scoped tokens on sign-in (access signed with the asymmetric key, `aud` = app, `user_id` + effective roles in claims; refresh stored in **Redis**). `Refresh` rotates the refresh token with **reuse detection** (a replayed/rotated token revokes the whole session family) and re-reads effective roles
- [ ] 4.7 Implement `Logout` (revoke the current session/refresh) and a revoke-all-sessions path
- [ ] 4.8 Implement `RequestPasswordReset` (single-use expiring token by email, rate-limited, **no account enumeration**) and `ResetPassword` (set new argon2id hash, invalidate existing sessions)
- [ ] 4.9 Implement `LinkIdentity` and `UnlinkIdentity` (authenticated): verify the credential/token, delegate to the `user` port; `ALREADY_EXISTS` when bound elsewhere; unlinking the last identity rejected; support local↔OIDC in both directions
- [ ] 4.10 Implement the **gRPC server adapter** (in `auth`) and **gRPC client adapter** (in `auth-port`)
- [ ] 4.11 Tests (fakes for `user` port, email sender, Redis): sign-up + verify + sign-in; duplicate email; weak password rejected; resend throttled; unverified blocked; wrong password; **lockout after N attempts**; OIDC sign-in + provision; unknown audience; token `aud` + scoped roles; **refresh rotation + reuse detection revokes session**; logout; password reset invalidates sessions; link/unlink incl. last-identity guard; contract test across both adapters

## 5. Server (composition root)

- [ ] 5.1 tonic server bootstrap: bind address, run all migrations, mount the `auth` and `user` gRPC **server adapters**, enable reflection in non-prod
- [ ] 5.2 Wire dependency injection: construct each module's **direct adapter**, inject the `user` direct adapter + Redis client + signing keys + rate-limiter into `auth`, install the internal-token interceptor — the single place adapters are chosen
- [ ] 5.3 gRPC Health service (readiness reflects DB + Redis reachability) **and the Axum HTTP surface** for `/.well-known/jwks.json` + `/healthz`, mounted alongside tonic
- [ ] 5.4 Initialize OpenTelemetry on startup and shut it down cleanly on exit
- [ ] 5.5 Tests: server builds with both modules + HTTP surface mounted; readiness toggles when DB or Redis is down

## 6. Observability (OpenTelemetry + Grafana)

- [ ] 6.1 Wire `tracing-opentelemetry` + the OpenTelemetry SDK + OTLP exporter in `platform`; endpoint + sampling configurable, export **disablable** for tests/offline runs
- [ ] 6.2 Emit a span per gRPC request (method, status, correlation id) with downstream DB/auth/Redis work as child spans; propagate trace context across async boundaries
- [ ] 6.3 Emit RED metrics (request rate/errors/duration) over OTLP
- [ ] 6.4 Emit **resource-consumption metrics** over OTLP: process CPU + resident memory (`sysinfo`), async-runtime saturation (`tokio-metrics`), and SQLx/Redis pool usage; enable the Collector **hostmetrics receiver** for host CPU/mem/disk/net
- [ ] 6.5 Bridge `tracing` events to the OpenTelemetry **Logs** signal via `opentelemetry-appender-tracing`, exported over OTLP; ensure each in-span log record carries `trace_id`/`span_id`; keep a console layer (pretty dev / JSON prod) in parallel; OTLP log export independently disablable
- [ ] 6.6 Assert in tests that no bearer token, password, secret, or PII appears in spans, metric labels, or logs
- [ ] 6.7 Add the observability stack to docker-compose (own profile): OTel Collector + Tempo + Prometheus + **Loki** + Grafana, with pre-provisioned datasources, a starter dashboard (RED + **resource** panels), and **trace↔logs correlation** (Tempo trace-to-logs + Loki `trace_id` derived field)
- [ ] 6.8 Write the observability technical docs: start the stack, point the OTLP endpoint at it, and find a sample request's trace, metrics, **resource usage**, and **correlated logs** in Grafana (incl. jumping trace↔logs)

## 7. Boundary enforcement, CI, coverage & docs

- [ ] 7.1 CI check asserting the module dependency graph (consumers depend on `-port` only; no impl→impl dependency); fail the build on violation
- [ ] 7.2 Integration test proving DB-level isolation: a module's role is denied when reading another schema's tables
- [ ] 7.3 Extend CI to build, `cargo fmt --check`, and `cargo clippy -D warnings` across all backend crates
- [ ] 7.4 `cargo llvm-cov` with the ≥80% gate, ignoring generated proto code and thin I/O glue (gRPC adapters, SQLx, Redis, JWKS/OIDC HTTP, SMTP, OTLP export) — domain logic and `*_core` helpers stay covered
- [ ] 7.5 Integration tests against docker-compose (Postgres + Redis + mock OIDC + Mailpit) covering sign-up/verify/resend/sign-in, OIDC sign-in, refresh reuse-detection, logout, password reset, link/unlink, and account get/update/delete
- [ ] 7.6 Backend README: modular-monolith layout, the port + dual-adapter pattern, the `IdentityVerifier` extension point, token/JWKS model, the extraction recipe, local setup, grpcurl smoke-tests
- [ ] 7.7 Run `openspec validate add-cymbra-id --strict` and ensure it passes
