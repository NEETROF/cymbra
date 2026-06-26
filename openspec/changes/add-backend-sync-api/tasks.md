## 1. Workspace & modular layout

- [ ] 1.1 Create the backend directory with workspace members: `platform`, a `server` binary, and two module crate pairs — `auth-port`/`auth` and `user-port`/`user`
- [ ] 1.2 Establish the dependency rule in the manifests: consumers depend on `<module>-port` only (`auth` depends on `user-port`); impl crates depend on `platform` + their own `-port`; `server` depends on all; no impl crate depends on another impl crate
- [ ] 1.3 Add shared dependencies: `tonic`, `prost`, `tokio`, `tower`, `sqlx` (postgres, runtime-tokio, migrate), `jsonwebtoken` (provider + internal tokens), `argon2`, `config`, `tracing`/`tracing-subscriber`, `tracing-opentelemetry`, `opentelemetry`, `opentelemetry-otlp`, an SMTP/email crate (e.g. `lettre`), `thiserror`/`anyhow`, `uuid` (with the `v7` feature — all internal ids are UUID v7, generated app-side via `Uuid::now_v7()`), `async-trait`
- [ ] 1.4 Set up `tonic-build` so each `<module>-port` owns its `proto/` and generated client/server stubs (packages `cymbra.auth.v1`, `cymbra.user.v1`)
- [ ] 1.5 Add docker-compose for local dev: Postgres + a mock OIDC issuer + Mailpit (SMTP sink)
- [ ] 1.6 Add a DB bootstrap that provisions schemas `auth` and `user_account`, each with a per-module Postgres role granted privileges **only** on its own schema (`search_path` pinned)
- [ ] 1.7 Add `.env.example` / config schema: DB roles, Google/Apple `iss`/`aud`, the **app-audience allow-list** (`music`/`live`), internal-token signing key + TTLs, SMTP settings, OTLP endpoint + toggle

## 2. Platform crate (cross-cutting)

- [ ] 2.1 Typed configuration loading (env + optional file) with fail-fast validation and a unit-tested `config_core` parser
- [ ] 2.2 Structured logging via `tracing` with request correlation ids; redact tokens/secrets
- [ ] 2.3 SQLx pool factory building a pool per module using that module's own DB role, plus a per-module migration runner
- [ ] 2.4 Shared `AuthIdentity { user_id, roles }` context type and gRPC error/status mapping helpers
- [ ] 2.5 Internal-token **JWT codec** (sign + verify) in a host-testable `token_core`, and the **internal-token interceptor** that validates the access token on protected methods and injects `AuthIdentity`; reject missing/invalid/expired with `UNAUTHENTICATED`
- [ ] 2.6 OIDC verification helper: JWKS fetch+cache and signature/`iss`/`aud`/`exp` checks in a host-testable `oidc_core`
- [ ] 2.7 argon2id password-hash helper and an **email-sender port** (fake for tests, SMTP impl for runtime)
- [ ] 2.8 Role-based guard `require_role(r)` / `is_admin` reading the role set from `AuthIdentity`; returns `PERMISSION_DENIED` when the role is absent
- [ ] 2.9 Tests: config validation; internal-token sign/verify (incl. roles claim) + interceptor outcomes; OIDC claim checks; guard allow/deny by role

## 3. User module (user-account)

- [ ] 3.1 In `user-port`: define the **port trait** (resolve-or-provision by `(provider, subject)`, link identity, list identities, get account, update account, read **effective roles for a scope** / `has_role(scope, role)`), DTOs, and `.proto`
- [ ] 3.2 In `user`: own schema `user_account`; migrations for `users` (`id` UUID v7, profile, preferences, `version`, timestamps — no role/provider columns), `user_identities` (`id`, `user_id`, `provider`, `subject`, `linked_at`, `UNIQUE(provider, subject)`), and `user_roles` (`user_id`, `scope`, `role`, `UNIQUE(user_id, scope, role)`)
- [ ] 3.3 In `user`: repositories scoped to the caller's `user_id`; resolve-or-provision (seed default role `(global, user)`) and link enforcing the uniqueness constraint (reject identity bound elsewhere); effective-roles query returns `global` + requested scope
- [ ] 3.4 Implement the **direct adapter**: resolve/provision, link, list identities, read roles, get account, update account with optimistic concurrency in a `version_core` helper (commit only if version matches; else `ABORTED` + current version)
- [ ] 3.5 Implement the **gRPC server adapter** (in `user`) and **gRPC client adapter** (in `user-port`)
- [ ] 3.6 Tests: first identity provisions with default `(global, user)`; known identity resolves; link attaches; already-linked rejected; multiple scoped roles stored once each; effective roles for a scope = `global` + that scope (excludes other apps); same set resolved across providers; list/get isolated per user; update increments version; stale update rejected; contract test across both adapters

## 4. Auth module (backend-auth)

