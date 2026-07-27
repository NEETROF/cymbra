## Why

Role administration today is keyed by an account's raw `id` (UUID), which an admin
never sees — they only know a user's handle (pseudo) or email. There is no way to
browse the accounts or to act on one without already knowing its id, so the
back-office Roles page is effectively unusable.

## What Changes

- Add an **admin-guarded, paginated `ListAccounts` RPC** to the user service that
  returns accounts with their music-scope roles (`{ id, handle, display_name,
  roles[] }`) plus a total count, and accepts an optional **query** that filters by
  handle (case-insensitive, via `handle_key`) or by identity email.
- Rework the back-office **Roles page** into a **paginated directory**: every
  account is listed with its current roles, and an admin **grants/revokes moderator
  or admin per row** (reusing the existing GrantRole/RevokeRole). A search box
  filters the list by handle/email. Selecting a row still shows its audit history.
- The manual "paste a UUID" field is **removed** — the directory replaces it.
- Regenerate the gRPC-web (back-office) and Flutter stubs for the new RPC.

## Capabilities

### New Capabilities
- `admin-account-directory`: an admin-only capability to browse accounts (paginated,
  with their roles), filter them by handle or email, and grant/revoke roles per
  account — the back-office surface for role administration.

### Modified Capabilities
<!-- None. GrantRole/RevokeRole/ListRoleGrants behaviour is unchanged; this change
     adds a listing that feeds them and reworks the console page. -->

## Impact

- **Backend**: `backend/user-port/proto/user.proto` (new `ListAccounts` RPC +
  paginated request/response with per-account roles); `backend/user-port`
  (generated types); user module `repo.rs` (trait method + row type), `pg.rs`
  (paginated query joining `user_roles` at `scope='music'`, optional handle/email
  filter, total count), `grpc.rs` (handler behind `require_admin`).
- **Guarding**: reuses `require_admin` (same guard as GrantRole/RevokeRole).
- **Codegen**: Flutter + back-office gRPC stubs regenerated.
- **Frontend (back-office)**: `RolesView.vue` becomes a paginated table with a
  filter box and per-row actions; `stores/roles.ts` gains a paginated `list` action
  (Async<T> union, store-only API call per the `vue-frontend-architecture` skill);
  i18n strings; unit + e2e coverage.
- No database migration (reads existing `users`, `user_identities`, `user_roles`).
