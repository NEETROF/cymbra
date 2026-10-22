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
**en-US** and **en-GB** are written for English speakers, who study Spanish — the es-en pair of
`enable-lingua-english-speakers` (change 34 of
[the language matrix programme](../../docs/lingua/language-matrix-programme.md)) — and French — the
fr-en pair of `enable-lingua-french` (change 52); the en-GB text is the en-US text, word for word
(M10: US English). **es-ES** and **es-MX** are written for Spanish speakers, who study English —
the en-es pair of `enable-lingua-spanish-speakers` (change 35), the first release carrying
`_locales/es` (M13's rule applied to Spanish) — and French if change 52 lists fr-es; the es-MX text
is the es-ES text, word for word (M10: tú, neutral Spanish, no vosotros, the RAE's numbers —
« 25,4 MB »). The locales are M16's, settled by the owner on 2026-10-09; French studied adds none: a
French speaker cannot study French, and fr-en is glossed in English, fr-es in Spanish.

With fr-FR primary, a storefront whose language has no localisation shows the French listing: a
German App Store user reads French while the app opens in English (M13, change 28).

Every English and Spanish text below is a draft for the owner's review (M9), true of what change 52
ships, and pasted with its release (M18), after the site deploy that publishes its figures: English
speakers study Spanish and French, glossed in English, the interface in English; Spanish speakers
study English — and French, if change 52 lists fr-es —, glossed in Spanish, the interface in
Spanish. **The Spanish texts are written twice** (add-lingua-french-listings, D1): with fr-es,
French named wherever a Spanish reader's languages are; without fr-es, change 37's text, where only
what English speakers study moves. The owner pastes the variant that matches the release. Should
change 34 or 35 be released before change 52, its texts are this file's version before
add-lingua-french-listings. The French texts change only where they say what another reader studies
(D3), and in the fr-FR description's step 3, aligned with the host app's French page by the owner on
2026-10-10. No locale calls the app a beta or a trial, or says anything of price (guidelines 2.2 and
2.3.7). Each count is the text's length in characters as App Store Connect counts them — every
character inside the block or after `> `, line breaks included — and equals the text it heads.

## Fields

| Field | Limit | fr-FR | en-US, en-GB | es-ES, es-MX |
|---|---|---|---|---|
| Name | 30 | `Cymbra Lingua` (13) | `Cymbra Lingua` (13) | `Cymbra Lingua` (13) |
| Subtitle | 30 | `Anglais et espagnol en lisant` (29) | `Spanish and French as you read` (30) | with fr-es `Inglés y francés mientras lees` (30); without, `Aprende inglés mientras lees` (28) |
| Promotional text | 170 | 154 characters, below | 130 characters, below | with fr-es 146 characters; without, 133 — below |
| Description | 4000 | 3186 characters, below | 3037 characters, below | with fr-es 3427 characters; without, 3015 — below |
| Keywords | 100 | 92 characters, below | 99 characters, below | with fr-es 90 characters (94 bytes); without, 89 (92 bytes) — below |
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

### en-US, en-GB (130 / 170)

> The Spanish or French words you don't know yet, highlighted on the page you're reading. The analysis runs on your device, offline.

### es-ES, es-MX, with fr-es (146 / 170)

> Las palabras en inglés o en francés que todavía no conoces, resaltadas en la página que lees. El análisis se hace en tu dispositivo, sin conexión.

### es-ES, es-MX, without fr-es (133 / 170)

> Las palabras en inglés que todavía no conoces, resaltadas en la página que lees. El análisis se hace en tu dispositivo, sin conexión.

## Keywords

Comma-separated, no spaces — a space costs a character and buys nothing. The app name and the
category are already indexed, so neither appears here.

### fr-FR (92 / 100)

```
anglais,espagnol,vocabulaire,lecture,traduction,extension,apprendre,mots,révision,CECRL,deck
```

French takes the place of « review » in English and of « repaso » in Spanish
(add-lingua-french-listings, *Measured*): at 99 and 98 of 100, the lists had no room for one more
word — confirmed by the owner on 2026-10-10. The French keywords do not move, « CECRL »
included: a search term no reader sees, the one French learners type (D7).

### en-US, en-GB (99 / 100)

```
spanish,french,vocabulary,reading,translation,extension,learn,words,CEFR,deck,flashcards,dictionary
```

### es-ES, es-MX, with fr-es (90 / 100; 94 bytes)

MCER is the CEFR's Spanish name, the one the interface uses (M19). Apple counts this field in
bytes, and an accented letter takes two: at 99 and 98 characters the Spanish lists were 103 and
101 bytes, over the limit, so « palabras » (which « vocabulario » already covers) left both on
2026-10-11, in the AI review of the Spanish drafts made on the owner's instruction.

```
inglés,francés,vocabulario,lectura,traducción,extensión,aprender,MCER,tarjetas,diccionario
```

### es-ES, es-MX, without fr-es (89 / 100; 92 bytes)

```
inglés,vocabulario,lectura,traducción,extensión,aprender,repaso,MCER,tarjetas,diccionario
```

## Description

Each description's « VOTRE LECTURE RESTE À VOUS » paragraph and its translations carry the sentence
of add-lingua-usage-counts (design Q5, accepted by the owner on 2026-10-14) where they promised
that, without an account or extended translation, nothing reached the network — the anonymous
counts make that false. Its subject is « l'extension », as everywhere in these texts, and its switch
is in « les Réglages de l'extension », kept apart from step 1's, the system's, as step 3 does; the
account paragraph says the counts leave the device too. Each count below was re-measured then
(+125 fr-FR, +114 en, +110 for each Spanish text).

### fr-FR (3186 / 4000)

```
Cymbra Lingua est une extension Safari qui surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne connaissez pas encore — sans rien changer à la mise en page. Une pastille vous dit quelle part du texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

APRÈS L'INSTALLATION
Cette app installe l'extension ; il reste à l'activer.
1. Ouvrez Réglages > Apps > Safari > Extensions (sur Mac : Safari > Réglages > Extensions).
2. Activez Cymbra Lingua, puis autorisez-la sur les sites que vous lisez.
3. Dans Safari, ouvrez l'extension depuis le menu de la barre d'adresse et choisissez votre niveau — sans lui, l'extension considère que vous ne connaissez aucun mot. Pour l'espagnol, cochez-le dans les Réglages de l'extension, onglet Langue, puis choisissez votre niveau d'espagnol.
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
Les dictionnaires et les moteurs d'analyse et de traduction sont dans l'app, et tout fonctionne hors ligne. Ce que vous lisez ne quitte jamais votre appareil. L'extension peut envoyer des comptages anonymes — installation, premiers pas, retours —, sans aucun identifiant ; un interrupteur dans les Réglages de l'extension les coupe.

UN COMPTE, SI VOUS EN VOULEZ UN
Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur vos autres appareils. Hormis les comptages anonymes, c'est la seule chose qui quitte votre machine, et vous pouvez effacer ces données depuis les réglages sans supprimer votre compte.

Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos chiffres sont sur cymbra.app/lingua.

L'interface est en français pour les francophones, qui apprennent l'anglais et l'espagnol, et en anglais pour les anglophones, qui apprennent l'espagnol et le français.
```

Its last line names what the readers of the other texts study, so it moves with them: English
speakers learn Spanish and French from change 52 (add-lingua-french-listings, D3). Its step 3 says
where a second language is ticked as the host app's French page does from change 52 (« coche-le
dans les Réglages de l'extension, onglet Langue », its D7), in the description's own register
(vous, where the page says tu): « cochez-le dans les Réglages de l'extension, onglet Langue », in
place of « dans Réglages › Langue » — the owner's decision of 2026-10-10 (+25 characters), which
keeps the extension's Réglages apart from step 1's, the system's. Both are pasted with change 52's
release, not before. A French speaker cannot study French, so nothing else in the French texts
moves.

### en-US, en-GB (3037 / 4000)

The same plan as the French — the activation first, because installing is not enabling — written
for a reader of English who studies Spanish and French. Its step 3 follows the host app's English
page as change 52 rewrites it (`Shared (App)/Resources/copy.js`, `en.step3`, its D7: "…choose your
level of Spanish. For French, check it in the extension's Settings, under Language, then choose your
level of French."), its labels the English catalogue (`apps/lingua-extension/src/i18n/en/card.ts`),
its grammar the English renderer ("past historic (passé simple)", pinned by
`test/baseline/word-card-fr-en.txt`), its sizes the model catalogue (es-en and fr-en, 26.2 MB each;
`apps/lingua-extension/STORE-LISTING.md` gives the bytes).

```
Cymbra Lingua is a Safari extension that highlights, on the page you are reading, the Spanish or French words you don't know yet — without changing the layout. A pill shows how much of the text is familiar to you, counted from what you actually marked rather than guessed.

AFTER INSTALLING
This app installs the extension; you still need to turn it on.
1. Open Settings > Apps > Safari > Extensions (on Mac: Safari > Settings > Extensions).
2. Turn on Cymbra Lingua, then allow it on the websites you read.
3. In Safari, open the extension from the address bar menu and choose your level of Spanish — until you do, the extension assumes you know no words at all. For French, check it in the extension's Settings, under Language, then choose your level of French.
Reload the tabs you already had open so they get highlighted.

READ
Tap a highlighted word: its meaning in English, its dictionary form and how rare it is in everyday use. The meanings come from Wiktionary, written by people, never machine-translated. The card also names the tense and the gender ("preterite indicative" in Spanish, "past historic (passé simple)" in French, "feminine noun"). Then decide — "I know it", "+ Deck" to review it later, or "Ignore". Select several words to capture a whole phrase with the sentence it came from.

REVIEW
The cards you make are reviewed in a panel beside your reading, with spaced repetition that picks the right moment for you. A statistics screen estimates your vocabulary level by level, from A1 to C2, from the words you have marked — for Spanish and French, levels estimated from word frequency.

READ YOUR BOOKS
Import your DRM-free EPUB books into the extension's library and read them offline, with the same highlighting. They stay on your device.

EXTENDED TRANSLATION (OPTIONAL)
Your selection is translated into English within its sentence, straight from Spanish or French, on your device, by the Firefox Translations engine, included in the app. Turning it on downloads from Cymbra, once, the translation model of each language you study (26.2 MB each); the text of the pages you read still never leaves your device. Turning it off deletes the models.

YOUR READING STAYS YOURS
The dictionaries and the analysis and translation engines are in the app, and everything works offline. What you read never leaves your device. The extension may send anonymous counts — install, first steps, returns — with no identifier of any kind; one switch in the extension's Settings turns them off.

AN ACCOUNT, IF YOU WANT ONE
Create a Cymbra account if — and only if — you want your words and cards on your other devices. Apart from the anonymous counts, that is the only thing that leaves your device, and you can erase it from the settings without deleting your account.

How many Spanish and French words our dictionary explains in English: our figures are at cymbra.app/en/lingua.

The interface is in English for English speakers, who learn Spanish and French, and in French for French speakers, who learn English and Spanish.
```

« Several languages at once » is absent on purpose: a listing speaks of no price
(add-lingua-french-listings, D2). The text carries the French one's sentence on where a second
language is ticked, never a line about price — and that line has left the extension (change 53b).

M15 is settled: extended translation opens with each pair. Should change 52 not offer fr-en's, its
clause goes from each text and every count only falls; recount before pasting.

### es-ES, es-MX, with fr-es (3427 / 4000)

The same plan again, written for a reader of Spanish who studies English and French. Its steps
follow the host app's Spanish page as change 52 rewrites it with fr-es (`copy.js`, `es.step1` to
`es.step3`, its D7: « … elige tu nivel de inglés. Para el francés, márcalo en los Ajustes de la
extensión, pestaña Idioma, y luego elige tu nivel de francés. »), its labels the Spanish catalogue
(`apps/lingua-extension/src/i18n/es/card.ts`: « La conozco », « + Mazo », « Ignorar »), its grammar
the Spanish renderer (`src/i18n/es/grammar.ts`: « pasado simple de go », pinned by
`test/word-grammar-es.spec.ts`; « pretérito perfecto simple de indicativo », pinned by
`test/baseline/word-card-fr-es.txt`), its sizes the model catalogue: en-es's one model,
`en-es/base-memory/2.1`, 22,698,792 + 2,265,250 + 409,312 B = « 25,4 MB »; French through English,
fr-en 2.0 then en-es 2.1, 26,234,715 + 25,373,354 B = « 51,6 MB », as the Spanish setting writes them
(`megabytes`, `src/reading/translation-setting.ts`). English's levels are not estimated: they come
from CEFR-J and Octanove (`scripts/lingua-data/tables/en-es/NOTICE`; `levels_estimated: false`,
`scripts/lingua-data/reduce-en-es.py`), named MCER (M19); French's are (`levels_estimated`, change
46). Its dictionary explains fewer French words than English ones — fr-es 83.2 / 70.8 / 56.8 % of
the 5,000 / 10,000 / 20,000 commonest lemmas glossed against en-es's 93.0 / 85.0 / 71.7 % (changes
49 and 22) —, which its next-to-last line says, as the French text says it of Spanish (M6).

