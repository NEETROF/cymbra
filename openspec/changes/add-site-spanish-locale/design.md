# Design — add-site-spanish-locale

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/site/astro.config.mjs` | `i18n: { defaultLocale: 'fr', locales: ['fr','en'], routing: { prefixDefaultLocale: false } }`; `localised404()` copies `dist/en/404/index.html` to `dist/en/404.html` and throws if it is missing |
| `src/layouts/Base.astro` | props `lang`, `frHref`, `enHref`; French and English nav arrays; a two-way switch; `hreflang` fr and en, no `x-default`; footers per language (the deletion page `/suppression-compte` or `/en/delete-account`) |
| `src/layouts/Legal.astro` | passes `title` and `lang` only: every Markdown page's switch and `hreflang` point at `/` and `/en/` |
| Pages | 12 French, 12 English twins: `cgu`↔`en/terms`, `confidentialite`↔`en/privacy`, `suppression-compte`↔`en/delete-account`, `support`; no Spanish |
| `src/lib/i18n.ts` | `type Lang = "fr" \| "en"`, 94 keys each, `t()`, `formatDate` `fr-FR`/`en-US`; islands (`SignInForm`, `DeleteAccountIsland`, …) take `lang` |
| `packages/web-auth/src/apple.ts` | Apple's SDK locale map `{en: en_US, fr: fr_FR}` |
| `apps/site/src/lib/plan-view.ts` | `appleReturnUrl` strips an `/en` prefix (`SignInForm.vue` always sends it when an Apple client id is set); `.env.example` and the README list `/redeem` and `/account` as registered Return URLs |
| `src/lib/pinned-routes.ts` | the single list of routes clients use, read by `yarn check:routes`; Music's legal pages, account and billing, `/en/delete-account/` (Play, Apple); not `/suppression-compte/` nor `/lingua/`, although Lingua's `flow.ts` and its listings' Homepage and Marketing URL point at them (`README.md` says the former is unpinned because nothing points at it — stale) |
| `test/post-build/pinned-routes.spec.ts` | every pinned route built; 404 pages `404.html` and `en/404.html` |
| Cloudflare Pages | serves the nearest `404.html`: without `es/404.html`, an unknown `/es/…` path answers in French |
| `apps/lingua-extension/src/account/flow.ts` | `deleteAccountUrl`: French page for French, English otherwise (change 17 keys it on the interface language) |

## Goals / Non-Goals

**Goals:**
- The pages M11 names in Spanish, reachable, linked from their French and English twins.
- One way for a page to name its translations, used by every layout.

**Non-Goals:**
- A Spanish home, account, redemption or checkout page; the Lingua page (30).
- Rewriting the legal texts: the Spanish is a translation of today's French.

## Decisions

### D1 — `es` at `/es/`, Spanish slugs

`locales: ['fr','en','es']`; Spanish pages under `src/pages/es/` with Spanish slugs
(`privacidad`, `terminos`, `soporte`, `eliminar-cuenta`), as the French pages carry French ones.
`localised404()` copies every non-default locale's 404 (`en`, `es`), and the routes check expects
`es/404.html`.

### D2 — `alternates`, one prop for every page

`Base.astro` takes `alternates: Partial<Record<"fr"|"en"|"es", string>>` in place of `frHref`/
`enHref`; the switch lists the page's other languages; `hreflang` names each alternate (no
`x-default`: the site has none today, and choosing one is the owner's); the nav and footer come from
one table per language. The Spanish table links the Spanish pages and, where a page has no Spanish
twin (home, Music, Lingua, account, code), its English page; the brand links `/en/`. `Legal.astro`
takes and forwards `alternates` from each Markdown page's front matter, labels `es` « Última
actualización », and passes each page's `description`. Pages that exist in one language pass that
one.

### D3 — The islands

`Lang = "fr" | "en" | "es"`; `es` typed `typeof fr`; `formatDate` adds `es-ES`; Apple's SDK
locale `es_ES` (`packages/web-auth/src/apple.ts`); `appleReturnUrl` (`apps/site/src/lib/plan-view.ts`)
strips a `/es` prefix as it strips `/en`, so `/es/eliminar-cuenta/` sends
`https://cymbra.app/eliminar-cuenta`, which the owner registers. Only the islands the
Spanish pages mount must work in Spanish (the deletion island and its sign-in); the dictionary is
complete anyway, as its type demands.

### D4 — Pins and the extension's link

`pinned-routes.ts` gains `/suppression-compte/` (Lingua's shipped `flow.ts`, the App Review notes)
and `/lingua/` (the listings' Homepage and Marketing URL); `/en/delete-account/`, pinned already,
names Lingua in its `pinnedBy`; `/es/eliminar-cuenta/` joins with the extension's link, `pinnedBy`
« Lingua — deletion link, Spanish interface ». The README's stale sentence goes. The link: after
change 17's `deleteAccountUrl(interfaceLanguage)` has merged, it answers `/es/eliminar-cuenta/` for a
Spanish interface (in `account/flow.ts`, or `account/locale.ts` if change 17 moved it). An extension
release carrying it ships after the site is deployed (D5). The not-found page of `/es/` is required by
the open `site-client-route-contract` (*An unmatched path SHALL NOT serve the home page*, in the
locale of the requested path); D1's copy satisfies it.

### D5 — Deploy order (owner)

The site deploy is manual (`site-deploy`); nothing links a Spanish page before it is live: the
extension's Spanish link ships in a release after the deploy, and change 32's e-mails after it too.

## Risks / Trade-offs

- **A Spanish legal text that says something the French does not** → a translation, reviewed by
  the owner (task 4.2), who may ask counsel.
- **An unknown `/es/` path answering in French** → D1's 404.
- **The French and English legal pages' switch moves** → it now opens the same page, a fix.

## Migration Plan

Merged, nothing is live until the owner deploys the site. The extension's Spanish deletion link
reaches readers in a release after that deploy.
