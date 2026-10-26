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
French speakers, who study English and Spanish, and Italian from the Italian programme's change 20
(it-fr, `enable-lingua-italian`); English, for English speakers, who study Spanish
(es-en, from change 34) and French (fr-en, from change 52); Spanish, for Spanish speakers, who study
English (en-es, from change 35) and French if change 52 lists fr-es (M6). A French speaker cannot
study French, so no French text offers it. Only French speakers study Italian: the English and
Spanish texts say so in one sentence and offer it to no reader of theirs (add-lingua-italian-listings,
D4); Italian adds no listing language. The interface speaks the reader's native language, so
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

**From the Italian programme's change 20** (`enable-lingua-italian`), which lists it-fr, the French
summary names Italian: « Lisez l'anglais et l'espagnol » no longer says what a French speaker
studies, and `yarn check:version` fails it once it-fr is listed. Change 20 commits it to
`_locales/fr/messages.json` and `manifest.json`'s French literal in **the pull request that lists
it-fr**, not before, as change 52 committed the English and Spanish ones. Its draft A (its D7 and
Open Question 3, the owner's words, M9), settled by the owner on 2026-10-25, 107 characters:

> Le web en anglais, espagnol ou italien : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé.

Change 20's other draft — « Lisez l'anglais, l'espagnol et l'italien sur le web : mots inconnus
surlignés, pourcentage honnête. Hors ligne. » (111) — was not chosen; should change 20 commit other
words, this quote follows what it commits. The English and Spanish summaries do not move with Italian: no pair
glossed in English or Spanish studies it.

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
Spanish, and Italian from the Italian programme's change 20; English speakers Spanish (change 34) and
French (change 52); Spanish speakers English
(change 35), and French if change 52 lists fr-es. Every claim in the English and Spanish texts below
is one change 52 ships; they are pasted with its release (M18), after the site deploy that publishes
fr-en's (and fr-es's) figures, under the dashboards' `en` and `es` (see _The dashboards' languages:
French studied_), and the owner reviews their wording (M9; M10: US English; tú, neutral Spanish, no
vosotros, the RAE's numbers). Each claim and the file that makes it true are in one table, after the
texts. Should change 34 or 35 be released before change 52, its text is this file's version before
add-lingua-french-listings — the English one naming Spanish alone, the Spanish one English alone.
Each text's count is its length as pasted — paragraphs a blank line apart, each paragraph's lines
joined, the bold marks dropped — re-measured by script at add-site-lingua-showcase (the counts
before it predated the « Traduction en ligne » paragraph).

The Spanish text is written twice (add-lingua-french-listings, D1): **with fr-es**, French named
everywhere a Spanish reader's languages are; **without fr-es**, change 37's text with its last line
alone made true (English speakers study Spanish and French). The owner pastes the variant that
matches the release. Change 52 listed fr-es (`packs.json`): the variant without it is history, kept
as written and not edited since. The French text names no other audience; add-lingua-french-listings
changed one line of it: « Plusieurs langues à la fois… » is gone, by the owner's decision of
2026-10-10 (no listing text speaks of price; change 53b took the line out of the extension and the
site).

Each text's local-analysis paragraph carries the sentence of add-lingua-usage-counts (design Q5,
accepted by the owner on 2026-10-14) where it promised that, with no account and no extended
translation, nothing reached the network — the anonymous counts make that false — and
its account paragraph names the counts beside « Traduction en ligne ». The four counts were
re-measured with it, the same way.

