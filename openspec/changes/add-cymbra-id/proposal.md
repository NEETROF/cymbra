## Why

Cymbra is growing into **two front-end apps** that must share the same users:
**Cymbra Music** (the existing music app) and **Cymbra Live** (RTC audio/video
channel streaming). Both need one shared identity and account — **Cymbra ID** —
so a user is the same person across products. Today everything is local: no
server, no accounts, no central identity.

This change establishes **Cymbra ID**, the base Rust backend that owns shared
identity and accounts: a modular monolith with an **auth module** (sign in via
Google, Apple, or a local email/password, multiple providers linkable to one
account) and a **user module** (account + **per-app roles**), issuing
**audience-scoped tokens** that Cymbra Music and Cymbra Live consume. It is wired
for first-class **observability** (OpenTelemetry + Grafana). The product apps
(Music, Live) and Live's RTC/media service are **separate** services that trust
Cymbra ID tokens and key their own domain data on the shared `user_id`.

Registration is **optional**: each app also runs in a fully **standalone,
no-account mode** (purely local, **zero server calls**), so Cymbra ID is reached
only when a user chooses to register or sign in — there is no anonymous/guest
server session. This is deliberately a foundation — clean, observable,
secure-by-default — not the full product backend.

## What Changes

- Add a new backend service built as a **modular monolith** Rust template
  (gRPC-first via **tonic**, Postgres via SQLx) under new workspace members: one
  crate pair per module plus a thin shared/platform crate and a composition-root
  binary.
- **Separation of concerns is a first-class requirement**: each module is
  self-contained, owns its own data, and is reachable only through a published
  port (Rust trait). Every port has **two interchangeable adapters** — a direct
  in-process implementation for inter-module calls, and a gRPC implementation for
  the public/network surface — so any module can be extracted into its own service
  by swapping the adapter, with no caller changes.
- Add an **auth module** that authenticates users via external **OIDC providers
  (Google and Apple)** **and a local email/password provider** (argon2id hash,
  **required email verification**), all behind a pluggable `IdentityVerifier` port
  (provider list stays open). It issues the backend's **own session tokens** —
  access (asymmetrically signed, public keys served at a **JWKS endpoint** so
  Cymbra Music/Live validate **offline**) + refresh (rotated, with **reuse
  detection**) — and the per-request interceptor validates only the internal token.
- Cover the full **session lifecycle**: `Refresh`, `Logout`/revoke-all,
  **password reset** and **resend verification** (single-use, expiring, throttled),
  with **password policy + rate-limiting/lockout** and **no account enumeration**.
  **Redis** backs sessions/refresh and rate-limit counters; durable data stays in
  Postgres.
- Support **multiple providers linked to one internal account** (`UNIQUE(provider,
  subject)`): **explicit** link **and unlink** (cannot unlink the last identity);
  no email-based auto-merge. **Account deletion** is in scope (erasure detailed at
  implementation). The backend stores no third-party passwords.
- Add a **user module** for account management (profile + preferences with
  versioned optimistic concurrency) and a **per-app role scaffold**: roles are
  scoped (`global` / `music` / `live`) via `user_roles(user_id, scope, role)`,
  carried by Cymbra ID, assigned server-side (never from OIDC claims), with a
  role-based guard. Fine-grained domain authorization stays in each app.
- Issue **audience-scoped session tokens**: sign-in targets an app audience
  (`music`/`live`), and the access token embeds only the effective roles for that
  audience (`global` + that scope). Concrete admin/role-assignment endpoints are
  out of scope.
- Add **OpenTelemetry** across all three signals — traces, metrics (RED **plus
  resource consumption**: process CPU/memory, async-runtime + DB-pool saturation,
  and host metrics via the Collector), and **logs** (`tracing` events bridged to
  the OTel Logs signal, carrying `trace_id`/`span_id`)
  — exported over OTLP, and a local **Grafana observability stack** (OpenTelemetry
  Collector + Tempo + Prometheus + **Loki** + Grafana) with **trace↔logs
  correlation** wired to the backend, plus **technical documentation** for using it.
