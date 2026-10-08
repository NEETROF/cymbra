# App Store listing — Cymbra Lingua (Apple host app)

The copy for App Store Connect, with Apple's limits counted. The privacy answers live in
[README.md](README.md) (App Store privacy); this file is the rest of the listing.

Two things make this listing different from the browser stores', and both change the text:

- **Installing the app is not enabling the extension.** Safari requires the reader to turn it
  on in Settings and allow it per site. A description that skips this produces one-star reviews
  from people who installed it and saw nothing; review notes that skip it produce a rejection.
- **Choosing a level is not optional.** With none chosen the engine knows no known word, so the
  page is a wall of highlighting and the pill reads 0%. It reads as broken, not as unconfigured.

Other browsers are deliberately not mentioned: nothing here needs them, and naming competing
platforms in an App Store description invites scrutiny it does not have to invite.

## Locales

**fr-FR** is the primary localisation, for French speakers, who study English and Spanish.
**en-US** and **en-GB** are drafted for English speakers, who study Spanish — the es-en pair of
`enable-lingua-english-speakers` (change 34 of
[the language matrix programme](../../docs/lingua/language-matrix-programme.md)), pasted with its
release (M13, M18). The locales are M16's recommendation, **still open: the owner settles them**;
the en-GB text is the en-US text, word for word (M10: US English).