**Italian studied** (add-lingua-italian-listings, the Italian programme's change 21). The French
text names Italian beside English and Spanish (D3): the languages highlighted, the card's tense and
gender, the route through English and its sizes, the dictionary sentence. The English text and the
Spanish one with fr-es say, in one sentence each, that French speakers study Italian too (D4). Each
Italian claim is one change 20 (`enable-lingua-italian`) ships, in the claims table below; the texts
are pasted with its release, after the site deploy that publishes it-fr's figures (I12, M18), and
the owner reviews their wording (M9). Until then — the next submission included, which comes
before change 20's — the French text to paste is **FR, current (until Italian ships)**, kept below
word for word, and the English and Spanish texts' next-to-last lines read as each one's heading
says. The figures are `main`'s when the texts were written (D6): it-en's served
size from the model catalogue (change 2), it-fr's committed coverage (change 16) and its marks
(change 19: 93 of 97 right, 3 % withheld — the first tier, so it-fr's translation is offered and
marked, and change 20's Open Question 2 does not arise). Two French sentences hang on a figure, and
each is written to be swapped whole: should change 20 not offer it-fr's translation, the
**Traduction étendue** paragraph reads « l'espagnol passe par l'anglais » and « (25,8 Mo pour
l'anglais, 52,0 Mo avec l'espagnol) » again; should the figures change 20 publishes not stand under
en-fr's at each top, the dictionary sentence reads « Pour l'espagnol, … » again.

**FR, current (until Italian ships)** — for French speakers, who study English and Spanish
(2,835 / 16,000 characters). The text to paste until change 20's release.

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

**L'analyse est locale.** Les dictionnaires et le moteur tournent dans votre navigateur, et
l'extension fonctionne hors ligne. Ce que vous lisez ne quitte jamais votre appareil. Lingua peut
envoyer des comptages anonymes — installation, premiers pas, retours —, sans aucun identifiant ; un
interrupteur dans les Réglages les coupe.

**Traduction étendue** (facultative, désactivée par défaut, sur Chrome, Firefox (ordinateur et
Android) et Safari) : votre sélection est traduite dans sa phrase, sur votre appareil, par le moteur de
Firefox Translations ; l'espagnol passe par l'anglais. L'activer télécharge une fois les modèles de
traduction depuis Cymbra (25,8 Mo pour l'anglais, 52,0 Mo avec l'espagnol) ; le texte des pages ne
quitte toujours pas votre appareil. La désactiver supprime les modèles.

Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos
chiffres sont publiés sur cymbra.app/lingua.

Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur
vos autres appareils. Avec la Traduction en ligne ci-dessous et les comptages anonymes, c'est la
seule chose qui quitte votre machine, et vous pouvez effacer ces données depuis les Réglages sans
supprimer votre compte.

**Traduction en ligne** (désactivée par défaut, pour les comptes qui y ont accès) : un bouton
envoie à un service de traduction en ligne la phrase que vous voulez mieux traduire, seulement quand
vous appuyez dessus ; personne ne la conserve.

Cymbra Lingua est disponible sur Chrome, sur Firefox (ordinateur et Android) et dans l'app Safari
(iPhone, iPad, Mac).

**FR, with Italian** — for French speakers, who study English and Spanish, and Italian (it-fr, from
the Italian programme's change 20); pasted with change 20's release (2,924 / 16,000 characters)

Cymbra Lingua surligne, sur la page que vous lisez, les mots d'anglais, d'espagnol ou d'italien que
vous ne connaissez pas encore — sans rien changer à la mise en page. Un pourcentage vous dit quelle
part du texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une
estimation.

Choisissez dans les Réglages les langues que vous apprenez : chaque page est lue dans la sienne.

Cliquez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté dans l'usage courant.
En espagnol et en italien, la carte nomme aussi le temps et le genre, comme on les apprend en classe
(« passé simple », « nom féminin »). Puis décidez : « Je connais », « + Deck » pour le réviser plus
tard, ou « Ignorer ». Sélectionnez plusieurs mots et appuyez sur Alt+L pour capturer une expression
entière avec sa phrase.

Les cartes que vous créez se révisent dans un panneau, à côté de votre lecture ou dans la barre
latérale, avec une répétition espacée qui décide toute seule du bon moment.

Lisez aussi vos livres : importez vos fichiers EPUB sans DRM dans la bibliothèque de
l'extension et lisez-les, hors ligne, avec le même surlignage, page par page — une tablette à
encre électronique comprise. Vos livres restent sur votre appareil.

**L'analyse est locale.** Les dictionnaires et le moteur tournent dans votre navigateur, et
l'extension fonctionne hors ligne. Ce que vous lisez ne quitte jamais votre appareil. Lingua peut
envoyer des comptages anonymes — installation, premiers pas, retours —, sans aucun identifiant ; un
interrupteur dans les Réglages les coupe.

**Traduction étendue** (facultative, désactivée par défaut, sur Chrome, Firefox (ordinateur et
Android) et Safari) : votre sélection est traduite dans sa phrase, sur votre appareil, par le moteur de
Firefox Translations ; l'espagnol et l'italien passent par l'anglais. L'activer télécharge une fois
les modèles de traduction depuis Cymbra (25,8 Mo pour l'anglais, 52,0 Mo avec l'espagnol ou
l'italien, 78,2 Mo avec les deux) ; le texte des pages ne quitte toujours pas votre appareil. La
désactiver supprime les modèles.

Pour l'espagnol et l'italien, le dictionnaire français est un peu moins complet que pour l'anglais :
nos chiffres sont publiés sur cymbra.app/lingua.

Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur
vos autres appareils. Avec la Traduction en ligne ci-dessous et les comptages anonymes, c'est la
seule chose qui quitte votre machine, et vous pouvez effacer ces données depuis les Réglages sans
supprimer votre compte.

**Traduction en ligne** (désactivée par défaut, pour les comptes qui y ont accès) : un bouton
envoie à un service de traduction en ligne la phrase que vous voulez mieux traduire, seulement quand
vous appuyez dessus ; personne ne la conserve.

Cymbra Lingua est disponible sur Chrome, sur Firefox (ordinateur et Android) et dans l'app Safari
(iPhone, iPad, Mac).

**EN** — for English speakers, who study Spanish (es-en, from change 34) and French (fr-en, from
change 52) (2,912 / 16,000 characters with Italian). Its next-to-last line names Italian for French
speakers, from change 20's release; until then (current, 2,903) it reads « The interface is in
English. Cymbra Lingua also teaches English and Spanish to French speakers, in French. »

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

**The analysis is local.** The dictionaries and the engine run in your browser, and the extension
works offline. What you read never leaves your device. Lingua may send anonymous counts — install,
first steps, returns — with no identifier of any kind; one switch in Settings turns them off.

**Extended translation** (optional, off by default, on Chrome, Firefox (desktop and Android), and
Safari):
your selection is translated into English within its sentence, straight from Spanish or French, on
your device, by the Firefox Translations engine. Turning it on downloads from Cymbra, once, the
translation model of each language you study (26.2 MB each); the text of the pages you read still
never leaves your device. Turning it off deletes the models.

How many Spanish and French words our dictionary explains in English: our figures are published at
cymbra.app/en/lingua.

Create a Cymbra account if — and only if — you want your words and cards on your other devices. With
Online translation below and the anonymous counts, that is the only thing that leaves your machine,
and you can erase it from Settings without deleting your account.

**Online translation** (off by default, for accounts that have access): a button sends the sentence
you want translated better to an online translation service, only when you press it; nobody keeps
it.

The interface is in English. Cymbra Lingua also teaches English, Spanish and Italian to French
speakers, in French.

Cymbra Lingua is available on Chrome, on Firefox (desktop and Android) and as a Safari app (iPhone,
iPad, Mac).

**ES, with fr-es** — for Spanish speakers, who study English (en-es, from change 35) and French
(fr-es, from change 52) (3,426 / 16,000 characters with Italian). Its next-to-last line names
Italian for French speakers, from change 20's release; until then (current, 3,416) it reads « La
interfaz está en español. Cymbra Lingua también enseña inglés y español a los francohablantes, en
francés, y español y francés a los anglohablantes, en inglés. »

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

**El análisis es local.** Los diccionarios y el motor se ejecutan en tu navegador, y la extensión
funciona sin conexión. Lo que lees nunca sale de tu dispositivo. Lingua puede enviar recuentos
anónimos —instalación, primeros pasos, regresos—, sin ningún identificador; un interruptor en los
Ajustes los desactiva.

**Traducción ampliada** (opcional, desactivada por defecto, en Chrome, Firefox (escritorio y
Android) y Safari): tu selección se traduce al español dentro de su frase, directamente desde el inglés y,
desde el francés, pasando por el inglés, en tu dispositivo, con el motor de Firefox Translations. Al
activarla se descargan una vez los modelos de traducción desde Cymbra (25,4 MB para el inglés, 51,6
MB con el francés); el texto de las páginas que lees sigue sin salir de tu dispositivo. Al
desactivarla se eliminan los modelos.

Cuántas palabras inglesas y francesas explica nuestro diccionario en español: nuestras cifras están
publicadas en cymbra.app/es/lingua. Para el francés, el diccionario es algo menos completo que para
el inglés.

Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros
dispositivos. Con la Traducción en línea de más abajo y los recuentos anónimos, es lo único que sale
de tu dispositivo, y puedes borrar esos datos desde los Ajustes sin eliminar tu cuenta.

**Traducción en línea** (desactivada por defecto, para las cuentas que tienen acceso): un botón envía
a un servicio de traducción en línea la frase que quieres traducir mejor, solo cuando lo pulsas;
nadie la conserva.

La interfaz está en español. Cymbra Lingua también enseña inglés, español e italiano a los
francohablantes, en francés, y español y francés a los anglohablantes, en inglés.

Cymbra Lingua está disponible en Chrome, en Firefox (escritorio y Android) y como app de Safari
(iPhone, iPad, Mac).

**ES, without fr-es** — history: change 52 listed fr-es, so this variant is a record, kept as
written and not edited for Italian. For Spanish speakers, who study English (en-es, from change 35),
if change 52 listed fr-en alone: change 37's text, its last line naming what English speakers study
(2,940 / 16,000 characters)

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

**El análisis es local.** Los diccionarios y el motor se ejecutan en tu navegador, y la extensión
funciona sin conexión. Lo que lees nunca sale de tu dispositivo. Lingua puede enviar recuentos
anónimos —instalación, primeros pasos, regresos—, sin ningún identificador; un interruptor en los
Ajustes los desactiva.

**Traducción ampliada** (opcional, desactivada por defecto, en Chrome, Firefox (escritorio y
Android) y Safari): tu selección se traduce al español dentro de su frase, directamente desde el inglés, en tu
dispositivo, con el motor de Firefox Translations. Al activarla se descarga una vez el modelo de
traducción desde Cymbra (25,4 MB); el texto de las páginas que lees sigue sin salir de tu
dispositivo. Al desactivarla se elimina el modelo.

Cuántas palabras inglesas explica nuestro diccionario en español: nuestras cifras están publicadas
en cymbra.app/es/lingua.

Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros
dispositivos. Con la Traducción en línea de más abajo y los recuentos anónimos, es lo único que sale
de tu dispositivo, y puedes borrar esos datos desde los Ajustes sin eliminar tu cuenta.

**Traducción en línea** (desactivada por defecto, para las cuentas que tienen acceso): un botón envía
a un servicio de traducción en línea la frase que quieres traducir mejor, solo cuando lo pulsas;
nadie la conserva.

La interfaz está en español. Cymbra Lingua también enseña inglés y español a los francohablantes, en
francés, y español y francés a los anglohablantes, en inglés.

Cymbra Lingua está disponible en Chrome, en Firefox (escritorio y Android) y como app de Safari
(iPhone, iPad, Mac).

What the English and Spanish texts check against, so that they promise nothing the packages do not
do — each claim and the file that makes it true once change 52 lists fr-en (and fr-es) in
`packs.json`, after es-en and en-es; and the French text's Italian claims (add-lingua-italian-listings,
D4), true once the Italian programme's change 20 lists it-fr after them — the rows whose text is FR,
the App Store's fr-FR texts' among them:

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
| Local analysis; what is read never leaves the device; anonymous counts, no identifier, one switch in Settings                                                         | EN, ES            | as the French text (_Remote code_, _Data usage disclosures_); the counts and their switch, "Anonymous usage counts" / « Recuentos anónimos » in Settings → Data (`src/usage/`, `src/i18n/{en,es}/settings.ts`; add-lingua-usage-counts)                                                                                                                                                                                                                                       |
| Extended translation straight from Spanish or French: 26.2 MB each                                                                                                    | EN                | routes `es-en` = `es-en/base-memory/2.0`, 23,288,494 + 2,543,246 + 409,312 B, and `fr-en` = `fr-en/base-memory/2.0`, 23,175,075 + 2,649,934 + 409,706 B = 26,234,715 B (`model-manifest.json`), "26.2 MB" each as the setting rounds them (`megabytes`, `src/reading/translation-setting.ts`); the setting adds the models of every language the reader studies (`modelsFor`, `src/translate/host/model-manifest.ts`, called by `model-controller.ts`): 52,475,767 B for both |
| Directly from English, through English from French: « 25,4 MB para el inglés, 51,6 MB con el francés »                                                                | ES with fr-es     | route `en-es` = `en-es/base-memory/2.1`, 22,698,792 + 2,265,250 + 409,312 = 25,373,354 B; route `fr-es` = fr-en 2.0 then en-es 2.1, 26,234,715 + 25,373,354 = 51,608,069 B — French needs both models whether or not English is studied, as Spanish does for a French speaker (« 52,0 Mo avec l'espagnol »)                                                                                                                                                                   |
| Extended translation on Chrome, Firefox (desktop and Android) and Safari; the models deleted when off                                                                 | EN, ES            | « Traducción ampliada » / "Extended translation" (`toggle`, `src/i18n/{en,es}/translation.ts`); the platforms are the French text's, the owner's wording                                                                                                                                                                                                                                                                                                                      |
| Available on Firefox for Android                                                                                                                                      | FR, EN, ES        | the Firefox package declares `browser_specific_settings.gecko_android` (`firefoxManifest`, `tool/manifests.mjs`, since #599): addons.mozilla.org lists the one zip for desktop and Android; extended translation runs in its event page there too (`TRANSLATION.md`)                                                                                                                                                                                                          |
| The coverage figures at cymbra.app/en/lingua and cymbra.app/es/lingua                                                                                                 | EN, ES            | change 52 writes fr-en's (and fr-es's) figures into `apps/site/src/data/lingua-coverage.json` (`gloss_coverage.py --write`); change 30's pages lead with the reader's pairs                                                                                                                                                                                                                                                                                                   |
| « Para el francés, el diccionario es algo menos completo que para el inglés » (M6)                                                                                    | ES with fr-es     | fr-es 83.2 / 70.8 / 56.8 % of the 5,000 / 10,000 / 20,000 commonest lemmas glossed (change 49) against en-es's 93.0 / 85.0 / 71.7 % (change 22), re-measured at change 52's commit — the French text's sentence about Spanish (es-fr 87.6 / 77.2 / 63.7 against en-fr 95.1 / 90.1 / 78.9), transposed. fr-en (93.6 / 86.9 / 76.3, change 48) stands beside es-en's 93.0 / 86.5 / 76.5: the English text says nothing of it                                                    |
| The account: optional, erasable from Settings without deleting it                                                                                                     | EN, ES            | "Manage my data" / « Gestionar mis datos » (`manageData`, `src/i18n/{en,es}/settings.ts`), "Erase my Lingua data…" / « Borrar mis datos de Lingua… » (`erase`, `src/i18n/{en,es}/account.ts`)                                                                                                                                                                                                                                                                                 |
| Who learns what: French speakers English and Spanish; English speakers Spanish and French; Spanish speakers English (and French)                                      | EN, ES            | `packs.json` after change 52: en-fr, es-fr, es-en, en-es, fr-en (, fr-es)                                                                                                                                                                                                                                                                                                                                                                                                     |
| French speakers study Italian too: "English, Spanish and Italian to French speakers" / « inglés, español e italiano a los francohablantes »                           | EN, ES            | `packs.json` after change 20: en-fr, es-fr, es-en, en-es, fr-en, fr-es, it-fr — it-fr glossed in French; no pair glossed in English or Spanish studies Italian (it-en and it-es are the Italian programme's stage 2), so neither text offers it to its readers                                                                                                                                                                                                                |
| Spanish stays the language an English reader starts in, English a Spanish reader's                                                                                    | EN, ES            | `defaultPair` is a native's first listed pair: change 52 lists fr-en after es-en and fr-es after en-es (add-lingua-french-listings, D12)                                                                                                                                                                                                                                                                                                                                      |
| No « Several languages at once »                                                                                                                                      | EN, ES            | a listing speaks of no price (add-lingua-french-listings, D2): the texts carry the French text's first sentence about the languages chosen in Settings, never a line about price — and the line itself has left the extension, in every interface language (change 53b)                                                                                                                                                                                                       |
| « les mots d'anglais, d'espagnol ou d'italien »: Italian highlighted for French speakers                                                                              | FR                | a French-native reader's pairs are en-fr, es-fr and it-fr once change 20 lists it-fr (`pairsOf("fr")`, `src/analyzer/pairs.ts`), Italian offered in Réglages › Langue after English and Spanish (`offerFor`, `src/reading/studied-languages-view.ts`); Italian tokenised, analysed and guarded from its neighbours by the Italian programme's changes 6–8                                                                                                                     |
| « En espagnol et en italien, la carte nomme aussi le temps et le genre… (« passé simple », « nom féminin ») »                                                         | FR                | the French renderer's Italian tenses (`TENSES.it`, `src/i18n/fr/grammar.ts`, change 18), pinned by `test/baseline/word-card-it-fr.txt` (« fu »: « 3e personne du singulier du passé simple d'essere »; « legge »: « [nom féminin] Loi »); Italian's genders and tenses from `tables/it/grammar.tsv` (change 13)                                                                                                                                                               |
| « l'espagnol et l'italien passent par l'anglais »; « 52,0 Mo avec l'espagnol ou l'italien, 78,2 Mo avec les deux »                                                    | FR                | route `it-fr` = `it-en/base-memory/2.0` then `en-fr/base-memory/2.0` (`model-manifest.json`, change 2): it-en 23,301,774 + 2,532,929 + 407,598 = 26,242,301 B, with en-fr's 25,752,472 B = 51,994,773 B, « 52,0 Mo » as es-fr's 51,993,524 B; Spanish and Italian studied (with or without English): en-fr + es-en + it-en = 78,235,825 B, « 78,2 Mo » — the setting takes each model once (`modelsFor`) and rounds as the texts do (`megabytes`)                             |
| Italian's translation offered, the selection marked in it                                                                                                             | FR                | it-fr in `MARKED_PAIRS` (`src/translate/markup.ts`): 93 of its 97 shown marks right, 3 % withheld, the programme's first tier (change 19, `measure-lingua-it-fr-translation-marks`; `tool/marks/README.md`)                                                                                                                                                                                                                                                                   |
| « Pour l'espagnol et l'italien, le dictionnaire français est un peu moins complet que pour l'anglais » (M6)                                                           | FR                | it-fr 89.4 / 82.0 / 73.4 % of the 5,000 / 10,000 / 20,000 commonest lemmas glossed (change 16, refreshed by change 17, #1062; `gloss_coverage.py --pair it-fr`, `tables/it-fr/README.md`) against en-fr's 95.1 / 90.1 / 78.9 %: under it at each top, as es-fr's 87.6 / 77.2 / 63.7 %; change 20 publishes the figures (`gloss_coverage.py --write`) on cymbra.app/lingua                                                                                                     |
| Italian's levels estimated from word frequency (the App Store's fr-FR RÉVISER)                                                                                        | FR                | `levels_estimated` in it-fr's pack (`tables/it-fr/manifest.json`, change 14), « Niveau d'italien estimé » (`levelTitleEstimated`, `src/i18n/fr/languages.ts`)                                                                                                                                                                                                                                                                                                                 |
| « Pour l'espagnol ou l'italien, cochez-le dans les Réglages de l'extension, onglet Langue, puis choisissez son niveau. » (the App Store's fr-FR step 3)               | FR                | change 20's activation page, `fr.step3` (`apps/lingua-apple/Shared (App)/Resources/copy.js`, its D7 draft), in the description's register: vous where the page says tu                                                                                                                                                                                                                                                                                                        |
| « une voix d'Italie pour la lecture à voix haute » (the App Store's fr-FR « What's New »)                                                                             | FR                | `PREFERRED_REGIONS.it` (`src/reading/speech.ts`, change 15): a voice of Italy ranked before another region's of the same quality                                                                                                                                                                                                                                                                                                                                              |

M15 is settled: extended translation opens with each pair. Should change 52 not offer one, its
clause goes from each text and every count only falls; recount before pasting.

---

## Graphics

| Asset            | Requirement                                                                                            | Where it is                                                                 |
| ---------------- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| Store icon       | 128×128                                                                                                | `icons/icon-128.png`                                                        |
| Screenshots      | 1280×800 or 640×400, JPEG or 24-bit PNG **without alpha**, at least one (five on the Chrome Web Store) | `store-captures/<lang>/1-card.png` … `5-reader.png`, made by `yarn capture` |
| Small promo tile | 440×280, no alpha                                                                                      | `store-captures/<lang>/promo-tile-440x280.png` (optional)                   |
| Marquee          | 1400×560, no alpha                                                                                     | optional                                                                    |

The screenshots come from the repository (add-site-lingua-showcase): `yarn capture` (README,
"Captures") builds the Chromium package with the reader declared on every page, loads it in Chrome
for Testing, restores a reader's state the extension itself recorded, and captures the real
extension on the articles and books committed under `tool/capture/fixtures/` — one set per listing
language, each in its interface language. Prerequisites: `yarn gen:wasm`, `yarn gen:pack:real`,
`yarn fetch:engine`, ffmpeg. The files land in `store-captures/<lang>/`, git-ignored: the owner
uploads them by hand, under each listing language on the Chrome Web Store, and the same files to
addons.mozilla.org (1280×800 is its recommended size). The tool refuses a PNG with an alpha channel,
naming it. The App Store's are not these: see `apps/lingua-apple/STORE-LISTING.md`.

Each set holds the same five, in this order:

| #   | File           | What it shows                                                                                                                                            |
| --- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `1-card.png`   | The first studied language's article, highlighted, the pill showing the share of the page known, the word card open on a verb whose card names its tense |
| 2   | `2-second.png` | The second studied language's article, highlighted, a noun's card open with its gender                                                                   |
| 3   | `3-review.png` | The first article with the review panel the extension draws in the page, on a revealed card: its sentence, its translation and the review's two answers  |
| 4   | `4-stats.png`  | The statistics page: the estimated vocabulary and the ladder, A1 to C2                                                                                   |
| 5   | `5-reader.png` | The EPUB reader on a public-domain chapter in the first studied language, highlighted                                                                    |

| Set  | Interface | Studied, first then second | The review's two answers (`src/i18n/<lang>/review.ts`) | The book                         |
| ---- | --------- | -------------------------- | ------------------------------------------------------ | -------------------------------- |
| `fr` | French    | English, then Spanish      | « Pas su » and « Su »                                  | _Pride and Prejudice_, chapter I |
| `en` | English   | Spanish, then French       | "Didn't know" and "Knew it"                            | _Marianela_, chapter I           |
| `es` | Spanish   | English, then French       | « No la sabía » and « La sabía »                       | _Pride and Prejudice_, chapter I |

The review shot is the panel the extension draws in the page — Firefox's only review panel, and
Chrome's on Alt+Shift+D — rather than Chrome's side panel, which is browser UI a page capture
cannot show; both hold the same card and the same two answers. The fifth slot is the book reader:
the second slot already shows the second studied language. The site shows these captures too, with
the reader in « E-ink contrasté » beside the reader; the stores do not carry that one.

Italian studied adds no capture (add-lingua-italian-listings, D8): the `fr` set is at the Chrome Web
Store's cap of five, its second capture on a Spanish page; moving that one to an Italian page would
need Italian capture fixtures and move the site's alt texts (that change's Open Question 3, settled
by the owner on 2026-10-25: the `fr` set unchanged). The App
Store's fr-FR set gains one on an Italian page (`apps/lingua-apple/STORE-LISTING.md`).

Where a dashboard takes screenshots per language, the `fr` set goes under the French listing, the
`en` set under the English one and the `es` set under the Spanish one; where it keeps one set for
every language, the owner chooses which set it keeps.

## Extra fields

| Field          | Value                                                                                                   |
| -------------- | ------------------------------------------------------------------------------------------------------- |
| Homepage URL   | `https://cymbra.app/lingua/` (EN: `https://cymbra.app/en/lingua/`, ES: `https://cymbra.app/es/lingua/`) |
| Support URL    | `https://cymbra.app/support/`                                                                           |
| Mature content | no                                                                                                      |

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
and French interfaces, with the Spanish labels of steps 1, 2 and 4, and, from the Italian
programme's change 20, that the French interface studies Italian too. Two texts follow: **current
(until Italian ships)**, true today and pasted at every submission before change 20's, and **with
Italian**, pasted with change 20's release.

The field caps at 1000 characters; the current text is 998, the one with Italian 993. Change 35's
text measured 996, so the
French path — « "Languages studied" > "French" adds French pages; » — was paid for by cutting
wording, and no step a reviewer needs (add-lingua-french-listings, D4): « No account is needed:
signing in only syncs a reader's words » became « No account needed: signing in only syncs words »,
and step 6 lost « "¿Cuál es tu nivel de inglés?" or »: a reviewer who switches to Spanish in Settings
is past the welcome tab, and meets the popup's « Elige tu nivel de inglés », which stays. Every step
is still there: the level and why it matters, the two ways to highlight, the pill, the card and its
three actions, capture and review, French, and the way to each interface. Each line break counts
one character, as everywhere in this file: a dashboard that counted one as two would put the text at
1,005 — read its counter on paste.

Italian (add-lingua-italian-listings, D5) is « "Français" English, Spanish and Italian » in step 6
(+9), paid for by cutting wording, and no step a reviewer needs: step 1's « Pick a level, e.g.
B1, at » became « Pick a level (B1) at », and « With none, every word is highlighted and the score
reads 0%. » became « With none, every word is highlighted, the score 0%. » (−14). The text with
Italian is 993 / 1,000 — 1,000 if a dashboard counted each line break as two.

**Current (until Italian ships)** (998 / 1,000):

```
No account needed: signing in only syncs words across devices. The interface follows the browser: French, Spanish, else English (studying Spanish).

1. Pick a level, e.g. B1, at "What's your Spanish level?" (welcome tab) or "Choose your Spanish level" (popup). With none, every word is highlighted and the score reads 0%.
2. Open a Spanish article. On Chrome, click "Analyze this page" in the popup, or grant "Always highlight (every page)".
3. Words above that level are highlighted; the pill shows the share of the page you know.
4. Click one for its card: English translation, dictionary form, frequency, and "I know it", "+ Deck", "Ignore".
5. Alt+L captures a selected phrase; Alt+Shift+S or the popup's "Review" opens the review panel.
6. Popup gear ("Settings") > "Language": "Languages studied" > "French" adds French pages; "Español" to study English, "Français" English and Spanish. In Spanish, steps 1-5 on English pages: "Elige tu nivel de inglés", "Analizar esta página", "La conozco".
```

**With Italian** (993 / 1,000; from change 20's release):

```
No account needed: signing in only syncs words across devices. The interface follows the browser: French, Spanish, else English (studying Spanish).

1. Pick a level (B1) at "What's your Spanish level?" (welcome tab) or "Choose your Spanish level" (popup). With none, every word is highlighted, the score 0%.
2. Open a Spanish article. On Chrome, click "Analyze this page" in the popup, or grant "Always highlight (every page)".
3. Words above that level are highlighted; the pill shows the share of the page you know.
4. Click one for its card: English translation, dictionary form, frequency, and "I know it", "+ Deck", "Ignore".
5. Alt+L captures a selected phrase; Alt+Shift+S or the popup's "Review" opens the review panel.
6. Popup gear ("Settings") > "Language": "Languages studied" > "French" adds French pages; "Español" to study English, "Français" English, Spanish and Italian. In Spanish, steps 1-5 on English pages: "Elige tu nivel de inglés", "Analizar esta página", "La conozco".
```

Every label above is quoted from the source, not from memory (both texts): the level prompts from
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

**Current (until Italian ships)** (593 characters):

Cymbra Lingua has one purpose: helping a reader understand and learn the vocabulary of the language
they study — Spanish or French for English speakers, English or French for Spanish speakers, English
or Spanish for French speakers — in what they are already reading. Every feature serves it —
highlighting unknown words, showing a word's translation on click, capturing words and phrases into
a deck, and reviewing that deck — on the web page they are reading, and in the books they import
themselves (DRM-free EPUB files, read in the extension's own reader page, with the same
highlighting).

**With Italian** (602 characters; from change 20's release):

Cymbra Lingua has one purpose: helping a reader understand and learn the vocabulary of the language
they study — Spanish or French for English speakers, English or French for Spanish speakers, English,
Spanish or Italian for French speakers — in what they are already reading. Every feature serves it —
highlighting unknown words, showing a word's translation on click, capturing words and phrases into
a deck, and reviewing that deck — on the web page they are reading, and in the books they import
themselves (DRM-free EPUB files, read in the extension's own reader page, with the same
highlighting).

The text with Italian is 602 characters, from the Italian programme's change 20, which lists it-fr
(add-lingua-italian-listings, D5). The field is one per item, in English, pasted with change 20's
release; until then it is the current one, change 52's, « English or Spanish for French speakers »
(593). Change 52's variant without fr-es,
« English for Spanish speakers » (583), is history: change 52 listed fr-es. Before change 52, the
field was change 35's (« Spanish for English speakers, English for Spanish speakers, English or
Spanish for French speakers »).

## Permission justifications (Chrome Web Store)

Each answer names the user-visible feature and the code path, because that is what a reviewer checks
against the bundle. The fields are one per item, in English, whatever the listing's languages: a
label is quoted in each interface that shows it — English for English speakers, who study Spanish
and French; Spanish for Spanish speakers, who study English, and French if change 52 lists fr-es;
French for French speakers, who study English and Spanish, and Italian from the Italian programme's
change 20. French studied adds no permission and no label: its readers' interface is English or
Spanish. Nor does Italian: its readers' interface is French, whose labels are quoted already. Code
paths are named by function, not by line, so that they survive the next edit.

The book reader (`reader.html`, `src/reader/`) **added no permission**: the file picker, the
IndexedDB database that keeps the books and an extension page of its own need none, and nothing
is injected into it. `tool/check_variants.mjs` fails a build whose manifest asks for more.

**`activeTab`** — The reader is injected only into the tab the user is looking at, when they click
"Analyze this page" (« Analizar esta página » in Spanish, « Analyser cette page » in French) in the
toolbar popup. This is the extension's default mode on Chrome: without it, the extension cannot read
the page it was asked to analyze. `src/popup/popup.ts` (`activeTabId`, then the injection below).

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

**`https://api.cymbra.app/*` (host permission)** — For the optional account: signing in,
then what a signed-in reader synchronizes across their own devices — word statuses, declared levels, cards
with their review history, and daily learning statistics (`SyncEngine`, `src/sync/sync.ts`).
And, only once the reader has pressed "Yes, count" (« Sí, contar » in Spanish, « Oui, compter » in
French), the anonymous usage counts (`cymbra.lingua.v1.UsageCountsService/Report`,
`src/usage/client.ts`): sent without the account's token, carrying only the day of installation,
the browser family and the version — see _Data usage disclosures_.
The origin is set at build time (`LINGUA_GRPC_WEB_URL`, `src/net/transport.ts`) and
granted by the build (`tool/manifests.mjs`, `hostPattern`). A reader who neither signs in nor
agrees to the counts never sends it a request.

## Remote code

None. Everything the extension runs ships inside the package, including both WebAssembly engines
(the analysis engine and the translation engine), the language data — a base per studied language
(`assets/bases/`) and a layer per listed pair (`assets/packs/`), which the engine assembles at load
(the Italian programme's changes 3 and 4) — one per pair of `packs.json` (en-fr, es-fr, es-en,
en-es, fr-en, fr-es, and it-fr from that programme's change 20) — and the book renderer (foliate-js, vendored
under `vendor/`). No script and no WebAssembly is fetched at runtime; the content security
policy is `script-src 'self' 'wasm-unsafe-eval'`. A book's own scripts never run: its pages are
rendered under that same policy (`test/reader-csp.spec.ts`).

The only files downloaded at runtime are **data**: the translation models' weights, the ones the
reader's pairs need (`model-manifest.json`, `routes`) — for a French speaker, en-fr, es-en when
they study Spanish and it-en (`it-en/base-memory/2.0`, from change 20) when they study Italian, since
both go through English; for an English speaker reading Spanish,
es-en alone, the direct model of its pair; for a Spanish speaker reading English, en-es alone
(`en-es/base-memory/2.1`), the direct model of its pair; for an English speaker reading French,
fr-en alone (`fr-en/base-memory/2.0`), the direct model of its pair; for a Spanish speaker reading
French, fr-en then en-es, French going through English. They are fetched from
`https://models.cymbra.app` only after the reader turns on extended translation (« Traducción
ampliada » in Spanish, « Traduction étendue » in French), and each file is used only once its sha256
matches the hash the package carries (`model-manifest.json`,
`src/translate/host/model-download.ts`). Nothing in them is executed.

## Data usage disclosures (Chrome Web Store)

Answer **yes** to collecting _"Personally identifiable information"_, _"Website content"_ and
_"User activity"_ — but only in the narrow senses below — and **no** to every other category
(health, financial, authentication information, personal communications, location, web history).

What is collected, and only for a signed-in reader: the email address used to create the
account, and the reader's own vocabulary state (word statuses, cards, review history) sent to
`https://api.cymbra.app` so their devices agree. What the reader reads — page content, browsing
history — is **not** collected: the analysis never leaves the browser. Nor are the books a
reader imports, their text or where the reader is in them: they stay on the device. Turning on
« Traduction étendue » downloads models from Cymbra; those requests carry nothing of the
reader's (no cookie, no identifier, no page text) and are not tied to an account.

Website content, only for a signed-in reader whose account has access to « Traduction en ligne »
(off by default, per device): the one sentence the reader sends by pressing its button, with the
selection's position and the language pair, is sent to `https://api.cymbra.app` and from there to
the translation provider (Google, or Anthropic as a fallback), translated, and kept by no one —
not by Cymbra, not by the provider, and not used for training. Nothing is sent without that
press.

User activity, **only after the reader agrees**: the anonymous usage counts. The extension asks
once, on the welcome tab or in the popup, in one sentence with two equal buttons, "Yes, count" and
"No, thanks" (« Sí, contar » / « No, gracias », « Oui, compter » / « Non merci »); nothing is sent
before "Yes, count", and "No, thanks" turns counting off. Six steps are then counted, each sent
once per installation: installed; set up (a first level chosen); first page analysed in a studied
language; back on day 1; back during days 7–13; back during days 30–59. A report carries those
steps, the day of installation, the browser family and the extension's version (major.minor) —
**no identifier** of any kind (no installation, device or account id, even for a signed-in reader)
and nothing about what is read. It goes to `https://api.cymbra.app` without the account's token, to
a service that authenticates no one; the server adds one to a counter per installation day, keeps
no network address and deletes the counters after 25 months. Each step is sent when it happens (or
at the extension's next start or use, if it could not leave), so an installation alone on its day
can be recognised in the totals, as the privacy policy says. The device keeps the day of
installation for at most 60 days. An installation made before the release that introduced the counts
is never asked and never counted. Turning "Anonymous usage counts" off in Settings → Data stops the
counts for good on that installation. While counting is on, removing the extension opens a cymbra.app page that asks
why, with six fixed answers and no free text; only an answer the reader taps is sent, counted the
same anonymous way. The figures are read only as aggregates, in Cymbra's own back office.

Google's documentation does not publish what the dashboard means by _"User activity"_ (read on
2026-10-14): read its description in the dashboard when ticking it (add-lingua-usage-counts, Q7).
It is ticked because each counted step is interaction with the extension: declaring it costs
nothing, while a declaration that does not match what the extension does can get it removed.

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

## The dashboards' languages: Italian studied (change 20's submission)

The Italian programme's change 20 (`enable-lingua-italian`) adds no language to what the dashboards
read: its package carries `_locales/fr`, `en` and `es`, as change 52's did, `default_locale` still
`en` (M13). What moves is the French summary — change 20's, in `_locales/fr` and `manifest.json`
(_Summary_) — and the French listing's text, so this submission is made by hand too, in this order
(the owner's, M18; task 5.2 of add-lingua-italian-listings), after the site deploy that publishes
it-fr's figures (I12). `add-lingua-french-listings`' procedure does not fit as it stands: its French
step keeps the summary, which this one moves.

1. **Upload without publishing**, as change 34's step 1: the tag's
   `cymbra-lingua-chromium-<version>.zip`, from its GitHub Release, uploaded through the dashboard
   (Package › Upload new package), which submits nothing.
2. **`fr` replaced.** The default language still `en`. Under `fr`: the French description **FR,
   with Italian** above, and the `fr` screenshots unchanged (_Graphics_; the owner's answer of
   2026-10-25). Check that the French summary reads as change 20 committed it (_Summary_).
3. **`en` and `es`, one sentence each.** Under `en` and `es`: the English description and the
   Spanish one with fr-es, as above with Italian — each changed only in its sentence on what French
   speakers study.
   Check that both summaries are unchanged.
4. **The single fields replaced.** The single purpose and the test instructions **with Italian**,
   and the remote-code answer, as above; the permission justifications are unchanged.
5. **Submit for review** from the dashboard.
6. **addons.mozilla.org**: dispatch `lingua-extension-release` with the tag, `publish` ticked and
   `stores: firefox`. After the submission, paste the French description into the French locale, the
   English one into the English (US) locale and the Spanish one into the Spanish locale change 35's
   procedure recorded, and check the French summary.
7. **Record both results below**, in the pull request that next touches this file, reading each
   dashboard's own counter as the text is pasted.

| Dashboard          | Checked on  | Default language after upload | French summary | Listing texts (fr, en, es) | Screenshots (fr) | Notes |
| ------------------ | ----------- | ----------------------------- | -------------- | -------------------------- | ---------------- | ----- |
| Chrome Web Store   | _to record_ | _to record_                   | _to record_    | _to record_                | _to record_      |       |
| addons.mozilla.org | _to record_ | _to record_                   | _to record_    | _to record_                | _to record_      |       |
