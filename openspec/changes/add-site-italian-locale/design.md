# Design — add-site-italian-locale

## Context

Measured on main at `e2564d9a6`. `yarn build` (35 pages), `yarn test` (237 tests) and `yarn
check:routes` (61 post-build tests) pass.

**The precedents:**
- Matrix change 29 (`add-site-spanish-locale`, #792) made the site multilingual:
  - `LOCALES` gained `es`; `localised404()` covers every non-default locale;
  - `Base.astro`'s `alternates` replaced the French/English pair;
  - five Spanish pages: privacy, terms, support, deletion, 404;
  - the islands' dictionary gained `es`;
  - the deletion page was pinned ahead of its consumer.
- Change 29b (#845) added `/es/` and `/es/music/`, and a home Lingua card that follows the shipped pairs.
- Since then: `/es/lingua/` (built from `[locale]/lingua.astro` when a pair glossed in Spanish ships),
  the leaving pages (`add-lingua-usage-counts`), and per-language Lingua and Music media.

**Visible words per page** (the built HTML: title, description, alt texts, main text):

| Page | fr | en | es |
|---|---|---|---|
| Home | 223 | 209 | 220 |
| Music | 456 | 415 | 418 |
| Lingua (six pairs) | 761 | 711 | 807 |
| Privacy | 3,469 | 3,249 | 3,585 |
| Terms | 720 | 674 | 720 |
| Support | 195 | 183 | 191 |
| Deletion | 174 | 163 | 169 |
| 404 | 68 | 71 | 72 |
| Leaving | 102 | 103 | 91 |

The islands' dictionary (`src/lib/i18n.ts`) holds 107 keys, 590 French words. Italian is about 6,800
French source words to translate, of which about 4,560 are legal.

**What adding `it` to `LANGS` forces** (a `vue-tsc` prototype gave 28 errors):
- the islands' dictionary and date locale;
- `lingua-leave.ts` (`THANKS_ID`, `SUPPORT_HREF`);
- `lingua-text.ts` (`LINGUA_TEXT`, `MUSIC_HREF`, `GRAMMAR`);
- `music-media.ts` (ten captions and alt texts);
- `stores.ts`;
- `AccountIsland.vue`;
- the `locale.spec` test.

In `.astro` files: `Base.astro`'s navigation and footer tables, labels and brand alt, `Legal.astro`,
`Community.astro`, `Downloads.astro`.

**What is not typed, and must be edited by hand:**
- `astro.config.mjs`'s `LOCALES`;
- the `alternates` of 24 page files;
- `plan-view.ts`'s `appleReturnUrl` pattern `(?:en|es)`;
- `packages/web-auth/src/apple.ts`'s locale;
- `speakers.it` in the French, English and Spanish tables of `lingua-text.ts`.

**Three traps:**
1. **`/it/lingua/` builds itself.** With `it` in `LANGS`, `linguaLocalePaths()` builds `/it/lingua/` as
   soon as a pair glossed in Italian enters `lingua-coverage.json` (change 45), and `lingua-media.ts`
   then fails the build without `src/assets/lingua/it/*` and the Italian demo video.
2. **No Italian Music video.** The Music page needs `apps/music/store/video/it/{showcase.mp4,
   poster.jpg}`, which does not exist.
3. **The media budget.** The site's media weigh 30,782,670 B against the 30 MB that
   `add-site-lingua-showcase` requires, 659 KB under it. One language's Music media are about 6.9 MB.

**Elsewhere:**
- The backend's `thanks_path` (`backend/server/src/web_lingua.rs`) sends an unknown language, `it`
  included, to the French thanks.
- Cymbra ID's Italian e-mails link `/en/terms/` and `/en/privacy/` (change 43's).
- The extension's `LEAVING_PAGES.it` and `PRIVACY_POLICY_URLS.it` are the English pages (change 33),
  and move to the Italian ones with change 45.

## Goals / Non-Goals

**Goals:**
- Every page Lingua and Cymbra ID send an Italian reader to exists in Italian, and is pinned before the
  first client links it.
- Italian visitors have a home page and a Music page in Italian.
- Nothing moves for French, English or Spanish visitors but their pages' heads.

**Non-Goals:**
- The extension's links (change 45), the e-mails' (change 43), and Music's app and listing, which keep
  the English pages (a Music change, as for Spanish).
- The Lingua page's Italian screenshots and demo video (change 45; trap 1).
- Account, code redemption and checkout in Italian: they stay in French and English, as 29b decided for
  Spanish.
- Rewriting the legal texts: they are translated as they stand.

## Decisions

### D1 — The locale

- `LOCALES` and `LANGS` gain `it`; `localised404()` writes `it/404.html`.
- Every page names its translations through `alternates`, so the switch and `hreflang` follow: the 24
  French, English and Spanish page files gain their Italian alternate.
- The Italian footer links French and English, as the Spanish one does.
- The header labels gain « IT ».

**Slugs**, which become permanent once pinned:

| Page | Slug |
|---|---|
| Home | `/it/` |
| Music | `/it/music/` |
| Privacy | `/it/privacy/` (Italian sites use the loanword: « Informativa sulla privacy ») |
| Terms | `/it/termini/` |
| Support | `/it/assistenza/` (Italian's word for customer support) |
| Deletion | `/it/elimina-account/` |
| Leaving | `/it/lingua/arrivederci/` (thanks at `#grazie`) |
| 404 | `/it/404` |

The reviewer agent (D9) may change a slug before the pull request pins it, never after.

### D2 — The legal pages: courtesy translations, the French prevails (I13)

The privacy policy, the terms and the deletion page are translated from the French as they stand, in
« tu » as the rest of the Italian site. Each of the three opens with « Traduzione di cortesia: fa fede la
versione francese », linking its French page, as I13 settles.

- **Faithful to the French.** Nothing is added or left out. The French names only the CNIL for
  complaints, so the Italian does too: no Garante, as the Spanish names no AEPD. A difference from the
  French would bind the publisher, and I13's notice protects little against a consumer.
- **The Lingua labels** the pages quote (Réglages paths, « Account → … ») are change 33's Italian
  catalogue, word for word.
- **Upkeep.** Every later edit to a French legal page is mirrored in Italian in the same pull request,
  as it is today for English and Spanish (eight privacy edits in fourteen days).

### D3 — Pinned ahead of their consumers

`/it/privacy/`, `/it/termini/`, `/it/elimina-account/` and `/it/lingua/arrivederci/` join
`PINNED_ROUTES` in this change, before the extension (change 45) and the e-mails (change 43) link them.
That departs from the file's rule that a route joins when a shipped client points at it, as change 29
did for `/es/eliminar-cuenta/`. The reason: once deployed, they are about to be compiled into clients,
and `check:routes` must refuse a build that drops them from then on. The leaving page is the strict
case: the browser opens its address after the extension is gone, so an installed build's address must
resolve forever. The pins' comments name their consumers.

### D4 — The home page and Cymbra Music's page

`/it/` and `/it/music/` are translations of the French pages, with Music's terms from its Italian app
(`app_it.arb`): « Accedi », « Abbonamento », « Cascata », « Spartito », « Corsi », « Libreria »,
« Informativa sulla privacy ». The Italian app's own inconsistencies, such as « Modalità Attesa » and
« Modalità attesa », are put to the reviewer agent. 29b's answers carry over:
- the « beta » badge is kept;
- the Music page's closing paragraph is dropped, as in Spanish;
- the Music app's own word names playback.

**The home page's Lingua card** follows the shipped pairs, as Spanish's does. Until a pair glossed in
Italian ships, it names the languages Lingua reads and the speakers it is made for (« Pensata per
francofoni, anglofoni e ispanofoni »), never says Lingua explains words in Italian, and links
`/en/lingua/`. Once en-it ships, it names English and links `/it/lingua/`. An Italian builder sits
beside Spanish's.

**The Music media.**
- The Italian gallery is copied from Music's store captures (`store/ios/…/it`), as the other languages'
  are.
- The Italian video is recorded by the existing harness (`melos run showcase-video -- it`, macOS), then
  `sync:music-media` and `gen:music-share` for Italian alone, so the other languages' share images do
  not move.
- The video's commit is a `chore(music)`, separate from the site's, so that release-please, which
  attributes a release by path, does not bump Music for it.

### D5 — The islands

- The islands' dictionary gains `it`, all 107 keys: sign-in, code redemption and checkout strings
  included, since the type requires them, though only sign-in, deletion, the leaving question and the
  errors are shown on Italian pages.
- Dates are `it-IT`.
- `packages/web-auth`'s Apple locale is `it_IT`.
- `appleReturnUrl` strips `/it`, and the owner registers `https://cymbra.app/elimina-account` on Apple's
  Services ID.
- The deletion island's confirmation word is the reviewer's to fix (« ELIMINA »).

### D6 — The Lingua page's Italian text, without its media

`LINGUA_TEXT.it` (56–61 strings), `GRAMMAR.it` (articles, « dall'inglese / dallo spagnolo / dal
francese », 20.000, 96%, and « 5000 » under five digits as change 33 writes it), `MUSIC_HREF.it` and
`speakers.it` are written here, because the types require them once `LANGS` holds `it`.

`/it/lingua/` itself is not built until a pair glossed in Italian ships (trap 1). Change 45, which lists
en-it, brings `src/assets/lingua/it/*` and the Italian demo video in the same pull request, captured
from the real extension in Italian. Change 42 adds Italian's pairs and coverage to the Lingua pages.

### D7 — The media budget at 40 MB

*The Lingua page's media stay within a weight budget* (held by `add-site-lingua-showcase`) is MODIFIED:
the site's images, posters and videos together within 40 MB instead of 30. This is the owner's answer
of 2026-10-22. A visitor loads only their own language's media, so the total weighs on the
deployment, not on a page. The per-file budgets (250 KB per image variant, 4 MB per video) and the
host's limits are unchanged. The Italian Music media (about 6.9 MB) and change 45's Lingua media fit,
with room for one more language.

### D8 — The backend's leaving thanks

`thanks_path("it")` returns `/it/lingua/arrivederci/#grazie`, with its test. It ships in the backend
before the site that links it is deployed, and both before change 45's release (I12), as
`add-lingua-usage-counts` ordered server, site, then extension. Its commit is the backend's own, so
release-please attributes it to the backend alone.

### D9 — Reviewed by the Italian reviewer agent (I2, I13)

Every Italian text is reviewed in the implementation pull request by change 33's agent:
- **Address**: « tu » everywhere, legal pages included, though the French site says « vous ». No
  gendered address: no « iscritto/a », no « connesso ».
- **The GDPR's Italian terms**: titolare del trattamento, interessato, base giuridica, legittimo
  interesse, cancellazione, portabilità, reclamo.
- **Product terms**: Music's, with its inconsistencies settled, and Lingua's labels from change 33's
  catalogue.
- **Language names** with their articles; the speakers (italofoni, francofoni, anglofoni, ispanofoni);
  QCER.
- **Typography**: numbers (20.000, 96%, 5000), « » without inner spaces, no space before « : ; ? ! ».
- **The leaving page**: its question and six answers.
- **The slugs**, before they are pinned (D1).

### D10 — What does not move

- Every French, English and Spanish page's visible text is byte for byte. Their heads gain the Italian
  `hreflang` and their switches « IT », where an Italian twin exists.
- The sitemap is built from the pages' heads and lists the Italian pages with no code change.

Found while measuring, not changed here: the extension still sends a Spanish interface to the English
deletion page (`apps/lingua-extension/src/account/locale.ts`). Change 29's agent task 3.2 never landed,
and `pinned-routes.ts`'s comments assume it did. `en/delete-account.astro` quotes the French
interface's labels. Both are reported to the owner as defects of their own.

## Risks / Trade-offs

- **A legal translation that says more than the French** → D2's faithful translation, I13's notice, the
  reviewer agent; the owner chose no lawyer.
- **The Italian Lingua page built without its media** → trap 1, closed by change 45 bringing both in one
  pull request; `lingua-media.ts` fails the build otherwise, so it cannot ship broken.
- **Four languages to keep in step on every legal edit** → the existing practice, one pull request per
  edit in every language; the byte-for-byte check catches a page left behind only in its own language.
- **A slug regretted after pinning** → slugs are reviewed before the pin (D1, D9).

## Migration Plan

The owner deploys the backend (D8), then the site, before change 45's release, and registers the Apple
Return URL. Nothing is released in a store. Rollback before change 45 is a revert; after it, the pinned
routes stay.

## Effort

6–10 ideal days, against the programme's 5–9:

| Part | Ideal days |
|---|---|
| The locale's plumbing | 0.75–1.25 |
| The legal pages (≈ 4,560 words) | 1.5–2.5 |
| The islands, the leaving page, the backend's thanks, the pins | 0.75–1.25 |
| The home and Music pages, the Italian video and share image | 1–1.5 |
| The Lingua page's text and the home card | 0.75–1.25 |
| The 404, tests, README, the byte-for-byte check | 0.75–1.25 |
| The reviewer agent's pass | 0.5–1 |