- [ ] 4.1 In `auth-port`: define the `AuthService` `.proto` and port (SignUpLocal, VerifyEmail, SignInLocal, SignInOidc, Refresh, LinkIdentity) + DTOs; sign-in/refresh carry a target **app audience**; depend on `user-port`
- [ ] 4.2 In `auth`: own schema `auth`; migration for `local_credentials` (email, argon2id hash, `email_verified`, verification token + expiry) and refresh-token state (for rotation/revocation)
- [ ] 4.3 Define the `IdentityVerifier` port; implement `OidcJwtVerifier` (Google + Apple, multi-issuer, selected by `iss`, using the platform OIDC helper) and `LocalCredentialVerifier` (email + argon2id)
- [ ] 4.4 Implement `SignUpLocal` (create local credential with argon2id hash, email unverified, send verification email via the email-sender port; reject duplicate with `ALREADY_EXISTS`) and `VerifyEmail` (single-use, expiring token)
- [ ] 4.5 Implement `SignInLocal` (verify password + `email_verified`; reject wrong password `UNAUTHENTICATED`, unverified `FAILED_PRECONDITION`) and `SignInOidc` (verify token → resolve/provision via `user` port); both validate the target audience against the allow-list (reject unknown with `INVALID_ARGUMENT`)
- [ ] 4.6 Implement audience-scoped internal-token issuance on successful sign-in (access + refresh), setting `aud` to the app and stamping `user_id` + the **effective roles for that audience** (read via the `user` port by scope) into the access-token claims; and `Refresh` (validate, rotate refresh, re-read effective roles for the same audience, reject revoked/expired)
- [ ] 4.7 Implement `LinkIdentity` (authenticated): verify the new credential/token, call the `user` port to link to the current account; surface `ALREADY_EXISTS` when bound elsewhere; support linking a local credential onto an OIDC-first account and vice versa
- [ ] 4.8 Implement the **gRPC server adapter** (in `auth`) and **gRPC client adapter** (in `auth-port`)
- [ ] 4.9 Tests (fakes for `user` port + email sender): sign-up + verify + sign-in happy path; duplicate email; unverified sign-in blocked; wrong password; OIDC sign-in + provision; unknown audience rejected; token `aud` + effective roles scoped to the audience; refresh rotation; link new provider; link already-bound rejected; contract test across both adapters

## 5. Server (composition root)

- [ ] 5.1 tonic server bootstrap: bind address, run all migrations, mount the `auth` and `user` gRPC **server adapters**, enable reflection in non-prod
- [ ] 5.2 Wire dependency injection: construct each module's **direct adapter**, inject the `user` direct adapter into `auth`, install the internal-token interceptor — the single place adapters are chosen
- [ ] 5.3 gRPC Health service with readiness reflecting DB reachability
- [ ] 5.4 Initialize OpenTelemetry on startup and shut it down cleanly on exit
- [ ] 5.5 Tests: server builds with both modules mounted; readiness toggles when the DB is down

## 6. Observability (OpenTelemetry + Grafana)

- [ ] 6.1 Wire `tracing-opentelemetry` + the OpenTelemetry SDK + OTLP exporter in `platform`; endpoint + sampling configurable, export **disablable** for tests/offline runs
- [ ] 6.2 Emit a span per gRPC request (method, status, correlation id) with downstream DB/auth work as child spans; propagate trace context across async boundaries
- [ ] 6.3 Emit RED metrics (request rate/errors/duration) over OTLP
- [ ] 6.4 Assert in tests that no bearer token, password, secret, or PII appears in spans, metric labels, or logs
- [ ] 6.5 Add the observability stack to docker-compose (own profile): OTel Collector + Tempo + Prometheus + Grafana, with pre-provisioned datasources and a starter dashboard
- [ ] 6.6 Write the observability technical docs: start the stack, point the OTLP endpoint at it, find a sample request's trace and metrics in Grafana

## 7. Boundary enforcement, CI, coverage & docs

- [ ] 7.1 CI check asserting the module dependency graph (consumers depend on `-port` only; no impl→impl dependency); fail the build on violation
- [ ] 7.2 Integration test proving DB-level isolation: a module's role is denied when reading another schema's tables
- [ ] 7.3 Extend CI to build, `cargo fmt --check`, and `cargo clippy -D warnings` across all backend crates
- [ ] 7.4 `cargo llvm-cov` with the ≥80% gate, ignoring generated proto code and thin I/O glue (gRPC adapters, SQLx, JWKS HTTP, SMTP, OTLP export) — domain logic and `*_core` helpers stay covered
- [ ] 7.5 Integration tests against docker-compose (Postgres + mock OIDC + Mailpit) covering local sign-up/verify/sign-in, OIDC sign-in, linking, and account get/update
- [ ] 7.6 Backend README: modular-monolith layout, the port + dual-adapter pattern, the `IdentityVerifier` extension point, the extraction recipe, local setup, grpcurl smoke-tests
- [ ] 7.7 Run `openspec validate add-backend-sync-api --strict` and ensure it passes
