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
(es-en, from change 34) and French (fr-en, from change 52); Spanish, for Spanish speakers, who study
English (en-es, from change 35) and French if change 52 lists fr-es (M6). A French speaker cannot
study French, so no French text offers it. The interface speaks the reader's native language, so
each text names what its readers study. The listing's default language follows `default_locale`:
English from change 34 (M13), and still English once `_locales/es` ships — see _The dashboards'
languages_.

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

**From change 52** (`enable-lingua-french`), which lists fr-en, and fr-es if its tables stand at or
above M6's floor, the English and Spanish summaries name French. The drafts of
add-lingua-french-listings (D6), approved by the owner on 2026-10-10, are committed to
`_locales/en/messages.json` (and `_locales/es/messages.json`) by **the pull request that lists the
pairs**, not before: committed earlier, they would tell every English and Spanish browser that
Lingua reads French.

**en** — for English speakers, who study Spanish and French (from change 52), 105 characters:

> Read Spanish and French on the web: unknown words highlighted, an honest percentage. Offline and private.

**es** — for Spanish speakers, who study English and French (from change 52, if it lists fr-es),
109 characters; without fr-es, the summary above stays:

> Lee inglés y francés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado.

The French summary does not move: a French speaker studies no French.

**What a summary names is checked.** `yarn check:version` (`tool/check_version.mjs`, run by
`lingua-extension-check`) fails when, for a native language a shipped pair is glossed in, its
description — `_locales/<language>`, and manifest.json's literal for French — does not name exactly
the languages its shipped pairs study, matched as whole words from the interface's own names
(`name` in `src/i18n/{fr,en,es}/languages.ts`). The error names the language, the languages the
summary names and the ones its pairs study. So change 52 cannot list fr-en under "Read Spanish on
the web", nor a summary name French before a pair glossed in its language studies it; a language no
shipped pair is glossed in is not read.

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
Spanish; English speakers Spanish (change 34) and French (change 52); Spanish speakers English
(change 35), and French if change 52 lists fr-es. Every claim in the English and Spanish texts below
is one change 52 ships; they are pasted with its release (M18), after the site deploy that publishes
fr-en's (and fr-es's) figures, under the dashboards' `en` and `es` (see _The dashboards' languages:
French studied_), and the owner reviews their wording (M9; M10: US English; tú, neutral Spanish, no
vosotros, the RAE's numbers). Each claim and the file that makes it true are in one table, after the
texts. Should change 34 or 35 be released before change 52, its text is this file's version before
add-lingua-french-listings — the English one naming Spanish alone, the Spanish one English alone.

The Spanish text is written twice (add-lingua-french-listings, D1): **with fr-es**, French named
everywhere a Spanish reader's languages are; **without fr-es**, change 37's text with its last line
alone made true (English speakers study Spanish and French). The owner pastes the variant that
matches the release. The French text names no other audience and is unchanged but for one line:
« Plusieurs langues à la fois… » is gone, by the owner's decision of 2026-10-10 (no listing text
speaks of price; change 53b took the line out of the extension and the site).

**FR** — for French speakers (2,442 / 16,000 characters)

Cymbra Lingua surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne
connaissez pas encore — sans rien changer à la mise en page. Un pourcentage vous dit quelle part du
texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

Choisissez dans les Réglages les langues que vous apprenez : chaque page est lue dans la sienne.

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

**EN** — for English speakers, who study Spanish (es-en, from change 34) and French (fr-en, from
change 52) (2,579 / 16,000 characters)

Cymbra Lingua highlights, right on the page you are reading, the Spanish or French words you don't
know yet — without changing the layout. A percentage tells you how much of the text is familiar,
counted from what you actually marked rather than guessed.

Choose the languages you study in Settings: each page is read in its own.

Click a highlighted word for its meaning in English, its dictionary form and how rare it is in
everyday use. The meanings come from Wiktionary, written by people, never machine-translated. The
card also names the tense and the gender ("preterite indicative" in Spanish,
"past historic (passé simple)" in French, "feminine noun"). Then decide: "I know it", "+ Deck" to
review it later, or "Ignore". Select several words and press Alt+L to capture a whole phrase with
the sentence it came from.

