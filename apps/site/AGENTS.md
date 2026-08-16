# apps/site — cymbra.app (Astro)

Part of the Cymbra monorepo since 2026-08-16 (imported from `NEETROF/cymbra-site`
with its history). Package manager: **Yarn Berry** (`corepack enable`), like
`apps/back-office` — never npm/pnpm.

```sh
yarn install
yarn dev        # localhost:4321
yarn check      # astro check (types)
yarn build      # → dist/
```

Static Astro site (fr default + en) served by Cloudflare Pages. It CONSUMES the
backend (web routes under `/web/*` on api.cymbra.app, cookie session — see
`backend/server/src/web_auth.rs`) and never redeclares an ID/Music capability
(OpenSpec prefix `site-`, see `openspec/config.yaml`).

## Development

When starting the dev server, use background mode:

```
yarn astro dev --background
```

Manage the background server with `yarn astro dev stop`, `yarn astro dev status`,
and `yarn astro dev logs`.

## Documentation

Full documentation: https://docs.astro.build

Consult these guides before working on related tasks:

- [Adding pages, dynamic routes, or middleware](https://docs.astro.build/en/guides/routing/)
- [Working with Astro components](https://docs.astro.build/en/basics/astro-components/)
- [Using React, Vue, Svelte, or other framework components](https://docs.astro.build/en/guides/framework-components/)
- [Adding or managing content](https://docs.astro.build/en/guides/content-collections/)
- [Adding styles or using Tailwind](https://docs.astro.build/en/guides/styling/)
- [Supporting multiple languages](https://docs.astro.build/en/guides/internationalization/)
