# Design — extend-site-spanish-locale

## Context

See proposal.md (Why). What exists on `main` (bac5cc51):

| Where | What |
|---|---|
| `apps/site/src/pages/index.astro`, `en/index.astro` | The hub: title, description, hero (`h1`, tagline), `#produits` / `#products` with one card per product (kicker, `h2`, body, button), `Community`, `#fonctionnalites` / `#features` with four cards. Literal markup; `alternates={{ fr: '/', en: '/en/' }}` |
| `apps/site/src/pages/music.astro`, `en/music.astro` | Hero (kicker, `h1`, tagline, `StoreButtons links={musicStores(lang)}`), eight feature cards, `Community`, a closing section « Déjà un compte Cymbra ? » linking `/account` and `/redeem` (plan, betas, subscription, access code). `alternates={{ fr: '/music', en: '/en/music' }}` |
| `apps/site/src/layouts/Base.astro` | One table per language (change 29, D2): the Spanish table's brand is `/en/`, its navigation and footer « Music » `/en/music`, « Cuenta » `/en/account`, « Código de acceso » `/en/redeem`, « Lingua » `linguaHref('es')`; the switch lists the page's other alternates; one `hreflang` per alternate, no `x-default` |
| `apps/site/src/pages/es/` | `privacidad.md`, `terminos.md`, `soporte.md`, `eliminar-cuenta.astro`, `404.astro` (change 29); the not-found page's « Inicio » opens `/en/`, « Cymbra Music » `/en/music` |
| `apps/site/src/lib/lingua-text.ts` | Change 30's text tables: Spanish names (« inglés », « español », « francés »), speakers (« francohablantes », « anglohablantes », « hispanohablantes »), the Spanish grammar (`of` « en inglés », `esOr`, `esAnd`), `fill`; `MUSIC_HREF.es = "/en/music"` for the closing of `/es/lingua/` |
| `apps/site/src/lib/lingua-pairs.ts` | `linguaPairs()`, `linguaPageLangs()`, `linguaHref('es')` — `/es/lingua` once a pair glossed in Spanish is in `src/data/lingua-coverage.json`, `/en/lingua` until then (change 30, D3) |
| `apps/site/src/lib/stores.ts` | `musicStores('es')`: « App Store (iOS, iPadOS, macOS) » and « Google Play » (language-neutral store URLs), « Windows / Linux — próximamente » dimmed |
| `apps/music/store/copy/es.md` | Music's Spanish listing (tú): « Modo Espera », « Notas que caen al estilo Synthesia (cascada) », « pentagrama », « batería electrónica », « dominio público », « tu propio SoundFont »; its URL fields `/en/music/`, `/en/support/`, `/en/privacy/` (pinned for en/it/es) |
| `apps/music/lib/l10n/app_es.arb` | The app's Spanish: « Cursos », « ¡Lección completada! », « Modo Espera », its two views « Pentagrama » and « Cascada », « tus propias SoundFonts » |
| `apps/site/src/lib/pinned-routes.ts` | No Spanish route but `/es/eliminar-cuenta/`, `/es/terminos/`, `/es/privacidad/` |
| `apps/site/test/post-build/alternates.spec.ts` | The legal pages' twins; *a page without a Spanish twin offers French and English alone*, on `/` and `/en/music` |
| Sitemap, `robots.txt` | None: no `@astrojs/sitemap`, and `public/` holds the icons alone |

The account, code redemption and checkout pages (`/account`, `/redeem`, `/checkout`,
`/checkout/done` and their English twins) are Vue islands (`AccountIsland`, `RedeemIsland`,
`CheckoutIsland`) or a static thank-you page, `noindex`; the islands' dictionary (`src/lib/i18n.ts`)
is complete in Spanish since change 29.

## Goals / Non-Goals

**Goals:**
- The home page and Cymbra Music's page in Spanish, reached from every Spanish page and named by
  their French and English twins.
- A Spanish home that tells no Spanish speaker Lingua explains words in Spanish before it does.
- Nothing else moves for French and English visitors.

