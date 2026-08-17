## Why

`add-premium-subscription` deliberately keeps three things **off the app and on the web**:
redeeming a beta access code (Apple 3.1.1 forbids in-app code entry), the merchant-of-record
checkout page for Linux/Windows (`CYMBRA_PADDLE_CHECKOUT_PAGE`), and the management of a web
subscription (the provider portal). The backend API for all three exists (`PlanService`:
`RedeemAccessCode`, `CreateWebCheckout`, the portal seam), the app already links to
`cymbra.app/account`, and `cymbra.app` now lives in this monorepo (`apps/site`) — but the
**pages do not exist**, and a static Astro site cannot call gRPC-web without dragging the
protobuf toolchain in. Without them the community beta cannot start (no way to redeem a code)
and desktop users cannot buy.

## What Changes

- **A thin JSON web API for browser clients** on the backend: `/web/plans/me`, `/web/plans/redeem`,
  `/web/plans/checkout`, `/web/plans/portal` — bearer-authenticated with the same short-lived
  access token the existing web sign-in already hands to the browser, CORS-restricted to the
  configured web origins, each a one-line delegation to `PlanService`. No new business rule.
- **A `web` audience** for the public site's sessions (next to `music`, `live`, `back-office`):
  the site signs in through the existing `/web/auth/*` surface (email / Google / Apple, HttpOnly
  refresh cookie on `.cymbra.app`); the store-build refusal on code redemption keys on the
  `music` audience and therefore lets `web` through.
- **A shared web-auth package** `packages/web-auth` (TypeScript, no build step) extracted from the
  back office — `useGoogleSignIn`, `useAppleSignIn`, the web-auth client and its "never persist
  tokens in JS storage" discipline — consumed by both `apps/back-office` and `apps/site` instead of
  a copy.
- **Four site pages** (fr + en), each an Astro page with a Vue island:
  - `/redeem` — sign in, enter the code (prefilled from `?code=`), get the campaign name / kind /
    end date, neutral refusals, "open the app" next step;
  - `/checkout` — the Paddle hosted-checkout page (Paddle.js overlay reading `_ptxn`), no sign-in
    needed, and `/checkout/done` telling the user to refresh in the app;
  - `/account` — the current plan, trial, betas, "manage" (web portal URL fetched at request time,
    or the store's own management page for Apple/Google rows), sign out;
  - the sign-in island itself, reused by `/redeem` and `/account`.
- **Configuration**: `https://cymbra.app` in the web origins, `web` in the allowed audiences and
  the OIDC audience mapping, the site origin declared on the Google web client and the Apple
  Services ID (one-time console entries).

## Capabilities

### New Capabilities
- `music-plan-web-api`: the bearer-authenticated JSON routes for browser clients (`me`, `redeem`,
  `checkout`, `portal`), their CORS/audience/rate-limit rules, error shapes that never leak provider
  detail, and their equivalence with the gRPC surface.
- `site-web-signin`: the site's sign-in island over `/web/auth/*` (email, Google, Apple), audience
  `web`, session persistence across pages via the refresh cookie, sign-out, no token in JS storage.
- `site-code-redemption`: the `/redeem` page — prefill, sign-in gate, outcome rendering, neutral
  refusals, rate-limit handling, next-step guidance; no discount, no price.
- `site-checkout`: the `/checkout` hosted-checkout page (Paddle overlay, sandbox/production by
  config) and `/checkout/done`.
- `site-account`: the `/account` page — plan status, betas, rights-end date, channel-aware manage
  action, sign-out.
- `platform-web-auth-package`: the shared `packages/web-auth` TypeScript package (Google / Apple
  sign-in composables + web-auth client) with one owner and two consumers.

### Modified Capabilities
- `music-access-codes`: the web-only redemption surface is `cymbra.app/redeem` calling
  `music-plan-web-api` (a signed-in `web` session), the RPC remaining the seam behind it.
- `web-auth-session`: the public site origin(s) are accepted alongside the back office; the `web`
  audience is a valid sign-in audience.

## Impact

Products (consumed vs new):

- **Cymbra ID (consumed + one config change)** — `/web/auth/*` sign-in / refresh / logout, cookie
  domain `.cymbra.app` (already the case for the back office); the `web` audience is added to
  `CYMBRA_ALLOWED_AUDIENCES` and to the Google/Apple audience mapping (`CYMBRA_GOOGLE_AUDIENCE`
  already accepts a comma list). No new identity concept.
- **Cymbra Music (new, thin)** — `backend/server/src/web_plans.rs`: axum JSON routes over
  `PlanService` (`snapshot`, `redeem`, `create_checkout`, `portal_url`), same rate limit as the RPC
  on redeem, mounted with the existing web CORS layer; `Platform::Web` for the snapshot.
- **Site (new)** — `apps/site`: Vue integration (`@astrojs/vue`), islands `SignIn`, `Redeem`,
  `Account`, `Checkout`, a tiny `web-plans` client (fetch + bearer), i18n fr/en, `PUBLIC_*` env
  (`PUBLIC_API_URL`, `PUBLIC_GOOGLE_CLIENT_ID`, `PUBLIC_APPLE_CLIENT_ID`, `PUBLIC_PADDLE_ENV`,
  `PUBLIC_PADDLE_CLIENT_TOKEN`), vitest for the island state logic, CI gate extended.
- **Back office (refactor only)** — imports the sign-in composables and web-auth client from
  `packages/web-auth`; behaviour unchanged, its tests keep passing.
- **Platform** — `CYMBRA_WEB_ORIGINS` (superset of the back-office origins) used by the CORS layers
  of the web routes; docs in `backend/.env.example`.
- **Legal / docs** — `apps/site/README.md`, `apps/music/store/SUBSCRIPTIONS.md` (checkout page and
  return URL now concrete). The divergence between the site's legal pages and `docs/legal/` is
  **out of scope** (own change).
- **Deploy** — the site is redeployed by `site-deploy.yml`; the backend by its release; the Google /
  Apple console entries are manual, one-time.
