# web-auth-session Specification

## Purpose
TBD - created by archiving change add-web-auth-cookies. Update Purpose after archive.
## Requirements
### Requirement: Web sign-in sets an HttpOnly refresh cookie

The web-auth surface SHALL provide a sign-in endpoint that authenticates a browser
client (local email/password or an OIDC id_token) via the auth module and, on
success, sets the refresh token in a cookie with the attributes `HttpOnly`, `Secure`,
`SameSite=Strict`, a path scoped to the auth endpoints, and a `Max-Age` equal to the
refresh-token lifetime. The access token SHALL be returned in the response body and
MUST NOT be placed in any cookie. The refresh token MUST NOT appear in the response
body.

#### Scenario: Successful web sign-in

- **WHEN** a browser posts valid credentials (or a valid OIDC id_token) to the web sign-in endpoint
- **THEN** the response sets an `HttpOnly; Secure; SameSite=Strict` refresh cookie and returns the access token in the body, and the refresh token is absent from the body

#### Scenario: Failed web sign-in sets no cookie

- **WHEN** the credentials are invalid
- **THEN** the endpoint returns an unauthenticated error, sets no refresh cookie, and returns no token

### Requirement: Web refresh uses the cookie, never JavaScript

The web-auth surface SHALL provide a refresh endpoint that reads the refresh token
from the request cookie (not from the body), rotates it via the auth module, sets the
rotated refresh token in a new cookie with the same attributes, and returns a new
access token in the body. The client JavaScript MUST NOT be able to read the refresh
token at any point.

#### Scenario: Refresh from the cookie

- **WHEN** a browser calls the refresh endpoint with a valid refresh cookie
- **THEN** the endpoint returns a new access token and re-sets a rotated refresh cookie

#### Scenario: Missing or invalid refresh cookie

- **WHEN** the refresh cookie is absent, expired, or already rotated (reuse)
- **THEN** the endpoint returns unauthenticated, clears the cookie, and the session is treated as ended

### Requirement: Web sign-out clears the cookie and revokes the session

The web-auth surface SHALL provide a sign-out endpoint that revokes the session via
the auth module and clears the refresh cookie (expired `Set-Cookie`).

#### Scenario: Sign-out

- **WHEN** a browser calls the sign-out endpoint
- **THEN** the session is revoked and the response clears the refresh cookie, so a subsequent refresh fails

### Requirement: The web client never persists tokens in JavaScript storage

The back office SHALL hold the access token in memory only and MUST NOT write any
token to `localStorage`, `sessionStorage`, or a non-`HttpOnly` cookie. On load it
SHALL obtain an access token by calling the refresh endpoint (credentialed); on a
gRPC `UNAUTHENTICATED` it SHALL refresh once via the cookie endpoint and retry.

#### Scenario: Access token is memory-only

- **WHEN** the back office signs in and then the page is reloaded
- **THEN** no token is present in `localStorage`/`sessionStorage`, and the app silently re-mints an access token from the refresh cookie

#### Scenario: Silent refresh on expiry

- **WHEN** a gRPC call returns `UNAUTHENTICATED` because the access token expired
- **THEN** the app calls the cookie refresh endpoint once and retries the call, without signing the user out (sign-out only if the refresh itself fails)

### Requirement: Cookie endpoints are CSRF-protected and credential-scoped

The web-auth cookie endpoints SHALL be protected against CSRF: the refresh cookie is
`SameSite=Strict`, the endpoints require a non-simple request (JSON content-type /
custom header forcing a CORS preflight), and CORS SHALL echo an exact allowed origin
with `Access-Control-Allow-Credentials: true` (never a wildcard origin with
credentials).

#### Scenario: Cross-site request cannot drive the cookie endpoints

- **WHEN** a page on another site attempts a cross-site request to the refresh or sign-out endpoint
- **THEN** the browser does not attach the `SameSite=Strict` cookie and/or the request is blocked by the preflight, so no session action occurs

#### Scenario: Credentialed CORS is origin-exact

- **WHEN** the web-auth surface answers a credentialed cross-origin request from an allowed back-office origin
- **THEN** it echoes that exact origin and `Access-Control-Allow-Credentials: true`, and never `Access-Control-Allow-Origin: *`

### Requirement: Native clients keep the bearer token flow

The gRPC `AuthService` `TokenPair` flow (SignInLocal/SignInOidc/Refresh/Logout) SHALL
remain available and unchanged for native (mobile/desktop) clients, which store tokens
in platform secure storage. The cookie surface is additive and web-only.

#### Scenario: Native sign-in is unchanged

- **WHEN** a native client signs in via the gRPC `AuthService`
- **THEN** it receives a `TokenPair` in the response as before, with no cookie involved

