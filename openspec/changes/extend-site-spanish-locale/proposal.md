# extend-site-spanish-locale — cymbra.app in Spanish: the home page and Cymbra Music's page

## Why

Change 29b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, outside the 57, decision M11 (settled): beyond change 29's privacy policy, terms,
support and account deletion pages, and change 30's `/es/lingua`, the site's **home page and
Cymbra Music's page** in Spanish too (`/es/`, `/es/music/`; the site has one Music page, and its
support page is Spanish since change 29), before en-es ships (change 35).

Today a Spanish visitor reads four Spanish pages and is sent to English ones for everything else:
the brand of every Spanish page opens `/en/`, the Spanish navigation's and footer's « Music » open
`/en/music`, and the Spanish not-found page's « Inicio » and « Cymbra Music » buttons open the
English pages (change 29, D2). Cymbra Music has been in Spanish since its first store release (the
app speaks Spanish, its store listing has a Spanish text, `apps/music/store/copy/es.md`); its site
page has not.

## What Changes

- **Two Spanish pages**, translations of the French ones (tú, neutral, no vosotros, M10), drafted
  for the owner's review (M9): `/es/` (the hub: both products, what they share) and `/es/music/`
  (hero, store buttons, the eight feature cards), 38 strings measured (36 distinct, 437 words), the
  Music terms taken from the app and its Spanish store listing so that the page, the app and the
  listing never name one feature two ways.
- **The home's Lingua card follows the shipped pairs**, as change 30's Lingua page does: once a pair
  glossed in Spanish ships it reads « Lee la web en inglés… » and opens `/es/lingua/`; until then it
  names the languages Lingua reads and the readers it is made for (« Pensada para francohablantes… »),
  and opens the English Lingua page. A deploy before change 35 never tells Spanish speakers that
  Lingua explains words in Spanish.
- **The Spanish pages link them**: the brand opens `/es/`, the navigation's and footer's « Music »
  open `/es/music`, the not-found page's two buttons open them, and the Spanish Lingua page (built
  once en-es ships) sends its readers to `/es/music`.
- **The French and English twins name them**: `/`, `/en/`, `/music/` and `/en/music/` gain the
  Spanish `hreflang` and « ES » in their language switch — measured on a prototype build, the only
  difference on those four pages; every other French and English page and every asset is byte for
  byte the previous build.
- **Out of scope, said so**: the account, code redemption and checkout pages stay French and
  English (interactive flows about plans and payments, reached from outside by fixed paths); the
  Music page's closing paragraph about plans, betas and access codes is not carried into Spanish;
  Music's store listings keep their URLs; nothing is pinned.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `site-locales` (introduced by change 29, `add-site-spanish-locale`, still open): two ADDED
  requirements — *The home page and Cymbra Music's page exist in Spanish* and *The Spanish home
  page describes Lingua for the readers it serves*. No requirement of change 29's is MODIFIED; this
  change is archived after it, and after change 30 (`add-site-lingua-matrix-pages`), whose rule for
  `/es/lingua/` the home's Lingua card follows.

## Impact

- **Products.** The site only (`apps/site`): two new pages, the Spanish table of
  `src/layouts/Base.astro`, `src/pages/es/404.astro`, the `alternates` of four pages, one link and
  one builder in `src/lib/lingua-text.ts`, the tests and the README. **Consumed**: change 29's
  locale, `alternates` and Spanish layout table; change 30's shipped pairs (`linguaPairs`,
  `linguaHref`) and its Spanish names; `musicStores('es')` and `Community` in Spanish (change 30).
  **New**: the two pages and the card builder. Cymbra ID, Cymbra Music (app, store listings),
  Cymbra Lingua, the back office and the backend are untouched.
- **Visible for French and English visitors**: « ES » in the language switch of the home and Music
  pages, and their Spanish `hreflang`; nothing else.
- **Visible for Spanish visitors**: a Spanish home and Music page, reached from every Spanish page;
  « Cuenta » and « Código de acceso » still open the English pages.
- **Deploy (owner, M18).** `site-deploy` is manual. A deploy cut from a `main` holding this change
  also publishes change 31's privacy annex, which says the native language reaches Cymbra with each
  card and day: the owner deploys only after change 10's server (`add-lingua-native-language-server`)
  is deployed and checked from outside (its task 5.2), as change 31's task 3.2 requires.
- **Order.** Before change 35 (`enable-lingua-spanish-speakers`), whose prerequisites list this
  change's pages live and its Spanish reviewed beside change 29's; independent of every other change
  of stage 2.
- **Not here.** Spanish account, code and checkout pages; Music's Spanish store listing pointing at
  `/es/music/`, `/es/soporte/` and `/es/privacidad/` (a listing change, open question 1); an
  `x-default`, canonical addresses or a sitemap (none exists; change 29's follow-up); the French
  and English homes' Lingua cards, which name English only (a separate change, their words are the
  owner's).
