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

## Fields

| Field | Limit | Value |
|---|---|---|
| Name | 30 | `Cymbra Lingua` (13) |
| Subtitle | 30 | `Anglais et espagnol en lisant` (29) |
| Promotional text | 170 | 154 characters, below |
| Description | 4000 | 2964 characters, below |
| Keywords | 100 | 92 characters, below |
| Support URL | — | `https://cymbra.app/support/` |
| Marketing URL | — | `https://cymbra.app/lingua/` |
| Privacy policy | — | `https://cymbra.app/confidentialite/` |
| Category | — | Education (secondary: Reference) |
| Age rating | — | 4+ |

## Promotional text (154 / 170)

Editable without a review, unlike everything else here.

> Les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne.

## Keywords (92 / 100)

Comma-separated, no spaces — a space costs a character and buys nothing. The app name and the
category are already indexed, so neither appears here.

```
anglais,espagnol,vocabulaire,lecture,traduction,extension,apprendre,mots,révision,CECRL,deck
```

## Description (2964 / 4000)

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

L'interface est en français : Cymbra Lingua enseigne l'anglais et l'espagnol à des francophones.
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

```
No account is needed to review this app: every feature works signed out. The Cymbra account is what saves the learner's progress in learning the language - known words, deck, level and statistics - so that it survives deleting and reinstalling the app, or replacing or resetting the device, and is the same on all their devices (iPhone, iPad, Mac). Without an account, that progress lives only on this device and is lost if the app is deleted. To test it, create an account from the extension (panel > Réglages > Données > Compte) with Sign in with Apple or Google - no invitation is needed.

IMPORTANT - this app is a Safari extension host. Installing it shows only a short explanatory screen: the extension must be enabled in Safari before anything happens.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu and pick an English level - B1 is a good default. With no level chosen the engine assumes zero known words, so every word is highlighted and the pill reads 0%, which looks broken rather than unconfigured.
3. Open any English-language page. A tab opened BEFORE the extension was enabled, or before the level was picked, must be reloaded.
4. Words above your level are highlighted and the pill shows the share of the page you already know. Tap a highlighted word: a card gives its translation, dictionary form and frequency, with "Je connais" (I know this), "+ Deck" (add to deck), "Ignorer" (ignore).
5. Tap the pill to open the panel: Révision (review), Stats (estimated vocabulary, A1 to C2), Réglages (settings, in four tabs; the account is under Données).

Account deletion: panel > Réglages > Données, signed in > "Gérer mes données" opens the account page. "Effacer mes données Lingua" erases the reader's Lingua data on the server and every device, keeping the account. "Supprimer mon compte Cymbra" opens https://cymbra.app/suppression-compte/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Traduction étendue", off by default (Settings). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app; nothing executable is downloaded. Turning it on downloads one data file - the translation model, 25.8 MB, from https://models.cymbra.app, checked against a pinned sha256 - and the selected sentence is then translated on the device. No page text, account or device identifier is sent. Turning it off deletes the model.

Purpose & audience: Cymbra Lingua helps French speakers learn English by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device (bundled WebAssembly engines + offline dictionary). No AI/LLM API, no analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the translation model when "Traduction étendue" is turned on; https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor: the app is free, with no in-app purchase.

Regional differences: none - the app behaves identically everywhere it is available. The interface is in French: it teaches English to French speakers.

Third-party material: the bundled dictionary combines sources licensed for commercial use - ESDB (the English Speller Database from the SCOWL project) for inflections, the wordfreq frequency list, a kaikki.org extract of the French Wiktionary, and the CEFR-J and Octanove vocabulary-level lists - credited in the extension's "Sources & confidentialité" panel. The translation model is Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

## Screenshots

Apple requires a set **per platform**, and the sizes are imposed:

| Platform | Size | Status |
|---|---|---|
| iPhone 6.9" | 1320 x 2868 or 1290 x 2796 | to capture |
| iPad 13" | 2064 x 2752 or 2048 x 2732 | to capture |
| macOS | 1280 x 800, 1440 x 900, 2560 x 1600 or 2880 x 1800 | to capture |

What they should show, in this order: a page being read with its highlighting and the pill, the
word card open on a highlighted word, the review panel, and the statistics screen with the
estimated vocabulary. The same four the browser listings use — they are what the product is.

## What the app itself says, and why it settles the wording

The host app's own first screen reads: *"Dans Safari, ouvre l'extension depuis le menu de la
barre d'adresse et choisis ton niveau d'anglais."* An earlier draft of this listing said to open
the app to pick the level. That was wrong — the app shows an explanatory screen and nothing
else; the level lives in the extension, inside Safari. Copy that contradicts the product is
worse than copy that says too little: the reader follows it, finds nothing, and concludes the
app is broken.

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
