# Store listings — Cymbra Lingua

Copy for the Chrome Web Store and addons.mozilla.org listings, kept here so the answers stay
the same in both dashboards and so a change to the extension can change its listing in the
same pull request.

Nothing here is submitted by CI: `lingua-extension-release` uploads a **new version of an
existing item**, and the listing itself is filled once by hand. What follows is what to paste.

Constants:

| Field             | Value                                                                                                                                         |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Name              | Cymbra Lingua                                                                                                                                 |
| Homepage          | `https://cymbra.app/lingua/` (EN: `https://cymbra.app/en/lingua/`, ES: `https://cymbra.app/es/lingua/`)                                       |
| Privacy policy    | `https://cymbra.app/confidentialite/` (EN: `https://cymbra.app/en/privacy/`, ES: `https://cymbra.app/es/privacidad/`) — Annex B covers Lingua |
| Support           | `https://cymbra.app/support/` (EN: `https://cymbra.app/en/support/`, ES: the owner's choice, below)                                           |
| Category          | Chrome: _Education_ · AMO: _Language support_                                                                                                 |
| Listing languages | French, English from change 34, Spanish from change 35 — see below                                                                            |
| Firefox add-on id | `lingua@cymbra.app`                                                                                                                           |

**Listing languages.** A text in each native language a shipped pair is glossed in: French, for
French speakers, who study English and Spanish; English, for English speakers, who study Spanish
(es-en, from change 34); Spanish, for Spanish speakers, who study English (en-es, from change 35).
The interface speaks the reader's native language, so each text names what its readers study. The
listing's default language follows `default_locale`: English from change 34 (M13), and still
English once `_locales/es` ships — see _The dashboards' languages_.

An « EN » or « ES » address goes in that language's listing where a dashboard takes the field per
language, and the French one stays where it takes one for every language. The Spanish support
address is the owner's choice (change 29): `https://cymbra.app/es/soporte/`, the Spanish
translation of `/support/` — like it and `/en/support/`, a page about Cymbra Music that gives the
contact address — or `https://cymbra.app/en/support/`. The Spanish homepage, `/es/lingua/`, is built
only once en-es is in the published figures (change 30, D3; change 35 writes them), so it goes in
once the site is deployed after change 35.

---

## Summary (short description)

**Not editable in the dashboards** — both stores take it from the package's manifest
`description`, one per language the package carries:

**fr** — for French speakers, who study English and Spanish (`manifest.json` and
`_locales/fr/messages.json`), 109 characters, in the owner's wording (add-lingua-spanish-listings):

> Lisez l'anglais et l'espagnol sur le web : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé.

**en** — for English speakers, who study Spanish (`_locales/en/messages.json`, shipped from change
34), 94 characters, the draft of localise-lingua-manifest the owner reviews (its task 3.3, M9):

> Read Spanish on the web: unknown words highlighted, an honest percentage. Offline and private.

**es** — for Spanish speakers, who study English (`_locales/es/messages.json`, shipped from change
35), 99 characters, the draft of localise-lingua-manifest the owner reviews (its task 3.3, M9):

> Lee inglés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado.

**The limit is 112, Apple's** — checked when the signed archive is uploaded to App Store Connect,
which is where `lingua-apple-v1.1.0` died after a full build. Chrome allows 132, so calibrating on
Chrome produces an archive Apple refuses. `yarn check:version` holds the 112 now, for every
committed language.

Changing one means changing its `_locales/<language>/messages.json` — for French, `manifest.json`
**and** `_locales/fr/messages.json` (the French has two homes, held equal by `yarn check:version`)
— and shipping a new package, so it is worth getting right before a submission.

**Its language follows the shipped pairs** (localise-lingua-manifest). While every shipped pair is
glossed in French, the package carries the literal French above and no `_locales`. Once a pair
glossed in another language ships (es-en, change 34; en-es, change 35), `tool/manifests.mjs` writes
the description as `__MSG_extensionDescription__` and packages `_locales/<language>/` for each
shipped native: a browser, and a store, then shows the summary of its own language — and
`default_locale` is `en` as soon as an English-glossed pair ships, `fr` otherwise, so a browser in
any other language reads the English (M13). The Chrome Web Store takes the listing's default
language from `default_locale` and a summary per `_locales` folder; what addons.mozilla.org does to
an existing listing's summary at upload is undocumented: _The dashboards' languages_, below, is
the first submission's procedure and where its result is recorded.

Each summary is in its readers' language because the interface is: it follows the reader's native
language — French for French speakers ("Analyser cette page", "Je connais", "Toujours surligner
(toutes les pages)"), English for English speakers ("Analyze this page", "I know it", "Always
highlight (every page)", `src/i18n/en/`), Spanish for Spanish speakers ("Analizar esta página", "La
conozco", "Resaltar siempre (todas las páginas)", `src/i18n/es/`) — and each names only what its
readers can study. A summary in a language no shipped pair is glossed in would send its readers to a
product they cannot use yet — which is why the build ships `_locales/<language>` only once a pair
glossed in that language does.

## Description

One text per listing language, each written for its readers: French speakers study English and
Spanish, English speakers study Spanish, Spanish speakers study English. Every claim in the English
text is one change 34 ships (es-en); it is pasted with that release, under the dashboards' `en` (see
_The dashboards' languages_), and the owner reviews its wording (M9, M10: US English). Every claim in
the Spanish text is one change 35 ships (en-es); it is pasted with that release, under `es`, and the
owner reviews its wording (M9, M10: tú, neutral Spanish, no vosotros, the RAE's numbers). The French
and English texts are unchanged (add-lingua-spanish-audience-listings, D2).

**FR** — for French speakers

Cymbra Lingua surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne
connaissez pas encore — sans rien changer à la mise en page. Un pourcentage vous dit quelle part du
texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

Choisissez dans les Réglages les langues que vous apprenez : chaque page est lue dans la sienne.
Plusieurs langues à la fois : gratuit pour l'instant.

Cliquez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté dans l'usage courant.
En espagnol, la carte nomme aussi le temps et le genre, comme on les apprend en classe (« passé
simple », « nom féminin »). Puis décidez : « Je connais », « + Deck » pour le réviser plus tard, ou
« Ignorer ». Sélectionnez plusieurs mots et appuyez sur Alt+L pour capturer une expression entière
avec sa phrase.

Les cartes que vous créez se révisent dans un panneau, à côté de votre lecture ou dans la barre
latérale, avec une répétition espacée qui décide toute seule du bon moment.

Lisez aussi vos livres : importez vos fichiers EPUB sans DRM dans la bibliothèque de
l'extension et lisez-les, hors ligne, avec le même surlignage, page par page — une tablette à
encre électronique comprise. Vos livres restent sur votre appareil.

**L'analyse est locale.** Les dictionnaires et le moteur tournent dans votre navigateur : aucune
page que vous lisez n'est envoyée nulle part, et l'extension fonctionne hors ligne. Sans
compte ni traduction étendue, elle ne fait aucune requête réseau.

**Traduction étendue** (facultative, désactivée par défaut, sur Chrome, Firefox pour ordinateur
et Safari) : votre sélection est traduite dans sa phrase, sur votre appareil, par le moteur de
Firefox Translations ; l'espagnol passe par l'anglais. L'activer télécharge une fois les modèles de
traduction depuis Cymbra (25,8 Mo pour l'anglais, 52,0 Mo avec l'espagnol) ; le texte des pages ne
quitte toujours pas votre appareil. La désactiver supprime les modèles.

Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos
chiffres sont publiés sur cymbra.app/lingua.

Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur
vos autres appareils. C'est la seule chose qui quitte votre machine, et vous pouvez effacer ces
données depuis les Réglages sans supprimer votre compte.

Cymbra Lingua est disponible sur Chrome, sur Firefox pour ordinateur et dans l'app Safari (iPhone, iPad, Mac).

**EN** — for English speakers, who study Spanish (es-en, from change 34)

Cymbra Lingua highlights, right on the page you are reading, the Spanish words you don't know yet —
without changing the layout. A percentage tells you how much of the text is familiar, counted from
what you actually marked rather than guessed.

Click a highlighted word for its meaning in English, its dictionary form and how rare it is in
everyday use. The meanings come from Wiktionary, written by people, never machine-translated. The
card also names the tense and the gender ("preterite indicative", "feminine noun"). Then decide:
"I know it", "+ Deck" to review it later, or "Ignore". Select several words and press Alt+L to
capture a whole phrase with the sentence it came from.

The cards you build are reviewed in a panel, beside your reading or in the sidebar, with spaced
repetition that picks the moment for you. Your statistics estimate your vocabulary level by level,
from A1 to C2 — levels estimated from word frequency, as no freely licensed CEFR list exists for
Spanish.

Read your own books too: import your DRM-free EPUB files into the extension's library and read
them offline with the same highlighting, page by page — on an e-ink tablet as well. Your books
stay on your device.

**The analysis is local.** The dictionaries and the engine run in your browser: no page you read
is ever sent anywhere, and the extension works offline. With no account and no extended
translation, it makes no network request at all.

**Extended translation** (optional, off by default, on Chrome, Firefox for desktop, and Safari):
your selection is translated into English within its sentence, straight from Spanish, on your
device, by the Firefox Translations engine. Turning it on downloads the translation model from
Cymbra once (26.2 MB); the text of the pages you read still never leaves your device. Turning it
off deletes the model.

How many Spanish words our dictionary explains in English: our figures are published at
cymbra.app/en/lingua.

Create a Cymbra account if — and only if — you want your words and cards on your other devices.
That is the only thing that leaves your machine, and you can erase it from Settings without
deleting your account.

The interface is in English. Cymbra Lingua also teaches English and Spanish to French speakers, in
French.

Cymbra Lingua is available on Chrome, on Firefox for desktop and as a Safari app (iPhone, iPad, Mac).

What the English text checks against, so that it promises nothing the package does not do:

- **The card**: its buttons are the English catalogue's (`src/i18n/en/card.ts`: "I know it",
  "+ Deck", "Ignore"); its grammar is the English renderer's (`src/i18n/en/grammar.ts`: "preterite
  indicative", "feminine noun"); its glosses are es-en's, the English Wiktionary's Spanish entries,
  else the Spanish Wiktionary's English translations (`scripts/lingua-data/tables/es-en/README.md`)
  — people's words, never a machine's (M5).
- **The levels**: Spanish's are estimated from word frequency, for every pack studying it
  (`tables/es-en/NOTICE`); the reason is the interface's own (`src/i18n/en/languages.ts`,
  `estimatedLevelsNote`).
- **Extended translation**: es-en's route is one model, `es-en/base-memory/2.0`
  (`model-manifest.json`), 23,288,494 + 2,543,246 + 409,312 B as served = 26.2 MB as the setting
  rounds it (`megabytes`, `src/reading/translation-setting.ts`). Its platforms are the French text's,
  the owner's wording.
- **No « Several languages at once »**: the French text's line is true for French speakers only.
  es-en is the one English-glossed pair, so an English speaker studies one language and the
  studied-languages box hides itself below two (`offerFor`, `src/reading/studied-languages-view.ts`).
- **The coverage figures**: `/en/lingua/` publishes es-en's once change 34 writes them
  (`apps/site/src/data/lingua-coverage.json`, change 30's page).

**If the owner settles M15 not to offer es-en's translation** (change 34, D5: the `es-en` route
leaves `model-manifest.json`), the « Extended translation » paragraph becomes the one below, and
« With no account and no extended translation, » becomes « With no account, »:

> **Extended translation** of your Spanish selections into English, on your device, comes later.

**ES** — for Spanish speakers, who study English (en-es, from change 35)

Cymbra Lingua resalta, en la misma página que estás leyendo, las palabras en inglés que todavía no
conoces, sin cambiar el diseño de la página. Un porcentaje te dice qué parte del texto te resulta
familiar, calculado a partir de lo que de verdad has marcado, no estimado.

Haz clic en una palabra resaltada para ver su significado en español, su forma de diccionario y su
frecuencia en el uso corriente. Los significados proceden del Wikcionario, escritos por personas,
nunca por una traducción automática. La tarjeta también nombra el tiempo verbal, como se aprende en
clase («pasado simple de go»). Luego decide: «La conozco», «+ Mazo» para repasarla más tarde, o
«Ignorar». Selecciona varias palabras y pulsa Alt+L para capturar una expresión entera con la frase
de la que procede.

Las tarjetas que creas se repasan en un panel, junto a tu lectura o en el panel lateral, con una
repetición espaciada que elige por ti el momento oportuno. Tus estadísticas estiman tu vocabulario
nivel por nivel, del A1 al C2, en la escala MCER.

Lee también tus propios libros: importa tus archivos EPUB sin DRM a la biblioteca de la extensión y
léelos sin conexión con el mismo resaltado, página a página, también en una tableta de tinta
electrónica. Tus libros se quedan en tu dispositivo.

**El análisis es local.** Los diccionarios y el motor se ejecutan en tu navegador: ninguna de las
páginas que lees se envía a ningún sitio, y la extensión funciona sin conexión. Sin cuenta ni
traducción ampliada, no hace ninguna solicitud de red.

**Traducción ampliada** (opcional, desactivada por defecto, en Chrome, Firefox para escritorio y
Safari): tu selección se traduce al español dentro de su frase, directamente desde el inglés, en tu
dispositivo, con el motor de Firefox Translations. Al activarla se descarga una vez el modelo de
traducción desde Cymbra (25,4 MB); el texto de las páginas que lees sigue sin salir de tu
dispositivo. Al desactivarla se elimina el modelo.

Cuántas palabras inglesas explica nuestro diccionario en español: nuestras cifras están publicadas
en cymbra.app/es/lingua.

Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros
dispositivos. Es lo único que sale de tu dispositivo, y puedes borrar esos datos desde los Ajustes
sin eliminar tu cuenta.

La interfaz está en español. Cymbra Lingua también enseña inglés y español a los francohablantes, en
francés, y español a los anglohablantes, en inglés.

Cymbra Lingua está disponible en Chrome, en Firefox para escritorio y como app de Safari (iPhone,
iPad, Mac).

What the Spanish text checks against, so that it promises nothing the package does not do — each
claim and the file that makes it true once change 35 lists en-es in `packs.json`:

| Claim                                                                                                       | Where it is true                                                                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| English words highlighted, the layout untouched                                                             | the same engine and highlighting as every pair (`src/reading/highlight.ts`); a Spanish-native reader's pairs are en-es alone (`pairsOf`, `src/analyzer/pairs.ts`)                                                                                                                                                                                        |
| The percentage, counted from what the reader marked                                                         | the popup's « de palabras conocidas en esta página » (`knownOnPage`, `src/i18n/es/popup.ts`), the pill's label (`src/i18n/es/hud.ts`)                                                                                                                                                                                                                    |
| The meaning in Spanish, the dictionary form, the frequency                                                  | en-es's `gloss.tsv` (`scripts/lingua-data/tables/en-es/`); the frequency band in words (« Rara », « Corriente »…: `rarityText`, `src/reading/selection-card.ts`, with `src/i18n/es/selection.ts`)                                                                                                                                                        |
| From Wiktionary, written by people, never machine-translated                                                | en-es's glosses are the Spanish Wiktionary's English entries, else the English Wiktionary's Spanish translations, else the Spanish Wiktionary's English translations read backwards (`tables/en-es/README.md`, `NOTICE`) — M5                                                                                                                            |
| The card names the tense: « pasado simple de go »                                                           | the Spanish renderer, English's tenses (`TENSES.en`, `src/i18n/es/grammar.ts`), pinned by `test/word-grammar-es.spec.ts`; English nouns carry no gender, so the text names none                                                                                                                                                                          |
| « La conozco », « + Mazo », « Ignorar »                                                                     | `src/i18n/es/card.ts` (`known`, `addToDeck`, `ignore`)                                                                                                                                                                                                                                                                                                   |
| Alt+L captures a phrase with its sentence                                                                   | the command's Spanish description, « Capturar la selección (palabra o expresión) con su frase » (`_locales/es/messages.json`)                                                                                                                                                                                                                            |
| Review in a panel, beside the reading or in the side panel                                                  | `src/i18n/es/drawer.ts` (« Repaso »), `src/i18n/es/sidepanel.ts`, « Abrir el panel lateral de repaso » (`_locales/es`)                                                                                                                                                                                                                                   |
| Statistics A1 to C2, MCER                                                                                   | the scale's Spanish name (`levelScale`, `src/i18n/es/languages.ts`, M19); English's levels are CEFR-J's and Octanove's, not estimated (`levels_estimated: false`, `scripts/lingua-data/reduce-en-es.py`), so the text says nothing of estimated levels                                                                                                   |
| Books: DRM-free EPUB, offline, on the device, e-ink                                                         | the Spanish books note (`booksNote`, `src/i18n/es/settings.ts`); the reader is every pair's (`src/reader/`)                                                                                                                                                                                                                                              |
| Local analysis; no request without an account or translation                                                | as the French and English texts (_Remote code_, _Data usage disclosures_)                                                                                                                                                                                                                                                                                |
| Extended translation: Chrome, Firefox for desktop, Safari; straight from English; 25,4 MB; deleted when off | « Traducción ampliada » (`toggle`, `src/i18n/es/translation.ts`); en-es's route is one model, `en-es/base-memory/2.1` (`model-manifest.json`), 22,698,792 + 2,265,250 + 409,312 B as served = « 25,4 MB » as the Spanish setting writes it (`megabytes`, `src/reading/translation-setting.ts`); the platforms are the French text's, the owner's wording |
| The coverage figures at cymbra.app/es/lingua                                                                | `/es/lingua/` is built once en-es is in `apps/site/src/data/lingua-coverage.json` (change 30, D3), which change 35 writes; its table is `es` in `apps/site/src/lib/lingua-text.ts`                                                                                                                                                                       |
| The account: optional, erasable from Ajustes without deleting it                                            | « Gestionar mis datos » (`manageData`, `src/i18n/es/settings.ts`), « Borrar mis datos de Lingua… » (`erase`, `src/i18n/es/account.ts`)                                                                                                                                                                                                                   |
| French speakers learn English and Spanish, English speakers Spanish                                         | `packs.json` after change 35: en-fr, es-fr, es-en, en-es                                                                                                                                                                                                                                                                                                 |
| No « Varios idiomas a la vez »                                                                              | the French text's line is true for French speakers only: en-es is the one pair glossed in Spanish, so a Spanish speaker studies one language, and the studied-languages box hides itself below two (`offerFor`, `src/reading/studied-languages-view.ts`) — its « gratis por ahora » never shows them                                                     |

**If the owner settles M15 not to offer en-es's translation** (the `en-es` route leaves
`model-manifest.json`), the « Traducción ampliada » paragraph becomes the one below, and « Sin cuenta
ni traducción ampliada, » becomes « Sin cuenta, »:

