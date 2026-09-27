## Context

The Users directory (`/admin/users`) is served by `UserService.ListAccounts`
(`backend/user`, schema `user_account`, role `user_svc`). Plan and beta filters are
resolved by the plans service into an `ids` set first, because the identity service must
not learn product criteria. Tokens are minted by the auth module (`backend/auth`,
`AuthModule::issue` at sign-in, `AuthModule::refresh` on rotation); both build the claims
through `access_claims`, which already calls the `UserPort` to resolve roles. Audiences
are `music`, `live`, `lingua` (the app scopes, `cymbra_platform::APP_SCOPES`), plus
`back-office` and `web`.

## Goals / Non-Goals

**Goals:** a durable "has used app X" fact per account; shown on the directory row;
filterable with exact pagination.

**Non-Goals:** activity analytics (counts, retention — `add-feature-usage-analytics` owns
that, keyed by a salted bucket, never by account); per-device or per-platform detail;
backfilling accounts whose sessions already expired.

## Decisions

### D1. The fact is "signed in to the app", owned by Cymbra ID

"Uses Music" could mean "has Music data" (plays, purchases) or "has signed in to Music".
Product data would need one port per product and leaves out an account that signed in
but has not played yet. Sign-in is the one signal every app shares, is already
audience-scoped, and belongs to the identity domain the directory already lives in.

It is stored in `user_account.account_apps`, not in `auth`: the directory query runs
under `user_svc`, whose `search_path` is `user_account` only, so a table in `auth` would
be unreadable there (and reading it would break module isolation). The auth module writes
through the `UserPort`, which it already holds.

```sql
CREATE TABLE account_apps (
    user_id       UUID        NOT NULL REFERENCES user_account.users (id) ON DELETE CASCADE,
    app           TEXT        NOT NULL CHECK (app IN ('music', 'live', 'lingua')),
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, app)
);
CREATE INDEX account_apps_app_idx ON account_apps (app, user_id);
```

`ON DELETE CASCADE` makes account deletion and the orphan reaper erase it with the
account, like `user_identities` and `user_roles`. The `CHECK` repeats the app-scope
vocabulary; widening it is a migration, like `user_roles_scope_vocabulary`.

### D2. Written on every token mint, throttled, best effort

`access_claims` runs for both sign-in and refresh, so recording there covers every
client, including one that stays signed in for months without signing in again. A
non-app audience (`back-office`, `web`) is ignored by the user module, not by the caller,
so the rule lives in one place.

```sql
INSERT INTO account_apps (user_id, app) VALUES ($1, $2)
ON CONFLICT (user_id, app) DO UPDATE SET last_seen_at = now()
WHERE account_apps.last_seen_at < now() - interval '1 hour'
```

The `WHERE` makes a refresh inside the hour a no-op write, so the cost is bounded by
accounts × apps × hours of use, not by refresh frequency.

A failed write is logged and swallowed: a missing icon is not worth refusing a sign-in.

### D3. Filter in the same query, AND semantics

`ListAccounts` takes `apps: repeated string`; an account matches when it has a row for
**every** listed app. It joins the existing `WHERE` (handle/email, `ids`, `exclude_ids`)
so `total` and `OFFSET` stay exact — pre-resolving it into an `ids` set, as the plan
filter does, would only be needed if the fact lived in another domain. The UI offers
any / Music / Lingua / both; "both" is `["music", "lingua"]`. Unknown app values are
`INVALID_ARGUMENT`.

The page's apps come back on each row (`AccountRow.apps`: app + last use),
fetched for the page ids in one extra query, like the roles.

### D4. Shown to every directory admin

The apps are not roles, so they are not restricted to the caller's administered scopes: a
`music/admin` sees that an account also uses Lingua. The directory already exposes the
account to that admin; which apps it uses is not more sensitive than its handle.

### D5. Icons, not tags

The row shows one small icon per app (the Music note and the Lingua book already used by
the sidebar), each with an accessible label and a tooltip giving the last use, and
`—` when the account used no app. Live has no icon until Live has users; a `live` row is
still returned by the API.

## Risks / Trade-offs

- **No backfill** → accounts with a live session but no refresh since the deploy show no
  icon until their app opens. Accepted: it converges on first use, and a live-session
  backfill would need a cross-schema read in a migration (`user_svc` cannot read `auth`).
- **One more write on the sign-in path** → bounded by D2's throttle and never blocking.
- **Vocabulary drift** between `APP_SCOPES` and the `CHECK` → a unit test asserts the
  migration's list equals `APP_SCOPES`, like the role-vocabulary drift test.
