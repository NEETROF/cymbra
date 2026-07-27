## Context

Role administration (grant/revoke/list-history) already exists in the user service
and back-office, keyed by account `id` (UUID). Admins only know handles/emails, and
there is no way to enumerate accounts, so the page is unusable. The data is present:
`users(id, display_name, handle, handle_key)`, `user_identities(provider, subject)`
(a `local` identity's `subject` is the email), and `user_roles(user_id, scope,
role)`. This change adds a read/list path plus a UI rework — no new storage.

## Goals / Non-Goals

**Goals:**
- Paginated `ListAccounts` returning `{ id, handle, display_name, roles[] }` (roles
  in the `music` scope) + `total`, admin-only.
- Optional filter matching handle (case-insensitive, `handle_key`) or local email.
- Back-office directory: list, paginate, filter, and grant/revoke per row.

**Non-Goals:**
- Fuzzy/full-text search or ranking — a prefix/exact filter is enough.
- Finding OIDC-only accounts (google/apple) by email — their `subject` is the OIDC
  sub, not an email; they still appear in the list and are filterable by handle.
- Managing scopes other than `music`, or editing account profile fields.
- Any schema change or new migration.

## Decisions

**1. One paginated `ListAccounts` RPC that also carries roles.**
`ListAccounts(limit, offset, query) -> { accounts: [{id, handle, display_name,
roles[]}], total }`. Bundling each account's music-scope roles avoids an N+1 of
per-row role fetches and lets the table render badges and decide toggles directly.
Alternative considered: a lookup-one endpoint + separate role reads — rejected; the
directory needs the whole page at once and admins want to see who has what.

**2. Query is an optional filter, not a separate lookup.**
Empty query → all accounts (ordered `handle` ascending, nulls last, then
`created_at`). Non-empty → `handle_key` matches `normalize(query)` (prefix) OR a
`local` identity's `lower(subject) = lower(query)`. This folds the earlier
"find by handle/email" need into the list. Handle disallows `@`, so an email query
naturally targets the email branch.

**3. Pagination mirrors the catalog search (limit/offset + total).**
Same shape the console + stores already use for the catalog, so the store's
`Async<{ accounts, total }>` and the table's pager are consistent with existing
code. Default page size 25.

**4. Per-row actions reuse GrantRole/RevokeRole (scope `music`).**
No new mutation. After a grant/revoke succeeds, the store re-lists the current page
so the row's role badges reflect the change (listener/refresh, not optimistic
mutation). Selecting a row still opens its `ListRoleGrants` audit history.

**5. `require_admin` guard; minimal projection.**
Same guard as the mutations it feeds. The response exposes only id/handle/
display_name/roles — no credentials, identities, or emails (email is a filter input,
not returned), limiting the linkage an admin can harvest to what they already
manage.

**6. Frontend follows `vue-frontend-architecture`.**
`stores/roles.ts` gains `list(query, limit, offset)` returning `Async<{accounts,
total}>`; `RolesView.vue` renders a filter box + paginated table with per-row
grant/revoke; all API calls stay in the store; not-found/empty/error are folded
into the union and shown as localized messages (no raw gRPC codes). The manual UUID
field is removed.

## Risks / Trade-offs

- **Large user base later** → limit/offset paginates; the default order uses the
  `users_handle_key_uniq` index and filters hit unique indexes. Fine at this scale;
  revisit keyset pagination if the table grows large.
- **Handle-less (onboarding-incomplete) accounts** → included with a blank handle
  (shown by display_name/id) and ordered last; they can still receive roles.
- **Email filter misses OIDC-only accounts** → acceptable; those are found by
  handle, and every account still appears unfiltered in the list.
- **Linkage disclosure (who has admin/moderator)** → bounded by `require_admin`;
  admins already perform these grants.