**es-ES** and **es-MX** are drafted for Spanish speakers, who study English — the en-es pair of
`enable-lingua-spanish-speakers` (change 35), pasted with its release, the first carrying
`_locales/es` (M13's rule applied to Spanish, M18). They are M16's recommendation too, **still
open**; the es-MX text is the es-ES text, word for word (M10: tú, neutral Spanish, no vosotros, the
RAE's numbers — « 25,4 MB »).

With fr-FR primary, a storefront whose language has no localisation shows the French listing: a
German App Store user reads French while the app opens in English (M13, change 28) — a
consequence to weigh when settling M16.

Every English text is a draft for the owner's review (M9), true of what change 34 ships: Spanish
studied, glossed in English, the interface in English. Every Spanish text is one too, true of what
change 35 ships: English studied, glossed in Spanish, the interface in Spanish. No locale calls
the app a beta or a trial, or says anything of price (guidelines 2.2 and 2.3.7). Each count is the
text's length in characters as App Store Connect counts them — every character inside the block or
after `> `, line breaks included — and equals the text it heads.

## Fields

| Field | Limit | fr-FR | en-US, en-GB | es-ES, es-MX |
|---|---|---|---|---|
| Name | 30 | `Cymbra Lingua` (13) | `Cymbra Lingua` (13) | `Cymbra Lingua` (13) |
| Subtitle | 30 | `Anglais et espagnol en lisant` (29) | `Learn Spanish while you read` (28) | `Aprende inglés mientras lees` (28) |
| Promotional text | 170 | 154 characters, below | 120 characters, below | 133 characters, below |
| Description | 4000 | 3021 characters, below | 2682 characters, below | 2895 characters, below |
| Keywords | 100 | 92 characters, below | 99 characters, below | 98 characters, below |
| Support URL | — | `https://cymbra.app/support/` | `https://cymbra.app/en/support/` | `https://cymbra.app/es/soporte/` or `https://cymbra.app/en/support/` — the owner chooses, below |
| Marketing URL | — | `https://cymbra.app/lingua/` | `https://cymbra.app/en/lingua/` | `https://cymbra.app/es/lingua/` |
| Privacy policy | — | `https://cymbra.app/confidentialite/` | `https://cymbra.app/en/privacy/` | `https://cymbra.app/es/privacidad/` |
| Category | — | Education (secondary: Reference) | the app's, not a locale's | the app's, not a locale's |
| Age rating | — | 4+ | the app's, not a locale's | the app's, not a locale's |

The Spanish URLs: the privacy policy is change 29's page, its Annex B Lingua's (change 31). The
marketing page is change 30's, built only once a pair glossed in Spanish is in the published
figures — en-es, written by change 35 — and live once the site is deployed after it; pasted before
that, the address would serve the home page (an unknown path is not a 404 there). The support page
`/es/soporte/` is change 29's translation of `/support/`; like it and `/en/support/`, the other
locales' support pages, it speaks of Cymbra Music (its subscription, its app menus), not of Lingua,
and gives the contact address. Change 29 leaves the choice to the owner: `/es/soporte/` or
`/en/support/`.

## Promotional text

Editable without a review, unlike everything else here.

### fr-FR (154 / 170)

> Les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne.

### en-US, en-GB (120 / 170)

> The Spanish words you don't know yet, highlighted on the page you're reading. The analysis runs on your device, offline.

### es-ES, es-MX (133 / 170)

> Las palabras en inglés que todavía no conoces, resaltadas en la página que lees. El análisis se hace en tu dispositivo, sin conexión.

## Keywords

Comma-separated, no spaces — a space costs a character and buys nothing. The app name and the
category are already indexed, so neither appears here.

### fr-FR (92 / 100)

```
anglais,espagnol,vocabulaire,lecture,traduction,extension,apprendre,mots,révision,CECRL,deck
```

### en-US, en-GB (99 / 100)

```
spanish,vocabulary,reading,translation,extension,learn,words,review,CEFR,deck,flashcards,dictionary
```

### es-ES, es-MX (98 / 100)

MCER is the CEFR's Spanish name, the one the interface uses (M19).

```
inglés,vocabulario,lectura,traducción,extensión,aprender,palabras,repaso,MCER,tarjetas,diccionario
```

## Description

### fr-FR (3021 / 4000)

```
Cymbra Lingua est une extension Safari qui surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne connaissez pas encore — sans rien changer à la mise en page. Une pastille vous dit quelle part du texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

APRÈS L'INSTALLATION
Cette app installe l'extension ; il reste à l'activer.
1. Ouvrez Réglages > Apps > Safari > Extensions (sur Mac : Safari > Réglages > Extensions).
2. Activez Cymbra Lingua, puis autorisez-la sur les sites que vous lisez.
3. Dans Safari, ouvrez l'extension depuis le menu de la barre d'adresse et choisissez votre niveau — sans lui, l'extension considère que vous ne connaissez aucun mot. Pour l'espagnol, cochez-le dans Réglages › Langue, puis choisissez votre niveau d'espagnol.
Rechargez les onglets déjà ouverts pour qu'ils soient surlignés.

LIRE
Touchez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté dans l'usage courant. En espagnol, la carte nomme aussi le temps et le genre, comme on les apprend en classe. Puis décidez — « Je connais », « + Deck » pour le réviser plus tard, ou « Ignorer ». Sélectionnez plusieurs mots pour capturer une expression entière avec la phrase d'où elle vient.

RÉVISER
Les cartes que vous créez se révisent dans un panneau, à côté de votre lecture, avec une répétition espacée qui décide toute seule du bon moment. Un écran de statistiques estime votre vocabulaire niveau par niveau, du A1 au C2, à partir des mots que vous avez marqués — pour l'espagnol, des niveaux estimés d'après la fréquence des mots.

LIRE VOS LIVRES
Importez vos livres EPUB sans DRM dans la bibliothèque de l'extension et lisez-les hors ligne, avec le même surlignage. Ils restent sur votre appareil.

TRADUCTION ÉTENDUE (FACULTATIVE)
Votre sélection est traduite dans sa phrase, sur votre appareil, par le moteur de Firefox Translations, inclus dans l'app ; l'espagnol passe par l'anglais. L'activer télécharge une fois les modèles de traduction depuis Cymbra (25,8 Mo pour l'anglais, 52,0 Mo avec l'espagnol) ; le texte des pages ne quitte toujours pas votre appareil. La désactiver supprime les modèles.

VOTRE LECTURE RESTE À VOUS
Les dictionnaires et les moteurs d'analyse et de traduction sont dans l'app. Aucune page que vous lisez n'est envoyée nulle part, et tout fonctionne hors ligne. Sans compte ni traduction étendue, l'extension ne fait aucune requête réseau.

UN COMPTE, SI VOUS EN VOULEZ UN
Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur vos autres appareils. C'est la seule chose qui quitte votre machine, et vous pouvez effacer ces données depuis les réglages sans supprimer votre compte.

Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos chiffres sont sur cymbra.app/lingua.

L'interface est en français pour les francophones, qui apprennent l'anglais et l'espagnol, et en anglais pour les anglophones, qui apprennent l'espagnol.
```

Its last line is the one this change rewrote (`add-lingua-english-listings`): it said the interface
was French-only. Like the English texts, it is pasted with change 34's release, not before — until
then the interface is French for every reader.

### en-US, en-GB (2682 / 4000)

The same plan as the French — the activation first, because installing is not enabling — written
for a reader of English who studies Spanish. Its step 3 follows the host app's English page
(`Shared (App)/Resources/copy.js`, `en.step3`), its labels the English catalogue
(`apps/lingua-extension/src/i18n/en/card.ts`), its sizes the model catalogue.

```
Cymbra Lingua is a Safari extension that highlights, on the page you are reading, the Spanish words you don't know yet — without changing the layout. A pill shows how much of the text is familiar to you, counted from what you actually marked rather than guessed.

AFTER INSTALLING
This app installs the extension; you still need to turn it on.
1. Open Settings > Apps > Safari > Extensions (on Mac: Safari > Settings > Extensions).
2. Turn on Cymbra Lingua, then allow it on the websites you read.
3. In Safari, open the extension from the address bar menu and choose your level of Spanish — until you do, the extension assumes you know no words at all.
Reload the tabs you already had open so they get highlighted.

READ
Tap a highlighted word: its meaning in English, its dictionary form and how rare it is in everyday use. The meanings come from Wiktionary, written by people, never machine-translated. The card also names the tense and the gender ("preterite indicative", "feminine noun"). Then decide — "I know it", "+ Deck" to review it later, or "Ignore". Select several words to capture a whole phrase with the sentence it came from.

REVIEW
The cards you make are reviewed in a panel beside your reading, with spaced repetition that picks the right moment for you. A statistics screen estimates your vocabulary level by level, from A1 to C2, from the words you have marked — for Spanish, levels estimated from word frequency.

READ YOUR BOOKS
Import your DRM-free EPUB books into the extension's library and read them offline, with the same highlighting. They stay on your device.

EXTENDED TRANSLATION (OPTIONAL)
Your selection is translated into English within its sentence, straight from Spanish, on your device, by the Firefox Translations engine, included in the app. Turning it on downloads the translation model from Cymbra once (26.2 MB); the text of the pages you read still never leaves your device. Turning it off deletes the model.

YOUR READING STAYS YOURS
The dictionaries and the analysis and translation engines are in the app. No page you read is sent anywhere, and everything works offline. Without an account or extended translation, the extension makes no network requests.

AN ACCOUNT, IF YOU WANT ONE
Create a Cymbra account if — and only if — you want your words and cards on your other devices. That is the only thing that leaves your device, and you can erase it from the settings without deleting your account.

How many Spanish words our dictionary explains in English: our figures are at cymbra.app/en/lingua.

The interface is in English for English speakers, who learn Spanish, and in French for French speakers, who learn English and Spanish.
```

« Several languages at once » is absent on purpose: an English speaker studies one language, es-en
being the one English-glossed pair, and the studied-languages box hides itself below two
(`offerFor`, `src/reading/studied-languages-view.ts`).

**If the owner settles M15 not to offer es-en's translation** (change 34, D5: the `es-en` route
leaves `model-manifest.json`), the « EXTENDED TRANSLATION (OPTIONAL) » paragraph is replaced by
the block below, « YOUR READING STAYS YOURS » drops « or extended translation », and the What's New
below drops its last sentence — every text shorter, so within its limit; recount before pasting.

```
EXTENDED TRANSLATION
Translating a Spanish selection into English on your device comes later.
```

### es-ES, es-MX (2895 / 4000)

The same plan again, written for a reader of Spanish who studies English. Its steps follow the host
app's Spanish page (`Shared (App)/Resources/copy.js`, `es.step1` to `es.step3`: « Ajustes → Apps →
Safari → Extensiones », « elige tu nivel de inglés »), its labels the Spanish catalogue
(`apps/lingua-extension/src/i18n/es/card.ts`: « La conozco », « + Mazo », « Ignorar »), its grammar the
Spanish renderer (`src/i18n/es/grammar.ts`: « pasado simple de go », pinned by
`test/word-grammar-es.spec.ts`), its size the model catalogue: en-es's one model,
`en-es/base-memory/2.1`, 22,698,792 + 2,265,250 + 409,312 B = 25.4 MB, « 25,4 MB » as the Spanish
setting writes it (`megabytes`, `src/reading/translation-setting.ts`). Its levels are not estimated:
English's come from CEFR-J and Octanove (`scripts/lingua-data/tables/en-es/NOTICE`;
`levels_estimated: false`, `scripts/lingua-data/reduce-en-es.py`), named MCER (M19).

