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
  (Google and Apple)**. It validates the provider ID token against the matching
  issuer's JWKS, then issues the backend's **own internal session tokens** (access
  + refresh); the per-request interceptor validates only the internal token. A
  pluggable `IdentityVerifier` port keeps the provider list open (e.g. Facebook
  later) without touching callers.
- Support **multiple providers linked to one internal account**: an account has
  one or more linked provider identities (`UNIQUE(provider, subject)`). Linking is
  **explicit** — an authenticated user links an additional provider; there is no
  email-based auto-merge. The backend stores no passwords.
- Add a **user module** for account management (profile + preferences with
  versioned optimistic concurrency) and a **per-app role scaffold**: roles are
  scoped (`global` / `music` / `live`) via `user_roles(user_id, scope, role)`,
  carried by Cymbra ID, assigned server-side (never from OIDC claims), with a
  role-based guard. Fine-grained domain authorization stays in each app.
- Issue **audience-scoped session tokens**: sign-in targets an app audience
  (`music`/`live`), and the access token embeds only the effective roles for that
  audience (`global` + that scope). Concrete admin/role-assignment endpoints are
  out of scope.
- Add **OpenTelemetry** tracing (and metrics) exported over OTLP, and a local
  **Grafana observability stack** (OpenTelemetry Collector + Tempo + Prometheus +
  Grafana) wired to the backend, with **technical documentation** for using it.
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
- `observability`: OpenTelemetry tracing/metrics exported over OTLP, a local
  Grafana exploration stack (Collector + Tempo + Prometheus + Grafana), and
  technical documentation for using them.

### Modified Capabilities
<!-- None — this change is additive and introduces a separate backend service
     without altering existing client-side spec requirements. -->

## Impact

- **New workspace members**: backend crates (`auth-port`/`auth` and `user-port`/
  `user` pairs, a `server` binary, and a thin `platform` crate) added to the Cargo
  workspace; CI build/test/coverage gates extended to cover them.
- **New dependencies**: `tonic`/`prost` (gRPC), `tokio`, `sqlx` (Postgres), an
  OIDC/JWT validation crate (`jsonwebtoken` + JWKS fetch) for both provider tokens
  and internal tokens, `config`, `tracing`, and OpenTelemetry crates
  (`opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`).
- **New infrastructure**: a Postgres database, external OIDC providers (Google,
  Apple) registered as relying-party clients, an internal token-signing key, and a
  local observability stack (OTel Collector, Tempo, Prometheus, Grafana) —
  provisioned for local dev via docker-compose and documented for deployment.
- **Protobuf contract**: new `.proto` definitions and generated code; these become
  the shared API contract the Flutter client will consume later.
- **Documentation**: a backend README (layout, ports/adapters, extraction recipe,
  local setup) and an observability guide (running the stack, finding traces and
  metrics in Grafana).
- **No change** to the existing Flutter app or on-device engine in this change.
- **Security surface**: introduces network-exposed authentication, internal-token
  issuance/refresh, provider-identity linking, and per-user account isolation.
