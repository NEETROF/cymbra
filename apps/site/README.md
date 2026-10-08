# cymbra.app — the Cymbra web site

Astro (fr default, en, es) static site deployed on Cloudflare Pages. Lives in the
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

## Pages

The landing side is a **hub + one page per product**, fr (default) and en:

| Page (fr / en) | What it holds |
|---|---|
| `/`, `/en/` | The Cymbra hub: positioning, one card per product, what both apps share (one account, EU hosting, offline, immediate feedback) |
| `/music`, `/en/music` | Cymbra Music — hero, store buttons, features. The copy mirrors `apps/music/store/copy/{fr,en}.md`, so the site and the store listings never claim different things |
| `/lingua`, `/en/lingua` | Cymbra Lingua — the browser extension, still unpublished: disabled Chrome/Firefox/Safari buttons plus the community invite when `PUBLIC_DISCORD_URL` is set |

Distribution links live in **one** place, `src/lib/stores.ts`, read by the product
pages and by the post-checkout `Downloads` block. A channel is either `live: true`
with a real URL, or dimmed — the App Store record `6789557194` covers iOS, iPadOS
and macOS, so the three share one button.

Spanish (`/es/`, change `add-site-spanish-locale`) holds the pages Cymbra Lingua sends
its Spanish readers to — `/es/privacidad`, `/es/terminos`, `/es/soporte`,
`/es/eliminar-cuenta` and `/es/404` — as translations of the French pages, with Spanish
slugs as the French pages have French ones. There is no Spanish home, product, account,
code or checkout page: the Spanish nav and footer link the English ones there.

**Every page names its translations.** `Base.astro` takes `alternates` — the page's
address in each language it exists in, its own included, e.g.
`{ fr: '/confidentialite', en: '/en/privacy', es: '/es/privacidad' }`; the twins are not
derivable from the path. They drive the header's language switch (every *other* language
the page exists in) and one `hreflang` per alternate (no `x-default`). A page that exists
in one language passes that one alone: no switch, its own `hreflang`. Markdown pages
declare them in their front matter, forwarded by `Legal.astro`, which refuses a page
whose `lang` is not one of the three or that names no alternates; `Base.astro` refuses
any page whose alternates omit its own language. The footer's `FR` / `EN` links are the
locale roots, as before. The French and English pages render as before: the only
markup differences against the previous build are the `hreflang` / switch entries and
`&#39;` in one footer label (« Code d'accès », now an expression Astro escapes).

Astro trims the whitespace at a text/element
boundary that falls on a source-line break — on **either** side, so both
`word\n<a>…` and `</a>\nword` lose their space in the build. Keep a sentence
containing inline links on one source line, and check the rendered HTML rather than
the source.

## Routes shipped clients depend on

Some routes here are not the site's to move. A Cymbra Music build already installed
on a device requests the path it was compiled with **for as long as it exists** — a
user on 1.30 will ask for `/cgu/` long after the site has been redesigned twice — and
a store listing field is read by reviewers and users without the site being consulted.
Neither can be updated retroactively.

The rule: a pinned route **may change what it serves, and may not move or disappear**.
Rewriting `/` from the Music landing page into the two-product hub was fine; deleting
`/en/privacy/` would not be. If one ever has to move, the `301` from the old path
ships in the *same* change, in `public/_redirects` — host-side, because a native app
opening an external browser never runs client-side script.

**The list lives in [`src/lib/pinned-routes.ts`](src/lib/pinned-routes.ts)**, with the
consumer pinning each route and where that consumer is declared. It is deliberately
not restated here: two copies drift, and a route list nobody trusts is the failure the
contract exists to prevent. `yarn check:routes` reads that file and asserts every
route survived `yarn build`; `site-check` runs it right after the build step, so a
deletion fails the pull request.

Adding a page does **not** pin it. A route joins the day a client or a listing field
starts requesting it, and the change that creates that consumer is the one that adds it
— `/suppression-compte/`, `/lingua/` and `/es/eliminar-cuenta/` joined with Lingua's
deletion link and its listings' Homepage / Marketing URL.

Why this is worth a gate rather than a note: the site answers an unmatched path with
the **nearest `404.html`**, and until `pin-music-site-url-contract` there was none, so
every unknown path returned the home page with a `200`. A dead `/en/privacy/` would
have served French marketing copy to someone asking for the English privacy policy,
with a status code no uptime check would flag. `src/pages/404.astro`,
`src/pages/en/404.astro` and `src/pages/es/404.astro` cover the three locales — the
English and Spanish ones are copied to `dist/<locale>/404.html` by a small integration in
`astro.config.mjs` (`localised404()`), since Astro only gives the root `404.astro` its
special filename; `yarn check:routes` asserts the three files.

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
mapped to fr/en/es copy in `src/lib/plan-view.ts` (`humanError`, `redeemError`) —
never shown raw.

Configuration is build-time `PUBLIC_*` env (`.env.example`): `PUBLIC_API_URL`,
`PUBLIC_GOOGLE_CLIENT_ID` / `PUBLIC_APPLE_CLIENT_ID` (a missing id hides that
button), `PUBLIC_PADDLE_ENV` + `PUBLIC_PADDLE_CLIENT_TOKEN` (missing token ⇒
"web checkout unavailable"). Backend side: `https://cymbra.app` in
`CYMBRA_WEB_ORIGINS`, `web` in `CYMBRA_ALLOWED_AUDIENCES`, the site origin on the
Google web client and the Apple Services ID. Apple matches Return URLs exactly: the
Services ID lists what `appleReturnUrl` (`src/lib/plan-view.ts`) sends from each page
that mounts the sign-in — the page's path without its locale prefix or trailing slash —
`https://cymbra.app/redeem`, `/account`, `/suppression-compte`, `/delete-account` and
`/eliminar-cuenta`.

Store builds of the app never link to `/redeem` (Apple 3.1.1); the app's "manage"
action for a web subscription opens `/account`. The site sets no CSP today (a
follow-up with the reverse proxy / `_headers`); the checkout page loads Paddle.js
from `cdn.paddle.com`, Google/Apple sign-in load their SDKs on demand.

## Deploy

`.github/workflows/site-deploy.yml`: on a `site-vX.Y.Z` tag (release-please) or a
manual dispatch, builds `dist/` and uploads it with wrangler to the Cloudflare
Pages project named by the repo variable `CF_PAGES_SITE_PROJECT` (dormant until
set). CI gate: `.github/workflows/site-check.yml` (`yarn check` + `yarn test` +
`yarn build`) on PRs touching `apps/site/**` or `packages/web-auth/**`.

Deploy order (change `add-site-spanish-locale`): nothing links a Spanish page before it
is live. The site is deployed, and `https://cymbra.app/eliminar-cuenta` registered on the
Services ID, before any release of the extension or of Cymbra ID that links a Spanish
page.

Migration note: the old repo's Cloudflare Pages project was connected to
`NEETROF/cymbra-site` directly. Either point that project at this monorepo
(root directory `apps/site`, build `yarn build`, output `dist`) or let the
workflow above deploy with wrangler — then archive the old repo.

## Legal texts

`src/pages/{cgu,confidentialite}.md`, `src/pages/en/{terms,privacy}.md` and
`src/pages/es/{terminos,privacidad}.md` are the published pages; the Spanish ones are
translations of the French, reviewed by the owner. The product-specific drafts live in `docs/legal/` at the repo
root; keeping them in one place (a build step or a shared source) is a follow-up.