> **Traducción ampliada** de tus selecciones en inglés al español, en tu dispositivo: llegará más adelante.

---

## Graphics

| Asset            | Requirement                                                             | Where it is                  |
| ---------------- | ----------------------------------------------------------------------- | ---------------------------- |
| Store icon       | 128×128                                                                 | `icons/icon-128.png`         |
| Screenshots      | 1280×800 or 640×400, JPEG or 24-bit PNG **without alpha**, at least one | captured by hand — see below |
| Small promo tile | 440×280, no alpha                                                       | optional                     |
| Marquee          | 1400×560, no alpha                                                      | optional                     |

Screenshots cannot be generated from the repository: they need the extension loaded in a
browser, reading a real page. Load `dist-chromium` unpacked, open an article in a studied language,
click the popup's analyse button, and capture the highlighted page, the word popup, the review panel
and the stats. Any capture can then be conformed to 1280×800 without alpha.

One set per listing language, captured by the owner:

| Language | Build                                                                   | Interface and page                                                              |
| -------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `fr`     | the released package                                                    | French (« Analyser cette page »), an English article                            |
| `en`     | a build of change 34 (`packs.json` lists es-en), the browser in English | English ("Analyze this page"), a Spanish article, a Spanish level chosen        |
| `es`     | a build of change 35 (`packs.json` lists en-es), the browser in Spanish | Spanish (« Analizar esta página »), an English article, an English level chosen |