**Non-Goals:**
- Spanish account, code redemption and checkout pages (D4).
- Music's store listings, its app's legal links, the Discord announcements' links.
- Rewording the French or English pages, their footers included.
- A sitemap, an `x-default`, canonical addresses.

## Decisions

### D1 — Two static pages, Spanish section ids

`src/pages/es/index.astro` and `src/pages/es/music.astro`, beside change 29's Spanish pages, with
the French pages' structure and classes; they mount no island. Section ids in Spanish (`productos`,
`funciones`), as change 30's Spanish Lingua page names its features `funciones`; nothing links the
French or English ids (searched: no `#produits`, `#products`, `#fonctionnalites` or `#features`
anywhere in the site, the app or the extension). `/es/` is a static page: unlike `/es/lingua/`
(change 30, D3), it describes Cymbra Music, which Spanish speakers can use today, so it is always
built; only its Lingua card follows the pairs (D3). `src/pages/[locale]/lingua.astro` and the new
`es/index.astro` do not collide (built together in the prototype, with today's pairs and with the
matrix, where `/es/lingua/` is built beside `/es/`).

### D2 — The Spanish text: a translation of the French, Music's own terms

Each string translates the French page (M10: tú, neutral, no vosotros), not the English one where
the two differ (the French says « notes qui tombent façon cascade », the English "Synthesia-style
falling notes": the Spanish follows the French). Where Cymbra Music has a Spanish term, the page
uses it, so the page, the app and the listing never name one feature two ways: the app's
(`app_es.arb`: « Cursos », « Lección », « Modo Espera », « Cascada », « tus propias SoundFonts »)
where the listing words it otherwise (« al estilo Synthesia (cascada) », « tu propio SoundFont »),
the listing's (`copy/es.md`) elsewhere (« pentagrama », « batería electrónica », « dominio
público », « Puntuación en tiempo real »). Lingua is named as change 30's Spanish Lingua page names
it, « extensión para el navegador ». Genders are avoided where French defaults to the masculine for
the reader (« connecté à votre instrument » → « con tu instrumento conectado »). Store buttons are
text, not badge images: `musicStores('es')` already gives their Spanish labels, and the store URLs
carry no language (each store shows its Spanish listing to a Spanish storefront), so nothing is
added.

Measured: **38 strings, 36 distinct** (the privacy card is on both pages), **437 words** (422
distinct) — 18 on the home page (one of them the Lingua card's audience sentence, which has no
French source, D3), 20 on the Music page. Neutral strings are not counted: « Cymbra Music · iOS,
iPadOS, macOS, Android », the Music page's kicker « Cymbra Music », « 🎹 Music », « 📖 Lingua », the
store labels. Drafts for the owner's review (M9); italics mark the `gradient` span and the badge,
braces what the pairs fill (D3):

