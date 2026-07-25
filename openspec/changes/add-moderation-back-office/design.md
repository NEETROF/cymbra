## Context

Change #3 of 3. #1 added `moderation_status` + `reviewed_by`/`reviewed_at` on
`catalog_scores`, made the hub accepted-only, and added a privileged (admin-only)
status filter + non-`accepted` fetch-bytes access. #2 adds app ratings and a
`needs_review` re-review flag. This change adds the **people and the tool**: the
`moderator` role, the accept/reject write path with audit, role granting, and the Vue
back office at `bo.cymbra.app`.

Backend facts that constrain this design:
- **gRPC-only over tonic**; `backend-service` forbids REST. No CORS/gRPC-web today.
- **Scoped RBAC exists**: `user_roles(user_id, scope, role)`, scopes `global|music|live`,
  roles `user|admin`; `effective_roles(user_id, audience)` unions `global` + that
  audience's scope into the token; guards `require_admin`/`require_role` exist. No
  `moderator` role and no admin RPCs yet; admin is DB-seeded.
- Access tokens are `aud`-scoped (one login per app), verified offline via JWKS.

## Goals / Non-Goals

**Goals:**
- A `music`-scoped `moderator` role; back-office access for `music/{admin,moderator}`
  (+ `global/admin` break-glass), no cross-app leakage.
- Accept/reject that records reviewer + timestamp (traceable rejections).
- A Vue SPA: table of all scores with the app's filters + a BO-only status filter + the
  re-review queue; click → read-only preview → accept/reject.
- Admin-managed role granting; a defined first-admin bootstrap.
- A browser transport (gRPC-web + CORS) that preserves "no REST".

**Non-Goals:**
- The app rating UI (#2). Bulk moderation, analytics dashboards, a full moderation
  event-history table. Changing how the Flutter app talks to the backend.

## Decisions

### D1 — `moderator` as a `music`-scoped role, reusing `user_roles`
Add `moderator` as a role value in scope `music`. `effective_roles` already unions
`global` + the audience scope, so a `music`-audience token naturally carries
`music/moderator`. Authorization for moderation ops = "is `admin` OR `moderator`".
- **Why scoped, not global**: the stakeholder explicitly rejected cross-app
  transversality. A `music/moderator` has zero power in `live`. `global/admin` remains a
  deliberate superuser escape hatch (already exists).
- **Alternative**: a separate `permissions` table / capability grants. Rejected —
  overkill for two roles; the scoped `user_roles` table already models exactly this.

### D2 — Back office authenticates with the `music` audience (+ a dedicated web OIDC client)
The console signs in against `aud = music` so `music`-scoped roles flow into the token
via the existing `effective_roles`. A **separate OIDC web client** (its own client id,
added to `CYMBRA_ALLOWED_AUDIENCES`/audience mapping) is used for the browser origin, but
it still targets the `music` audience.
- **Why not a new `bo` audience/scope**: `effective_roles(user, "bo")` would union
  `global` + `bo` scope and miss `music/moderator`. Reusing `music` keeps roles where the
  data lives. Trade-off: the back office and the app share an audience; acceptable since
  authorization is role-gated per method regardless of client.
- **Alternative considered**: `bo` audience with `effective_roles` special-cased to also
  union `music`. Rejected as a surprising special case; revisit only if BO must span
  multiple modules.

### D3 — Browser transport: `tonic-web` + CORS, no REST
Wrap the gRPC services with `tonic-web` and a `tower-http` `CorsLayer` allowing only the
back-office origin(s). The Flutter app is unaffected (native HTTP/2 gRPC). This is a
MODIFIED `backend-service` requirement: the foundation stays gRPC (gRPC-web is gRPC over
HTTP/1.1/2 framing), and **no REST API is introduced**.
- **Why gRPC-web over a REST gateway**: keeps a single service definition and the no-REST
  invariant; the Vue client uses a generated gRPC-web stub from the same protos.
- **Alternative**: an Envoy sidecar for gRPC-web. Rejected for now — `tonic-web` in-process
  is simpler for this scale; revisit at higher scale.

### D4 — Evaluate: `SetModerationStatus` writes status + audit atomically
A new RPC sets a score's `moderation_status` to `accepted` or `rejected` and stamps
`reviewed_by = caller` + `reviewed_at = now()` in the same update. Guarded by
"admin-or-moderator". Setting back to `pending` is allowed (re-queue). The write is a
single conditional UPDATE.
- **Why store only last reviewer**: matches #1's columns; enough to trace "who rejected
  this". A full history table is a later change if needed.
- **Widen #1's guards**: the privileged status filter and non-`accepted` fetch-bytes,
  admin-only in #1, are widened here to admin-or-moderator (a guard change; no proto/schema
  change). Expressed as new `moderation-access-control` requirements so this change's specs
  stay additive and independent of #1's archive order.

