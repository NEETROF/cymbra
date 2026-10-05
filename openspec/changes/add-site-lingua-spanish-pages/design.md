# Design

## Context

The site (`apps/site`, Astro, French and English) has one Lingua page per language,
`src/pages/lingua.astro` and `src/pages/en/lingua.astro`. Each page has:
- a hero, with a « bêta » badge;
- eight feature cards;
- the community block;
- a closing section about Cymbra Music.

The pages present English alone, and their last card promises other pairs.

The figures the programme promised exist only in prose:
- the es-fr tables' README gives Spanish's share of glossed lemmas;
- the programme's study gave English's.

Nothing keeps either in step with the tables, which change monthly (`lingua-pack-update`).

The site deploys by hand: `site-deploy` runs on a dispatch only.

## Goals / Non-Goals

**Goals:**
- Both pages name English and Spanish, and say what Spanish has and lacks.
- Coverage figures, measured the same way for both languages, that cannot go stale silently.

**Non-Goals:**
- A page per language, or a separate Spanish page.
- Translating the site into Spanish.
- Changing what the extension does.

## Decisions

### D1 — The pages (settled by the owner, 2026-10-05; French then English)

| Element | French | English |
|---|---|---|
| Meta description | « Une extension navigateur qui surligne, sur la page que vous lisez, les mots d'anglais ou d'espagnol que vous ne connaissez pas encore. Analyse hors-ligne, aucun compte requis. » | "A browser extension that highlights, on the page you are reading, the English or Spanish words you do not know yet. Offline analysis, no account required." |
| Tagline | « …les mots d'anglais ou d'espagnol que vous ne connaissez pas encore, directement sur la page… » (the rest unchanged) | "…the English or Spanish words you do not know yet, right on the page…" |
| 👆 card | adds « En espagnol, la carte nomme aussi le temps et le genre. » | adds "In Spanish, the card also names the tense and the gender." |
| 📈 card | « Une estimation de votre vocabulaire, adossée à l'échelle A1→C2 plutôt qu'à un badge maison. Pour l'espagnol, les niveaux sont estimés d'après la fréquence des mots, faute de liste CECR libre de droits. » | "An estimate of your vocabulary, anchored to A1→C2 rather than to a homemade badge. For Spanish, the levels are estimated from word frequency, as no CEFR list can be shipped freely." |
| 🧭 card | title « Anglais et espagnol → français », text « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. Plusieurs langues à la fois : gratuit pour l'instant. » | title "English and Spanish → French", text "Choose your languages in Settings: each page is read in its own. Several languages at once: free for now." |

The hero's availability line names Safari among the live channels, and the Safari store button
links to the app's App Store record (`apps/site/src/lib/stores.ts`, `LINGUA_APP_STORE`): the owner
reported the app live on iOS and macOS while this change was implemented.

The « gratuit pour l'instant » line is the owner's decision, shown in the extension's language
choice too. The site already carries a « bêta » badge; App Store rules do not apply to it.

### D2 — The coverage section

A section after the cards: French « Ce que couvre le dictionnaire », English "What the dictionary
covers", with the anchor `#couverture`.

> La part des mots les plus courants qui ont une définition en français, mesurée de la même façon
> pour chaque langue, sur les dictionnaires livrés avec l'extension.

| Mots les plus courants | Anglais | Espagnol |
|---|---|---|
| 5 000 | 95 % | 88 % |
| 10 000 | 90 % | 77 % |
| 20 000 | 79 % | 64 % |

> Les définitions viennent du Wiktionnaire et de traductions écrites par des personnes, jamais d'une
> traduction automatique. La traduction étendue, facultative, traduit aujourd'hui l'anglais ;
> l'espagnol suivra.

What is measured:
- the N lemmas the pair ranks commonest (`freq.tsv`);
- of those, the share that have a French gloss (`gloss.tsv`);
- whole percentages, rounded half up.

Today's figures, from the committed tables:
- English: 95.1, 90.1 and 78.9 %;
- Spanish: 87.6, 77.2 and 63.7 %. These are the es-fr README's figures.

Not 1,000: each frequency list carries noise in its first thousand, so that row would compare noise,
not dictionaries.
- English counts contractions as lemmas (« it's », « don't »).
- Spanish counts English words, letters and abbreviations from its corpus (« the », « c », « etc »).
- The row would put Spanish above English: 97 % against 95 %.

### D3 — The figures follow the tables

`scripts/lingua-data/gloss_coverage.py` is stdlib-only and works on the Python that runs the lingua-data
tests.
- It measures every pair `apps/lingua-extension/packs.json` ships.
- `--write` writes `apps/site/src/data/lingua-coverage.json`: per pair, the three shares to one
  decimal.
- `--check` exits 1 when that file differs from a fresh measure.

The pages import the JSON and round it in their locale: « 95 % » in French, "95%" in English.

`test_gloss_coverage.py` runs in the lingua-data unit tests of `lingua-extension-check`, which already
run on every change to the tables. It covers two things:
- the measure, on a small fixture;
- the committed JSON against the committed tables.

So a table update that moves a figure fails until the JSON is written again. `lingua-pack-update`
writes it after rebuilding a pair's tables.

### D4 — Publication

Nothing deploys on merge. The owner dispatches `site-deploy` with the release that ships Spanish,
which is also when the store listings (change 29) are pasted. The listings point to this section.

## Risks / Trade-offs

- **Figures that look worse for Spanish.** They are the programme's accepted gap (decision D4),
  stated like for like rather than hidden.
- **A check that blocks a table update.** It fails with the command to run, and
  `lingua-pack-update` runs that command itself.
