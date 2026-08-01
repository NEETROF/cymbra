## Why

The role model is genuinely multi-scope (`global` / `music` / `live`), but the back
office is wired to `music` only three ways: it authenticates against the `music`
audience, its roles store defaults `scope = "music"` and never overrides it, and
`ListAccounts` joins `user_roles` with `r.scope = 'music'` hardcoded. Worse, the
grant/revoke guard is **not** scope-matched — `require_admin` only proves "admin in the
token's scope" while the handler accepts any target `scope` — so a `music/admin` can
already grant `live` roles, violating scope isolation. We want a proper scope-matched
RBAC for role administration: `global/admin` administers `music` **and** `live`;
`music/admin` only `music`; `live/admin` only `live`.

## What Changes

- **BREAKING (authorization tightening)**: Granting/revoking a role in scope `S` SHALL
  require the caller to hold `admin` in `S`, or `global/admin`. A `music/admin` can no
  longer affect `live` roles (and vice-versa). This closes the current gap where any
  admin token could mutate any scope.
- Access tokens/identities carry the caller's roles **with their scope**, so the guard
  can answer "is the caller admin in scope `S`" instead of checking a single flattened
  role set.
- A back-office sign-in obtains a session whose token reflects the admin's **actual**
  roles across `global ∪ music ∪ live` (a back-office audience), replacing the hardcoded
  `music`-only audience — so one session can administer every scope the admin is
  entitled to.
- `ListAccounts` returns each account's roles **per scope** and exposes **only** the
  scopes the calling admin is authorized for (enforced in the backend, not merely hidden
  in the UI).
- The back-office Roles page gains scope selection/columns and grants/revokes within a
  chosen authorized scope; an admin sees only the scopes they may administer (a
  `music`-only admin never sees `live` or `global`).
- Roles are administrable in **every** scope the caller is admin for, **including
  `global`**: a `global/admin` can grant/revoke `global` roles from the UI (e.g. promote
  another `global/admin`). The scope-matched guard makes this safe — only a `global/admin`
  can act in the `global` scope — so `global` is no longer bootstrap/DB-only.

## Capabilities

### New Capabilities

- `back-office-admin-session`: A back-office sign-in yields an access token that
  reflects the administrator's real roles in **every** scope they hold
  (`global`, `music`, `live`), so a single session can administer the scopes they are
  entitled to — instead of a `music`-only audience token.

### Modified Capabilities

- `moderation-access-control`: Role grant/revoke authorization becomes **scope-matched**
  — mutating a role in scope `S` requires `admin` in `S` or `global/admin`; the token's
  audience no longer confers cross-scope authority. Requires identities to expose the
  caller's roles per scope.
- `admin-account-directory`: The directory returns each account's roles **per scope** and
  restricts the returned scopes to those the calling admin may administer
  (backend-enforced). The back-office Roles page presents scope selection/columns and
  operates within the chosen authorized scope.

## Impact

- **Backend – platform**: `backend/platform/src/token.rs` (Claims carry scoped roles),
  `identity.rs` (`AuthIdentity` exposes per-scope roles / `has_role_in_scope`),
  `guard.rs` (new scope-matched `require_admin_in_scope`), `config.rs`
  (allowed audiences / back-office audience).
- **Backend – auth**: `backend/auth/src/module.rs` `issue()` / `effective_roles` — mint a
  token carrying the caller's roles across the relevant scopes for the back-office
  audience; `session.rs` audience binding.
- **Backend – user**: `backend/user/src/module.rs` (`effective_roles`, scope/role
  validation), `grpc.rs` (`grant_role` / `revoke_role` scope-matched guard, `list_accounts`
  scope filtering), `pg.rs` + `repo.rs` (drop the hardcoded `r.scope = 'music'` join;
  return roles per authorized scope), `user-port` proto/types (`AccountRow.roles` shape).
- **Front – back office**: `apps/back-office/src/stores/auth.ts` (audience),
  `stores/roles.ts` (pass scope explicitly), `views/RolesView.vue` (scope selector /
  columns, show only authorized scopes), i18n strings pinned to "music".
- **Security**: authorization change — must be covered by tests proving cross-scope
  denial (`music/admin` cannot touch `live`) and `global/admin` break-glass across scopes.
- **Compatibility**: existing `music` / `live` **app** tokens keep working unchanged; the
  scoped-role additions are additive. Only the back-office login audience changes.
