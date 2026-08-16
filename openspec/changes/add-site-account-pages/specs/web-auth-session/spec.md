## ADDED Requirements

### Requirement: The public site is a first-party web client with its own audience

The web sign-in surface SHALL accept the public site's origin(s) alongside the back office
(`CYMBRA_WEB_ORIGINS`), and SHALL accept `web` as a sign-in audience — a plain-user audience
that carries no console semantics: no scoped admin roles are derived from it, and store-build
checks that key on the `music` audience are unaffected. The refresh cookie SHALL be first-party
for the site (same registrable domain as the web-auth surface).

#### Scenario: Site sign-in

- **WHEN** the site signs a user in with audience `web`
- **THEN** a session is created, the refresh cookie is set on the shared domain, and the access token identifies the `web` audience

#### Scenario: `web` grants no console power

- **WHEN** a `web`-audience token calls a console-only method
- **THEN** it is refused exactly like a plain `music` user

### Requirement: Browser clients read their own account summary through a bearer JSON route

The backend SHALL expose `GET /web/account/me`, bearer-authenticated with the same
short-lived access token as the plan routes and served under the same web-origins CORS
policy, answering the caller's own account summary: `handle`, `display_name`, `locale`,
and the linked identities as `{ provider, email, linked_at }` where `email` is set only
for the `local` provider (the OIDC subjects of Google / Apple MUST NOT be exposed). A
missing or invalid bearer SHALL be refused with `401` before any read. The route is
read-only: no account mutation is offered on the web.

#### Scenario: Summary for a signed-in web session

- **WHEN** a signed-in web session calls `GET /web/account/me`
- **THEN** the JSON carries the handle and the identity providers, with the e-mail for the local identity only

#### Scenario: Missing bearer

- **WHEN** the route is called without a valid bearer
- **THEN** the answer is `401` and no account data is read
