# Design

## Context

Two files hold the listing copy, pasted by hand into the dashboards (CI uploads packages, not
listings):

| File | Store | Fields |
|---|---|---|
| `apps/lingua-extension/STORE-LISTING.md` | Chrome Web Store and addons.mozilla.org | Summary, description, single purpose, permissions, remote code, data usage, test instructions |
| `apps/lingua-apple/STORE-LISTING.md` | App Store, the Safari app | Subtitle, promotional text, keywords, description, « What's New » |

The summary comes from `manifest.json`. It names both languages since change 28, in the owner's
wording (109 characters).

## Goals / Non-Goals

**Goals:**
- Every listing names English and Spanish, and says honestly what Spanish has and lacks.
- A full draft for the owner to correct, within each store's limits.

**Non-Goals:**
- Screenshots, which are captured by hand.
- Listings in another language than French, with English for the extension's second description.
- Submitting anything.

## Decisions

### D1 — One listing per store, both languages

Each store keeps its single listing (programme decision D10). English comes first everywhere, as it
is the default studied language. The listing language stays French: the interface is French, and the
product teaches English and Spanish to French speakers.

### D2 — The extension listing (settled by the owner, 2026-10-05)

**Description, French:**

> Cymbra Lingua surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne
> connaissez pas encore — sans rien changer à la mise en page. Un pourcentage vous dit quelle part du
> texte vous est familière, calculée sur ce que vous avez réellement marqué, pas sur une estimation.
>
> Choisissez dans les Réglages les langues que vous apprenez : chaque page est lue dans la sienne.
> Plusieurs langues à la fois : gratuit pour l'instant.
>
> Cliquez un mot surligné : sa traduction, sa forme du dictionnaire et sa rareté dans l'usage courant.
> En espagnol, la carte nomme aussi le temps et le genre, comme on les apprend en classe (« passé
> simple », « nom féminin »). Puis décidez : « Je connais », « + Deck » pour le réviser plus tard, ou
> « Ignorer ». Sélectionnez plusieurs mots et appuyez sur Alt+L pour capturer une expression entière
> avec sa phrase.
>
> *(the review, books, local analysis and account paragraphs stay as they are, with « Le
> dictionnaire » becoming « Les dictionnaires »)*
>
> **Traduction étendue** (facultative, désactivée par défaut, sur Chrome, Firefox pour ordinateur et
> Safari) : votre sélection en anglais est traduite dans sa phrase, sur votre appareil, par le moteur
> de Firefox Translations. L'activer télécharge une fois le modèle de traduction (25,8 Mo) depuis
> Cymbra ; le texte des pages ne quitte toujours pas votre appareil. La désactiver supprime le modèle.
> Pour l'espagnol, la traduction étendue arrivera plus tard.
>
> Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos
> chiffres sont publiés sur cymbra.app/lingua.

**Description, English:** the same text, translated:
- « the English or Spanish words you do not know yet »;
- « Choose the languages you study in Settings: each page is read in its own. Several languages at
  once: free for now. »;
- « In Spanish, the card also names the tense and the gender the way French schools teach them. »;
- « your English selection is translated… For Spanish, extended translation comes later. »;
- « For Spanish, the French dictionary is a little less complete than for English: our figures are
  published at cymbra.app/lingua. »

**Single purpose:** « helping a reader understand and learn English or Spanish vocabulary in what
they are already reading ». The rest of the answer is unchanged.

**Remote code:** « the language packs, one per language » instead of « the language pack ».

**Test instructions** (978 of 1,000 characters). Step 1 is shortened and step 6 is added:

```
No account is needed: signing in only syncs a reader's vocabulary between their own devices. The interface is French; the extension teaches English and Spanish to French speakers.

1. Open the toolbar popup and pick a level at "Choisis ton niveau d'anglais" (B1 is a good default). With no level, every word is highlighted and the score reads 0%.
2. Open an English article and click "Analyser cette page". On Chrome the extension reads a page only when asked, or once the optional "Toujours surligner" permission is granted.
3. Words above that level are highlighted; the pill shows the share of the page you already know.
4. Click a highlighted word: a card gives its translation, its dictionary form and how common it is, with "Je connais", "+ Deck" and "Ignorer".
5. Alt+L captures a multi-word selection; Alt+Shift+S, or the popup's "Réviser" button, opens the review panel.
6. Spanish: in the popup's "Réglages", tab "Langue", tick "Espagnol", then open a Spanish article.
```

The new labels are quoted from the source, like the others:
- « Réglages » is the popup's settings button (`popup.html`);
- « Langue » is the settings tab (`settings-view.ts`);
- « Espagnol » is the box under « Langues étudiées » (`language-labels.ts`).

### D3 — The App Store listing (settled by the owner, 2026-10-05)

| Field | Limit | Draft |
|---|---|---|
| Subtitle | 30 | `Anglais et espagnol en lisant` (29) |
| Promotional text | 170 | « Les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne. » (154) |
| Keywords | 100 | `anglais,espagnol,vocabulaire,lecture,traduction,extension,apprendre,mots,révision,CECRL,deck` (92) |

**Description:** the current text, with these changes (well within 4,000 characters):
- First paragraph: « les mots d'anglais ou d'espagnol que vous ne connaissez pas encore ».
- APRÈS L'INSTALLATION, step 3:
  > Dans Safari, ouvrez l'extension depuis le menu de la barre d'adresse et choisissez votre niveau —
  > sans lui, l'extension considère que vous ne connaissez aucun mot. Pour l'espagnol, cochez-le dans
  > Réglages › Langue, puis choisissez votre niveau d'espagnol.
- LIRE: « sa rareté dans l'usage courant. En espagnol, la carte nomme aussi le temps et le genre, comme
  on les apprend en classe. »
- RÉVISER: « … du A1 au C2 — pour l'espagnol, des niveaux estimés d'après la fréquence des mots. »
- TRADUCTION ÉTENDUE: « Votre sélection en anglais est traduite… Pour l'espagnol, elle arrivera plus
  tard. »
- A new line: « Pour l'espagnol, le dictionnaire français est un peu moins complet que pour l'anglais : nos
  chiffres sont sur cymbra.app/lingua. »
- The last line: « L'interface est en français : Cymbra Lingua enseigne l'anglais et l'espagnol à des
  francophones. »

**« What's New »:** a paragraph for the release that ships Spanish. The owner completes the rest of
the notes at release.

> Espagnol : lisez aussi l'espagnol. Cochez-le dans Réglages › Langue : mots surlignés, carte avec le
> temps et le genre, niveaux estimés, et une voix d'Espagne pour la lecture à voix haute.

The App Store listing carries neither « bêta » nor anything about price, and so not the « gratuit
pour l'instant » line of the extension's listing. App Store review guideline 2.2 keeps betas, demos
and trials off the store, and the line already shows inside the app.

### D4 — Translation, like for like

The programme's decision D3 has translation ship per platform, its cost stated like for like.
- English: extended translation is offered, with its download (25.8 MB).
- Spanish: the listings say it comes later. Changes 26 and 27 add its download size to these files
  when they ship it.

### D5 — Coverage: a pointer, not figures

The listings say that the French dictionary is a little less complete for Spanish, and point to
cymbra.app/lingua. The figures (`add-site-lingua-spanish-pages`, change 30) live on the site:
- a store description is revised only with a package and a review, so figures written there would
  go stale;
- one page can show both languages side by side.

### D6 — What does not change

- **Permissions:** the Spanish pack ships inside the package and asks for no permission.
- **Data disclosures:** the synced data is the same for every language.
- **Graphics:** screenshots stay captured by hand.

## Risks / Trade-offs

- **The listings point to figures that are not live yet.** Change 30's pages deploy by hand, with
  the release. The listings are pasted with the same release, so the link and the figures go out
  together.
- **A longer description.** It still fits every store's limit. Apple's description stays under 3,000
  characters of its 4,000.