The `en` set, in this order:

1. **Reading** — a Spanish article (a Spanish Wikipedia page reads well), highlighted, the pill
   showing the share of the page known.
2. **The word card** — open on a verb form whose card names a tense: « dijo » reads "third-person
   singular preterite indicative of decir" (`test/baseline/word-card-es-en.txt`), its English
   gloss below, "I know it", "+ Deck" and "Ignore".
3. **Review** — Chrome's side panel ("Review") on a card, its sentence shown, the four answers
   ("Again", "Hard", "Good", "Easy").
4. **Statistics** — the ladder "My estimated Spanish level", A1 to C2, with its note that the
   levels are estimated from word frequency.

The `es` set, in this order:

1. **Reading** — an English article (an English Wikipedia page reads well), highlighted, the pill
   showing the share of the page known.
2. **The word card** — open on a past form: « went » reads « pasado simple de go »
   (`src/i18n/es/grammar.ts`, pinned by `test/word-grammar-es.spec.ts`; change 24's snapshot of the
   Spanish card, `test/baseline/word-card-en-es.txt`, gives the real readings once merged), its
   Spanish gloss below, « La conozco », « + Mazo » and « Ignorar ».
3. **Review** — Chrome's side panel (« Repaso ») on a card, its sentence shown, the four answers
   (« Otra vez », « Difícil », « Bien », « Fácil »).
