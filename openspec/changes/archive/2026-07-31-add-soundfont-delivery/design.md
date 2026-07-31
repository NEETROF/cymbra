## Context

Catalog score bytes live in a **private** OVH S3 bucket `cymbra-scores`
(`object_store` 0.11 `aws`, endpoint `s3.gra.io.cloud.ovh.net`, region `gra`), fronted by
a `LocalFirstStore` (local-disk warm cache + S3 origin; reads local-first, S3 fallback,
writes to S3). Postgres stores only the `object_key` pointer. The browser **never touches
OVH directly** — the back-office calls gRPC-web `GetCatalogScoreBytes`, and the handler
reads the bytes server-side and returns them inline. The reverse proxy (`backend/deploy/
Caddyfile`) exposes two path groups on the API domain: Axum on `:8081` (`/.well-known/*`,
`/healthz`, `/readyz`, `/web/auth/*`) and everything else → tonic gRPC/gRPC-web on `:50051`.
gRPC-web + web-auth CORS both allow-list `CYMBRA_BACK_OFFICE_ORIGINS` (prod
`https://bo.cymbra.app`), echoing an exact origin with credentials.

SoundFonts are a different concern: large (57 MB default, up to ~296 MB), redistributable
today (CC0/CC-BY) but **paid tomorrow**. They must be stored and served with the same
"private OVH + backend-proxied" shape as scores, but isolated (separate bucket/ACL/cache)
and access-controlled so paid fonts can be gated. Consumers: the back-office score-preview
playback (`add-wasm-notation-preview`) and the app's `piano-sound-selection`
download-on-first-use. Repo conventions: pure logic host-tested; IO/FFI glue thin and
coverage-excluded; trait-seam doubles are mockall.

## Goals / Non-Goals

**Goals:**
- Store SoundFonts in a dedicated **private** bucket `cymbra-soundfonts`, isolated from
  `cymbra-scores`.
- Serve a font to an **authenticated** client by id via a **range-capable streaming** HTTP
  route on the existing API origin — no public bucket, no browser→S3.
- A **per-font entitlement seam**: trivially "allowed" for free fonts now, gate-ready for
  paid fonts later, with no redesign.
- Reuse the proven `LocalFirstStore` (local warm cache + OVH origin) so repeat fetches are
  cheap and the scores cache is never bloated.
- Be the single host for both consumers (back-office + app), needing no client CSP change.

**Non-Goals:**
- The **purchase/entitlement system itself** (payments, ownership records) — only the
  check *seam* is built now; its free-font implementation is trivial.
- **Client wiring** — `VITE_SOUNDFONT_URL` in the back-office and the app's download source
  belong to their own changes.
- Uploading/managing fonts in-app, font transcoding/subsetting, or a CDN (a later
  optimisation; the streaming route is CDN-frontable).
- Any change to the scores bucket, catalog serving, or the gRPC surface.

## Decisions

### Decision: A separate private bucket + second LocalFirstStore, not a prefix in cymbra-scores
Add `cymbra-soundfonts` as its own bucket with its own `LocalFirstStore` instance (own
warm-cache root), reusing the OVH endpoint/region/credentials.
- *Why:* scores are strictly private/copyright; soundfonts have a different ACL trajectory
  (free now, **paid** later) and size profile (~296 MB). Separate buckets keep policies and
  the scores warm cache clean. **Alternatives:** a `soundfonts/` prefix in `cymbra-scores`
  (rejected: one ACL for two very different concerns, and 300 MB fonts bloat the scores
  cache); one giant store (same problem).
- *Trade-off:* a second storage target to configure. Mitigated by reusing the existing
  builder/credentials — only bucket + cache root differ.

