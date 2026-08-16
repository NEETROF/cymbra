# cymbra.app — the Cymbra web site

Astro (fr default, en) static site deployed on Cloudflare Pages. Lives in the
monorepo (`apps/site`, imported 2026-08-16 from `NEETROF/cymbra-site` with its
history) because it is becoming a **product surface** — Cymbra ID sign-in, access
code redemption (`/redeem`), the web checkout page (`/checkout`), subscription
management (`/account`) — that shares the backend's web API, the sign-in code and
the legal texts with the rest of the repo.

## Commands (Yarn Berry — `corepack enable`)

| Command | Action |
|---|---|
| `yarn install` | install |
| `yarn dev` | dev server on `localhost:4321` |
| `yarn check` | `astro check` (types of `.astro` / `.ts`) |
| `yarn typecheck` | `vue-tsc` over the Vue islands + tests |
| `yarn test` | vitest (islands' logic + components, jsdom) |
| `yarn build` | production build → `dist/` |
| `yarn preview` | preview the build |

## Account pages (change: add-site-account-pages)

Four pages are **interactive islands** (Astro + Vue, `client:load`), everything
else stays static:

| Page (fr / en) | Island | Talks to |
|---|---|---|
| `/redeem`, `/en/redeem` | `RedeemIsland` — sign-in gate, `?code=` prefill, neutral refusals | `POST /web/plans/redeem` |
| `/account`, `/en/account` | `AccountIsland` — who is signed in (handle, sign-in methods), plan, betas, rights-end date, manage (web portal / store page / web checkout), sign-out | `GET /web/account/me`, `GET /web/plans/me`, `/portal`, `POST /checkout` |
| `/checkout` (+ `/en/`) | `CheckoutIsland` — Paddle.js overlay for `_ptxn`, no sign-in | Paddle.js (CDN) |
| `/checkout/done` (+ `/en/`) | static — "go back to the app and refresh" | — |

Sign-in (`SignInForm.vue`: email + Google + Apple) and the session come from the
shared **`packages/web-auth`** package (`@cymbra/web-auth`, Yarn `portal:`), the
same code as the back office: audience `web`, access token **in memory only**,
persistence through the backend's HttpOnly refresh cookie (re-minted on mount).
Field names of the plan JSON are the proto's (`src/lib/web-plans.ts`); errors are
mapped to fr/en copy in `src/lib/plan-view.ts` (`humanError`, `redeemError`) —
never shown raw.

Configuration is build-time `PUBLIC_*` env (`.env.example`): `PUBLIC_API_URL`,
`PUBLIC_GOOGLE_CLIENT_ID` / `PUBLIC_APPLE_CLIENT_ID` (a missing id hides that
button), `PUBLIC_PADDLE_ENV` + `PUBLIC_PADDLE_CLIENT_TOKEN` (missing token ⇒
"web checkout unavailable"). Backend side: `https://cymbra.app` in
`CYMBRA_WEB_ORIGINS`, `web` in `CYMBRA_ALLOWED_AUDIENCES`, the site origin on the
Google web client and the Apple Services ID (Return URLs `/redeem`, `/account`).

Store builds of the app never link to `/redeem` (Apple 3.1.1); the app's "manage"
action for a web subscription opens `/account`. The site sets no CSP today (a
follow-up with the reverse proxy / `_headers`); the checkout page loads Paddle.js
from `cdn.paddle.com`, Google/Apple sign-in load their SDKs on demand.

## Deploy

`.github/workflows/site-deploy.yml`: on a `site-vX.Y.Z` tag (release-please) or a
manual dispatch, builds `dist/` and uploads it with wrangler to the Cloudflare
Pages project named by the repo variable `CF_PAGES_SITE_PROJECT` (dormant until
set). CI gate: `.github/workflows/site.yml` (`yarn check` + `yarn test` +
`yarn build`) on PRs touching `apps/site/**` or `packages/web-auth/**`.

Migration note: the old repo's Cloudflare Pages project was connected to
`NEETROF/cymbra-site` directly. Either point that project at this monorepo
(root directory `apps/site`, build `yarn build`, output `dist`) or let the
workflow above deploy with wrangler — then archive the old repo.

## Legal texts

`src/pages/{cgu,confidentialite}.md` and `src/pages/en/{terms,privacy}.md` are the
published pages. The product-specific drafts live in `docs/legal/` at the repo
root; keeping them in one place (a build step or a shared source) is a follow-up.