```
Cymbra Lingua es una extensión de Safari que resalta, en la página que lees, las palabras en inglés que todavía no conoces, sin cambiar el diseño de la página. Una pastilla te muestra qué parte del texto te resulta familiar, calculada a partir de lo que de verdad has marcado, no estimada.

DESPUÉS DE INSTALARLA
Esta app instala la extensión; todavía tienes que activarla.
1. Abre Ajustes > Apps > Safari > Extensiones (en Mac: Safari > Ajustes > Extensiones).
2. Activa Cymbra Lingua y luego permítela en los sitios web que lees.
3. En Safari, abre la extensión desde el menú de la barra de direcciones y elige tu nivel de inglés: mientras no lo hagas, la extensión supone que no conoces ninguna palabra.
Recarga las pestañas que ya tenías abiertas para que se resalten.

LEER
Toca una palabra resaltada: su significado en español, su forma de diccionario y su frecuencia en el uso corriente. Los significados proceden del Wikcionario, escritos por personas, nunca por una traducción automática. La tarjeta también nombra el tiempo verbal («pasado simple de go»). Luego decide: «La conozco», «+ Mazo» para repasarla más tarde, o «Ignorar». Selecciona varias palabras para capturar una expresión entera con la frase de la que procede.

REPASAR
Las tarjetas que creas se repasan en un panel junto a tu lectura, con una repetición espaciada que elige por ti el momento oportuno. Una pantalla de estadísticas estima tu vocabulario nivel por nivel, del A1 al C2 en la escala MCER, a partir de las palabras que has marcado.

LEER TUS LIBROS
Importa tus libros EPUB sin DRM a la biblioteca de la extensión y léelos sin conexión, con el mismo resaltado. Se quedan en tu dispositivo.

TRADUCCIÓN AMPLIADA (OPCIONAL)
Tu selección se traduce al español dentro de su frase, directamente desde el inglés, en tu dispositivo, con el motor de Firefox Translations, incluido en la app. Al activarla se descarga una vez el modelo de traducción desde Cymbra (25,4 MB); el texto de las páginas que lees sigue sin salir de tu dispositivo. Al desactivarla se elimina el modelo.

TU LECTURA ES SOLO TUYA
Los diccionarios y los motores de análisis y de traducción están en la app. Ninguna de las páginas que lees se envía a ningún sitio, y todo funciona sin conexión. Sin cuenta ni traducción ampliada, la extensión no hace ninguna solicitud de red.

UNA CUENTA, SI LA QUIERES
Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros dispositivos. Es lo único que sale de tu dispositivo, y puedes borrar esos datos desde los ajustes sin eliminar tu cuenta.

Cuántas palabras inglesas explica nuestro diccionario en español: nuestras cifras están en cymbra.app/es/lingua.

La interfaz está en español para los hispanohablantes, que aprenden inglés; en francés para los francohablantes, que aprenden inglés y español; y en inglés para los anglohablantes, que aprenden español.
```