| Page | Slot | French (source) | Spanish (draft) |
|---|---|---|---|
| `/es/` | title | Cymbra — apprendre en pratiquant, pas en révisant | Cymbra — aprender practicando, no repasando |
| | description | Cymbra conçoit deux applications : Music, pour apprendre la musique connecté à votre instrument, et Lingua, pour enrichir son vocabulaire en lisant le web. | Cymbra crea dos aplicaciones: Music, para aprender música con tu instrumento conectado, y Lingua, para ampliar tu vocabulario leyendo la web. |
| | `h1` | Apprendre en pratiquant, *pas en révisant* | Aprender practicando, *no repasando* |
| | tagline | Cymbra fait deux applications construites sur la même idée : on progresse dans ce qu'on fait déjà. Devant votre instrument, ou devant une page web. | Cymbra crea dos aplicaciones con una misma idea: progresas en lo que ya haces. Frente a tu instrumento, o frente a una página web. |
| | Music card | Branchez votre clavier MIDI et jouez : l'application suit la partition en temps réel, vous dit ce qui est juste et vous fait avancer leçon par leçon. | Conecta tu teclado MIDI y toca: la aplicación sigue la partitura en tiempo real, te dice qué está bien y te hace avanzar lección a lección. |
| | Music button | Découvrir Music | Descubrir Music |
| | Lingua kicker | Cymbra Lingua · extension navigateur *bêta* | Cymbra Lingua · extensión para el navegador *beta* |
| | Lingua card (D3) | Lisez le web en anglais, les mots que vous ne connaissez pas encore sont surlignés sur place. Un pourcentage honnête par page, un clic pour la traduction, et votre vocabulaire qui se construit tout seul. | Lee la web {en inglés} con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo. |
| | Lingua audience (D3) | — | Pensada para {francohablantes}, con la interfaz y las traducciones en su idioma. |
| | Lingua button | Découvrir Lingua | Descubrir Lingua |
| | card | 🪪 Un seul compte — Le compte Cymbra vous suit d'une application à l'autre, sur tous vos appareils. | 🪪 Una sola cuenta — Tu cuenta Cymbra te acompaña de una aplicación a otra, en todos tus dispositivos. |
| | card | 🔒 Respect de votre vie privée — Données hébergées en France (UE), aucune revente, aucun pisteur publicitaire. | 🔒 Respeto por tu privacidad — Datos alojados en Francia (UE), sin reventa y sin rastreadores publicitarios. |
| | card | 📴 Ça marche hors-ligne — L'analyse et la lecture se font sur votre appareil : pas d'aller-retour serveur à chaque note ou à chaque mot. | 📴 Funciona sin conexión — El análisis y la lectura se hacen en tu dispositivo: sin ida y vuelta al servidor por cada nota ni por cada palabra. |
| | card | 🎯 Retour immédiat — Pas de quiz déconnecté du réel : le retour arrive pendant que vous jouez ou pendant que vous lisez. | 🎯 Respuesta inmediata — Nada de cuestionarios desconectados de la realidad: la respuesta llega mientras tocas o mientras lees. |
| `/es/music/` | title | Cymbra Music — pratique et cours de musique connectés | Cymbra Music — práctica y cursos de música conectados |
| | description | Branchez votre clavier MIDI et jouez : Cymbra Music suit la partition en temps réel, vous donne un retour immédiat et vous fait progresser leçon par leçon. | Conecta tu teclado MIDI y toca: Cymbra Music sigue la partitura en tiempo real, te da una respuesta inmediata y te hace progresar lección a lección. |
| | `h1` | Apprenez la musique, *connecté à votre instrument* | Aprende música, *con tu instrumento conectado* |
| | tagline | Cymbra transforme la pratique en une expérience guidée : suivez des cours, jouez en temps réel avec votre instrument, et progressez pas à pas. | Cymbra convierte la práctica en una experiencia guiada: sigue cursos, toca en tiempo real con tu instrumento y progresa paso a paso. |
| | card | 🎹 Connecté à vos instruments — Branchez n'importe quel clavier MIDI USB ou batterie électronique — ou jouez directement à l'écran. | 🎹 Conectado a tus instrumentos — Conecta cualquier teclado MIDI USB o batería electrónica, o toca directamente en la pantalla. |
| | card | 👀 Deux façons de lire — Partition classique sur la portée pour travailler la lecture, ou notes qui tombent façon cascade pour apprendre vite. On change de vue à tout moment. | 👀 Dos formas de leer — Partitura clásica en el pentagrama para practicar la lectura, o notas que caen en cascada para aprender rápido. Cambia de vista en cualquier momento. |
| | card | ⏸️ Mode Attente — La partition se fige jusqu'à ce que vous jouiez la bonne note. Main gauche et main droite séparément, avec un code couleur par main. | ⏸️ Modo Espera — La partitura se detiene hasta que tocas la nota correcta. Mano izquierda y mano derecha por separado, con un color para cada mano. |
| | card | 🥁 La batterie aussi — De vraies parties de batterie, sur la portée ou sur un kit animé. Associez vos pads une fois, isolez un élément pour travailler un groove. | 🥁 También la batería — Partes de batería reales, en el pentagrama o en un kit animado. Asocia tus pads una vez y aísla un elemento para trabajar un groove. |
| | card | 🎯 Notation en temps réel — Précision et timing notés pendant que vous jouez, avec un résumé de session à la fin de chaque passage. | 🎯 Puntuación en tiempo real — Precisión y ritmo puntuados mientras tocas, con un resumen de la sesión tras cada intento. |
| | card | 📚 Une bibliothèque qui s'étoffe — Partitions du domaine public du débutant à l'avancé, sons de piano de qualité, et vos propres SoundFonts si vous voulez. | 📚 Una biblioteca que crece — Partituras de dominio público, de principiante a avanzado, sonidos de piano de calidad y tus propias SoundFonts si quieres. |
| | card | 🌍 Quatre langues — Français, anglais, italien et espagnol. | 🌍 Cuatro idiomas — Español, inglés, francés e italiano. (the listing's order, the page's language first, as the English page does) |
| | card | 🔒 Respect de votre vie privée — (as the home page) | 🔒 Respeto por tu privacidad — (as the home page) |
| | closing | Déjà un compte Cymbra ? — Retrouvez votre formule, vos bêtas et la gestion de votre abonnement sur votre compte. Un code d'accès reçu sur la communauté ? Utilisez-le ici. | not carried (D4) |

