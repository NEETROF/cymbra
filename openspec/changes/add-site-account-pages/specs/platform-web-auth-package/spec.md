## ADDED Requirements

### Requirement: One shared web-auth package for every browser front-end

The repository SHALL provide `packages/web-auth`, a source-only TypeScript package holding the
Google and Apple sign-in composables and the web-auth client (sign-in local/OIDC, refresh,
logout, in-memory access token, bearer fetch helper). Both `apps/back-office` and `apps/site`
SHALL depend on it; no browser front-end SHALL carry its own copy of that code. The package
MUST NOT persist tokens in web storage and MUST NOT depend on any app.

#### Scenario: Two consumers, one source

- **WHEN** the repository is searched for the Google / Apple sign-in composables
- **THEN** exactly one implementation exists, under `packages/web-auth`, imported by both apps

#### Scenario: Back office unchanged in behaviour

- **WHEN** the back office's sign-in tests run after the extraction
- **THEN** they pass unchanged