« Varios idiomas a la vez » is absent for the same reason as in English: en-es is the one pair
glossed in Spanish, so a Spanish speaker studies one language, and the studied-languages box hides
itself below two (`offerFor`, `src/reading/studied-languages-view.ts`). The French and English
descriptions are unchanged (this change's D2): their last lines name no Spanish speakers, which the
owner may add in their own wording.

**If the owner settles M15 not to offer en-es's translation** (the `en-es` route leaves
`model-manifest.json`), the « TRADUCCIÓN AMPLIADA (OPCIONAL) » paragraph is replaced by the block
below, « TU LECTURA ES SOLO TUYA » drops « ni traducción ampliada » (« Sin cuenta, la extensión no
hace ninguna solicitud de red. »), and the What's New below drops its last sentence — every text
shorter, so within its limit; recount before pasting.

```
TRADUCCIÓN AMPLIADA
La traducción de una selección en inglés al español en tu dispositivo llegará más adelante.
```

## What's New — the release that ships English (es-en)

The paragraph for English speakers, for the release that ships es-en (change 34); the rest of its
notes is written with the release, like every « What's New ». If M15 withholds es-en's
translation, its last sentence goes.

### en-US, en-GB (302 / 4000)

```
Cymbra Lingua now speaks English, for English speakers learning Spanish: the interface, the word card, review and statistics are in English, and Spanish words come with their meaning in English, written by people. Extended translation turns your Spanish selection straight into English, on your device.
```