```
Cymbra Lingua es una extensión de Safari que resalta, en la página que lees, las palabras en inglés o en francés que todavía no conoces, sin cambiar el diseño de la página. Una pastilla te muestra qué parte del texto te resulta familiar, calculada a partir de lo que de verdad has marcado, no estimada.

DESPUÉS DE INSTALARLA
Esta app instala la extensión; todavía tienes que activarla.
1. Abre Ajustes > Apps > Safari > Extensiones (en Mac: Safari > Ajustes > Extensiones).
2. Activa Cymbra Lingua y luego permítela en los sitios web que lees.
3. En Safari, abre la extensión desde el menú de la barra de direcciones y elige tu nivel de inglés: mientras no lo hagas, la extensión supone que no conoces ninguna palabra. Para el francés, márcalo en los Ajustes de la extensión, pestaña Idioma, y luego elige tu nivel de francés.
Recarga las pestañas que ya tenías abiertas para que se resalten.

LEER
Toca una palabra resaltada: su significado en español, su forma de diccionario y su frecuencia en el uso corriente. Los significados proceden del Wikcionario, escritos por personas, nunca por una traducción automática. La tarjeta también nombra el tiempo verbal («pasado simple de go»; en francés, «pretérito perfecto simple de indicativo») y el género de los sustantivos franceses. Luego decide: «La conozco», «+ Mazo» para repasarla más tarde, o «Ignorar». Selecciona varias palabras para capturar una expresión entera con la frase de la que procede.

REPASAR
Las tarjetas que creas se repasan en un panel junto a tu lectura, con una repetición espaciada que elige por ti el momento oportuno. Una pantalla de estadísticas estima tu vocabulario nivel por nivel, del A1 al C2 en la escala MCER, a partir de las palabras que has marcado; para el francés, niveles estimados según la frecuencia de las palabras.

LEER TUS LIBROS
Importa tus libros EPUB sin DRM a la biblioteca de la extensión y léelos sin conexión, con el mismo resaltado. Se quedan en tu dispositivo.

TRADUCCIÓN AMPLIADA (OPCIONAL)
Tu selección se traduce al español dentro de su frase, directamente desde el inglés y, desde el francés, pasando por el inglés, en tu dispositivo, con el motor de Firefox Translations, incluido en la app. Al activarla se descargan una vez los modelos de traducción desde Cymbra (25,4 MB para el inglés, 51,6 MB con el francés); el texto de las páginas que lees sigue sin salir de tu dispositivo. Al desactivarla se eliminan los modelos.

TU LECTURA ES SOLO TUYA
Los diccionarios y los motores de análisis y de traducción están en la app, y todo funciona sin conexión. Lo que lees nunca sale de tu dispositivo. La extensión puede enviar recuentos anónimos —instalación, primeros pasos, regresos—, sin ningún identificador; un interruptor en los Ajustes de la extensión los desactiva.

UNA CUENTA, SI LA QUIERES
Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros dispositivos. Aparte de los recuentos anónimos, es lo único que sale de tu dispositivo, y puedes borrar esos datos desde los ajustes sin eliminar tu cuenta.

Para el francés, nuestro diccionario en español es algo menos completo que para el inglés: nuestras cifras están en cymbra.app/es/lingua.

La interfaz está en español para los hispanohablantes, que aprenden inglés y francés; en francés para los francohablantes, que aprenden inglés y español; y en inglés para los anglohablantes, que aprenden español y francés.
```

