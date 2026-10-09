# add-lingua-french-listings — the listings and the site, for the readers who study French

## Why

Change 53 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the last of stage 3 (French studied: fr-en for English speakers, fr-es for Spanish speakers), with
M6, M7, M9, M10, M13, M16, M18 and M19. Change 52 (`enable-lingua-french`) lists fr-en in
`packs.json`, and fr-es only if its committed tables stand at or above the floor M6 fixed
(81.4 / 68.8 / 54.5 %, change 49). What a reader of English or Spanish is told about Cymbra Lingua
does not know it:

- **The store listings** (`apps/lingua-extension/STORE-LISTING.md` for the Chrome Web Store and
  addons.mozilla.org, `apps/lingua-apple/STORE-LISTING.md` for the App Store) were written by
  changes 36 and 37 for English speakers learning Spanish and Spanish speakers learning English.
  Every English text names Spanish alone (« Learn Spanish while you read », keywords with no
  "french", "levels estimated … for Spanish", one model of 26.2 MB); every Spanish text names English
  alone and names its levels on the MCER scale with no word of estimation; the fields that are one per store (the test
  instructions at 996 / 1,000 characters, the App Store review notes at 3,976 / 4,000) have no French
  path and no room for one.
- **The stores' summary** is the package's own description (`_locales/en`: "Read Spanish on the
  web…", `_locales/es`: « Lee inglés en la web… »). Nothing ties it to the shipped pairs: a release
  listing fr-en would tell every English browser that Lingua reads Spanish.
- **The site's Lingua pages** (change 30) are built from the shipped pairs, so French appears in
  their audiences, tables and notes without an edit — but two sentences are Spanish's own: « En
  espagnol, la carte nomme aussi le temps et le genre » and « Pour l'espagnol, les niveaux sont
  estimés … faute de liste CECR libre de droits », while French's card names the tense and the gender
  too (change 51) and French's levels are estimated (M7, change 46). That French sentence also names
  the scale « CECR », where the same page's card title and the extension's French interface say
  « CEFR » (M19: French unchanged) — change 46 handed the choice here.
- **The coverage table** has one column per pair once pairs of two native languages are listed:
  measured at a phone's width (375 px, 335 px of text), it is 421–439 px wide with stage 2's four
  pairs and 583–612 px with the six, so the whole page scrolls sideways.
- **The English home page's Lingua card** reads « Read the English web… »: written for French
  speakers, wrong for the English speakers who land on `/en/` from change 34 on, and more so once
  they read French. Change 29b gives the Spanish home's card its words from the shipped pairs and
  leaves the French and English cards literal.

## What Changes

- **The English texts gain French** (fr-en), for English speakers: the Chrome Web Store and AMO
  description, the App Store en-US and en-GB subtitle, promotional text, keywords, description and a
  « What's New » paragraph — French words highlighted, the languages chosen in Settings, the card's
  tense and gender ("past historic (passé simple)"), French's levels **estimated** (M7), translation
  straight from French (fr-en 2.0, 26.2 MB, change 50), the coverage at `cymbra.app/en/lingua`.
- **The Spanish texts gain French only if fr-es ships** (M6): the same fields in es-ES and es-MX,
  French through English (« 51,6 MB con el francés »), its levels estimated, and — as the French text
  says of Spanish — that the Spanish dictionary is less complete for French than for English, the
  figures published. Below the floor, the Spanish texts change only where they name what English
  speakers study.
- **The French texts change only where they say what another reader studies** (the App Store fr-FR
  description's last line); a French speaker cannot study French (change 39), and no French text
  offers it.
- **The fields that are one per store**: the single purpose, the remote-code answer (fr-en direct,
  fr-es through English) and the permission justifications updated in place; the test instructions
  gain the French path within 1,000 characters (998); the App Store review notes gain it, the
  French sources and the models within 4,000 (3,995 with fr-es, 3,954 without), wording cut, no step
  lost.
- **The stores' summary follows the pairs**: drafted here ("Read Spanish and French on the web…",
  105 characters; « Lee inglés y francés en la web… », 109), and held by `yarn check:version`, which
  fails when a shipped native's description does not name exactly the languages its shipped pairs
  study — so change 52 commits the drafts in the pull request that lists the pairs, and cannot list
  fr-en with "Read Spanish on the web".
- **The site's Lingua pages** say, per studied language, which cards name the tense and the gender
  and which levels are estimated — « En espagnol et en français », "In Spanish and French", « En
  español y en francés » — from a table of what each studied language's packs are, a language it does
  not describe failing the build; « CECR » becomes « CEFR », the one word that moves on `/lingua/`
  with today's pairs.
- **The coverage table turns** once pairs of two native languages are listed: one row per pair,
  four columns, in a box that scrolls on its own below 375 px; with today's pairs, unchanged.
- **The English home's Lingua card follows the shipped pairs**, as the Spanish home's does
  (change 29b): once a pair glossed in English ships it reads « Read the web in Spanish [or French]… »;
  until then, today's paragraph byte for byte.
- **Screenshots**: one capture more per locale that gains French (a French page, the card on
  « fut »); **the dashboards**: no language is added, the procedure and the record of what each
  dashboard shows.
- **The wording is the owner's** (M9): full drafts in the files, reviewed before change 52's release.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-english-listings` and `add-lingua-spanish-audience-listings`, so this change is
  archived after both): a text that names the levels of a language whose levels are estimated says
  so; every scenario kept, three added. ADDED *The stores' summary names what its readers can study*,
  beside change 27's *The manifest speaks the browser's language among the shipped natives*, which it
  does not modify.
- `site-lingua-page`: ADDED *The Lingua page says what each studied language's card and levels are*
  and *The coverage table reads at a phone's width*; change 30's *The Lingua page names its languages
  and publishes their coverage* is not modified (this change is archived after it).
- `site-locales`: ADDED *The English home page describes Lingua for the readers it serves*, beside
  change 29b's Spanish requirement, which it does not modify.

## Impact

- **Products.** Lingua: the two listing files and `tool/check_version.mjs` (a check, run by
  `lingua-extension-check`; nothing it builds moves). The site (`apps/site`): `src/lib/lingua-text.ts`,
  `src/components/LinguaPage.astro`, `src/styles/global.css`, `src/pages/en/index.astro`, their tests
  and one fixture. **Consumed**: change 30's shipped pairs and text tables, change 29b's home-card
  builder, change 27's `_locales`, the catalogue's routes and sizes (changes 25, 50). Cymbra ID,
  Music, the back office and the backend are untouched.
- **Nothing studied moves**: no table, pack, analyser, engine or interface string is edited, so S0,
  the es-fr, es-en, en-es and French goldens (`fr-en.golden`) and the French interface cannot move;
  the extension's packages are byte for byte what they were.
- **What a visitor sees before change 52**: `/lingua/` says « CEFR » for « CECR »; nothing else on any
  page while today's pairs ship. From change 34, the coverage table's rows and the English home's
  card; from change 52, French.
- **Order.** The listing texts are true of what change 52 ships, and pasted with its release (M18).
  The pull request may merge before change 52, as changes 36 and 37 merged before 34 and 35: the
  site's code is inert until the pairs are listed, and the check makes 52 commit the summaries. It
  builds on changes 29b, 30, 36, 37, 45, 46, 48–51 and is archived after them.
- **The owner** reviews the English and Spanish drafts (M9), captures the screenshots, pastes the
  listings with change 52's release after the site deploy that publishes its figures, and records what
  the dashboards show (M18).