### D5 — Console table = the app's filters + a privileged status filter + queue
The console lists **all** scores in a simple table using the **same catalog search** the
app uses (same text/author/level/facet filters), plus the privileged `moderation_status`
filter (from #1) that only moderators/admins may send. A default "queue" view orders work:
`pending` first (e.g. newest or by source), then `accepted` scores flagged `needs_review`
by #2. Row click → read-only preview (reusing the score bytes + a web renderer or an
embedded view) → accept/reject.
- **Why reuse catalog search**: the requirement is explicitly "the same filters as the
  hub". One search surface, one set of filters, with the status filter as the BO-only
  extra. Avoids a parallel query surface.
- **Preview in the browser**: the console fetches score bytes (moderators may fetch
  non-`accepted`, D4) and renders MusicXML in a web view. Exact renderer (e.g. a JS music
  notation lib) is a console-local choice; out of scope for the backend.

### D6 — Role granting + first-admin bootstrap
`GrantRole(user, scope, role)` / `RevokeRole(...)` RPCs, guarded by `require_admin`,
let a `music/admin` grant `moderator`/`admin` within `music`. Only an `admin` can create
another `admin`. The **first `music/admin` is seeded out-of-band** by an operator with DB
access (a small `backend/scripts` command or SQL, consistent with today's "admin seeded in
DB"), breaking the chicken-and-egg. After that, admins self-serve from the console.
- **Why ops-seed first admin**: there is no trusted in-app path to mint the very first
  admin; an operator with DB access is the root of trust, as today.

## Risks / Trade-offs

- **CORS/gRPC-web misconfig exposes the API to other origins** → restrict `CorsLayer` to
  the exact back-office origin(s); every method still enforces auth + role guards, so a
  wrong origin cannot escalate. Review the origin allow-list in config.
- **Empty hub until moderation catches up** (from #1) → the console's queue + #2's flag
  prioritize work; consider onboarding moderators before #1's prod migration.
- **Shared `music` audience for app + BO** (D2) → a leaked BO token is a `music` token;
  mitigated by short-lived access tokens, role gating per method, and the BO's own OIDC
  client. Revisit a dedicated audience if isolation becomes a requirement.
- **Privilege escalation via GrantRole** → only `require_admin` may grant; granting
  `admin` requires being an `admin`; log all grants. Consider a grant audit table later.
- **New web app maintenance** → a Vue SPA is new surface; keep it thin (table + preview +
  two actions) and reuse the marketing-site deploy pattern (Cloudflare Pages).

## Migration Plan

1. Backend: add `moderator` handling in `effective_roles`/guards; `SetModerationStatus`,
   `GrantRole`/`RevokeRole` RPCs; widen #1 guards to admin-or-moderator; add `tonic-web` +
   CORS. All additive; the app is unaffected.
2. Seed the first `music/admin` via the ops command in the target environment.
3. Ship the Vue SPA to `bo.cymbra.app`; the seeded admin promotes moderators.
4. **Rollback**: remove the web app + RPCs + CORS/gRPC-web layer; roles in `user_roles`
   are harmless if unused. The app never depended on any of it.

## Open Questions

- **Vue vs. alternative** — the stakeholder flagged "VueJS à challenger". Vue 3 + Vite SPA
  is the default here; a lighter option (e.g. a small Preact/vanilla + gRPC-web) could be
  weighed at implementation. Decide before scaffolding the web app.
- **Preview renderer in the browser** — which MusicXML/notation renderer the console uses
  (and whether to reuse any Rust/wasm rendering) is undecided; backend only serves bytes.
- **Grant audit** — do we need a `role_grants` audit trail now, or is logging enough?
  Assumed logging for v1.
