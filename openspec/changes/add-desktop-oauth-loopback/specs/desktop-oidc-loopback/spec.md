## ADDED Requirements

### Requirement: Desktop Google sign-in via browser loopback

The app SHALL obtain a Google `id_token` on desktop platforms where the native Google
SDK is unavailable (Windows/Linux) via the OAuth 2.0 authorization-code flow with PKCE
and a loopback redirect: it MUST open the system browser, capture the redirect on a
local `127.0.0.1` port, exchange the code for tokens, and return the `id_token` to the
same `SignInOidc` / `LinkIdentity` path used elsewhere.

#### Scenario: Successful desktop Google sign-in

- **WHEN** a user on Windows/Linux chooses "Continue with Google"
- **THEN** the system browser opens, the user authenticates, the app captures the loopback redirect, exchanges the code, and signs in with the returned `id_token`

#### Scenario: User cancels in the browser

- **WHEN** the user closes the browser or denies consent
- **THEN** the local server stops, the port is freed, and the flow is a no-op (no error, treated like a cancelled native sheet)

### Requirement: Desktop flow preserves the single audience

The desktop loopback flow SHALL produce an `id_token` whose audience matches the
backend's configured Google audience (the web client, Option A), so that `SignInOidc`
accepts it without a backend change. If the chosen OAuth client type makes this
impossible, the audience handling MUST be reconciled before release (per design D3).

#### Scenario: Backend accepts the desktop token unchanged

- **WHEN** a desktop `id_token` from the loopback flow is sent to `SignInOidc`
- **THEN** the backend accepts it under its existing Google audience configuration

### Requirement: Desktop availability gating

When the desktop loopback flow is configured, `googleAvailable` SHALL be true on
Windows/Linux, so the Google entry button and the "Link Google" action are offered
there. When it is not configured, Google SHALL remain hidden on those platforms.
Apple SHALL remain unavailable on desktop (out of scope).

#### Scenario: Configured desktop shows Google

- **WHEN** the loopback flow is configured and the app runs on Windows/Linux
- **THEN** the Google entry button and the "Link Google" action are available

#### Scenario: Unconfigured desktop hides Google

- **WHEN** the loopback flow is not configured on Windows/Linux
- **THEN** Google stays hidden (only email/guest), as today

### Requirement: Desktop flow security

The flow SHALL use PKCE (S256) and a `state` parameter validated on return, listen
only on `127.0.0.1` (never `0.0.0.0`), use a single-use server shut down after the
redirect, enforce a timeout, and MUST NOT log the authorization code, PKCE verifier,
or tokens.

#### Scenario: Mismatched state is rejected

- **WHEN** the redirect's `state` does not match the value the app generated
- **THEN** the flow fails without exchanging the code

#### Scenario: Timeout frees resources

- **WHEN** no redirect arrives within the timeout
- **THEN** the local server is stopped, the port freed, and the attempt ends as a no-op
