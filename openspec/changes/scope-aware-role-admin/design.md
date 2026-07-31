## Context

Roles are `(user_id, scope, role)` triples with `scope ∈ {global, music, live}` and
`role ∈ {user, admin, moderator}`. A user's *effective* roles for an app = that app's
scope ∪ `global`. The backend write path (`grant_role`/`revoke_role`, `role_grants`
audit, `validate_scope_role`) already accepts any valid scope. The gaps are all on the
**read/authorization/UI** side:

- `require_admin` (`backend/platform/src/guard.rs:19`) checks a single flattened role set
  (`AuthIdentity.roles`), which is "effective roles for the token's audience". The
  grant/revoke handlers (`backend/user/src/grpc.rs:145-172`) pass the request's `scope`
  straight through **without** comparing it to the caller's scope, so a `music/admin`
  token can already mutate `live`.
- Tokens are single-audience: `effective_roles(user_id, scope)` unions only
  `["global", scope]` (`backend/user/src/module.rs:189`), and `Claims.aud` is one string.
  A back-office login uses `AUDIENCE = "music"` (`apps/back-office/src/stores/auth.ts:7`).
- `ListAccounts` hardcodes `LEFT JOIN user_roles r ... AND r.scope = 'music'`
  (`backend/user/src/pg.rs:361`), mirrored in the in-memory fake (`repo.rs:422`), and the
  proto `AccountRow.roles` is a flat "music-scope roles" list.

Allowed audiences default to `["music","live"]` (`backend/platform/src/config.rs:363`).

## Goals / Non-Goals

**Goals:**

- Scope-matched authorization for role administration: mutate scope `S` ⇒ caller is
  `admin` in `S` or `global/admin`.
- One back-office session can administer every scope the admin is entitled to.
- Directory + UI show only the scopes the caller may administer, enforced server-side.
- No regression for the `music`/`live` **app** tokens (additive changes only).

**Non-Goals:**

- No new roles or scopes (still `user/admin/moderator` × `global/music/live`).
- No change to the grant/revoke DB write path, audit schema, or bootstrap flow.
- No per-scope UI beyond the Roles page (moderation console etc. unchanged).
- No fine-grained per-capability permissions — this stays coarse RBAC.

## Decisions

### D1 — Identity carries roles per scope, not a flat set

Extend `Claims` to carry roles **with their scope** and surface it on `AuthIdentity` as a
per-scope map (e.g. `roles_by_scope: BTreeMap<String, Vec<String>>`), keeping the existing
flat `roles`/`has_role` for backward compatibility. Add `AuthIdentity::has_role_in_scope(scope, role)`
with semantics: `roles_by_scope[global].contains(role) || roles_by_scope[scope].contains(role)`.

- *Why:* the guard cannot answer "admin in `live`?" from a flattened union. Scope must be
  preserved end-to-end (token → identity → guard).
- *Backward-compat:* app tokens (`music`/`live`) populate `roles_by_scope` with just
  `{global: …, <aud>: …}`; `has_role`/`require_admin` keep working from the flattened
  view, so existing guards in other modules are untouched.
- *Alternative rejected:* keep `Claims` flat and re-query the DB in the guard — couples the
  platform guard to the user repo and adds latency on every call. Rejected.

### D2 — A `back-office` audience whose effective roles span all scopes

Add `back-office` to the allowed audiences. When `issue()` mints a token for this audience,
`effective_roles` returns the caller's roles across **all** scopes (`global`, `music`,
`live`), grouped by scope, rather than `["global", aud]`. Introduce a
`scoped_effective_roles(user_id)` on the user port that returns the per-scope map;
`issue()` uses it for the `back-office` audience and the existing single-scope path for
app audiences.

- *Why:* realizes the user's model — a `music/admin` in the back office sees only `music`,
  a `global/admin` sees all — in a single session, without audience switching.
