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