## What's New — the release that ships Spanish speakers' English (en-es)

The paragraph for Spanish speakers, for the release that ships en-es (change 35); the rest of its
notes is written with the release, like every « What's New ». If M15 withholds en-es's
translation, its last sentence goes.

### es-ES, es-MX (335 / 4000)

```
Cymbra Lingua ahora habla español, para los hispanohablantes que aprenden inglés: la interfaz, la tarjeta, el repaso y las estadísticas están en español, y las palabras inglesas llegan con su significado en español, escrito por personas. La traducción ampliada traduce tu selección en inglés directamente al español, en tu dispositivo.
```

## What's New — the release that ships Spanish

The Spanish paragraph (add-lingua-spanish-listings). The rest of the notes is written with the
release, like every « What's New ».

```
Espagnol : lisez aussi l'espagnol. Cochez-le dans Réglages › Langue : mots surlignés, carte avec le temps et le genre, niveaux estimés, une voix d'Espagne pour la lecture à voix haute, et la traduction étendue en passant par l'anglais.
```

## What's New — 1.4.0 (830 / 4000)

The extension inside the app goes from 1.3.0 to 1.6.0. Only what a Safari reader can see: the
Spanish work of 1.5.0–1.6.0 is not listed, as the app still ships the English → French pair
alone (`apps/lingua-extension/packs.json`), and Safari offers no « Lier Google / Apple ».

```
Réglages réorganisés en quatre onglets : Langue, Apparence, Pages & livres, Données.

Compte : connexion et déconnexion directement dans Réglages › Données. Nouveau « Comptes connectés » : ajoutez un mot de passe à un compte créé avec Apple ou Google, pour vous connecter aussi par e-mail, ou retirez une méthode de connexion.

Couleurs : choisissez celles du surlignage et de la page, avec des préréglages pour liseuse à encre électronique. La taille du texte agrandit aussi toute l'interface.

Livres : les pages peuvent glisser en tournant, et le lecteur passe en plein écran sur Mac.

Carte de mot : la forme du mot (temps, pluriel…) et une traduction rangée par nature.

Statistiques : les mots enseignés et estimés à chaque niveau, de A1 à C2.

Corrections, dont le menu de l'extension sur iPad, désormais large et défilant.
```

## What's New — 1.1.0 (373 / 4000)

```
Première version publique.

Lecture assistée sur iPhone, iPad et Mac : surlignage des mots que vous n'avez pas encore marqués, traduction et rareté au toucher, capture d'expressions, révision par répétition espacée, et une estimation de votre vocabulaire du A1 au C2.

Vos données restent sur l'appareil ; le compte Cymbra est facultatif et ne sert qu'à la synchronisation.
```

## Review notes (App Review Information)

Apple's first review (2026-09-22, iOS 1.1.0) was a Guideline 2.1 "Information Needed": an
account with little review history is asked, in reply **and** in this field for every later
submission, for a screen recording on a physical device (from launch, with sign-in and account
deletion), the app's purpose and audience, setup steps, external services, regional differences
and third-party material. The notes below answer all of it; the recording goes with the reply.

Paste only what is inside the block: text around it has been pasted into App Store Connect
before.

