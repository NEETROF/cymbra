## Context

- The backend already exposes everything the site needs, over gRPC: `PlanService`
  (`GetMyPlan`, `RedeemAccessCode` — refused for the `music` audience —, `CreateWebCheckout`)
  and the `WebBillingProvider::portal_url` seam (Paddle). The web sign-in surface
  (`/web/auth/signin|refresh|logout`, `backend/server/src/web_auth.rs`) hands the browser a
  short-lived **access token** and an HttpOnly refresh cookie on the configured cookie domain;
  the back office calls gRPC-web with that bearer.
- The back office owns the only browser sign-in code: `useGoogleSignIn.ts`,
  `useAppleSignIn.ts`, `lib/web-auth.ts` (never persist tokens in JS storage — spec
  `web-auth-session`).
- `apps/site` is a static Astro site (fr default, en), Yarn Berry, deployed on Cloudflare
  Pages; no framework islands yet, no protobuf toolchain, and it must stay that way (a static
  site with a couple of interactive islands).
- The app already sends web subscribers to `https://cymbra.app/account` for "manage", and the
  Paddle checkout page is configured by `CYMBRA_PADDLE_CHECKOUT_PAGE`.

## Goals / Non-Goals

**Goals:**

- A beta tester can redeem a code on `cymbra.app/redeem` in one sitting (sign in → code → done)
  and see it in the app at the next refresh; a desktop user can buy on the web and manage there.
- Zero new authentication code: one shared package, two consumers.
- The site stays static; the interactive parts are small islands talking JSON to the backend.
- Store builds remain untouched (no code entry, no external link) — everything here is web.

**Non-Goals:**

- Unifying the site's legal pages with `docs/legal/` (own change).
- A full account-management site (email change, deletion, linked identities): the app owns
  those; `/account` shows the plan and offers manage/sign-out only.
- Server-side rendering / auth-gated routing in Astro (the islands gate themselves).
- Discount / promo handling on the web (Paddle offers, later).

## Decisions

### D1 — JSON routes for the browser, bearer-authenticated, next to `/web/auth/*`

`/web/plans/me` (GET), `/web/plans/redeem` (POST `{code}`), `/web/plans/checkout` (POST
`{productId}`), `/web/plans/portal` (GET) — axum handlers in `backend/server/src/web_plans.rs`
that verify the `Authorization: Bearer <access token>` header with the same verifier the
SoundFont HTTP routes use (`SoundfontAuth`-style identity extraction), then delegate to
`PlanService` (`snapshot` with `Platform::Web`, `redeem` behind the same `ratelimit::check`
scopes as the RPC, `create_checkout` guarded by `plans.enabled` + `billing.web.enabled` +
"not already subscribed elsewhere", `portal_url` for an active `web` row). Responses are the
same shapes as the proto (`GetMyPlanResponse` → JSON with the same field names), errors are
`{error}` with the RPC's neutral messages. Mounted with the web CORS layer restricted to
`CYMBRA_WEB_ORIGINS`.

*Alternative considered*: gRPC-web from the site (as the back office does). Correct but drags
`protoc` + Connect-ES generation into an otherwise static Astro build and Cloudflare's build
image; the four calls do not justify it. *Alternative*: cookie-authenticated routes with CSRF.
The refresh cookie is deliberately **not** an access credential (spec `web-auth-session`); the
bearer discipline is kept.

### D2 — A `web` audience for the site

Site sessions sign in with `audience: "web"`. It is added to `CYMBRA_ALLOWED_AUDIENCES` (the
strict/optional interceptors accept it), to the OIDC audience mapping (Google web client id and
Apple Services ID — the same web clients the back office uses, with `https://cymbra.app` added
as an authorized origin / return URL in the consoles), and it is exactly what makes
`RedeemAccessCode` accept the call (it refuses `music` only). Effective roles for `web` are the
`global` ones (nobody needs a role to redeem a code or read their plan).

*Alternative considered*: reusing the `back-office` audience. It carries admin semantics
(console scopes) and would let a plain user's site session pass audience checks written for
the console. Rejected.

### D3 — `packages/web-auth`: one shared TypeScript package, no build step

`useGoogleSignIn`, `useAppleSignIn` (Vue composables over Google Identity Services / Sign in
with Apple JS, unchanged), the web-auth client (`signInLocal`, `signInOidc`, `refresh`,
`logout`, in-memory access token) and the tiny bearer `fetchJson` helper move to
`packages/web-auth/src`. Both apps depend on it with a Yarn `portal:` (workspace-less monorepo:
each app keeps its own `yarn.lock`, the package is source-only so Vite compiles it in place).
The back office's tests keep running against the moved code (import paths change, behaviour
does not).