- Add database schema + migrations, typed configuration, health/readiness, and
  structured logging for the new service.

This is foundational and additive; it does **not** change existing client
capabilities. Flutter client integration is a follow-up, not part of this base.

**Explicitly out of scope (future):** a **billing / entitlements** module —
purchases are platform-specific and multiplatform-sensitive, so this base only
**wires the seams** (entitlements keyed by `user_id`, app `scope`, and a
`ReceiptVerifier` pattern mirroring `IdentityVerifier`); the actual store
integrations (App Store / Google Play / Stripe), server-to-server webhooks, and
cross-platform entitlement sharing come in a later `billing` module (possibly via
RevenueCat). Also future: the **Cymbra Live RTC/media** service; Steam identity
(dropped, outside OIDC) and Facebook (addable as one more `IdentityVerifier`);
user **file** sync/upload (scores, SoundFonts) and its **S3** storage. The
architecture leaves clean seams for all of these.

## Capabilities

### New Capabilities
- `backend-service`: Service foundation / template — the tonic gRPC server
  scaffold, the modular-monolith crate layout (ports + dual adapters), typed
  configuration, Postgres connection pool + migrations, health/readiness, and
  structured logging.
- `backend-auth`: The auth module — multi-issuer OIDC verification (Google,
  Apple) and local email/password behind an `IdentityVerifier` port, sign-in
  targeting an app audience and issuing **audience-scoped** internal session
  tokens (with the effective role set), refresh, the per-request internal-token
  interceptor, explicit provider linking, and the role-based guard.
- `user-account`: The user module — the internal account plus its linked provider
  identities (1→N), provisioning/linking by `(provider, subject)`, account
  management (profile + preferences, optimistic concurrency), and **per-app scoped
  roles** (`user_roles(user_id, scope, role)`).
- `observability`: OpenTelemetry traces/metrics/logs exported over OTLP — metrics
  include RED **and resource consumption** (process CPU/memory, runtime + DB-pool
  saturation, host metrics); logs bridged from `tracing` and correlated to traces —
  a local Grafana exploration stack (Collector + Tempo + Prometheus + Loki +
  Grafana) with trace↔logs correlation, and technical documentation for using them.

### Modified Capabilities
<!-- None — this change is additive and introduces a separate backend service
     without altering existing client-side spec requirements. -->

## Impact

- **New workspace members**: backend crates (`auth-port`/`auth` and `user-port`/
  `user` pairs, a `server` binary, and a thin `platform` crate) added to the Cargo
  workspace; CI build/test/coverage gates extended to cover them.
- **New dependencies**: `tonic`/`prost` (gRPC), `axum` (JWKS + health HTTP
  endpoint), `tokio`, `sqlx` (Postgres), `redis`/`deadpool-redis` (sessions +
  rate-limit), `jsonwebtoken` (EdDSA/RS256 internal tokens + provider verification),
  `argon2`, `config`, `tracing`, and OpenTelemetry crates
  (`opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`,
  `opentelemetry-appender-tracing` for the logs signal, `tokio-metrics` + `sysinfo`
  for resource metrics).
- **New infrastructure**: a Postgres database, a **Redis** instance, an SMTP
  sender (Mailpit in dev), external OIDC providers (Google, Apple) registered as
  relying-party clients, an **asymmetric token-signing keypair published via JWKS**,
  and a local observability stack (OTel Collector, Tempo, Prometheus, Loki,
  Grafana) — provisioned for local dev via docker-compose and documented for
  deployment.
- **Protobuf contract**: new `.proto` definitions and generated code; these become
  the shared API contract the Flutter client will consume later.
- **Documentation**: a backend README (layout, ports/adapters, extraction recipe,
  local setup) and an observability guide (running the stack, finding traces,
  metrics, and correlated logs in Grafana).
- **No change** to the existing Flutter app or on-device engine in this change.
- **Security surface**: introduces network-exposed authentication, internal-token
  issuance/refresh, provider-identity linking, and per-user account isolation.