« lectura » in the offline card keeps the French « lecture », which covers reading a score and a
page; the English page says "playback" — the owner chooses (M9).

### D3 — The home's Lingua card follows the shipped pairs

The French card reads « Lisez le web en anglais »: true for French speakers. Translated as it
stands, the Spanish card would tell Spanish speakers « Lee la web en inglés » while every shipped
pair explains words in French — and the site will be deployed before en-es ships (change 31's
annex goes live after change 10's server and before change 12's release). Holding the whole page
back as `/es/lingua/` is (change 30, D3) would also hold back Cymbra Music, which Spanish speakers
use today; leaving the language out (« Lee la web en otro idioma ») would say less than the French
says. So the card's words come from the pairs, as change 30's Lingua page does, with change 30's
Spanish names, speakers and grammar (`lingua-text.ts`); one builder there, inserted with `set:html`
and `fill`'s escaping like every slot of the Lingua page:

- **A pair glossed in Spanish ships**: « Lee la web {en <the languages read with a Spanish gloss>}
  con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por
  página, un clic para la traducción y tu vocabulario, que se construye solo. » The button opens
  `/es/lingua` (`linguaHref('es')`).
- **None ships**: the same sentence names every language read, and one sentence says for whom:
  « Pensada para {the speakers of every native language}, con la interfaz y las traducciones en su
  idioma. » The button opens `/en/lingua`.

Measured on the prototype (scratchpad, not committed), with the pair lists of the programme:

| Pairs | Card | Button |
|---|---|---|
| en-fr, es-fr (today) | Lee la web en inglés o en español con las palabras… solo. Pensada para francohablantes, con la interfaz y las traducciones en su idioma. (49 words) | `/en/lingua` |
| + es-en (change 34) | … Pensada para francohablantes y anglohablantes, con la interfaz y las traducciones en su idioma. (51) | `/en/lingua` |
| + en-es (change 35) | Lee la web en inglés con las palabras… solo. (34) | `/es/lingua` |
| + fr-en, fr-es (stage 3) | Lee la web en inglés o en francés con las palabras… solo. (37) | `/es/lingua` |

The kicker keeps the badge (« beta »), as the French and English homes and change 30's three Lingua
tables carry it (open question 2). The French and English homes' cards stay literal: their bytes do
not move, and their naming English alone is theirs to change.

### D4 — Account, code redemption and checkout stay French and English

M11 names the home page and Cymbra Music's page; these are not landing pages:

- They are account flows: plans, betas, subscriptions, access codes, payment (Cymbra Music's store
  builds never link `/redeem`, Apple 3.1.1).