4. **Statistics** — the ladder « Mi nivel de inglés », A1 to C2 (MCER), English's levels not
   estimated.

Where a dashboard takes screenshots per language, the `en` set goes under its English listing and
the `es` set under its Spanish one; where it keeps one set for every language, the owner chooses
which set it keeps.

## Extra fields

| Field          | Value                                                                                                                                                                                              |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Homepage URL   | `https://cymbra.app/lingua/` — **only once the site is deployed**; the live site predates that page, and an unknown path there serves the home page rather than a 404, so it looks fine and is not |
| Support URL    | `https://cymbra.app/support/`                                                                                                                                                                      |
| Mature content | no                                                                                                                                                                                                 |

## Test instructions (for the reviewer)

Paste this into the store's test-instructions field — one field per store, one text for both.
**Step 1 is not optional**: on a fresh install `needsLevelChoice` is true
(`src/state/level-choice.ts`), and until a level is picked the engine knows of no known word —
every word is highlighted and the pill reads 0%. A reviewer who skips it sees what looks like a
broken extension, not a missed step.

It follows the path a reviewer takes: their browser is almost never in French or Spanish, so a new
install presets the English interface, studying Spanish — the native language is preset from the
browser's, French or Spanish when the browser is in one and English for any other language
(`add-lingua-native-language-choice`, D4) — and step 6 says how to reach the Spanish and French
interfaces, with the Spanish labels of steps 1, 2 and 4. It is true from change 35 and pasted with
its release; until then the dashboards keep the text change 34 pasted (this file's version before
add-lingua-spanish-audience-listings), whose first line gives the English interface to a browser in
any language but French.