The field is **one, in English, for every locale, within 4,000 characters**. The block before
`add-lingua-english-listings` measured 4,042, over it; that change cut it to 3,973, and this one
(`add-lingua-spanish-audience-listings`) adds the Spanish path and measures **3976**. A
reviewer's device that is not in French or Spanish gets the English interface studying Spanish (the
native language is preset from the browser's language: `add-lingua-native-language-choice`, D4), so
the steps follow that path, every label quoted from `apps/lingua-extension/src/i18n/en/` —
`card.ts` ("I know it", "+ Deck", "Ignore"), `drawer.ts` and `hud.ts` (Review, Stats, Settings),
`settings.ts` (the tabs, "Manage my data"), `account.ts` ("Erase my Lingua data…", "Delete my Cymbra
account"), `translation.ts` ("Extended translation"), `review.ts` ("Sources & privacy"). "Purpose &
audience" and "Regional differences" say which interface each reader gets and how to switch; the
latter gives a device in Spanish its path, English studied, in the Spanish catalogue's labels
(`src/i18n/es/`: `languages.ts` « Elige tu nivel de inglés », the popup's level prompt; `card.ts`
« La conozco », « + Mazo », « Ignorar »; `drawer.ts` « Repaso », « Ajustes »). "Third-party
material" names the sources of the glosses in each language — en-es's are the Spanish Wiktionary's
definitions of English words and the English and Spanish Wiktionaries' translation tables
(`scripts/lingua-data/tables/en-es/README.md`), beside es-en's and es-fr's; the models are es-en's
and en-es's, their sizes from `model-manifest.json` (Spanish to English 23,288,494 + 2,543,246 +
409,312 B = 26.2 MB; English to Spanish 22,698,792 + 2,265,250 + 409,312 B = 25.4 MB, as the setting
rounds them). The deletion link is the English interface's (`deleteAccountUrl`,
`src/account/flow.ts`).

To fit the Spanish path — 243 characters added — 240 were cut, wording only, no step a reviewer
needs: the account's paragraph says « across reinstalls and devices » instead of spelling both out;
« B1 is a good default » became « e.g. B1 »; « Open any Spanish-language page » became « Open a
Spanish page »; « you already know » lost « already »; the OPTIONAL paragraph lost « nothing
executable is downloaded », which its next sentence's « downloads data only » already says;
"External services" points to the model « above » instead of repeating when it is fetched;
"Regional differences" lost « the app behaves identically everywhere », which its « none » says,
and « French for French speakers … English for any other » became « French … else English ». Every
step, the deletion path, every external service and every source are still there.

If M15 withholds es-en's translation, the OPTIONAL paragraph says it is offered to readers of the
French interface only, with their models (English to French, 25.8 MB; with Spanish, 52.0 MB); if it
withholds en-es's, « , English to Spanish 25.4 MB » goes.

### English, for every locale (3976 / 4000)

