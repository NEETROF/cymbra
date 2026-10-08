# add-site-spanish-locale — cymbra.app in Spanish: privacy, terms, support and account deletion

## Why

Change 29 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, decision M11 (open: "Spanish site pages: at least privacy, account deletion, support
and `/es/lingua`", before en-es). Spanish-speaking readers of English (en-es, change 35) read the
extension in Spanish; the pages it and its store listings point at — the privacy policy, the
account deletion page, support — exist in French and English only, and Cymbra ID's e-mails link a
Spanish reader to the English legal pages. M14 also assumes a Spanish privacy policy before the
release that sends the native language.

The site (`apps/site`, Astro) is built for two languages: `astro.config` lists `fr` (at the root)
and `en` (`/en/`); the layout takes a French and an English address for its language switch and
its `hreflang`; the islands' dictionary is `fr` and `en`; the 404 is copied for `en` alone. The
legal pages, written in Markdown through one layout, do not even pass their counterpart: their
switch and `hreflang` point at `/` and `/en/`.

## What Changes

- **A third locale**, `es`, at `/es/`, with Spanish slugs as the French pages have French ones:
  `/es/privacidad/`, `/es/terminos/`, `/es/soporte/`, `/es/eliminar-cuenta/`, and `/es/404`. The
  Lingua page in Spanish is change 30's (it is built once a pair glossed in Spanish ships).
- **Pages that know their translations**: the layout takes `alternates` — the page's address in
  each language it exists in — for its language switch (every other language the page exists in)
  and its `hreflang` (each alternate); the legal layout forwards them, which fixes the French and
  English legal pages' switch and `hreflang`.
- **The islands in Spanish**: `Lang` gains `es`, the dictionary's Spanish entries typed after the
  French (the 94 keys), dates in `es-ES`, Apple's sign-in locale `es_ES`, and the Apple return
  address recognising `/es/`.
- **The routes pinned** that shipped clients and listings already use and the list omits —
  Lingua's `/suppression-compte/` and `/lingua/` (`/en/delete-account/`, pinned already, names Lingua
  too) — and the Spanish deletion page once the extension links it: after change 17's
  implementation, the extension's deletion link for a Spanish interface becomes `/es/eliminar-cuenta/`
  (change 17 sends it to the English page until this page exists).
- **Drafts for the owner**: the Spanish legal texts are translations of the French ones, for the
  owner's review (M9), who may also want them read by counsel before they are published.

## Capabilities

### New Capabilities

- `site-locales`: the public site's languages, how a page names its translations, and the 404 of
  each language.

### Modified Capabilities

- `lingua-privacy`: MODIFIED *Cymbra account deletion is reachable from Lingua* (held by no open
  change): the link follows the interface language — the French page for French, the Spanish page
  for Spanish, the English page otherwise; every scenario kept, one added.
- `lingua-interface-language`: MODIFIED *The account's e-mails follow the interface language*
  (change 17's, so this change is archived after it): its scenario *A Spanish-native reader* now opens
  the Spanish deletion page; every other scenario verbatim.

## Impact

- **Products.** The site (`apps/site`: config, layouts, pages, islands, pinned routes, the routes
  check) and Cymbra Lingua's one link (`apps/lingua-extension/src/account/flow.ts`). Cymbra ID's
  e-mails are change 32's; Music's own legal links are untouched (a Music change may follow).
- **Visible for French and English visitors**: the legal pages' language switch now opens the same
  page in the other language instead of the home page.
- **The owner deploys the site** (`site-deploy`, manual) before any release of the extension or
  of Cymbra ID that links a Spanish page (M18), and registers on Apple's Services ID the Return URL
  the Spanish deletion page sends (`https://cymbra.app/eliminar-cuenta`).
- **Not here.** `/es/lingua` (30); the Lingua annex's update (31); the e-mails' links (32); a
  Spanish home page, account, code redemption or checkout pages (beyond M11).
