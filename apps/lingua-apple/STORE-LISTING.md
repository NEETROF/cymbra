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
| Subtitle | 30 | `Vocabulaire anglais en lisant` (29) |
| Promotional text | 170 | 140 characters, below |
| Description | 4000 | 2116 characters, below |
| Keywords | 100 | 83 characters, below |
| Support URL | — | `https://cymbra.app/support/` |
| Marketing URL | — | `https://cymbra.app/lingua/` |
| Privacy policy | — | `https://cymbra.app/confidentialite/` |
| Category | — | Education (secondary: Reference) |
| Age rating | — | 4+ |

## Promotional text (140 / 170)

Editable without a review, unlike everything else here.

> Les mots d'anglais que vous ne connaissez pas encore, surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne.

## Keywords (83 / 100)

Comma-separated, no spaces — a space costs a character and buys nothing. The app name and the
category are already indexed, so neither appears here.

```
anglais,vocabulaire,lecture,traduction,extension,apprendre,mots,révision,CECRL,deck
```

## Description (2116 / 4000)

**LIRE VOS LIVRES** describes the book reader (`add-lingua-reader`). Paste it only once that
change's iPhone measurement (task 1.5) has kept the reader in the Safari variant; if it has
not, the paragraph goes and the count drops back to 1947 (the 1838 this file used to state was stale).

```
Cymbra Lingua est une extension Safari qui surligne, sur la page que vous lisez, les mots d'anglais que vous ne connaissez pas encore — sans rien changer à la mise en page. Une pastille vous dit quelle part du texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.

APRÈS L'INSTALLATION
Cette app installe l'extension ; il reste à l'activer.
1. Ouvrez Réglages > Apps > Safari > Extensions (sur Mac : Safari > Réglages > Extensions).
2. Activez Cymbra Lingua, puis autorisez-la sur les sites que vous lisez.
3. Dans Safari, ouvrez l'extension depuis le menu de la barre d'adresse et choisissez votre niveau d'anglais — sans lui, l'extension considère que vous ne connaissez aucun mot.
Rechargez les onglets déjà ouverts pour qu'ils soient surlignés.

LIRE
Touchez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté en anglais courant. Puis décidez — « Je connais », « + Deck » pour le réviser plus tard, ou « Ignorer ». Sélectionnez plusieurs mots pour capturer une expression entière avec la phrase d'où elle vient.

RÉVISER
Les cartes que vous créez se révisent dans un panneau, à côté de votre lecture, avec une répétition espacée qui décide toute seule du bon moment. Un écran de statistiques estime votre vocabulaire niveau par niveau, du A1 au C2, à partir des mots que vous avez marqués.

LIRE VOS LIVRES
Importez vos livres EPUB sans DRM dans la bibliothèque de l'extension et lisez-les hors ligne, avec le même surlignage. Ils restent sur votre appareil.

VOTRE LECTURE RESTE À VOUS
Le dictionnaire et le moteur d'analyse sont dans l'app. Aucune page que vous lisez n'est envoyée nulle part, et tout fonctionne hors ligne. Sans compte, l'extension ne fait aucune requête réseau.

UN COMPTE, SI VOUS EN VOULEZ UN
Créez un compte Cymbra si — et seulement si — vous voulez retrouver vos mots et vos cartes sur vos autres appareils. C'est la seule chose qui quitte votre machine, et vous pouvez effacer ces données depuis les réglages sans supprimer votre compte.

L'interface est en français : Cymbra Lingua enseigne l'anglais à des francophones.
```

## What's New — 1.1.0 (373 / 4000)

```
Première version publique.

Lecture assistée sur iPhone, iPad et Mac : surlignage des mots que vous n'avez pas encore marqués, traduction et rareté au toucher, capture d'expressions, révision par répétition espacée, et une estimation de votre vocabulaire du A1 au C2.

Vos données restent sur l'appareil ; le compte Cymbra est facultatif et ne sert qu'à la synchronisation.
```

## Review notes (App Review Information)

```
No account is needed to review this app. Signing in only synchronises a reader's own
vocabulary between their devices; every other feature works signed out.

IMPORTANT — the app is a Safari extension host. Installing it shows only a short
explanatory screen: the extension must be enabled in Safari before anything happens.

1. Open Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions),
   enable "Cymbra Lingua", and allow it on the site you will test.
2. In SAFARI, not in the app, open the extension from the address-bar menu and pick an
   English level (B1 is a good default). This matters: with no level chosen the engine
   assumes zero known words, so every word is highlighted and the pill reads 0% — which
   looks like a broken extension rather than an unconfigured one. A tab opened before the
   extension was enabled, or before the level was picked, must be reloaded: the level
   applies to pages analysed after it is set.
3. In Safari, open any English-language page (a news article works well). Words above
   your level are highlighted and a pill shows the share of the page you already know.
4. Tap a highlighted word: a card gives its translation, its dictionary form and how
   common it is, with three actions — "Je connais", "+ Deck", "Ignorer".
5. Open the extension's panel to review the cards you captured, and the statistics
   screen for the estimated vocabulary.

The interface is in French: the app teaches English to French speakers.
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
