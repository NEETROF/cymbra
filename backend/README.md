# Cymbra ID — backend

The shared identity/account service for Cymbra Music and Cymbra Live: a
gRPC-first **modular monolith** in Rust (tonic + SQLx/Postgres + Redis), with
OIDC (Google/Apple) + local email/password sign-in, audience-scoped session
tokens, per-app scoped roles, and first-class OpenTelemetry observability.

Spec: [`openspec/changes/add-cymbra-id`](../openspec/changes/add-cymbra-id).

## Layout (ports + dual adapters)

```
backend/
  platform/     cross-cutting: config, telemetry, token codec + interceptor,
                JWKS, OIDC/JWKS verify, argon2, email port, Redis, rate-limit
  auth-port/    contract: AuthPort trait + DTOs + cymbra.auth.v1 proto + gRPC client
  auth/         impl: IdentityVerifier, local credentials, sessions, gRPC server
  user-port/    contract: UserPort trait + DTOs + cymbra.user.v1 proto + gRPC client
  user/         impl: account aggregate (users/identities/roles), gRPC server
  server/       composition root (binary `cymbra-id`): wires everything, serves
                gRPC + the Axum JWKS/health surface
```

Each module is a **port** (Rust trait) with two interchangeable adapters: a
**direct** in-process impl and a **gRPC** impl (server + client). Consumers depend
on `<module>-port` only; the dependency rule is enforced in CI
([`scripts/check_boundaries.py`](scripts/check_boundaries.py)). Per-module Postgres
roles confine each module to its own schema. Tokens are signed with an asymmetric
key and published at a JWKS endpoint so Music/Live validate them offline.

### Extraction recipe

To split a module into its own service: give it a `main` that mounts the gRPC
**server** adapter it already has, point it at its own infra, and in `server` swap
the module's **direct** adapter for its `<module>-port` gRPC **client** adapter.
No changes to other modules' domain code.

## Local development

```bash
# infra: Postgres (+ per-module roles), Redis, a mock OIDC issuer, Mailpit
docker compose -f backend/docker-compose.yml up -d

cp backend/.env.example backend/.env   # fill in the signing keypair + OIDC ids
set -a; . backend/.env; set +a
cargo run -p cymbra-server --bin cymbra-id
```

Generate the internal-token signing keypair:

```bash
openssl genpkey -algorithm ed25519 -out priv.pem
openssl pkey -in priv.pem -pubout -out pub.pem
# set CYMBRA_TOKEN_SIGNING_KEY_PEM / CYMBRA_TOKEN_PUBLIC_KEY_PEM from these
```

Observability stack + docs: [`observability/README.md`](observability/README.md).

## Smoke test (grpcurl)

Reflection is not enabled, so pass the proto:

```bash
grpcurl -plaintext -proto backend/auth-port/proto/auth.proto \
  -d '{"email":"a@example.com","password":"a-strong-passphrase"}' \
  localhost:50051 cymbra.auth.v1.AuthService/SignUpLocal

curl -s localhost:8081/.well-known/jwks.json   # JWKS for downstream apps
curl -s localhost:8081/readyz                  # readiness (DB + Redis)
```

## Tests

```bash
cargo test --workspace                 # unit tests (fakes, no infra)
# integration (needs the infra above):
cargo test -p cymbra-auth -p cymbra-user -- --ignored
```

CI: [`rust.yml`](../.github/workflows/rust.yml) (fmt/clippy/coverage ≥80% +
boundary check) and [`backend-it.yml`](../.github/workflows/backend-it.yml)
(integration against Postgres + Redis).
