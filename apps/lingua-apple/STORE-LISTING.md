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

With fr-FR primary, a storefront whose language has no localisation shows the French listing: a
German App Store user reads French while the app opens in English (M13, change 28) — a
consequence to weigh when settling M16.

Every English text is a draft for the owner's review (M9), true of what change 34 ships: Spanish
studied, glossed in English, the interface in English. No locale calls the app a beta or a trial,
or says anything of price (guidelines 2.2 and 2.3.7). Each count is the text's length in
characters as App Store Connect counts them — every character inside the block or after `> `,
line breaks included — and equals the text it heads.

## Fields

| Field | Limit | fr-FR | en-US, en-GB |
|---|---|---|---|
| Name | 30 | `Cymbra Lingua` (13) | `Cymbra Lingua` (13) |
| Subtitle | 30 | `Anglais et espagnol en lisant` (29) | `Learn Spanish while you read` (28) |
| Promotional text | 170 | 154 characters, below | 120 characters, below |
| Description | 4000 | 3021 characters, below | 2682 characters, below |
| Keywords | 100 | 92 characters, below | 99 characters, below |
| Support URL | — | `https://cymbra.app/support/` | `https://cymbra.app/en/support/` |
| Marketing URL | — | `https://cymbra.app/lingua/` | `https://cymbra.app/en/lingua/` |
| Privacy policy | — | `https://cymbra.app/confidentialite/` | `https://cymbra.app/en/privacy/` |
| Category | — | Education (secondary: Reference) | the app's, not a locale's |
| Age rating | — | 4+ | the app's, not a locale's |

## Promotional text

Editable without a review, unlike everything else here.

### fr-FR (154 / 170)

> Les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne.

### en-US, en-GB (120 / 170)

> The Spanish words you don't know yet, highlighted on the page you're reading. The analysis runs on your device, offline.

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

## What's New — the release that ships English (es-en)

The paragraph for English speakers, for the release that ships es-en (change 34); the rest of its
notes is written with the release, like every « What's New ». If M15 withholds es-en's
translation, its last sentence goes.

### en-US, en-GB (302 / 4000)

```
Cymbra Lingua now speaks English, for English speakers learning Spanish: the interface, the word card, review and statistics are in English, and Spanish words come with their meaning in English, written by people. Extended translation turns your Spanish selection straight into English, on your device.
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
`add-lingua-english-listings` measured 4,042, over it; this one measures **3973**. A reviewer's
device that is not in French gets the English interface studying Spanish (the native language is
preset from the browser's language: `add-lingua-native-language-choice`, D4), so the steps follow
that path, every label quoted from `apps/lingua-extension/src/i18n/en/` — `card.ts` ("I know it",
"+ Deck", "Ignore"), `drawer.ts` and `hud.ts` (Review, Stats, Settings), `settings.ts` (the tabs,
"Manage my data"), `account.ts` ("Erase my Lingua data…", "Delete my Cymbra account"),
`translation.ts` ("Extended translation"), `review.ts` ("Sources & privacy"). "Purpose &
audience" and "Regional differences" say which interface each reader gets and how to switch;
"Third-party material" names the Spanish sources (the English Wiktionary's Spanish section and the
Spanish Wiktionary's translations for es-en, beside es-fr's); the model is es-en's, its size from
`model-manifest.json` (23,288,494 + 2,543,246 + 409,312 B = 26.2 MB, as the setting rounds it). The
deletion link is the English interface's (`deleteAccountUrl`, `src/account/flow.ts`). If M15
withholds es-en's translation, the OPTIONAL paragraph says it is offered to readers of the French
interface only, with their models (English to French, 25.8 MB; with Spanish, 52.0 MB).

### English, for every locale (3973 / 4000)

```
No account is needed to review this app: every feature works signed out. The Cymbra account saves the learner's progress - known words, deck, level and statistics - so that it survives reinstalling the app or changing device, and is the same on iPhone, iPad and Mac. Without one, it lives only on this device. To test it, create an account from the extension (panel > Settings > Data > Account) with Sign in with Apple or Google - no invitation is needed.

IMPORTANT - this app is a Safari extension host: its own screen only explains. The extension must be enabled in Safari before anything happens.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level - B1 is a good default. With no level chosen the engine assumes zero known words: every word is highlighted and the pill reads 0%.
3. Open any Spanish-language page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you already know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs; the account is under Data).

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases that data on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app; nothing executable is downloaded. Turning it on downloads data only - the model for the reader's languages (Spanish to English: 26.2 MB) from https://models.cymbra.app, checked against a pinned sha256 - and the selected sentence is then translated on the device. No page text, account or device identifier is sent. Turning it off deletes the model.

Purpose & audience: Cymbra Lingua helps people learn a language by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked. English speakers learn Spanish; French speakers learn English and Spanish.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device (bundled WebAssembly engines + offline dictionaries). No AI/LLM API, no analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the translation model when "Extended translation" is turned on; https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor: the app is free, with no in-app purchase.

Regional differences: none - the app behaves identically everywhere. The interface follows the device's language: French for French speakers (learning English and Spanish), English for any other (learning Spanish); Settings > Language switches.

Third-party material: the bundled dictionaries combine sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD treebank for Spanish; wordfreq frequency lists and kaikki.org extracts of the English, French and Spanish Wiktionaries for both - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
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

## What the app itself says, and why it settles the wording

The host app's own first screen reads: *"Dans Safari, ouvre l'extension depuis le menu de la
barre d'adresse et choisis ton niveau d'anglais."* An earlier draft of this listing said to open
the app to pick the level. That was wrong — the app shows an explanatory screen and nothing
else; the level lives in the extension, inside Safari. Copy that contradicts the product is
worse than copy that says too little: the reader follows it, finds nothing, and concludes the
app is broken. Its English page (`localise-lingua-apple-host`, `copy.js`) says the same to an
English speaker — *"In Safari, open the extension from the address bar menu and choose your level
of Spanish."* — and the en-US description's step 3 follows it.

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
