## ADDED Requirements

### Requirement: Back-office session carries the admin's roles across every scope

The back office SHALL authenticate against a dedicated back-office audience whose access
token reflects the administrator's **actual** roles in every scope they hold — the union
of `global`, `music`, and `live` — rather than a single app scope. The token MUST carry
each role together with the scope it belongs to, so downstream authorization can tell in
which scope the caller is an `admin`. An account that holds no role in a scope
contributes nothing for that scope. This audience is for the administration console only;
the `music` and `live` **app** audiences are unchanged and continue to mint single-scope
tokens.

#### Scenario: Multi-scope admin gets all their scopes in one token

- **WHEN** an account holding `music/admin` and `live/moderator` signs in to the back-office audience
- **THEN** its access token carries `admin` in the `music` scope and `moderator` in the `live` scope

#### Scenario: Global admin's global role is present for every scope

- **WHEN** an account holding `global/admin` signs in to the back-office audience
- **THEN** its access token conveys `admin` authority applicable to both the `music` and `live` scopes

#### Scenario: Scope with no role contributes nothing

- **WHEN** an account holding only `music/admin` signs in to the back-office audience
- **THEN** its token carries the `music` admin role and conveys no `admin`/`moderator` authority in the `live` scope

#### Scenario: App audiences remain single-scope

- **WHEN** the same account signs in to the `music` app audience
- **THEN** the token carries only its `music`-and-`global` effective roles, exactly as before this change

### Requirement: Back-office session uses the existing web-auth cookie flow

The back-office sign-in, refresh, and sign-out for the back-office audience SHALL reuse
the existing web-auth surface (HttpOnly refresh cookie, in-memory access token, refresh
on `UNAUTHENTICATED`). Refreshing a back-office session MUST preserve the back-office
audience and re-resolve the caller's current per-scope roles, so a role change takes
effect on the next refresh.

#### Scenario: Refresh preserves the back-office audience

- **WHEN** a back-office session is refreshed via the cookie endpoint
- **THEN** the new access token is again a back-office multi-scope token, with roles re-resolved from the account's current per-scope roles

#### Scenario: Access token stays out of JavaScript storage

- **WHEN** the back office holds a back-office session
- **THEN** the access token is kept in memory only and the refresh token remains in an HttpOnly cookie, unchanged from the current web-auth behavior