*Alternative considered*: copy the three files into the site. Two copies of a security-relevant
piece; rejected — the point of bringing the site into the monorepo.

### D4 — Astro + Vue islands, `PUBLIC_*` config

`@astrojs/vue` integration; islands: `SignInIsland` (email / Google / Apple, emits the session),
`RedeemIsland` (wraps SignIn, prefills `?code=`, calls `/web/plans/redeem`, renders the
outcome), `AccountIsland` (wraps SignIn, `GET /web/plans/me`, manage: `GET /web/plans/portal`
for `web` rows, store management URLs for `apple`/`google` rows, sign-out), `CheckoutIsland`
(loads Paddle.js, `Paddle.Initialize({token, environment})`, `Paddle.Checkout.open({transactionId
: _ptxn, settings: {successUrl: /checkout/done}})`). Config through Astro `PUBLIC_*` env
(`PUBLIC_API_URL`, `PUBLIC_GOOGLE_CLIENT_ID`, `PUBLIC_APPLE_CLIENT_ID`, `PUBLIC_PADDLE_ENV`,
`PUBLIC_PADDLE_CLIENT_TOKEN`); a missing client id hides that button (like the back office).
Copy is fr/en through the page's `lang` (the site's existing convention), no i18n library.

### D5 — Session across pages: refresh-on-load

Each island calls `/web/auth/refresh` on mount (cookie present ⇒ a fresh access token, no UI;
absent ⇒ the sign-in form). No token ever touches `localStorage`/`sessionStorage`. Sign-out
calls `/web/auth/logout` and clears the in-memory token.

### D6 — Store-side "manage" from the web

An `apple` / `google` subscriber who lands on `/account` sees "managed on the App Store /
Google Play" with the store's own management link — the site never tries to cancel a store
subscription (impossible and out of policy).

### D7 — Testing

Backend: unit tests for `web_plans.rs` with an axum test app over a `PlanService` built on
mocks (the crate's `mock` feature) — auth missing → 401, `music` audience redeem → 403-class
error, redeem success shape, checkout refused when disabled / subscribed elsewhere, portal for
web rows only, CORS origin. Site: vitest on the islands' pure state (redeem outcome mapping,
manage-action mapping, prefill parsing) and on the shared web-auth client (fake fetch); the
back office's existing tests cover the moved composables. Playwright for the site is deferred
(no fake seam yet); the CI gate is `astro check` + build + vitest.

## Risks / Trade-offs

- **CORS / cookie misconfiguration breaks sign-in silently** → one env (`CYMBRA_WEB_ORIGINS`)
  documented in `.env.example`; the refresh cookie domain is already `.cymbra.app`; a
  same-site check in the island surfaces a clear "site not authorized" message.
- **Google / Apple console entries forgotten** → the buttons hide when the client id is unset,
  email sign-in always works, and the store checklist lists the console steps.
- **Paddle.js from a CDN on the checkout page** → allowed by that page's CSP only; the page
  carries no session and no personal data (the transaction was bound server-side).
- **`web` audience widening** → it grants nothing: no roles, and every plan-aware RPC still
  decides per user; store builds are unaffected (they never present a `web` token).
- **Two `yarn.lock`s + a portal package** → the package is dependency-free TypeScript; if that
  ever changes, promote to a Yarn workspace root.

## Migration Plan

1. Backend: `web` audience + web origins config, `web_plans.rs` routes (dark: they only work for
   `web` tokens, which nobody has yet).
2. `packages/web-auth` extraction; back office switched to it (behaviour-neutral, tests green).
3. Site: Vue integration, islands, pages, env; deploy behind the existing site deploy.
4. Consoles: add `https://cymbra.app` to the Google web client and the Apple Services ID.
5. Point `CYMBRA_PADDLE_CHECKOUT_PAGE` / `CYMBRA_PADDLE_SUCCESS_URL` at the new pages.

**Rollback**: remove `web` from the allowed audiences (site sign-in refused, nothing else
affected); the pages degrade to a message; the app's "manage" link still opens the account
page, which then only shows the store links.

## Open Questions

- Should `/account` also expose the app's download links per platform (nice landing after a
  web purchase)? Cheap, cosmetic — leaning yes.
- Vue islands vs a lighter runtime (Svelte/Preact): Vue reuses the composables as-is; keeping
  Vue avoids porting the Google/Apple wrappers.