### es-ES, es-MX, without fr-es (3015 / 4000)

If change 52 lists fr-en alone: change 37's text, its last line alone made true — English speakers
learn Spanish and French. Its steps follow the host app's Spanish page, which change 52 leaves as it
is without fr-es (« … y elige tu nivel de inglés. »).

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
Los diccionarios y los motores de análisis y de traducción están en la app, y todo funciona sin conexión. Lo que lees nunca sale de tu dispositivo. La extensión puede enviar recuentos anónimos —instalación, primeros pasos, regresos—, sin ningún identificador; un interruptor en los Ajustes de la extensión los desactiva.

UNA CUENTA, SI LA QUIERES
Crea una cuenta de Cymbra si —y solo si— quieres recuperar tus palabras y tus tarjetas en tus otros dispositivos. Aparte de los recuentos anónimos, es lo único que sale de tu dispositivo, y puedes borrar esos datos desde los ajustes sin eliminar tu cuenta.

Cuántas palabras inglesas explica nuestro diccionario en español: nuestras cifras están en cymbra.app/es/lingua.

La interfaz está en español para los hispanohablantes, que aprenden inglés; en francés para los francohablantes, que aprenden inglés y español; y en inglés para los anglohablantes, que aprenden español y francés.
```

« Varios idiomas a la vez » is absent for the same reason as in English: a listing speaks of no
price (add-lingua-french-listings, D2).

M15 is settled: extended translation opens with each pair. Should change 52 not offer fr-es's, its
clause goes from the text with fr-es and every count only falls; recount before pasting.

## What's New — the release that ships French (fr-en, fr-es)

The paragraphs for English and Spanish speakers, for the release of change 52; the rest of its notes
is written with the release, like every « What's New ». The Spanish one is pasted only if the
release lists fr-es; without it, the Spanish notes say nothing of French. The French notes say
nothing of it either: a French speaker studies no French.

### en-US, en-GB (380 / 4000)

```
Cymbra Lingua now reads French too, for English speakers: check French in Settings › Language. French words come with their meaning in English, written by people; the card names the tense and the gender ("past historic (passé simple)"), and your French level is estimated from word frequency. Extended translation turns your French selection straight into English, on your device.
```

### es-ES, es-MX, with fr-es (449 / 4000)

```
Cymbra Lingua ahora también lee francés, para los hispanohablantes: márcalo en Ajustes › Idioma. Las palabras francesas llegan con su significado en español, escrito por personas; la tarjeta nombra el tiempo verbal y el género («pretérito perfecto simple de indicativo»), y tu nivel de francés se estima según la frecuencia de las palabras. La traducción ampliada traduce tu selección en francés al español, pasando por el inglés, en tu dispositivo.
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