The field caps at 1000 characters; the text below is 996. Change 34's text measured 992, so the
Spanish path was paid for by cutting wording, and no step a reviewer needs: « between their own
devices » became « across devices »; « (the tab opened on install) » and « (the toolbar popup) »
became « (welcome tab) » and « (popup) »; « B1 is a good default » became « e.g. B1 » and « With no
level » « With none »; step 4's « Click a highlighted word: its card gives the … with » became
« Click one for its card: … and »; step 5's « a multi-word selection » became « a selected phrase »
and « the popup's "Review" button » « the popup's "Review" »; step 6's « French interface (French
speakers learn English and Spanish): the popup's gear ("Settings"), tab "Language", choose
"Français" » became one line for both interfaces. Every step is still there: the level and why it
matters, the two ways to highlight, the pill, the card and its three actions, capture and review,
and the way to each interface.

```
No account is needed: signing in only syncs a reader's words across devices. The interface follows the browser: French, Spanish, else English (studying Spanish).

1. Pick a level, e.g. B1, at "What's your Spanish level?" (welcome tab) or "Choose your Spanish level" (popup). With none, every word is highlighted and the score reads 0%.
2. Open a Spanish article. On Chrome, click "Analyze this page" in the popup, or grant "Always highlight (every page)".
3. Words above that level are highlighted; the pill shows the share of the page you know.
4. Click one for its card: English translation, dictionary form, frequency, and "I know it", "+ Deck", "Ignore".
5. Alt+L captures a selected phrase; Alt+Shift+S or the popup's "Review" opens the review panel.
6. Popup gear ("Settings") > "Language": "Español" to study English, "Français" English and Spanish. In Spanish, steps 1-5 on English pages: "¿Cuál es tu nivel de inglés?" or "Elige tu nivel de inglés", "Analizar esta página", "La conozco".
```