The cards you build are reviewed in a panel, beside your reading or in the sidebar, with spaced
repetition that picks the moment for you. Your statistics estimate your vocabulary level by level,
from A1 to C2 — levels estimated from word frequency, as no freely licensed CEFR list exists for
Spanish or French.

Read your own books too: import your DRM-free EPUB files into the extension's library and read them
offline with the same highlighting, page by page — on an e-ink tablet as well. Your books stay on
your device.

**The analysis is local.** The dictionaries and the engine run in your browser: no page you read is
ever sent anywhere, and the extension works offline. With no account and no extended translation, it
makes no network request at all.

**Extended translation** (optional, off by default, on Chrome, Firefox for desktop, and Safari):
your selection is translated into English within its sentence, straight from Spanish or French, on
your device, by the Firefox Translations engine. Turning it on downloads from Cymbra, once, the
translation model of each language you study (26.2 MB each); the text of the pages you read still
never leaves your device. Turning it off deletes the models.

How many Spanish and French words our dictionary explains in English: our figures are published at
cymbra.app/en/lingua.

Create a Cymbra account if — and only if — you want your words and cards on your other devices. That
is the only thing that leaves your machine, and you can erase it from Settings without deleting your
account.

The interface is in English. Cymbra Lingua also teaches English and Spanish to French speakers, in
French.

Cymbra Lingua is available on Chrome, on Firefox for desktop and as a Safari app (iPhone, iPad,
Mac).

**ES, with fr-es** — for Spanish speakers, who study English (en-es, from change 35) and French
(fr-es, from change 52) (3,045 / 16,000 characters)

Cymbra Lingua resalta, en la misma página que estás leyendo, las palabras en inglés o en francés que
todavía no conoces, sin cambiar el diseño de la página. Un porcentaje te dice qué parte del texto te
resulta familiar, calculado a partir de lo que de verdad has marcado, no estimado.

Elige en los Ajustes los idiomas que estudias: cada página se lee en el suyo.

Haz clic en una palabra resaltada para ver su significado en español, su forma de diccionario y su
frecuencia en el uso corriente. Los significados proceden del Wikcionario, escritos por personas,
nunca por una traducción automática. La tarjeta también nombra el tiempo verbal, como se aprende en
clase («pasado simple de go»; en francés, «pretérito perfecto simple de indicativo»), y el género de
los sustantivos franceses. Luego decide: «La conozco», «+ Mazo» para repasarla más tarde, o
«Ignorar». Selecciona varias palabras y pulsa Alt+L para capturar una expresión entera con la frase
de la que procede.

Las tarjetas que creas se repasan en un panel, junto a tu lectura o en el panel lateral, con una
repetición espaciada que elige por ti el momento oportuno. Tus estadísticas estiman tu vocabulario
nivel por nivel, del A1 al C2, en la escala MCER; para el francés, niveles estimados según la
frecuencia de las palabras, a falta de una lista MCER de uso libre.

Lee también tus propios libros: importa tus archivos EPUB sin DRM a la biblioteca de la extensión y
léelos sin conexión con el mismo resaltado, página a página, también en una tableta de tinta
electrónica. Tus libros se quedan en tu dispositivo.

**El análisis es local.** Los diccionarios y el motor se ejecutan en tu navegador: ninguna de las
páginas que lees se envía a ningún sitio, y la extensión funciona sin conexión. Sin cuenta ni
traducción ampliada, no hace ninguna solicitud de red.

**Traducción ampliada** (opcional, desactivada por defecto, en Chrome, Firefox para escritorio y
Safari): tu selección se traduce al español dentro de su frase, directamente desde el inglés y
pasando por el inglés desde el francés, en tu dispositivo, con el motor de Firefox Translations. Al
activarla se descargan una vez los modelos de traducción desde Cymbra (25,4 MB para el inglés, 51,6
MB con el francés); el texto de las páginas que lees sigue sin salir de tu dispositivo. Al
desactivarla se eliminan los modelos.