- Their consumers open fixed, unprefixed, pinned paths: Music's « manage » action `/account`,
  Paddle `/checkout` and its return `/checkout/done`, the access-code links `/redeem`. A Spanish twin
  would be reached from the Spanish navigation alone; sending Spanish users to it from the app or
  from Paddle is a Music or backend change.
- They are `noindex` personal surfaces: no `hreflang` reaches a search engine.

What a Spanish visitor sees, unchanged from change 29: « Cuenta » and « Código de acceso » (the
Spanish navigation and footer), the support page's « Mi cuenta » and the not-found page's « tu
cuenta » open `/en/account` and `/en/redeem`, in English, whose switch offers « FR »; checkout is
reached at `/checkout` from Paddle, as for every buyer. The cost of a later Spanish twin is small and
recorded here: the islands' dictionary is complete in Spanish, the shells are ≈ 14 strings
(title, description and heading of three pages, and the thank-you page's five), and
`appleReturnUrl` strips `/es`, so their sign-in sends the Return URLs already registered
(`/account`, `/redeem`).

For the same reason the Music page's closing paragraph (« Retrouvez votre formule, vos bêtas et la
gestion de votre abonnement… Un code d'accès… ») is not carried into Spanish: it exists to send the
reader to the account and code pages, and both its links would open English pages. The Spanish page
ends with the community band (when an invite is set); the account stays in the navigation.

Account deletion is not among these pages: `/es/eliminar-cuenta/` is Spanish since change 29, and
the extension's link (`deleteAccountUrl`, `account/flow.ts` today, `account/locale.ts` once change
17 moves it and keys it on the interface language) opens it for a Spanish interface. This change
moves only its brand and « Music » links (D5).

### D5 — The links that move, and those that do not

Measured on the prototype build (31 pages instead of 29):

| Where | Link | Before → after |
|---|---|---|
| `Base.astro`, Spanish table | brand | `/en/` → `/es/` |
| | navigation « Music » | `/en/music` → `/es/music` |
| | footer « Music » | `/en/music` → `/es/music` |
| `es/404.astro` | « Inicio » | `/en/` → `/es/` |
| | « Cymbra Music » | `/en/music` → `/es/music` |
| `lingua-text.ts` | `MUSIC_HREF.es` (closing of `/es/lingua/`, built once en-es ships) | `/en/music` → `/es/music` |
| `index.astro`, `en/index.astro` | `alternates` | gain `es: '/es/'` |
| `music.astro`, `en/music.astro` | `alternates` | gain `es: '/es/music'` |

Unchanged: the Spanish « Cuenta », « Código de acceso », « Lingua » (`linguaHref('es')`) and the
footer's « FR » / « EN »; the French and English footers (no « ES »: the switch of every page with a
Spanish twin already offers it); the store URLs; `apps/music/store/copy/es.md`'s URL fields
(open question 1); Music's in-app legal links; the Discord announcements (`/en/music/`, French or
English only); Lingua's listings.