```
No account is needed to review this app: every feature works signed out. The Cymbra account saves the learner's progress - known words, deck, level and statistics - across reinstalls and devices (iPhone, iPad, Mac); without one, it stays on this device. To test it, create an account from the extension (panel > Settings > Data > Account) with Sign in with Apple or Google - no invitation is needed.

IMPORTANT - this app is a Safari extension host: its own screen only explains. The extension must be enabled in Safari before anything happens.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level chosen the engine assumes zero known words: every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs; the account is under Data).

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases that data on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app. Turning it on downloads data only - the model of the reader's pair (Spanish to English 26.2 MB, English to Spanish 25.4 MB) from https://models.cymbra.app, checked against a pinned sha256 - and the selected sentence is then translated on the device. No page text, account or device identifier is sent. Turning it off deletes the model.

Purpose & audience: Cymbra Lingua helps people learn a language by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked. English speakers learn Spanish, Spanish speakers English, French speakers both.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device (bundled WebAssembly engines + offline dictionaries). No AI/LLM API, no analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the model above; https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor: the app is free, with no in-app purchase.

Regional differences: none. The interface follows the device's language: French (learning English and Spanish), Spanish (learning English), else English (learning Spanish); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries combine sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD treebank for Spanish; wordfreq frequency lists for both; kaikki.org extracts of the English, French and Spanish Wiktionaries (definitions and translation tables) for the glosses - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

## Screenshots

Apple requires a set **per platform and per locale**, at the sizes each slot names (see *what the
slots actually asked for*, below): iPhone 6.5" 1284 x 2778 — a 6.9" capture is refused there —,
iPad 13" 2064 x 2752, macOS 1280 x 800, 1440 x 900, 2560 x 1600 or 2880 x 1800.

What they should show, in this order: a page being read with its highlighting and the pill, the
word card open on a highlighted word, the review panel, and the statistics screen with the
estimated vocabulary. The same four the browser listings use — they are what the product is.

| Locale | iPhone 6.5" | iPad 13" | macOS | Taken from |
|---|---|---|---|---|
| fr-FR | sent | sent | to capture | the French interface on an English page |
| en-US | to capture | to capture | to capture | a build of change 34, the device in English: the English interface on a Spanish page |
| en-GB | en-US's set | en-US's set | en-US's set | — |
| es-ES | to capture | to capture | to capture | a build of change 35, the device in Spanish: the Spanish interface on an English page |
| es-MX | es-ES's set | es-ES's set | es-ES's set | — |

The en-US set, captured by the owner (the device in English, a Spanish level chosen — B1):

1. **Reading** — a Spanish article in Safari (a Spanish Wikipedia page reads well), highlighting
   on, the pill showing the share of the page known.
2. **The word card** — open on a highlighted verb form whose card names a tense: « dijo » reads
   "third-person singular preterite indicative of decir" (`test/baseline/word-card-es-en.txt`),
   its English gloss below, "I know it", "+ Deck" and "Ignore".
3. **Review** — the panel ("Review") on a card, its sentence shown, the four answers ("Again",
   "Hard", "Good", "Easy").
4. **Statistics** — "Stats", the ladder "My estimated Spanish level" from A1 to C2, with its note
   that the levels are estimated from word frequency.

On macOS, the same four in Safari for Mac.

The es-ES set, captured by the owner (the device in Spanish, an English level chosen — B1):

1. **Reading** — an English article in Safari (an English Wikipedia page reads well), highlighting
   on, the pill showing the share of the page known.
2. **The word card** — open on a highlighted past form: « went » reads « pasado simple de go »
   (`src/i18n/es/grammar.ts`, pinned by `test/word-grammar-es.spec.ts`; change 24's snapshot of the
   card in Spanish, once merged, gives the real readings), its Spanish gloss below, « La conozco »,
   « + Mazo » and « Ignorar ».
3. **Review** — the panel (« Repaso ») on a card, its sentence shown, the four answers (« Otra vez »,
   « Difícil », « Bien », « Fácil »).
4. **Statistics** — « Stats », the ladder « Mi nivel de inglés » from A1 to C2 (MCER), its levels
   English's own lists, not estimated.

On macOS, the same four in Safari for Mac.

## What the app itself says, and why it settles the wording

The host app's own first screen reads: *"Dans Safari, ouvre l'extension depuis le menu de la
barre d'adresse et choisis ton niveau d'anglais."* An earlier draft of this listing said to open
the app to pick the level. That was wrong — the app shows an explanatory screen and nothing
else; the level lives in the extension, inside Safari. Copy that contradicts the product is
worse than copy that says too little: the reader follows it, finds nothing, and concludes the
app is broken. Its English page (`localise-lingua-apple-host`, `copy.js`) says the same to an
English speaker — *"In Safari, open the extension from the address bar menu and choose your level
of Spanish."* — and the en-US description's step 3 follows it. Its Spanish page says it to a
Spanish speaker — *« En Safari, abre la extensión desde el menú de la barra de direcciones y elige tu
nivel de inglés. »* — and the es-ES description's step 3 follows it.

Found by running the app, not by reading the code.

## Screenshots — what the slots actually asked for

Captured from the simulators, at the sizes App Store Connect names on the page itself:

| Slot | Accepted sizes | What was sent |
|---|---|---|
| iPhone 6.5" | 1242 x 2688, 1284 x 2778 | 1284 x 2778, rescaled from a 6.9" capture |
| iPad 13" | 2064 x 2752, 2048 x 2732 | 2064 x 2752, captured natively |
| macOS | 1280 x 800, 1440 x 900, 2560 x 1600, 2880 x 1800 | still to capture |

A 6.9" capture (1320 x 2868) is **refused** by the 6.5" slot. The dashboard states its own
sizes; read them there rather than assuming the newest device is the right one.