- *Guarding the audience:* the back-office audience carries no authority by itself; every
  privileged op is still gated by `require_admin_in_scope`. So minting a back-office token
  for a plain `user` is harmless (they'll be denied everything and see an empty directory).
- *Alternative rejected:* audience-switching (mono-scope tokens, toggle music/live in the
  UI) — worse UX, and a non-`global` admin of both scopes still couldn't see both at once.

### D3 — `require_admin_in_scope(id, scope)` for grant/revoke/list

New guard: `require_admin_in_scope(id, scope) = id.has_role_in_scope(scope, "admin")`.
`grant_role`/`revoke_role` call it with the **request's target scope** (`r.scope`) instead
of `require_admin(&id)`. This applies uniformly to **every** scope including `global`:
granting/revoking a `global` role requires `has_role_in_scope("global","admin")`, which is
exactly `global/admin` — so only a `global/admin` can promote or demote another
`global/admin`. `list_accounts` computes the caller's **authorized scopes** =
`{ s ∈ SCOPES : has_role_in_scope(s,"admin") }` (this naturally yields all of
`global/music/live` for a `global/admin`, and just the caller's own scope otherwise) and
passes them to the read query.

- *Why:* this is the load-bearing enforcement. Keeping it in the gRPC layer matches the
  current design (module comment: "Authorization is enforced at the gRPC layer").
- Granting `admin` uses the same check (admin-in-target-scope), preserving "granting admin
  requires admin" per-scope.

### D4 — `ListAccounts` returns roles grouped by scope, filtered to authorized scopes

Replace the hardcoded `r.scope = 'music'` join with a join filtered to the caller's
authorized scopes (`r.scope = ANY($scopes)`), aggregating roles per scope. Change the
proto `AccountRow.roles` from a flat `repeated string` to a per-scope shape — e.g.
`repeated ScopeRoles roles_by_scope { string scope; repeated string roles; }` — and update
the port `AccountSummary` + in-memory fake accordingly.

- *Why:* the directory must not leak roles from scopes the caller can't administer; doing
  it in SQL keeps it authoritative (not UI-hidden).
- *Proto compat:* this is a wire-shape change to an admin-only, internally-consumed RPC
  (back office only). We rename the field to avoid silent misreads; the back office is
  updated in lockstep. Documented as BREAKING for the RPC in the proposal.

### D5 — Back office: audience, store, and RolesView

- `auth.ts`: `AUDIENCE = "back-office"`. `isAdmin`/`isModerator` stay derived from the
  token, now interpreted per scope (an admin in *any* scope may reach the Roles page;
  route guard keeps gating `/roles` on holding `admin` in ≥1 scope).
- `roles.ts`: `grant`/`revoke` take an explicit required `scope` (drop the `"music"`
  default) sourced from the selected scope.
- `RolesView.vue`: derive the **authorized scopes** from the token; render a scope selector
  only when >1 is authorized; the table shows the selected scope's roles per row. The audit
  history sub-table already renders `scope` and stays as-is.
- Reuse the `Async<T>` ts-pattern union and the injectable client seam per the
  `vue-frontend-architecture` skill; no API calls from components.

## Risks / Trade-offs

- **[Authorization tightening is behavior-changing]** A `music/admin` that (incorrectly)
  relied on mutating `live` today will now be denied. → Intended fix; call it out in the PR
  and cover both directions with tests (`music/admin`→`live` denied; `global/admin`→`live`
  allowed).
- **[Proto field reshape]** Changing `AccountRow.roles` breaks any other consumer. → The
  RPC is admin-only and consumed only by the back office in this repo; update in lockstep,
  regenerate `gen/user_pb.ts`, and grep for other callers before merge.
- **[New audience widens token surface]** A back-office token aggregates scopes. →
  Authority still comes only from `has_role_in_scope`; the audience alone grants nothing.
  Keep `back-office` off the app clients (only the back office requests it).
- **[Identity shape change ripples across guards]** Other modules use `AuthIdentity`. →
  Keep the flat `roles`/`has_role`/`require_admin` intact and additive; only role-admin
  paths adopt `_in_scope`. Run the full workspace test + clippy.
- **[Token size]** Aggregating three scopes slightly enlarges the JWT. → Negligible (a
  handful of short strings); no action.

## Migration Plan

1. Backend platform: extend `Claims`/`AuthIdentity` (additive), add `has_role_in_scope`
   and `require_admin_in_scope`. Ship first — no behavior change yet.
2. User module: add `scoped_effective_roles`; auth `issue()` handles the `back-office`
   audience; add `back-office` to allowed audiences (config/env
   `CYMBRA_ALLOWED_AUDIENCES`). Deploy config before flipping the front end.
3. User grpc: switch grant/revoke/list to the scope-matched guard + per-scope read.
   Reshape `AccountRow` proto; regenerate bridges/`gen`.
4. Back office: audience → `back-office`, scope-aware store + RolesView. Deploy last.
5. **Rollback:** revert the front end to `AUDIENCE="music"` and the grpc layer to
   `require_admin`; the platform/identity additions are inert if unused. Keep `music` in
   allowed audiences throughout so app tokens never break.

Prod note: `CYMBRA_ALLOWED_AUDIENCES` must include `back-office` (and keep `music`,`live`)
before the back office deploys — otherwise back-office sign-in fails `check_audience`.

## Open Questions

- Audience name: `back-office` vs `admin` vs `console` — cosmetic; `back-office` chosen for
  clarity. Confirm no collision with any existing OIDC `aud` mapping.
- **Resolved:** `global`-scope roles **are** grantable from the back office. The
  scope-matched guard makes this safe by construction — only a `global/admin` can grant or
  revoke a `global` role (see D3). The Roles page therefore offers `global` as a selectable
  scope to a `global/admin`, alongside `music` and `live`.
