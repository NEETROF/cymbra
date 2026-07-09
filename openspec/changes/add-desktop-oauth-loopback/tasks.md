## 1. Spike: client type vs audience (do first — gates the rest)

- [x] 1.1 Determine whether the **web** OAuth client can run the loopback flow with `http://127.0.0.1` redirect URIs + PKCE so the `id_token` `aud` stays the web client (preserves Option A) — recorded in design.md (D3 resolution): web client can, but is confidential (must ship a secret)
- [x] 1.2 If a **Desktop** client is required (different `aud`), decide: backend accepts a second Google audience (config + verifier list) vs a desktop-specific audience; record the outcome in design.md — recorded (agnostic build: optional client_secret serves either path)
- [x] 1.3 Register the loopback redirect URIs on the chosen Google Cloud client — done: a **Desktop app** client (`cymbra desktop`) auto-registers loopback; backend `CYMBRA_GOOGLE_AUDIENCE` extended to include it (comma-separated)

## 2. Pure core (host-testable)

- [x] 2.1 PKCE helpers: verifier + S256 challenge
- [x] 2.2 Authorization-URL builder (client id, redirect, scope `openid email`, `state`, `code_challenge`, `prompt=select_account` when forcing the chooser)
- [x] 2.3 `state` generation + validation; redirect-query parsing (`code` / `error`)
- [x] 2.4 Unit tests for 2.1–2.3 (challenge correctness, URL params, state match/mismatch, error redirect)

## 3. Desktop OIDC source

- [x] 3.1 Loopback adapter: ephemeral `127.0.0.1` `HttpServer`, open browser via `url_launcher`, capture redirect, success page, timeout + cancel, single-use shutdown
- [x] 3.2 Token exchange (code + verifier → tokens) and `id_token` extraction
- [x] 3.3 `DesktopOidcTokenSource implements OidcTokenSource`: `googleIdToken({forceChooser})`, `signOut()`, `googleAvailable` true on Windows/Linux when configured, Apple unavailable
- [x] 3.4 Platform selection in `oidcTokenSourceProvider`: Windows/Linux → desktop source; macOS/iOS/Android → native source (unchanged)
- [x] 3.5 Fake the browser/HttpServer/token-exchange glue; tests for success, cancel, timeout, state-mismatch — no real browser/network

## 4. Wiring + config

- [x] 4.1 Build-time config for the desktop flow (client id / loopback enablement) via `--dart-define`; keep secrets out of the repo (mirror the existing OAuth config approach)
- [x] 4.2 Confirm the entry screen + the Connected accounts screen (account-linking) now show Google on configured Windows/Linux — entry screen gates on `googleAvailable` (now true on desktop); no linking UI exists in-repo yet (account-linking not archived), it inherits the same gate when it lands
- [x] 4.3 README: document desktop Google setup (client type, redirect URIs, dart-defines)

## 5. Validation

- [x] 5.1 `flutter analyze` + `dart run custom_lint` + `dart format` clean
- [x] 5.2 `flutter test --coverage` green, coverage ≥ 80% (pure core covered; browser/HttpServer glue behind the fake/seam, excluded like other native adapters)
- [x] 5.3 Manual: Google sign-in **and** "Link Google" on Windows and Linux end-to-end against the backend — **sign-in verified end-to-end on Windows AND Linux** (Desktop client + secret + backend multi-audience). "Link Google" not applicable yet (no linking UI in-repo; account-linking not merged)
- [x] 5.4 `openspec validate add-desktop-oauth-loopback --strict` passes