## What's New — 1.8.1

Submitted on 2026-10-16: iOS build 260, macOS build 261, in the five locales (en-GB as en-US, es-MX
as es-ES), released automatically once approved. 1.8.0 never left TestFlight: its page called a
local backend (#963), so 1.8.1 is the first public build since 1.7.0 and its notes cover both. What
a reader sees change: signing in from the app, with an email and a password too (add-lingua-app-email-signin),
the app syncing its own reviews (add-lingua-app-deck-sync), editing and ending a review
(add-lingua-card-edit, the end of a session in progress), a translation kept on the card, the count
on the toolbar icon and the anonymous counts (add-lingua-usage-counts). Online translation is still
not listed: a beta opened per account. The anonymous counts are said, as they are on by default in
Safari. Every label is quoted from `apps/lingua-extension/src/i18n/`. The promotional text was pasted
again (es-ES and es-MX with fr-es), and the review notes carried over from 1.7.0 unchanged.

### fr-FR (755 / 4000)

```
Connexion dans l'app : connectez-vous avec Apple, Google ou une adresse e-mail et un mot de passe. Les révisions faites dans l'app se synchronisent avec vos autres appareils.

Révision : « Modifier la carte » corrige une carte pendant la révision, et « Terminer » arrête la séance en cours. Une traduction trouvée à la révision est gardée sur la carte.

Dans Safari, l'icône de l'extension peut afficher le nombre de cartes à réviser (Réglages › Afficher les cartes à réviser sur l'icône).

Comptages anonymes : l'extension envoie quelques comptages d'utilisation, jamais ce que vous lisez. Vous pouvez les couper dans Réglages (Envoyer des comptages anonymes).

Corrections, dont la traduction des verbes modaux anglais et la ponctuation autour des mots.
```

### en-US, en-GB (555 / 4000)

```
Sign in from the app: with Apple, Google, or an email address and a password. Reviews done in the app sync with your other devices.

Review: "Edit card" corrects a card during review, and "End" stops the session in progress. A translation found at review stays on the card.

In Safari, the extension's icon can show the number of cards to review (Settings › Show the cards to review on the icon).

Anonymous usage counts: the extension sends a few usage counts, never what you read. You can turn them off in Settings (Send anonymous usage counts).

Fixes.
```

### es-ES, es-MX (638 / 4000)

```
Inicio de sesión en la app: con Apple, Google o una dirección de correo y una contraseña. Los repasos hechos en la app se sincronizan con tus otros dispositivos.

Repaso: «Editar la tarjeta» corrige una tarjeta durante el repaso, y «Terminar» detiene la sesión en curso. Una traducción encontrada en el repaso se guarda en la tarjeta.

En Safari, el icono de la extensión puede mostrar el número de tarjetas por repasar (Ajustes › Mostrar las tarjetas por repasar en el icono).

Recuentos anónimos: la extensión envía algunos recuentos de uso, nunca lo que lees. Puedes desactivarlos en Ajustes (Enviar recuentos anónimos).

Correcciones.
```

## What's New — 1.7.0

Submitted on 2026-10-13: iOS build 240, macOS build 241, in the five locales (en-GB as en-US, es-MX
as es-ES). What a reader sees change since 1.6.0: the app reviews the deck (add-lingua-apple-review),
the review of refine-lingua-review-session, add-lingua-review-practice and
add-lingua-review-repeat-hint, the review translation, the chapter recap and the pages left. Online
translation is not listed: it is a beta opened per account, said in the review notes only. Every
label is quoted from `apps/lingua-extension/src/i18n/`, except one: App Store Connect refuses « ᵉ »
(U+1D49) as an invalid character, so the French notes write « 2e passage » where the review says
« 2ᵉ passage ». The promotional text is not carried over to a new version: paste it again with
each one.

### fr-FR (869 / 4000)

```
L'app Cymbra Lingua révise votre deck : ouvrez-la pour réviser les mots capturés dans Safari, sur iPhone, iPad et Mac.

Révision repensée : des séances courtes de 10 cartes, les plus fragiles d'abord. La carte montre la phrase où vous avez rencontré le mot, avec deux réponses, « Pas su » et « Su », et « Ne plus me le montrer ». Une carte ratée revient dans la séance (« À revoir · 2e passage »). Les nouveaux mots entrent en révision à votre rythme (Réglages › Nouveaux mots par jour). Rien à réviser ? « S'entraîner quand même », sans rien décaler.

Une carte sans traduction dans le dictionnaire est traduite à la révision, sur l'appareil, si la traduction étendue est activée.

Livres : à la fin d'un chapitre, triez les mots que vous avez regardés. Le pied de page affiche les pages restantes d'un toucher.

Corrections, dont une déconnexion inattendue du compte.
```

### en-US, en-GB (779 / 4000)

```
The Cymbra Lingua app now reviews your deck: open it to review the words you captured in Safari, on iPhone, iPad and Mac.

Review, redesigned: short sessions of 10 cards, the most fragile first. The card shows the sentence where you met the word, with two answers, "Didn't know" and "Knew it", and "Don't show it again". A missed card comes back in the session ("Again · 2nd time"). New words enter review at your pace (Settings › New words per day). Nothing due? "Practice anyway", without moving any schedule.

A card the dictionary has no translation for is translated at review, on your device, when extended translation is on.

Books: at the end of a chapter, sort the words you looked up. The footer shows the pages left, at a tap.

Fixes, including an unexpected sign-out.
```

### es-ES, es-MX (880 / 4000)

```
La app Cymbra Lingua ahora repasa tu mazo: ábrela para repasar las palabras que guardaste en Safari, en iPhone, iPad y Mac.

Repaso renovado: sesiones cortas de 10 tarjetas, las más frágiles primero. La tarjeta muestra la frase donde encontraste la palabra, con dos respuestas, «No la sabía» y «La sabía», y «No volver a mostrarla». Una tarjeta fallada vuelve en la sesión («Otra vez · 2.ª vez»). Las palabras nuevas entran en el repaso a tu ritmo (Ajustes › Palabras nuevas al día). ¿Nada que repasar? «Practicar de todos modos», sin mover ninguna fecha.

Una tarjeta sin traducción en el diccionario se traduce en el repaso, en tu dispositivo, si la traducción ampliada está activada.

Libros: al final de un capítulo, clasifica las palabras que consultaste. El pie de página muestra las páginas restantes con un toque.

Correcciones, entre ellas un cierre de sesión inesperado.
```

## What's New — 1.5.0 (696 / 4000)

Submitted on 2026-10-07: iOS build 210, macOS build 211. The app now ships the Spanish → French
pair beside the English one (`apps/lingua-extension/packs.json`). The first paragraph is the
Spanish paragraph of add-lingua-spanish-listings; the rest is what a Safari reader sees change
since 1.4.0.

```
Espagnol : lisez aussi l'espagnol. Cochez-le dans Réglages › Langue : mots surlignés, carte avec le temps et le genre, niveaux estimés, une voix d'Espagne pour la lecture à voix haute, et la traduction étendue en passant par l'anglais.

Révision : elle se fait dans la langue de la page ou du livre ouvert, et ses compteurs suivent cette langue.

Carte de mot : la fréquence réelle du mot dans l'usage courant, et un bouton pour écouter aussi sa forme du dictionnaire.

Traduction étendue : votre sélection est mise en gras dans la phrase traduite, en espagnol comme en anglais, et les appels de note des pages sont ignorés.

Corrections, dont les statistiques qui restent dans la langue choisie.
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
`add-lingua-english-listings` measured 4,042, over it; that change cut it to 3,973,
`add-lingua-spanish-audience-listings` added the Spanish path (3,976), and
`add-lingua-french-listings` added French (3976 with fr-es, 3935 without), the correction for
« Traduction en ligne » (below) measured 3988 and 3947, and `add-lingua-usage-counts` names the
anonymous counts on top of it and measures **3984** with fr-es, **3943** without it. A
reviewer's device that is not in French or Spanish gets the English interface studying Spanish (the
native language is preset from the browser's language: `add-lingua-native-language-choice`, D4;
Spanish stays an English reader's first language once change 52 lists fr-en after es-en), so the
steps follow that path, every label quoted from `apps/lingua-extension/src/i18n/en/` — `card.ts`
("I know it", "+ Deck", "Ignore"), `drawer.ts` and `hud.ts` (Review, Stats, Settings), `settings.ts`
(the tabs, "Languages studied", "Manage my data"), `languages.ts` ("French"), `account.ts` ("Erase
my Lingua data…", "Delete my Cymbra account"), `translation.ts` ("Extended translation"),
`review.ts` ("Sources & privacy"). Step 6 adds French. "Purpose & audience" says, once, what each
reader studies; "Regional differences" gives a device in Spanish its path, English studied, in the
Spanish catalogue's labels (`src/i18n/es/`: `languages.ts` « Elige tu nivel de inglés », the popup's
level prompt; `card.ts` « La conozco », « + Mazo », « Ignorar »; `drawer.ts` « Repaso »,
« Ajustes »). "Third-party material" names the sources of the glosses in each language — the
English, French and Spanish Wiktionaries' definitions and translation tables (`tables/*/README.md`)
— and UD French-GSD beside Spanish-GSD; the models are es-en's, fr-en's and en-es's, their sizes
from `model-manifest.json` (Spanish to English 23,288,494 + 2,543,246 + 409,312 B and French to
English 23,175,075 + 2,649,934 + 409,706 B, 26.2 MB each; English to Spanish 22,698,792 + 2,265,250
+ 409,312 B = 25.4 MB, as the setting rounds them; French to Spanish chains fr-en and en-es). The
deletion link is the English interface's (`deleteAccountUrl`, `src/account/flow.ts`).

To fit French — step 6 (+104), the models (+40), the audiences said once in "Purpose & audience"
(+37) — wording was cut, and no step a reviewer needs (add-lingua-french-listings, D4): the account
paragraph lost « ; without one, it stays on this device », which its first sentence already says
(every feature works signed out), and « The Cymbra account » / « an account » became « An optional
Cymbra account » / « one » (−37); "External services" lost « (bundled WebAssembly engines + offline
dictionaries) », said where the engine ships and the dictionaries are bundled, and its last sentence
became « No payment processor and no in-app purchase » — no listing text speaks of price, and the
payment processor, the external service Apple asks about, is still answered (−71); "Regional differences" lost the audiences, now in "Purpose &
audience" (−68); "Third-party material" lost « for Spanish » and « for both » (−5). Every step, the
deletion path, every external service and every source are still there. Without fr-es,
« Spanish speakers English and French, » reads « Spanish speakers English, » and « ; French to
Spanish chains two » goes.

M15 is settled: extended translation opens with each pair; should change 52 not offer one, its
models go from the OPTIONAL paragraph and the count only falls.

Corrected on 2026-10-14 for « Traduction en ligne » (`add-lingua-online-translation`): "External
services" said « No AI/LLM API », which stopped being true when online translation shipped. It now
says that api.cymbra.app, for accounts an operator granted it, relays "Online translation" (the
English label, `settings.ts`): off by default, one sentence per press of the card's button, sent to
Google Gemini or, when Gemini fails, to Anthropic Claude (`backend/lingua/src/providers/mod.rs`,
Gemini first by default), and kept by no one. That sentence costs +152; the rest was paid for by
cutting wording (−140): the opening sentence lost « : every feature works signed out », no longer
true of that feature; « the learner's », « is », « chosen » and « real » went; step 5 lost « the
account is under Data », already in the account paragraph; « that data » became « it », « The
translation engine » « Its engine », « the reader's » « their », « combine » « use », « frequency
lists » « frequencies »; the `https://` of models.cymbra.app and api.cymbra.app went, and the
Safari sentence was reworded. Every step, the deletion path, every external service and every
source are still there: **3988** with fr-es, **3947** without it.

To name the anonymous counts (add-lingua-usage-counts, written on 2026-10-14 and applied on
2026-10-16 on top of the correction above), "External services" no longer says « none by
default » or « no analytics »: it says the counts are on by default under a legitimate interest
once the popup has said so, that they are sent as they happen, gives their six steps, the three
things sent with them (install day, browser, version), no identifier and no page text, and where
they are turned off (+281). Wording was cut, and no step a reviewer needs: the account paragraph
lost « (iPhone, iPad, Mac) », its progress list became « (words, deck, level, statistics) » and
its test path « To test it: panel > Settings > Data > Account, … » (−67); the IMPORTANT line says
« Enable the extension in Safari first. » (−20); step 2 lost « the engine assumes zero known
words: », the reason for what it shows (−37); "OPTIONAL" lost « - then the selected sentence is
translated on the device », which "External services" says, and gives the models' sizes as one
range, « 25.4-26.2 MB each » (−109); "Purpose & audience" lost « (A1-C2) », which step 5 gives
(−8); "External services" lost « the » before « level estimate » and says « No payment processor,
no in-app purchase » (−7); "Third-party material" lost « (definitions and translation tables) »
(−37). Every step, the deletion path, every external service and every source are still there:
**3984** with fr-es, **3943** without it.

### For the next submission (3956 / 4000)

The notes on the dashboard are still 1.7.0's (below), carried unchanged to 1.8.1, and they say
« No analytics » — which the anonymous counts (`add-lingua-usage-counts`, shipped in 1.8.1) have made
untrue, and which that change's requirement forbids an App Review note to say. The block written
for the counts (« English, for every locale, with fr-es », 3984, below) was never pasted, and it
predates 1.7.0: it has no step 7, and its IMPORTANT line says the app's screen only explains. This
is that block made true of 1.7.0 and later: step 7 restored as 1.7.0 wrote it (+70), the IMPORTANT
line « IMPORTANT - this app hosts a Safari extension: enable it in Safari first. » (−43), and, to
stay within 4,000 characters, the account paragraph without « (words, deck, level, statistics) »
(−33), the engine without « , WebAssembly » (−13) and « highlighted in place » become
« highlighted » (−9). Every step, the deletion path, every external service and every source are
still there: **3956**. Paste it at the next submission, whatever it ships.

```
No account is needed to review this app. An optional Cymbra account saves progress across reinstalls and devices. To test it: panel > Settings > Data > Account, Sign in with Apple or Google - no invitation needed.

IMPORTANT - this app hosts a Safari extension: enable it in Safari first.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level, every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs).
6. French: Settings > Language > "Languages studied" > "French", then a French level, on a French page.
7. Open the app: it reviews the deck built in Safari (on the device).

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases it on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). Its engine (Mozilla's Firefox Translations) ships inside the app. Turning it on downloads data only - a model per pair studied (25.4-26.2 MB each; French to Spanish chains two) from models.cymbra.app, checked against a pinned sha256. No page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps people learn a language by reading web pages and their own DRM-free EPUB books: unknown words are highlighted, looked up offline, captured into a deck reviewed with spaced repetition, and their CEFR level is estimated from the words they marked. English speakers learn Spanish and French, Spanish speakers English and French, French speakers English and Spanish.

External services: highlighting, lookup, level estimate and translation run on the device; no ads. By default (legitimate interest), once the popup has said so, anonymous counts go to api.cymbra.app (our backend) as they happen: install, level chosen, first page analysed, return on day 1, 7-13, 30-59 - each once, with install day, browser, version; no identifier, no page text. Off: Settings > Data > "Anonymous usage counts". Only on the reader's action: models.cymbra.app serves the models above; the backend syncs the word list when signed in and, for accounts we grant it, relays the AI-based "Online translation" (off by default): each button press sends one sentence to Google Gemini, else Anthropic Claude, kept by no one; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor, no in-app purchase.

Regional differences: none. The interface follows the device's language (French, Spanish, else English); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries use sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD and French-GSD treebanks; wordfreq frequencies; kaikki.org extracts of the English, French and Spanish Wiktionaries for the glosses - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

### As submitted with 1.7.0 (3976 / 4000)

Pasted on 2026-10-13 for iOS build 240 and macOS build 241: the block with fr-es below, changed
where 1.7.0 made it untrue. The app is no longer an explanation only: it reviews the deck built in
Safari (step 7). « No AI/LLM API » went: online translation sends the sentence a reader chooses to
our backend and on to Google Gemini or Anthropic Claude, said as a beta enabled per account. To stay
within 4,000 characters, the account paragraph, the audiences and the dictionary sources were
shortened; every step, the deletion path, every external service and every source are still there.

```
No account is needed: every feature but online translation works signed out. An optional Cymbra account saves progress (known words, deck, level, statistics) across reinstalls and devices. To test it: panel > Settings > Data > Account, Sign in with Apple or Google - no invitation needed.

IMPORTANT - this app hosts a Safari extension, which must be enabled in Safari before anything happens. Until then the app only explains how.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level, every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs; the account is under Data).
6. French: Settings > Language > "Languages studied" > "French", then a French level, on a French page.
7. Open the app: it reviews the deck built in Safari (on the device).

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases that data on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app. Turning it on downloads data only - a model per pair studied (Spanish or French to English 26.2 MB each, English to Spanish 25.4 MB; French to Spanish chains two) from https://models.cymbra.app, checked against a pinned sha256 - and the selected sentence is then translated on the device. No page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps people learn a language by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked. Readers study English, Spanish or French, other than their own.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device. No analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the models above; https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). "Online translation" (a beta enabled per account on request, then turned on in Settings): its button sends the selected sentence to our backend, translated by Google Gemini or Anthropic Claude, kept by no one. No payment processor and no in-app purchase.

Regional differences: none. The interface follows the device's language (French, Spanish, else English); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries combine sources licensed for commercial use - ESDB (SCOWL), CEFR-J and Octanove for English; the UD Spanish-GSD and French-GSD treebanks; wordfreq; kaikki.org extracts of the English, French and Spanish Wiktionaries - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

### English, for every locale, with fr-es (3984 / 4000)

```
No account is needed to review this app. An optional Cymbra account saves progress (words, deck, level, statistics) across reinstalls and devices. To test it: panel > Settings > Data > Account, Sign in with Apple or Google - no invitation needed.

IMPORTANT - this app is a Safari extension host: its own screen only explains. Enable the extension in Safari first.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level, every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs).
6. French: Settings > Language > "Languages studied" > "French", then a French level, on a French page.

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases it on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). Its engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app. Turning it on downloads data only - a model per pair studied (25.4-26.2 MB each; French to Spanish chains two) from models.cymbra.app, checked against a pinned sha256. No page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps people learn a language by reading web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and their CEFR level is estimated from the words they marked. English speakers learn Spanish and French, Spanish speakers English and French, French speakers English and Spanish.

External services: highlighting, lookup, level estimate and translation run on the device; no ads. By default (legitimate interest), once the popup has said so, anonymous counts go to api.cymbra.app (our backend) as they happen: install, level chosen, first page analysed, return on day 1, 7-13, 30-59 - each once, with install day, browser, version; no identifier, no page text. Off: Settings > Data > "Anonymous usage counts". Only on the reader's action: models.cymbra.app serves the models above; the backend syncs the word list when signed in and, for accounts we grant it, relays the AI-based "Online translation" (off by default): each button press sends one sentence to Google Gemini, else Anthropic Claude, kept by no one; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor, no in-app purchase.

Regional differences: none. The interface follows the device's language (French, Spanish, else English); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries use sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD and French-GSD treebanks; wordfreq frequencies; kaikki.org extracts of the English, French and Spanish Wiktionaries for the glosses - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

### English, for every locale, without fr-es (3943 / 4000)

```
No account is needed to review this app. An optional Cymbra account saves progress (words, deck, level, statistics) across reinstalls and devices. To test it: panel > Settings > Data > Account, Sign in with Apple or Google - no invitation needed.

IMPORTANT - this app is a Safari extension host: its own screen only explains. Enable the extension in Safari first.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level, every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs).
6. French: Settings > Language > "Languages studied" > "French", then a French level, on a French page.

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases it on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). Its engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app. Turning it on downloads data only - a model per pair studied (25.4-26.2 MB each) from models.cymbra.app, checked against a pinned sha256. No page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps people learn a language by reading web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and their CEFR level is estimated from the words they marked. English speakers learn Spanish and French, Spanish speakers English, French speakers English and Spanish.

External services: highlighting, lookup, level estimate and translation run on the device; no ads. By default (legitimate interest), once the popup has said so, anonymous counts go to api.cymbra.app (our backend) as they happen: install, level chosen, first page analysed, return on day 1, 7-13, 30-59 - each once, with install day, browser, version; no identifier, no page text. Off: Settings > Data > "Anonymous usage counts". Only on the reader's action: models.cymbra.app serves the models above; the backend syncs the word list when signed in and, for accounts we grant it, relays the AI-based "Online translation" (off by default): each button press sends one sentence to Google Gemini, else Anthropic Claude, kept by no one; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor, no in-app purchase.

Regional differences: none. The interface follows the device's language (French, Spanish, else English); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries use sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD and French-GSD treebanks; wordfreq frequencies; kaikki.org extracts of the English, French and Spanish Wiktionaries for the glosses - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

### As submitted with 1.5.0 (3950 / 4000)

Pasted on 2026-10-07 for iOS build 210 and macOS build 211, before the English interface: the
labels are the French ones, and Spanish is the one new path. A record, not a text to paste again:
it also predates « Traduction en ligne » (2026-10-12), so its « No AI/LLM API » no longer holds —
paste one of the blocks above.

```
No account is needed to review this app: every feature works signed out. The optional Cymbra account saves the learner's progress (known words, deck, level, statistics) so that it survives reinstalling the app or changing device, on iPhone, iPad and Mac alike; without one, it stays on this device and is lost with the app. To test it: panel > Réglages > Données > Compte, Sign in with Apple or Google - no invitation is needed.

IMPORTANT - this app is a Safari extension host. Installing it shows only a short explanatory screen: the extension must be enabled in Safari before anything happens.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu and pick an English level (B1 is a good default). With none, the engine assumes zero known words: every word is highlighted and the pill reads 0%, which looks broken.
3. Open any English-language page. A tab opened BEFORE the extension was enabled, or before the level was picked, must be reloaded.
4. Words above your level are highlighted and the pill shows the share of the page you already know. Tap a highlighted word: a card gives its translation, dictionary form and frequency, with "Je connais" (I know this), "+ Deck" (add to deck), "Ignorer" (ignore).
5. Tap the pill to open the panel: Révision (review), Stats (estimated vocabulary, A1 to C2), Réglages (settings).

NEW IN 1.5.0 - Spanish: in Réglages > Langue, tick "Espagnol" and pick a Spanish level, then open a Spanish-language page (e.g. es.wikipedia.org).

Account deletion: panel > Réglages > Données, signed in > "Gérer mes données" opens the account page. "Effacer mes données Lingua" erases the Lingua data on the server and every device, keeping the account. "Supprimer mon compte Cymbra" opens https://cymbra.app/suppression-compte/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Traduction étendue" (Settings, off by default). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app; nothing executable is downloaded. Turning it on downloads only the translation models - 25.8 MB, 52.0 MB with Spanish - from https://models.cymbra.app, checked against pinned sha256. The sentence is translated on the device; no page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps French speakers learn English and Spanish by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device (bundled engines and dictionaries). No AI/LLM API, no analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the translation models (Traduction étendue); https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor: the app is free, with no in-app purchase.

Regional differences: none - the app behaves identically everywhere it is available. The interface is in French: it teaches English and Spanish to French speakers.

Third-party material: the bundled dictionaries combine sources licensed for commercial use - kaikki.org extracts of the French, English and Spanish Wiktionaries, wordfreq frequencies, ESDB (SCOWL) for English inflections, the Universal Dependencies Spanish-GSD treebank, and the CEFR-J and Octanove vocabulary-level lists - credited in the extension's "Sources & confidentialité" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

## Screenshots

Apple requires a set **per platform and per locale**, at the sizes each slot names (see *what the
slots actually asked for*, below): iPhone 6.5" 1284 x 2778 — a 6.9" capture is refused there —,
iPad 13" 2064 x 2752, macOS 1280 x 800, 1440 x 900, 2560 x 1600 or 2880 x 1800.

What they should show, in this order: a page being read with its highlighting and the pill, the
word card open on a highlighted word, the review panel, and the statistics screen with the
estimated vocabulary — and, where a locale gains French, a fifth on a French page (App Store
Connect takes ten per slot). Every slot, macOS's 1280 x 800 included, is captured in Safari by hand,
and never filled from the browser listings' captures (`yarn capture` in `apps/lingua-extension`,
change add-site-lingua-showcase): those are Chromium's rendering of the extension, not the Safari
app's, and App Review asks for the app in use (Guideline 2.3.3).

| Locale | iPhone 6.5" | iPad 13" | macOS | Taken from |
|---|---|---|---|---|
| fr-FR | sent | sent | to capture | the French interface on an English page |
| en-US | to capture | to capture | to capture | a build of change 34, the device in English: the English interface on a Spanish page; the fifth from a build of change 52, on a French page |
| en-GB | en-US's set | en-US's set | en-US's set | — |
| es-ES | to capture | to capture | to capture | a build of change 35, the device in Spanish: the Spanish interface on an English page; the fifth, if change 52 lists fr-es, from its build, on a French page |
| es-MX | es-ES's set | es-ES's set | es-ES's set | — |

The en-US set, captured by the owner (the device in English, a Spanish level chosen — B1):

1. **Reading** — a Spanish article in Safari (a Spanish Wikipedia page reads well), highlighting
   on, the pill showing the share of the page known.
2. **The word card** — open on a highlighted verb form whose card names a tense: « dijo » reads
   "third-person singular preterite indicative of decir" (`test/baseline/word-card-es-en.txt`),
   its English gloss below, "I know it", "+ Deck" and "Ignore".
3. **Review** — the panel ("Review") on a card, its answer revealed below its sentence, the two
   answers ("Didn't know" and "Knew it").
4. **Statistics** — "Stats", the ladder "My estimated Spanish level" from A1 to C2, with its note
   that the levels are estimated from word frequency.
5. **French** (from change 52, a French level chosen) — a French article in Safari (a French
   Wikipedia page reads well), the card open on « fut »: "third-person singular past historic
   (passé simple) of être" (`apps/lingua-extension/test/baseline/word-card-fr-en.txt`), its English
   gloss below.

On macOS, the same five in Safari for Mac.

The es-ES set, captured by the owner (the device in Spanish, an English level chosen — B1):

1. **Reading** — an English article in Safari (an English Wikipedia page reads well), highlighting
   on, the pill showing the share of the page known.
2. **The word card** — open on a highlighted past form: « went » reads « pasado simple de go »
   (`src/i18n/es/grammar.ts`, pinned by `test/word-grammar-es.spec.ts`; change 24's snapshot of the
   card in Spanish, once merged, gives the real readings), its Spanish gloss below, « La conozco »,
   « + Mazo » and « Ignorar ».
3. **Review** — the panel (« Repaso ») on a card, its answer revealed below its sentence, the two
   answers (« No la sabía » and « La sabía »).
4. **Statistics** — « Stats », the ladder « Mi nivel de inglés » from A1 to C2 (MCER), its levels
   English's own lists, not estimated.
5. **French** (if change 52 lists fr-es, a French level chosen) — the same French page, the card
   open on « fut »: « tercera persona del singular del pretérito perfecto simple de indicativo de
   être » (`apps/lingua-extension/test/baseline/word-card-fr-es.txt`), its Spanish gloss below.

On macOS, the same five in Safari for Mac (four without fr-es).

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

From change 52 (`enable-lingua-french`, D7) each page's step 3 also says where another language is
ticked, so that it stays true for a reader of several: *"… choose your level of Spanish. For French,
check it in the extension's Settings, under Language, then choose your level of French."*; with
fr-es, *« … elige tu nivel de inglés. Para el francés, márcalo en los Ajustes de la extensión,
pestaña Idioma, y luego elige tu nivel de francés. »*; and in French, *« … choisis ton niveau
d'anglais. Pour l'espagnol, coche-le dans les Réglages de l'extension, onglet Langue, puis choisis
ton niveau d'espagnol. »* The en-US description's step 3 and the es-ES one with fr-es carry that
sentence word for word — « of the extension » keeping its Settings apart from step 1's, the
system's —, and the fr-FR one in its own register (« cochez-le dans les Réglages de l'extension,
onglet Langue »: vous, where the page says tu). The owner settled all three on 2026-10-10.

## Screenshots — what the slots actually asked for

Captured from the simulators, at the sizes App Store Connect names on the page itself:

| Slot | Accepted sizes | What was sent |
|---|---|---|
| iPhone 6.5" | 1242 x 2688, 1284 x 2778 | 1284 x 2778, rescaled from a 6.9" capture |
| iPad 13" | 2064 x 2752, 2048 x 2732 | 2064 x 2752, captured natively |
| macOS | 1280 x 800, 1440 x 900, 2560 x 1600, 2880 x 1800 | still to capture |

A 6.9" capture (1320 x 2868) is **refused** by the 6.5" slot. The dashboard states its own
sizes; read them there rather than assuming the newest device is the right one.