Cuántas palabras inglesas y francesas explica nuestro diccionario en español: nuestras cifras están
publicadas en cymbra.app/es/lingua. Para el francés, es algo menos completo que para el inglés.

Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros
dispositivos. Es lo único que sale de tu dispositivo, y puedes borrar esos datos desde los Ajustes
sin eliminar tu cuenta.

La interfaz está en español. Cymbra Lingua también enseña inglés y español a los francohablantes, en
francés, y español y francés a los anglohablantes, en inglés.

Cymbra Lingua está disponible en Chrome, en Firefox para escritorio y como app de Safari (iPhone,
iPad, Mac).

**ES, without fr-es** — for Spanish speakers, who study English (en-es, from change 35), if change 52
lists fr-en alone: change 37's text, its last line naming what English speakers study
(2,586 / 16,000 characters)

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
francés, y español y francés a los anglohablantes, en inglés.

Cymbra Lingua está disponible en Chrome, en Firefox para escritorio y como app de Safari (iPhone,
iPad, Mac).

What the English and Spanish texts check against, so that they promise nothing the packages do not
do — each claim and the file that makes it true once change 52 lists fr-en (and fr-es) in
`packs.json`, after es-en and en-es:

| Claim                                                                                                                                                                 | Text              | Where it is true                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Spanish or French words highlighted, the layout untouched                                                                                                             | EN                | the same engine and highlighting as every pair (`src/reading/highlight.ts`); an English-native reader's pairs are es-en and fr-en (`pairsOf("en")`, `src/analyzer/pairs.ts`)                                                                                                                                                                                                                                                                                                  |
| English (or, with fr-es, French) words highlighted                                                                                                                    | ES                | a Spanish-native reader's pairs are en-es, and fr-es if listed (`pairsOf("es")`)                                                                                                                                                                                                                                                                                                                                                                                              |
| The percentage, counted from what the reader marked                                                                                                                   | EN, ES            | the popup's "of words known on this page" / « de palabras conocidas en esta página » (`knownOnPage`, `src/i18n/{en,es}/popup.ts`), the pill's label (`src/i18n/{en,es}/hud.ts`)                                                                                                                                                                                                                                                                                               |
| "Choose the languages you study in Settings: each page is read in its own" / « Elige en los Ajustes los idiomas que estudias… » (the French text's sentence, carried) | EN; ES with fr-es | the studied-languages box, shown from two pairs of the reader's native language (`offerFor`, `src/reading/studied-languages-view.ts`), "Languages studied" / « Idiomas estudiados » (`studiedLanguages`, `src/i18n/{en,es}/settings.ts`), its note "Each page is read in whichever of your languages it holds" (`src/i18n/en/studied-languages.ts`)                                                                                                                           |
| The meaning in English, from Wiktionary, written by people, never machine-translated                                                                                  | EN                | es-en's glosses are the English Wiktionary's Spanish entries, else the Spanish Wiktionary's English translations (`scripts/lingua-data/tables/es-en/README.md`); fr-en's the English Wiktionary's French section (change 48, `tables/fr-en/README.md`) — M5                                                                                                                                                                                                                   |
| The meaning in Spanish, from Wiktionary, written by people                                                                                                            | ES                | en-es's glosses are the Spanish Wiktionary's English entries, else the English Wiktionary's Spanish translations, else the Spanish Wiktionary's English translations read backwards (`tables/en-es/README.md`, `NOTICE`); fr-es's the Spanish Wiktionary's French section, then the French Wiktionary's Spanish translations, then the Spanish Wiktionary's French translations read backwards (change 49, `tables/fr-es/README.md`) — M5                                     |
| The card names the tense and the gender: "preterite indicative", "past historic (passé simple)", "feminine noun"                                                      | EN                | the English renderer (`TENSES` keyed by studied language, `src/i18n/en/grammar.ts`), pinned by `test/baseline/word-card-es-en.txt` (« dijo »: "third-person singular preterite indicative of decir") and `word-card-fr-en.txt` (« fut »: "third-person singular past historic (passé simple) of être", "[feminine noun] house"); French's genders from `tables/fr/grammar.tsv` (change 45)                                                                                    |
| « pasado simple de go »; « pretérito perfecto simple de indicativo »; the gender of French nouns                                                                      | ES                | the Spanish renderer (`src/i18n/es/grammar.ts`), pinned by `test/word-grammar-es.spec.ts` and `test/baseline/word-card-fr-es.txt` (« fut »: « tercera persona del singular del pretérito perfecto simple de indicativo de être », « [sustantivo femenino] Casa »); English nouns carry no gender, so the text names one for French alone                                                                                                                                      |
| "I know it", "+ Deck", "Ignore" / « La conozco », « + Mazo », « Ignorar »                                                                                             | EN, ES            | `src/i18n/{en,es}/card.ts` (`known`, `addToDeck`, `ignore`)                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Alt+L captures a phrase with its sentence                                                                                                                             | EN, ES            | the command's description in `_locales/{en,es}/messages.json` (« Capturar la selección (palabra o expresión) con su frase »)                                                                                                                                                                                                                                                                                                                                                  |
| Review in a panel, beside the reading or in the side panel                                                                                                            | EN, ES            | `src/i18n/{en,es}/drawer.ts` ("Review", « Repaso »), `src/i18n/{en,es}/sidepanel.ts`                                                                                                                                                                                                                                                                                                                                                                                          |
| Spanish's and French's levels estimated from word frequency, no freely licensed CEFR list (M7)                                                                        | EN                | `levels_estimated` in every Spanish pack (`tables/es-en/NOTICE`) and every French one (change 46, D5; `reduce-fr-en.py`); the interface's own reason (`estimatedLevelsNote`, `src/i18n/en/languages.ts`), "Estimated French level" (`levelTitleEstimated`)                                                                                                                                                                                                                    |
| Statistics A1 to C2, MCER; English's levels not estimated, French's estimated                                                                                         | ES                | the scale's Spanish name (`levelScale`, `src/i18n/es/languages.ts`, M19); English's levels are CEFR-J's and Octanove's (`levels_estimated: false`, `reduce-en-es.py`); French's estimated (`reduce-fr-es.py`), « Nivel de francés estimado » (`levelTitleEstimated`)                                                                                                                                                                                                          |
| Books: DRM-free EPUB, offline, on the device, e-ink                                                                                                                   | EN, ES            | the books note (`booksNote`, `src/i18n/{en,es}/settings.ts`); the reader is every pair's (`src/reader/`)                                                                                                                                                                                                                                                                                                                                                                      |
| Local analysis; no request without an account or translation                                                                                                          | EN, ES            | as the French text (_Remote code_, _Data usage disclosures_)                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Extended translation straight from Spanish or French: 26.2 MB each                                                                                                    | EN                | routes `es-en` = `es-en/base-memory/2.0`, 23,288,494 + 2,543,246 + 409,312 B, and `fr-en` = `fr-en/base-memory/2.0`, 23,175,075 + 2,649,934 + 409,706 B = 26,234,715 B (`model-manifest.json`), "26.2 MB" each as the setting rounds them (`megabytes`, `src/reading/translation-setting.ts`); the setting adds the models of every language the reader studies (`modelsFor`, `src/translate/host/model-manifest.ts`, called by `model-controller.ts`): 52,475,767 B for both |
| Directly from English, through English from French: « 25,4 MB para el inglés, 51,6 MB con el francés »                                                                | ES with fr-es     | route `en-es` = `en-es/base-memory/2.1`, 22,698,792 + 2,265,250 + 409,312 = 25,373,354 B; route `fr-es` = fr-en 2.0 then en-es 2.1, 26,234,715 + 25,373,354 = 51,608,069 B — French needs both models whether or not English is studied, as Spanish does for a French speaker (« 52,0 Mo avec l'espagnol »)                                                                                                                                                                   |
| Extended translation on Chrome, Firefox for desktop and Safari; the models deleted when off                                                                           | EN, ES            | « Traducción ampliada » / "Extended translation" (`toggle`, `src/i18n/{en,es}/translation.ts`); the platforms are the French text's, the owner's wording                                                                                                                                                                                                                                                                                                                      |
| The coverage figures at cymbra.app/en/lingua and cymbra.app/es/lingua                                                                                                 | EN, ES            | change 52 writes fr-en's (and fr-es's) figures into `apps/site/src/data/lingua-coverage.json` (`gloss_coverage.py --write`); change 30's pages lead with the reader's pairs                                                                                                                                                                                                                                                                                                   |
| « Para el francés, es algo menos completo que para el inglés » (M6)                                                                                                   | ES with fr-es     | fr-es 83.2 / 70.8 / 56.8 % of the 5,000 / 10,000 / 20,000 commonest lemmas glossed (change 49) against en-es's 93.0 / 85.0 / 71.7 % (change 22), re-measured at change 52's commit — the French text's sentence about Spanish (es-fr 87.6 / 77.2 / 63.7 against en-fr 95.1 / 90.1 / 78.9), transposed. fr-en (93.6 / 86.9 / 76.3, change 48) stands beside es-en's 93.0 / 86.5 / 76.5: the English text says nothing of it                                                    |
| The account: optional, erasable from Settings without deleting it                                                                                                     | EN, ES            | "Manage my data" / « Gestionar mis datos » (`manageData`, `src/i18n/{en,es}/settings.ts`), "Erase my Lingua data…" / « Borrar mis datos de Lingua… » (`erase`, `src/i18n/{en,es}/account.ts`)                                                                                                                                                                                                                                                                                 |
| Who learns what: French speakers English and Spanish; English speakers Spanish and French; Spanish speakers English (and French)                                      | EN, ES            | `packs.json` after change 52: en-fr, es-fr, es-en, en-es, fr-en (, fr-es)                                                                                                                                                                                                                                                                                                                                                                                                     |
| Spanish stays the language an English reader starts in, English a Spanish reader's                                                                                    | EN, ES            | `defaultPair` is a native's first listed pair: change 52 lists fr-en after es-en and fr-es after en-es (add-lingua-french-listings, D12)                                                                                                                                                                                                                                                                                                                                      |
| No « Several languages at once »                                                                                                                                      | EN, ES            | a listing speaks of no price (add-lingua-french-listings, D2): the texts carry the French text's first sentence about the languages chosen in Settings, never a line about price — and the line itself has left the extension, in every interface language (change 53b)                                                                                                                                                                                                       |

M15 is settled: extended translation opens with each pair. Should change 52 not offer one, its
clause goes from each text and every count only falls; recount before pasting.

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

| Language | Build                                                                                                                        | Interface and page                                                                                                                    |
| -------- | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `fr`     | the released package                                                                                                         | French (« Analyser cette page »), an English article                                                                                  |
| `en`     | a build of change 34 (`packs.json` lists es-en), the browser in English; the fifth from a build of change 52 (fr-en)         | English ("Analyze this page"), a Spanish article, a Spanish level chosen; the fifth on a French article, a French level chosen        |
| `es`     | a build of change 35 (`packs.json` lists en-es), the browser in Spanish; the fifth, if change 52 lists fr-es, from its build | Spanish (« Analizar esta página »), an English article, an English level chosen; the fifth on a French article, a French level chosen |

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
5. **French** (from change 52) — a French article (a French Wikipedia page reads well), highlighted,
   the card open on « fut »: "third-person singular past historic (passé simple) of être"
   (`test/baseline/word-card-fr-en.txt`), its English gloss below.

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
5. **French** (if change 52 lists fr-es) — the same French page, the card open on « fut »:
   « tercera persona del singular del pretérito perfecto simple de indicativo de être »
   (`test/baseline/word-card-fr-es.txt`), its Spanish gloss below.

Where a dashboard takes screenshots per language, the `en` set goes under its English listing and
the `es` set under its Spanish one; where it keeps one set for every language, the owner chooses
which set it keeps. The Chrome Web Store takes five, so the French capture is the last it holds;
each is taken by the owner from a build of the release that ships it.

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
(`add-lingua-native-language-choice`, D4), and Spanish stays an English reader's first language
once change 52 lists fr-en after es-en — step 6 says how to add French and how to reach the Spanish
and French interfaces, with the Spanish labels of steps 1, 2 and 4. It is true from change 52 and
pasted with its release, the same text whether fr-es ships or not; until then the dashboards keep
the text change 35 pasted (this file's version before add-lingua-french-listings).

The field caps at 1000 characters; the text below is 998. Change 35's text measured 996, so the
French path — « "Languages studied" > "French" adds French pages; » — was paid for by cutting
wording, and no step a reviewer needs (add-lingua-french-listings, D4): « No account is needed:
signing in only syncs a reader's words » became « No account needed: signing in only syncs words »,
and step 6 lost « "¿Cuál es tu nivel de inglés?" or »: a reviewer who switches to Spanish in Settings
is past the welcome tab, and meets the popup's « Elige tu nivel de inglés », which stays. Every step
is still there: the level and why it matters, the two ways to highlight, the pill, the card and its
three actions, capture and review, French, and the way to each interface. Each line break counts
one character, as everywhere in this file: a dashboard that counted one as two would put the text at
1,005 — read its counter on paste.

```
No account needed: signing in only syncs words across devices. The interface follows the browser: French, Spanish, else English (studying Spanish).

1. Pick a level, e.g. B1, at "What's your Spanish level?" (welcome tab) or "Choose your Spanish level" (popup). With none, every word is highlighted and the score reads 0%.
2. Open a Spanish article. On Chrome, click "Analyze this page" in the popup, or grant "Always highlight (every page)".
3. Words above that level are highlighted; the pill shows the share of the page you know.
4. Click one for its card: English translation, dictionary form, frequency, and "I know it", "+ Deck", "Ignore".
5. Alt+L captures a selected phrase; Alt+Shift+S or the popup's "Review" opens the review panel.
6. Popup gear ("Settings") > "Language": "Languages studied" > "French" adds French pages; "Español" to study English, "Français" English and Spanish. In Spanish, steps 1-5 on English pages: "Elige tu nivel de inglés", "Analizar esta página", "La conozco".
```

Every label above is quoted from the source, not from memory: the level prompts from
`src/i18n/en/languages.ts` with the language "Spanish" (`levelQuestion`, the welcome tab's;
`chooseLevelPrompt`, the popup's); "Analyze this page", "Always highlight (every page)", "Review"
and "Settings" (the gear's label) from the popup's copy, `src/i18n/en/popup.ts`; the three word
actions from the word card's, `src/i18n/en/card.ts`; "Language", the settings tab, from
`src/i18n/en/settings.ts`. The Spanish ones from `src/i18n/es/`: « Elige tu nivel de inglés »
(`languages.ts`, `chooseLevelPrompt` with « de inglés »), « Analizar esta página » (`popup.ts`,
`analyse`), « La conozco » (`card.ts`, `known`). French's: "Languages studied" (`studiedLanguages`,
`src/i18n/en/settings.ts`), the box's "French" (`name`, `src/i18n/en/languages.ts`), shown once two
pairs are glossed in English (`offerFor`, `src/reading/studied-languages-view.ts`). « Español » and
« Français », the native-language choice's options, are each language's own name in every interface
(`ownNames`, `src/i18n/en/languages.ts`, read by `src/analyzer/language-labels.ts`). An approximate
label sends the reviewer looking for a control that does not exist.

## Single purpose (Chrome Web Store)

Cymbra Lingua has one purpose: helping a reader understand and learn the vocabulary of the language
they study — Spanish or French for English speakers, English or French for Spanish speakers, English
or Spanish for French speakers — in what they are already reading. Every feature serves it —
highlighting unknown words, showing a word's translation on click, capturing words and phrases into
a deck, and reviewing that deck — on the web page they are reading, and in the books they import
themselves (DRM-free EPUB files, read in the extension's own reader page, with the same
highlighting).

593 characters, from change 52 with fr-es. Without fr-es, « English or French for Spanish
speakers » reads « English for Spanish speakers » (583). The field is one per item, in English,
pasted with change 52's release; until then it is change 35's (« Spanish for English speakers,
English for Spanish speakers, English or Spanish for French speakers »).

## Permission justifications (Chrome Web Store)

Each answer names the user-visible feature and the code path, because that is what a reviewer checks
against the bundle. The fields are one per item, in English, whatever the listing's languages: a
label is quoted in each interface that shows it — English for English speakers, who study Spanish
and French; Spanish for Spanish speakers, who study English, and French if change 52 lists fr-es;
French for French speakers, who study English and Spanish. French studied adds no permission and no
label: its readers' interface is English or Spanish. Code paths are named by function, not by line,
so that they survive the next edit.

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
en-fr, es-fr; from changes 34 and 35, es-en and en-es; from change 52, fr-en, and fr-es if its
tables stand at or above their floor), and the book renderer (foliate-js,
vendored under `vendor/`). No script and no WebAssembly is fetched at runtime; the content security
policy is `script-src 'self' 'wasm-unsafe-eval'`. A book's own scripts never run: its pages are
rendered under that same policy (`test/reader-csp.spec.ts`).

The only files downloaded at runtime are **data**: the translation models' weights, the ones the
reader's pairs need (`model-manifest.json`, `routes`) — for a French speaker, en-fr, and es-en when
they study Spanish, since Spanish goes through English; for an English speaker reading Spanish,
es-en alone, the direct model of its pair; for a Spanish speaker reading English, en-es alone
(`en-es/base-memory/2.1`), the direct model of its pair; for an English speaker reading French,
fr-en alone (`fr-en/base-memory/2.0`), the direct model of its pair; for a Spanish speaker reading
French, fr-en then en-es, French going through English. They are fetched from
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

## The dashboards' languages: French studied (change 52's submission)

Change 52's package adds no language to what the dashboards read: it carries `_locales/fr`, `en`
and `es`, as change 35's did, `default_locale` still `en` (M13), with new `en` summaries — and new
`es` ones if it lists fr-es (_Summary_). What moves is each listing's text, so this submission is
made by hand too, in this order (the owner's, M18; task 6.2 of add-lingua-french-listings), after
the site deploy that publishes fr-en's (and fr-es's) figures:

1. **Upload without publishing**, as change 34's step 1: the tag's
   `cymbra-lingua-chromium-<version>.zip`, from its GitHub Release, uploaded through the dashboard
   (Package › Upload new package), which submits nothing.
2. **`fr` checked.** The default language still `en`; the French listing's text as above (its
   « Plusieurs langues à la fois… » line gone) and its summary unchanged.
3. **`en` and `es` replaced.** Under `en`: the English description above and the fifth `en`
   screenshot (_Graphics_). Under `es`: the Spanish description in the variant that matches the
   release — with fr-es or without — and, with fr-es, the fifth `es` screenshot. Check that each
   summary reads as quoted in _Summary_ for that release.
4. **The single fields replaced.** The single purpose (the variant that matches the release), the
   permission justifications, the remote-code answer and the test instructions as above.
5. **Submit for review** from the dashboard.
6. **addons.mozilla.org**: dispatch `lingua-extension-release` with the tag, `publish` ticked and
   `stores: firefox`. After the submission, paste the English description into the English (US)
   locale and the Spanish one into the Spanish locale change 35's procedure recorded, and check
   both summaries.
7. **Record both results below**, in the pull request that next touches this file. Read each
   dashboard's own counter as the text is pasted: every count in this file takes a line break as one
   character.

| Dashboard          | Checked on  | Default language after upload | Summaries (en, es) | Listing texts (en, es; the variant pasted) | Notes |
| ------------------ | ----------- | ----------------------------- | ------------------ | ------------------------------------------ | ----- |
| Chrome Web Store   | _to record_ | _to record_                   | _to record_        | _to record_                                |       |
| addons.mozilla.org | _to record_ | _to record_                   | _to record_        | _to record_                                |       |