Every label above is quoted from the source, not from memory: the level prompts from
`src/i18n/en/languages.ts` with the language "Spanish" (`levelQuestion`, the welcome tab's;
`chooseLevelPrompt`, the popup's); "Analyze this page", "Always highlight (every page)", "Review"
and "Settings" (the gear's label) from the popup's copy, `src/i18n/en/popup.ts`; the three word
actions from the word card's, `src/i18n/en/card.ts`; "Language", the settings tab, from
`src/i18n/en/settings.ts`. The Spanish ones from `src/i18n/es/`: « ¿Cuál es tu nivel de inglés? »
and « Elige tu nivel de inglés » (`languages.ts`, `levelQuestion` and `chooseLevelPrompt` with « de
inglés »), « Analizar esta página » (`popup.ts`, `analyse`), « La conozco » (`card.ts`, `known`).
Two are not in the catalogue yet: « Español » and « Français », the native-language choice's
options, which `add-lingua-native-language-choice` (change 20, D4) names each in its own language in
every interface — the owner checks them against change 20's merged copy before pasting. An
approximate label sends the reviewer looking for a control that does not exist.

## Single purpose (Chrome Web Store)

Cymbra Lingua has one purpose: helping a reader understand and learn the vocabulary of the language
they study — Spanish for English speakers, English for Spanish speakers, English or Spanish for
French speakers — in what they are already reading. Every feature serves it — highlighting unknown
words, showing a word's translation on click, capturing words and phrases into a deck, and reviewing
that deck — on the web page they are reading, and in the books they import themselves (DRM-free EPUB
files, read in the extension's own reader page, with the same highlighting).

## Permission justifications (Chrome Web Store)

Each answer names the user-visible feature and the code path, because that is what a reviewer checks
against the bundle. The fields are one per item, in English, whatever the listing's languages: a
label is quoted in each interface that shows it — English for English speakers, who study Spanish;
Spanish for Spanish speakers, who study English; French for French speakers, who study English and
Spanish. Code paths are named by function, not by line, so that they survive the next edit.

The book reader (`reader.html`, `src/reader/`) **added no permission**: the file picker, the
IndexedDB database that keeps the books and an extension page of its own need none, and nothing
is injected into it. `tool/check_variants.mjs` fails a build whose manifest asks for more.

**`activeTab`** — The reader is injected only into the tab the user is looking at, when they click
"Analyze this page" (« Analizar esta página » in Spanish, « Analyser cette page » in French) in the
toolbar popup. This is the extension's default mode on Chrome: without it, the extension cannot read
the page it was asked to analyse. `src/popup/popup.ts` (`activeTabId`, then the injection below).

**`scripting`** — To inject that reader. Two paths: `chrome.scripting.executeScript` on the
popup's explicit request (`analyseCurrentPage`, `src/popup/popup.ts`), and
`chrome.scripting.registerContentScripts` once — and only once — the user has granted the
optional `<all_urls>` permission, so highlighting survives a page reload
(`syncReaderRegistration`, `src/background.ts`). Revoking the permission unregisters it.

**`storage`** — The reader's own data, kept on their machine: which words they marked as known,
learning or ignored, the cards they captured, their review history and their preferences. No
page content is stored. `src/state/`, and IndexedDB for the parts that grow. The books a reader
imports are kept in IndexedDB too (`src/reader/library.ts`), on the device only.

**`sidePanel`** — The review panel, opened by the user from the popup or the Alt+Shift+S
shortcut, so they can review cards beside the page instead of over it.
`src/sidepanel/`, opened by `openReviewSurface` (`src/popup/popup.ts`) and by the
`lingua-side-panel` command (`src/background.ts`).

**`identity`** — Only for the optional account. `chrome.identity.launchWebAuthFlow` runs the
Google or Apple sign-in flow and returns to `chrome.identity.getRedirectURL()` (the account
block of `src/background.ts`). The extension never sees a password. A reader who does not sign in
never reaches this code.

**`offscreen`** — Only for extended translation (« Traducción ampliada » in Spanish, « Traduction
étendue » in French), an optional setting that is off by default. The translation engine and the
download of its model run in a dedicated worker, off every thread that paints; a Manifest V3 service
worker cannot start a worker itself, so an offscreen document (reason `WORKERS`) owns it. It is
created when the reader turns the setting on or asks for a translation, and closed once idle
(`src/translate/host/offscreen-engine.ts`, `src/translate/host/offscreen.ts`).

**`<all_urls>` (optional, not requested at install)** — Offered behind "Always highlight (every
page)" (« Resaltar siempre (todas las páginas) » in Spanish, « Toujours surligner (toutes les
pages) » in French) for readers who want highlighting on every page without clicking the toolbar
each time. It is requested by `chrome.permissions.request` from a user gesture, the click on that button
(`src/popup/popup.ts`), and can be revoked at any time. The extension is fully usable without it.

