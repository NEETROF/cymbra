## 1. Backend — web-auth HTTP surface

- [ ] 1.1 Add a web-auth route group on the existing HTTP server (`/web/auth/signin`, `/web/auth/refresh`, `/web/auth/logout`) that wraps the auth module; JSON in/out. No change to the gRPC `AuthService`.
- [ ] 1.2 Sign-in handler: authenticate (local + OIDC) via the auth module, set the refresh token as `HttpOnly; Secure; SameSite=Strict; Path=/web/auth; Max-Age=<refresh TTL>` (Domain per-env), return `{ accessToken }` in the body (never the refresh token).
- [ ] 1.3 Refresh handler: read the refresh token from the cookie (not the body), rotate via the auth module, re-set the rotated cookie, return `{ accessToken }`; on missing/invalid/reused cookie → 401 + clear cookie.
- [ ] 1.4 Logout handler: revoke the session via the auth module and clear the cookie (expired `Set-Cookie`).

## 2. Backend — CORS, CSRF, config

- [ ] 2.1 CORS for the web-auth surface: `Access-Control-Allow-Credentials: true` + exact origin echo from the existing back-office origins allow-list (never `*` with credentials).
- [ ] 2.2 CSRF: require a JSON content-type / custom header (preflight-forcing) on the cookie endpoints; optionally a double-submit CSRF token for refresh/logout. Document the threat model.
- [ ] 2.3 Cookie config per environment (Domain, Secure on/off for localhost) via config, reusing the refresh TTL from auth config.

## 3. Backend — tests

- [ ] 3.1 Host-testable handler tests: sign-in sets the cookie + returns access token (no refresh in body); refresh reads cookie → rotates → re-sets; logout clears + revokes; invalid/reused cookie → 401 + clear.
- [ ] 3.2 CORS/CSRF tests: credentialed response echoes exact origin (never `*`); a request without the required non-simple header is rejected.

## 4. Back office — memory-only session

- [ ] 4.1 `stores/auth.ts`: hold the access token in memory only; remove all `localStorage` token reads/writes. Expose `isAuthenticated` from the in-memory token (or a bootstrapped flag).
- [ ] 4.2 Sign-in/sign-out views + store call the web-auth endpoints (credentialed `fetch`) instead of the gRPC `SignIn*`/`Logout`; store the returned access token in memory.
- [ ] 4.3 `lib/transport.ts`: the token refresher calls the cookie refresh endpoint (credentialed) instead of the JS refresh RPC; the gRPC transport keeps sending the in-memory access token as `Authorization: Bearer`. On load, mint the access token from the cookie.
- [ ] 4.4 Handle the "no session" boot path: a failed initial refresh routes to sign-in (no token, no console error).

## 5. Back office — tests

- [ ] 5.1 Unit-test the auth store: sign-in stores the access token in memory and nothing in `localStorage`; sign-out clears it.
- [ ] 5.2 Update the refresh interceptor tests to the cookie refresher seam; keep retry/single-flight/no-recursion coverage.
- [ ] 5.3 e2e: assert `localStorage`/`sessionStorage` hold no token after sign-in; reload silently re-mints from the (fake) cookie seam; expiry → refresh-and-retry, not sign-out.

## 6. Docs & checks

- [ ] 6.1 Document the same-site deployment constraint (`api.` + `bo.` under one registrable domain) and the reverse-proxy header-form CSP (`frame-ancestors`) + HSTS.
- [ ] 6.2 Run the gates: backend `cargo test`/`clippy`/`llvm-cov`; back-office `yarn lint && yarn format:check && yarn typecheck && yarn test:coverage && yarn e2e`.
- [ ] 6.3 `openspec validate add-web-auth-cookies --strict` passes.