Nothing is pinned: no shipped client and no listing field requests `/es/` or `/es/music/`
(`pinned-routes.ts`'s rule: a route joins with its first consumer). `/` and `/en/` are pinned as
locale roots — entry points, and the listings' website field before the product pages; `/es/` was
never a listing's field. `yarn check:routes` still checks every pinned route and the three not-found
pages; its `alternates` and `locale` specs gain the two pages (tasks 2.1, 2.2).

### D6 — Nothing else moves for French and English visitors: measured

The prototype was built twice from the same directory — `main` (bac5cc51), then with this change —
because an island's `uid` hashes its component's path (`astro/dist/runtime/server/render/component.js`)
and two checkouts at different paths differ on every island page for that reason alone. Result:

- 29 pages (31 HTML files, with the two copied not-found pages) before, 31 pages (33 files) after —
  `es/index.html`, `es/music/index.html`; `_astro/` identical.
- `/`, `/en/`, `/music/`, `/en/music/`: two elements more each,
  `<link rel="alternate" hreflang="es" href="https://cymbra.app/es/[music]">` and
  `<a href="/es/[music]">ES</a>` in the switch; nothing else.
- The six Spanish files (`es/404.html`, `es/404/index.html`, `eliminar-cuenta`, `privacidad`,
  `soporte`, `terminos`): their brand and Music links (D5), and the not-found page's two buttons.
- The 21 other files, the Lingua pages and the account, code and checkout pages included: byte for
  byte.
- Two existing tests move: `test/lingua-text.spec.ts` (the Spanish closing now links `/es/music`)
  and `test/post-build/alternates.spec.ts` (*a page without a Spanish twin* used `/` and `/en/music`).

### D7 — Deploy order (owner)

`site-deploy` is manual (M18). A deploy publishes everything merged: with this change, change 31's
privacy annex, which says the native language reaches Cymbra with each card and day, so the deploy
comes after change 10's server is deployed and checked from outside (its task 5.2), as change 31's
3.2 requires. These pages mount no sign-in, so change 29's 4.3 is unchanged: if this deploy is the
first to publish `/es/eliminar-cuenta/`, `https://cymbra.app/eliminar-cuenta` is registered on the
Services ID with it. The Spanish home and Music page are then live before change 35, as M11 asks;
their Lingua card says nothing in Spanish's favour until en-es's figures are in the build (D3).

## Risks / Trade-offs

- **A Spanish page that claims something the French does not** → a translation string by string
  (D2's table), Music's terms from its app and its Spanish listing, the owner's review (M9).
- **Lingua promised to Spanish speakers before it serves them** → D3, its four states tested.
- **A Spanish visitor sent to English pages for the account** → as since change 29; D4 records why
  and what a Spanish twin would cost.
- **The French and English pages move** → D6, re-measured on the implementation (task 3.1).
- **Search engines see a third language of the hub** → the same `hreflang` cluster as change 29's
  legal pages; no `x-default` (change 29's follow-up, the owner's).

## Migration Plan

Merged, nothing is live until the owner deploys the site (D7). No data, no client and no listing
moves; reverting the change removes two pages nothing outside the site links.

## Open Questions

1. **Music's Spanish store listing** — **settled by the owner on 2026-10-09: pointed at the Spanish pages in a Music listing change of its own, after this deploy and change 29's legal review.** It points at `/en/music/`, `/en/support/` and `/en/privacy/`.
   Recommendation: point it at `/es/music/`, `/es/soporte/` and `/es/privacidad/` in a Music listing
   change of its own, after change 29's Spanish legal pages are reviewed (its 4.2) and this change's
   pages deployed — `copy/es.md` and `pinned-routes.ts` in the same pull request, the console paste
   after the deploy (es-ES and es-MX on App Store Connect, Spanish on Play). That change also
   modifies `store-distribution`'s *Store-listing URL fields*, which fixes `/en/music/` for `es` and
   whose scenario asks for the product page « in the listing's locale »; the requirement is held by
   the open `pin-music-site-url-contract`, so the listing change follows its archive. Not here: this
   change pins nothing.
2. **The « beta » badge** — **settled on 2026-10-09: kept, as on the other pages.** The badge on the Spanish home's Lingua kicker is carried from the French and English
   homes, which show it today, as do the Lingua pages' three tables (`lingua-text.ts`) —
   `/es/lingua/`'s among them, the Marketing URL of Lingua's Spanish listing (change 37). The App
   Store rule that keeps « beta » out of Lingua's listing texts (guidelines 2.2 and 2.3.7,
   `apps/lingua-apple/STORE-LISTING.md`) governs the listings, not cymbra.app, and no listing field
   points at `/es/`. Keep it on the Spanish home, or drop it — from the Spanish home alone, or from
   every home and Lingua page (a change of its own)?
3. **The Music page's closing paragraph** — **settled on 2026-10-09: left out of Spanish.** It is left out of Spanish (D4). Keep it out, or add a neutral
   line linking the (English) account page?
4. **« lectura » or « reproducción »** in the offline card (D2) — **settled on 2026-10-09: « reproducción ».**