**`https://api.cymbra.app/*` (host permission)** — Only for the optional account: signing in,
then what a signed-in reader synchronises across their own devices — word statuses, declared levels, cards
with their review history, and daily learning statistics (`SyncEngine`, `src/sync/sync.ts`).
The origin is set at build time (`LINGUA_GRPC_WEB_URL`, `src/net/transport.ts`) and
granted by the build (`tool/manifests.mjs`, `hostPattern`). A reader who does not sign in never
sends it a request.

## Remote code

None. Everything the extension runs ships inside the package, including both WebAssembly engines —
the analysis engine and the translation engine — the language packs, one per pair (`packs.json`:
en-fr, es-fr and, from changes 34 and 35, es-en and en-es), and the book renderer (foliate-js,
vendored under `vendor/`). No script and no WebAssembly is fetched at runtime; the content security
policy is `script-src 'self' 'wasm-unsafe-eval'`. A book's own scripts never run: its pages are
rendered under that same policy (`test/reader-csp.spec.ts`).

The only files downloaded at runtime are **data**: the translation models' weights, the ones the
reader's pairs need (`model-manifest.json`, `routes`) — for a French speaker, en-fr, and es-en when
they study Spanish, since Spanish goes through English; for an English speaker reading Spanish,
es-en alone, the direct model of its pair; for a Spanish speaker reading English, en-es alone
(`en-es/base-memory/2.1`), the direct model of its pair. They are fetched from
`https://models.cymbra.app` only after the reader turns on extended translation (« Traducción
ampliada » in Spanish, « Traduction étendue » in French), and each file is used only once its sha256
matches the hash the package carries (`model-manifest.json`,
`src/translate/host/model-download.ts`). Nothing in them is executed.

## Data usage disclosures (Chrome Web Store)

Answer **yes** to collecting _"Personally identifiable information"_ — but only in the narrow
sense below — and **no** to every other category (health, financial, authentication
information, personal communications, location, web history, user activity, website content).

What is collected, and only for a signed-in reader: the email address used to create the
account, and the reader's own vocabulary state (word statuses, cards, review history) sent to
`https://api.cymbra.app` so their devices agree. Reading activity, page content and browsing
history are **not** collected — the analysis never leaves the browser. Nor are the books a
reader imports, their text or where the reader is in them: they stay on the device. Turning on
« Traduction étendue » downloads models from Cymbra; those requests carry nothing of the
reader's (no cookie, no identifier, no page text) and is not tied to an account.

Tick the three certifications: the data is not sold to third parties, it is not used or
transferred for a purpose unrelated to the item's single purpose, and it is not used or
transferred to determine creditworthiness or for lending.

## Source code (addons.mozilla.org)

AMO requires the human-readable source because the package is built (esbuild bundle, a `.wasm`
compiled from Rust, and the translation engine compiled from `mozilla/translations` with
Emscripten — its recipe is `tool/build_engine.sh`, pinned by `engine-pin.json`). `lingua-extension-release` attaches it automatically; its build
instructions are in [REVIEWERS.md](REVIEWERS.md), which becomes the archive's `README.md`.