### Decision: Authenticated streaming Axum route, not the gRPC-web bytes path
Serve fonts via a new **Axum** `GET /soundfonts/{id}` on `:8081`, streaming the object
body (range-capable), added to the Caddy `@http` matcher.
- *Why:* the existing `GetCatalogScoreBytes` returns bytes **inline over gRPC-web**, which
  base64-inflates and buffers the whole payload — unacceptable for 57–296 MB. A streaming
  HTTP GET lets Caddy/the browser do ranges + caching, and the browser fetches it from the
  API origin already permitted by CORS/CSP. **Alternatives:** a gRPC streaming method
  (works, but browsers can't range/cache it and it fights the byte-inline pattern); a public
  bucket + direct fetch (rejected — paid fonts can't be public, and it needs bucket-side
  CORS this repo doesn't manage).

### Decision: Auth + a per-font entitlement seam
The handler **requires a valid session** (same identity the gRPC/web-auth surfaces use) and
calls an `entitlement`-style trait: `may_access(identity, font_id) -> bool`. Today it
returns true for fonts flagged **free**; paid fonts (none yet) would consult a purchase
record. Unauthorized → `403`; unknown id → `404`.
- *Why:* the whole reason for a private bucket is future paid fonts — the gate must exist
  from the start, even if trivial, so adding paid fonts is data + one impl, not a redesign.
  The pure id→key + free/paid classification + decision logic is **host-tested**; the store
  IO and HTTP glue stay thin.
- *Trade-off:* a tiny amount of indirection now for a large amount of rework avoided later.

### Decision: A small font catalog (id → key, license, free/paid), server-owned
Fonts are addressed by a **stable id** (e.g. `upright-piano-kw`) mapped to an object key,
a license/attribution, and a `free|paid` tier. v1 lists the free defaults; the map is the
single source the route and the entitlement check read.
- *Why:* decouples client-facing ids from bucket keys, carries the CC-BY attribution the
  license requires, and is where paid tiers land later. Aligns with `piano-sound-selection`'s
  catalog-of-pianos framing so the app can share the same ids.

### Decision: Config-gated + graceful, mirroring scores storage
`CYMBRA_SOUNDFONT_S3_BUCKET` gates the feature (unset → the route is disabled / `503`,
like the scores bucket gates upload/serving). `CYMBRA_SOUNDFONT_LOCAL_ROOT` is the warm
cache. An absent object → `404`; an origin fetch failure surfaces a clean error, never a
panic.

## Risks / Trade-offs

- **Large-object streaming/memory** → stream from `LocalFirstStore` in chunks (honour
  `Range`); never buffer the whole 296 MB in the handler. Warm cache makes repeats disk-fast.
- **Cold-cache first fetch is slow** (pull 57–296 MB from OVH) → acceptable/on-demand;
  document it; a warm-on-deploy step can pre-pull the default. Client Cache-API means it's a
  once-per-browser cost.
- **Entitlement seam under-built** → keep it a real trait with a free-font impl + tests, so
  the paid path is additive; don't hardcode "always allow" inline.
- **Auth coupling** → reuse the existing session/identity extraction the Axum web-auth
  surface already has; if that identity isn't yet available to arbitrary Axum routes, expose
  it via the same middleware — no new auth model.
- **Bucket/credential sprawl** → reuse the OVH endpoint/region/keys; only bucket + cache
  root are new. Document that scores and soundfont buckets are distinct with distinct ACLs.
- **CDN later** → the route is CDN-frontable (cache by id + auth at the edge) if traffic
  grows; out of scope now.

## Migration Plan

Additive: new config (gated off until `CYMBRA_SOUNDFONT_S3_BUCKET` is set), a new storage
instance, a new Axum route + Caddy matcher entry, and an ops upload of the default font.
Nothing changes for existing scores/gRPC paths. Rollout: create the bucket, upload the
default `.sf2`, set the env, add the Caddy route (needs the box `git pull` + `caddy reload`
per the deploy runbook), then consumers point at it. Rollback: unset the env (route
disabled) / revert the route + Caddy entry; no data migration, scores untouched.

## Open Questions

- **Dedicated vs shared OVH credentials** for the soundfont bucket — reuse the scores keys
  or mint a separate key scoped to `cymbra-soundfonts` (better blast-radius isolation for
  paid content)? Lean dedicated key.
- **Route shape** — `GET /soundfonts/{id}` returning the raw `.sf2`, vs `/soundfonts/{id}.sf2`;
  and whether to expose a `GET /soundfonts` catalog listing (ids/labels/licenses) the app
  picker could reuse. Decide with `piano-sound-selection`.
- **Free-tier authz breadth** — any signed-in identity, or also unauthenticated for the CC0
  default (simpler for the app's first-run)? Default to requiring a session for uniformity;
  revisit if the app needs the font pre-login.
- **Cache invalidation / versioning** — ids are stable; if a font's bytes change, version the
  key (e.g. dated filename, already the convention) so client Cache-API stays correct.
