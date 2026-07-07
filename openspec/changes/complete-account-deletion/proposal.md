## Why

`DeleteAccount` today only deletes the `user_account` schema row (cascading to
`user_identities` / `user_roles`). It leaves the **auth** module's data behind:
`auth.local_credentials` (email PK + argon2 password hash + verification/reset
tokens) and `auth.sessions` (refresh tokens). That is a **GDPR erasure gap**
(email + password hash survive a "deletion"), a **functional bug** (email is the
`local_credentials` primary key, so re-signing-up with the same address fails with
"already registered"), and a **security gap** (the deleted user's refresh tokens
stay valid until they expire). The app already ships the in-app deletion UI
(`delete_account_screen.dart` → `DeleteAccount`), which the App Store and Play
Store require — but the backend must actually erase everything.

## What Changes

- Account deletion becomes a **complete, cross-module erasure**: on `DeleteAccount`
  the backend removes the user's `user_account` data **and** `auth.local_credentials`
  (matched by email, resolved from `user_id` via the `local` identity) **and**
  `auth.sessions` (by `user_id`).
- The erasure is **atomic** (all-or-nothing) and **idempotent** (re-running for an
  already-deleted user is a no-op success), so a retry after a partial failure
  converges rather than stranding data.
- After deletion, **re-signing-up with the same email succeeds**, the user's
  refresh tokens are **rejected**, and no OIDC identity remains.
- Respect per-module schema isolation: cross-schema writes go through the ops path
  designed for it (`admin_svc` via the worker), not by widening a module role.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `account-management`: the **Account deletion** requirement gains backend
  completeness — deletion erases *all* personal data across modules, and a
  previously-deleted email can register again.
- `backend-auth`: new requirement — when an account is deleted, the auth module's
  `local_credentials` and `sessions` for that user MUST be erased (auth owns that
  data and must not retain it after erasure).

## Impact

- **Backend**: `backend/user` (DeleteAccount orchestration / enqueue), `backend/auth`
  (a delete-credentials-by-email + delete-sessions-by-user path; the latter query
  already exists in `session_pg` but is uncalled), `backend/jobs` + `backend/worker`
  (a `purge_user` job run as `admin_svc` if the job approach is chosen — decided in
  design.md).
- **Data**: destructive deletes across the `user_account` and `auth` schemas.
- **APIs**: no gRPC signature change — `DeleteAccount` keeps its contract, its
  effect becomes complete.
- **Tests**: coverage that after deletion the same email can re-register, sessions
  are gone/rejected, and credentials/identities are removed (Rust ≥80%).
- **No change** to the Flutter client — the UI + `DeleteAccount` call already exist.