License to declare on AMO: **Apache-2.0** (the repository's, see `LICENSE`).

## The dashboards' languages (change 34's first submission)

The first package carrying `_locales` — change 34's, es-en (or change 35's, should en-es ship
first) — changes what the dashboards read from it. `default_locale` becomes `en` (M13), and the
Chrome Web Store takes the listing's default language from it: uploaded and published the usual
way, the item's listing would be re-keyed to English with the French texts no longer the default
— the risk localise-lingua-manifest names. What addons.mozilla.org does with `__MSG_` at upload is
undocumented. So that submission is made by hand, in this order (the owner's, M18; task 3.2 of
add-lingua-english-listings):

1. **Upload without publishing.** `lingua-extension-release` has no input that uploads without
   calling `:publish`, so the Chrome Web Store package goes through the dashboard: the tag's run
   attaches `cymbra-lingua-chromium-<version>.zip` to its GitHub Release, and the owner uploads that
   file as the item's new package (Package › Upload new package), which submits nothing.
2. **`fr` re-entered.** In the Store listing, under `fr`: the French description above, and every
   other per-language field as it stood before the upload.
3. **`en` filled.** Under `en`: the English description above, the English addresses of
   _Constants_ where the field is per language, and the `en` screenshots (_Graphics_). The summary
   of each language is the package's (_Summary_): check that both read as quoted there.
4. **The single fields.** The single purpose, the permission justifications and the remote-code
   answer as above, and the test instructions (_Test instructions_).
5. **Submit for review** from the dashboard.
6. **addons.mozilla.org**: dispatch `lingua-extension-release` with the tag, `publish` ticked and
   `stores: firefox` (the Chrome Web Store already holds the version, and a second submission
   would be refused). After the submission, check the listing's default locale and its summary in
   French and in English; paste the English description into the English (US) locale.
7. **Record both results below**, in the pull request that next touches this file.

| Dashboard          | Checked on  | Default language after upload | Summary per language | Listing texts | Notes |
| ------------------ | ----------- | ----------------------------- | -------------------- | ------------- | ----- |
| Chrome Web Store   | _to record_ | _to record_                   | _to record_          | _to record_   |       |
| addons.mozilla.org | _to record_ | _to record_                   | _to record_          | _to record_   |       |

## The dashboards' languages: Spanish (change 35's first submission)

The first package carrying `_locales/es` — change 35's, en-es — adds a language to what the
dashboards read. `default_locale` stays `en` (M13: change 27 sets it to `en` as soon as an
English-glossed pair ships, which change 34 did), so the listing's default language should not
move; what arrives is a Spanish summary, from `_locales/es`, and with it a Spanish listing to fill.
Whether either dashboard does anything else with a new `_locales` folder at upload is not
documented, so this submission is made by hand too, in the same order (the owner's, M18; task 3.2
of add-lingua-spanish-audience-listings):

1. **Upload without publishing**, as change 34's step 1: the tag's
   `cymbra-lingua-chromium-<version>.zip`, from its GitHub Release, uploaded through the dashboard
   (Package › Upload new package), which submits nothing.
2. **`fr` and `en` checked.** The default language still `en`, the French and English listings'
   texts as they stood before the upload, and their summaries as quoted in _Summary_.
3. **`es` filled.** Under `es`: the Spanish description above, the « ES » addresses of _Constants_
   where the field is per language (the support address the owner chose), and the `es` screenshots
   (_Graphics_). Check that the Spanish summary reads as quoted in _Summary_.
4. **The single fields.** The single purpose, the permission justifications, the remote-code answer
   and the test instructions as above: this change gave each its Spanish path.
5. **Submit for review** from the dashboard.
6. **addons.mozilla.org**: dispatch `lingua-extension-release` with the tag, `publish` ticked and
   `stores: firefox`. After the submission, check the listing's default locale (unchanged) and its
   Spanish summary; paste the Spanish description into the Spanish locale. If the dashboard offers
   more than one Spanish locale, paste it into the one that shows the `_locales/es` summary, and
   record which.
7. **Record both results below**, in the pull request that next touches this file.

If changes 34 and 35 ship in one release, the two procedures are one submission: `fr` re-entered,
`en` and `es` filled in the same pass, and both tables filled.

| Dashboard          | Checked on  | Default language after upload | Spanish summary | Spanish listing (locale, texts) | Notes |
| ------------------ | ----------- | ----------------------------- | --------------- | ------------------------------- | ----- |
| Chrome Web Store   | _to record_ | _to record_                   | _to record_     | _to record_                     |       |
| addons.mozilla.org | _to record_ | _to record_                   | _to record_     | _to record_                     |       |
