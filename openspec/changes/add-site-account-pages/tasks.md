## 1. Backend: `web` audience + web origins + JSON plan routes (D1, D2)

- [ ] 1.1 Config: `CYMBRA_WEB_ORIGINS` (csv, superset of `CYMBRA_BACK_OFFICE_ORIGINS`, defaults to it) in `cymbra_platform::config` + `.env.example`; document adding `web` to `CYMBRA_ALLOWED_AUDIENCES` and to the Google/Apple audience mapping (`CYMBRA_GOOGLE_AUDIENCE`, Apple Services ID)
- [ ] 1.2 `backend/server/src/web_plans.rs`: axum router `web_plans_router(state)` with `GET /web/plans/me`, `POST /web/plans/redeem`, `POST /web/plans/checkout`, `GET /web/plans/portal`; bearer identity extraction (same verifier seam as the SoundFont routes), `Platform::Web` snapshot serialized with the proto field names, redeem behind `ratelimit::check` (same scopes as the RPC) and refused for the `music` audience, checkout gated (plans + web channel + product + not-subscribed-elsewhere), portal for active `web` rows via `WebBillingProvider::portal_url`; `{error}` bodies with the RPC's neutral messages
- [ ] 1.3 CORS layer over the web routes restricted to `CYMBRA_WEB_ORIGINS` (+ `Authorization`), mounted in `main.rs` when the plan service is wired
- [ ] 1.4 Tests (axum test app + `cymbra-plans` mocks): 401 without bearer, `me` shape, redeem success / neutral refusal / throttle / `music` audience refused, checkout refused when disabled or subscribed elsewhere, portal only for web rows, CORS allow-list

## 2. Shared web-auth package (D3)

- [ ] 2.1 Create `packages/web-auth` (source-only TS: `package.json` `{ "name": "@cymbra/web-auth", "type": "module", "exports": "./src/index.ts" }`, `tsconfig.json`), move `useGoogleSignIn.ts`, `useAppleSignIn.ts`, `lib/web-auth.ts` (client: signInLocal / signInOidc / refresh / logout, in-memory token, `webAuthBaseUrl` from an injected base URL) + a bearer `fetchJson` helper; peer deps `vue`
- [ ] 2.2 Back office: depend on it (`portal:../../packages/web-auth`), replace the imports, delete the copies; `yarn lint`, `yarn test`, e2e green (behaviour unchanged)
- [ ] 2.3 vitest for the package (fake fetch: sign-in stores nothing in web storage, refresh-on-load path, logout clears) run from the back-office test suite or a tiny package-level vitest config

## 3. Site: Vue islands + pages (D4, D5, D6)

- [ ] 3.1 `apps/site`: add `@astrojs/vue`, `vue`, `@cymbra/web-auth` (portal), `PUBLIC_API_URL` / `PUBLIC_GOOGLE_CLIENT_ID` / `PUBLIC_APPLE_CLIENT_ID` / `PUBLIC_PADDLE_ENV` / `PUBLIC_PADDLE_CLIENT_TOKEN` env (`.env.example`), a `web-plans.ts` client (bearer fetch to `/web/plans/*`)
- [ ] 3.2 `SignInIsland.vue`: email/password + Google/Apple buttons (hidden without client id), audience `web`, refresh-on-mount, in-memory token, sign-out, localized errors (fr/en via a `lang` prop)
- [ ] 3.3 `/redeem` (fr) + `/en/redeem`: `RedeemIsland.vue` — prefill from `?code=`, sign-in gate, submit → outcome (campaign name, kind, end date, "open/refresh the app"), neutral refusals incl. rate limit
- [ ] 3.4 `/checkout` + `/checkout/done` (fr/en): `CheckoutIsland.vue` loads Paddle.js (CSP for that page only), `Paddle.Initialize({ token, environment })`, `Paddle.Checkout.open({ transactionId: _ptxn, settings: { successUrl } })`; missing `_ptxn` → localized message; done page → "refresh in the app" + link to `/account`
- [ ] 3.5 `/account` (fr/en): `AccountIsland.vue` — sign-in gate, plan / trial / betas / rights-end line, manage (portal URL for web, store pages for apple/google), optional web checkout when `can_purchase_here`, sign-out, links to the app; no account-management features
- [ ] 3.6 Nav / footer entries ("Mon compte" / "Account"), `Base.astro` CSP additions scoped to the pages that need them (accounts.google.com, appleid.apple.com, Paddle)
- [ ] 3.7 vitest (`apps/site`): redeem outcome mapping, manage-action mapping, prefill parsing, web-plans client error mapping; `yarn check` + `yarn build` green; extend `site.yml` with `yarn test`

## 4. Wiring + docs

- [ ] 4.1 `SUBSCRIPTIONS.md`: checkout page = `https://cymbra.app/checkout`, success URL `/checkout/done`, console steps (Google web client origin + Apple Services ID return URL `https://cymbra.app`), `CYMBRA_WEB_ORIGINS`
- [ ] 4.2 App: the web "manage" target stays `https://cymbra.app/account` (no app change); the app's beta copy points to the community, not to a code field — verify no `/redeem` link exists in store builds
- [ ] 4.3 `apps/site/README.md`: pages, env, islands, how the site consumes the backend
- [ ] 4.4 `openspec validate add-site-account-pages --strict`

## 5. Verification

- [ ] 5.1 Rust: fmt / clippy / tests / coverage gate green with the new routes (glue in the ignore regex, tests on the handler logic)
- [ ] 5.2 Back office: lint / test / e2e green after the extraction
- [ ] 5.3 Site: check / build / test green; a local run against a dev backend: sign in (email + Google), redeem a minted code, see it in the app after refresh, open the account page, portal for a sandbox web subscription
- [ ] 5.4 Manual: Google / Apple consoles updated for `https://cymbra.app`; a Paddle sandbox checkout from a Windows build lands on `/checkout`, completes, returns to `/checkout/done`, and the app shows premium after refresh
