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
| `yarn check` | `astro check` (types) |
| `yarn build` | production build → `dist/` |
| `yarn preview` | preview the build |

## Deploy

`.github/workflows/site-deploy.yml`: on a `site-vX.Y.Z` tag (release-please) or a
manual dispatch, builds `dist/` and uploads it with wrangler to the Cloudflare
Pages project named by the repo variable `CF_PAGES_SITE_PROJECT` (dormant until
set). CI gate: `.github/workflows/site.yml` (`yarn check` + `yarn build`) on PRs
touching `apps/site/**`.

Migration note: the old repo's Cloudflare Pages project was connected to
`NEETROF/cymbra-site` directly. Either point that project at this monorepo
(root directory `apps/site`, build `yarn build`, output `dist`) or let the
workflow above deploy with wrangler — then archive the old repo.

## Legal texts

`src/pages/{cgu,confidentialite}.md` and `src/pages/en/{terms,privacy}.md` are the
published pages. The product-specific drafts live in `docs/legal/` at the repo
root; keeping them in one place (a build step or a shared source) is a follow-up.
