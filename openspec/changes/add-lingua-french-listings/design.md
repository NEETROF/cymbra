# Design — add-lingua-french-listings

## Context

See proposal.md (Why). What exists on `main` (4222a013) and in the changes this one follows:

| Where | What |
|---|---|
| `apps/lingua-extension/STORE-LISTING.md` | Constants, the listing languages (French; English from change 34; Spanish from change 35), the summary per language (from `_locales`), the FR, EN and ES descriptions, the EN bullet list and the ES claims table, the screenshots per language, the single purpose, the permission justifications, the remote-code answer, the data usage, the test instructions (one field per store, **996 / 1,000**), the dashboards' languages for changes 34 and 35 |
| `apps/lingua-apple/STORE-LISTING.md` | fr-FR (primary), en-US = en-GB, es-ES = es-MX (M16): name, subtitle, promotional text, keywords, description, « What's New » per release, URLs; the review notes (one English field, **3,976 / 4,000**); the screenshots per platform and locale |
| `apps/lingua-extension/_locales/{fr,en,es}/messages.json` | `extensionDescription`, the stores' summary: fr « Lisez l'anglais et l'espagnol sur le web… » (109), en "Read Spanish on the web…" (94), es « Lee inglés en la web… » (99). `tool/check_version.mjs` (`yarn check:version`, run by `lingua-extension-check`) holds each to Apple's 112 and the French to `manifest.json`'s literal; nothing holds what it names |
| `apps/site/src/lib/lingua-text.ts` | Change 30's text tables (fr, en, es) and `linguaPageText`: the audiences, the tagline, the 🧭 card, the coverage table and the translation note are built from the shipped pairs; `spanishCard` and `spanishLevels` are appended when a listed pair studies Spanish; the French `spanishLevels` says « CECR », the French level card's title « Votre niveau, en CEFR » |
| `apps/site/src/components/LinguaPage.astro`, `global.css` | the coverage table: one column per pair, `.prose table { width: 100% }`, cells padded 0.5rem × 0.7rem; `test/fixtures/lingua/main.{fr,en}.html` pin `/lingua/` and `/en/lingua/` byte for byte on today's pairs |
| `apps/site/src/pages/en/index.astro` | the English home: the Lingua card's paragraph is literal, « Read the English web… » |
| Change 29b (`extend-site-spanish-locale`, #841, open) | the Spanish home's Lingua card built from the pairs (« Lee la web en inglés o en francés… » with fr-en and fr-es), the French and English cards left literal |
| `apps/back-office/src/i18n/locales/{fr,en}.json` | `lingua.languages`: « Anglais », « Espagnol » / "English", "Spanish"; a language with no name shows its code (`languageLabel`, `src/i18n/language-label.ts`), so French reads « fr » from change 52. Changes 39 and 52 hand its name here |
| `apps/lingua-extension/model-manifest.json` | routes `fr-en` = [`fr-en/base-memory/2.0`], `fr-es` = [`fr-en/base-memory/2.0`, `en-es/base-memory/2.1`] (change 50, inert until 52) |
| Changes 45, 46, 48, 49, 51 | French's readings (genders included), French's estimated levels (`levels_estimated`), fr-en's glosses (the English Wiktionary's French section), fr-es's (the Spanish Wiktionary's French section, then the French Wiktionary's Spanish translations, then the Spanish Wiktionary's French translations read backwards), the card's French wording ("past historic (passé simple)", « pretérito perfecto simple de indicativo ») |

The requirement *The store listings name each studied language* is held by changes 36 and 37 (the
same text, 37's adding a scenario); *The Lingua page names its languages and publishes their
coverage* by change 30; *The Spanish home page describes Lingua for the readers it serves* by 29b.

## Goals / Non-Goals

**Goals:**
- Every listing text true of what change 52 ships, for each audience, French named where its readers
  can study it and nowhere else; French's levels said to be estimated wherever levels are named (M7).
- The stores' summary held to the shipped pairs by a check, not by memory.
- The site's Lingua pages and the English home true of French, readable at a phone's width.
- Nothing studied moves; the French interface and today's pages do not move but for one word.

**Non-Goals:**
- Uploading listings (`add-lingua-asc-localisations-upload`, optional, outside the counts).
- A new App Store locale or `_locales` folder: fr-en is glossed in English and fr-es in Spanish, both
  shipped by then (M16 settled).
- The host app's activation page (`copy.js`, change 28), the extension's copy, the card's wording
  (changes 51, 52, 56).
- The French home's Lingua card (« Lisez le web en anglais »), which French studied does not change.
- Publishing the share of definitions among fr-es's glosses on the site (D11).

## Decisions

### D1 — French in the English texts always, in the Spanish texts only if fr-es ships

Change 52 lists fr-en; it lists fr-es only if its committed tables stand at or above M6's floor
(change 49, D8). The English texts are written for fr-en. The Spanish texts are written twice: for
fr-es shipping (French named everywhere a Spanish reader's languages are) and for fr-es held back,
where only the sentences that say what English speakers study change (« y español y francés a los
anglohablantes »). The owner pastes the variant that matches the release. Every count below is given
for both where they differ.

### D2 — What a text says about French, and the file that makes each claim true

Like for like with what the texts already say of Spanish (changes 36 and 37):

| Claim | Where it is true once change 52 ships |
|---|---|
| French words highlighted, for English speakers (and Spanish speakers if fr-es ships) | `packs.json` lists fr-en (and fr-es); `pairsOf("en")` is es-en, fr-en (`src/analyzer/pairs.ts`) |
| « Choose the languages you study in Settings: each page is read in its own » (the French text's sentence, carried) | the studied-languages box, shown from two pairs of the reader's native language (`offerFor`, `src/reading/studied-languages-view.ts`), "Languages studied" / « Idiomas estudiados » (`studiedLanguages`, `src/i18n/{en,es}/settings.ts`), its note "Each page is read in whichever of your languages it holds" (`src/i18n/en/studied-languages.ts`) |
| The meaning in English, from Wiktionary, written by people | fr-en's `gloss.tsv`, the English Wiktionary's French section (change 48, `tables/fr-en/README.md`), M5 |
| The meaning in Spanish, from Wiktionary, written by people (fr-es) | fr-es's `gloss.tsv`, the Spanish Wiktionary's French section, then the French and Spanish Wiktionaries' translation tables (change 49), M5 |
| The card names the tense and the gender: "past historic (passé simple)", « pretérito perfecto simple de indicativo », "feminine noun" | change 51's renderers (`TENSES` keyed by studied language, `src/i18n/{en,es}/grammar.ts`), pinned by its snapshots; the genders from change 45's `tables/fr/grammar.tsv`. Quoted from change 51's scenarios: the owner checks them against the merged snapshot before pasting, as changes 36 and 37 had « Español » checked against change 20 |
| French's levels estimated from word frequency (M7) | `levels_estimated` in every French pack's manifest (change 46, D5); the labels "Estimated French level", « Nivel de francés estimado » (`levelTitleEstimated`, `estimatedLevelsNote`, `src/i18n/{en,es}/languages.ts`), whose key change 52 widens |
| Extended translation straight from French into English: 26.2 MB | route `fr-en` = `fr-en/base-memory/2.0`: 23,175,075 + 2,649,934 + 409,706 B = 26,234,715 B, "26.2 MB" as the setting rounds it (`megabytes`, `src/reading/translation-setting.ts`); the setting adds the models of every language the reader studies (`modelsFor`, `src/translate/host/model-manifest.ts`, called by `model-controller.ts`): es-en and fr-en together 52,475,767 B, so "26.2 MB each" |
| Into Spanish through English: « 25,4 MB para el inglés, 51,6 MB con el francés » (fr-es) | route `fr-es` = fr-en 2.0 then en-es 2.1: 26,234,715 + 25,373,354 = 51,608,069 B; en-es alone 25,373,354 B — French needs both models whether or not English is studied, as Spanish does for a French speaker (« 52,0 Mo avec l'espagnol ») |
| The figures at `cymbra.app/en/lingua` (and `/es/lingua`) | change 52 writes fr-en's (and fr-es's) figures into `apps/site/src/data/lingua-coverage.json` (`gloss_coverage.py --write`); change 30's pages lead with the reader's pairs |
| « Para el francés, el diccionario español es algo menos completo que para el inglés » (fr-es, M6) | fr-es 83.2 / 70.8 / 56.8 % of the 5,000 / 10,000 / 20,000 commonest lemmas against en-es's 93.0 / 85.0 / 71.7 % (changes 49 and 22), re-measured at change 52's commit; the French text's sentence about Spanish (es-fr's 87.6 / 77.2 / 63.7 against en-fr's 95.1 / 90.1 / 78.9), transposed. fr-en (93.6 / 86.9 / 76.3, change 48) stands beside es-en's committed 93.0 / 86.5 / 76.5 (`tables/es-en/README.md`, the figures change 34 publishes): the English texts say nothing of it |
| The step "For French, check it in Settings › Language, then choose your level of French" | the "Language" tab (`tabLanguage`), "Languages studied", the box's "French" (`languageName`), "Choose your French level" (`chooseLevelPrompt`) |
| Spanish stays the language an English reader starts in (the steps and the test instructions say "pick a Spanish level") | `defaultPair("en")` is the first listed English-glossed pair: change 52 lists fr-en after es-en (and fr-es after en-es), a hand-over (D12) |

What the texts say, per field, is in *Measured*. No text calls the app a beta or speaks of price
(guidelines 2.2, 2.3.7; the programme's rule for every listing) — the review notes' « the app is
free » included (D4).

**« Several languages at once » stays out, for a new reason.** Changes 36 and 37 left the French
text's second line (« Plusieurs langues à la fois : gratuit pour l'instant ») out because their
readers studied one language and the studied-languages box hid itself (`offerFor`). From change 52
an English reader has two pairs (a Spanish reader too, with fr-es), the box shows, and its own line
reads "Several languages at once: free for now." (`severalLanguagesOffer`,
`src/i18n/en/studied-languages.ts`). The texts carry the French text's first sentence and not that
line: a listing speaks of no price. The owner decided on 2026-10-09 that the line itself
leaves the extension, in French, English and Spanish, in a change of its own; the listings are
unchanged by that change, since they never carried it. The notes of both files that give the old
reason (« es-en is the one English-glossed pair… », « en-es is the one pair glossed in
Spanish… ») are rewritten with this one (tasks 1.1, 1.3).

### D3 — The French texts: only what another reader studies

A French speaker cannot study French (change 39), so no French text names French as a language to
learn. The French texts that say what English or Spanish speakers study are made true: the App Store
fr-FR description's last line (« …et en anglais pour les anglophones, qui apprennent l'espagnol et
le français. », 3,021 → 3,036). The Chrome Web Store FR text names no other audience and is
unchanged, as are the French summary, subtitle, promotional text and keywords (« CECRL » stays: a
search term, D7).

### D4 — The fields that are one per store

- **Single purpose** (English): "Spanish or French for English speakers, English or French for
  Spanish speakers, English or Spanish for French speakers" (573 → 593 characters; 583 without fr-es).
- **Permission justifications**: their labels are unchanged by French; the sentence that says who
  studies what names French for English (and Spanish) speakers.
- **Remote code**: the packs listed per change (fr-en, and fr-es, from change 52); the models: for an
  English speaker reading French, fr-en alone, the direct model of its pair; for a Spanish speaker
  reading French, fr-en then en-es, French going through English.
- **Test instructions** (one field per store, ≤ 1,000): step 6 gains « "Languages studied" > "French"
  adds French pages »; paid for by wording, every step kept — « No account is needed: signing in only
  syncs a reader's words » → « No account needed: signing in only syncs words », and step 6's welcome
  question « "¿Cuál es tu nivel de inglés?" or » dropped: a reviewer who switches to Spanish in
  Settings is past the welcome tab and meets the popup's « Elige tu nivel de inglés », which stays.
  **996 → 998.** The same text whether fr-es ships or not.
- **App Store review notes** (one English field, ≤ 4,000): a step 6 for French (« Settings > Language >
  "Languages studied" > "French", then a French level, on a French page »), the models per pair
  (« Spanish or French to English 26.2 MB each, English to Spanish 25.4 MB; French to Spanish chains
  two »), the audiences once, in "Purpose & audience", UD French-GSD among the sources. Cut, wording
  only: the account paragraph's « ; without one, it stays on this device » (its first sentence says
  every feature works signed out), "External services"' « (bundled WebAssembly engines + offline
  dictionaries) » (the engine is said to ship inside the app, the dictionaries to be bundled),
  "Regional differences"' audiences (now in "Purpose & audience", once), « for Spanish » and « for
  both » in "Third-party material", and "External services"' « No payment processor: the app is
  free, with no in-app purchase » → « No payment processor and no in-app purchase » (D2: no text
  speaks of price; the payment processor, the external service Apple asks about, is still
  answered). **3,976**, today's count, with fr-es; **3,935** without it (its « and French » and
  « ; French to Spanish chains two » go). Per paragraph: account −37, steps +104, OPTIONAL +40,
  purpose +37, external −71, regional −68, sources −5.

### D5 — The App Store locales

No locale is added (M16 settled: fr-FR primary, en-US, en-GB, es-ES, es-MX). The en-GB text is the
en-US text, the es-MX text the es-ES text (M10). Each field's draft and count is in *Measured*. The
en-US and es-ES descriptions gain, after the activation steps, the French text's own sentence for a
second language (« Pour l'espagnol, cochez-le dans Réglages › Langue, puis choisissez votre niveau
d'espagnol »), transposed; their « What's New » paragraph is for change 52's release.

### D6 — The stores' summary is held to the shipped pairs

The summary is the package's description, read by the browser from `_locales` and by both stores as
the listing's summary — and, by M13, the English one by every browser in a language Lingua does not
speak: it is listing copy that lives in the package. Its drafts are written here
(en "Read Spanish and French on the web: unknown words highlighted, an honest percentage. Offline and
private.", 105 / 112; es « Lee inglés y francés en la web: palabras desconocidas resaltadas,
porcentaje honesto. Sin conexión y privado. », 109 / 112). Committed now, they would claim French in
every English and Spanish browser before fr-en ships; left alone, change 52 would ship French under
"Read Spanish on the web".

So `tool/check_version.mjs` gains one rule, beside the 112 it holds: for each native language a
shipped pair is glossed in (`packs.json`), its description — `_locales/<native>`, and `manifest.json`'s
literal for French — names, in that language, every language the native's shipped pairs study and no
other. Names are matched whole and case-insensitively, from a table of three names per language held
equal to the catalogue's (`name` in `src/i18n/{fr,en,es}/languages.ts`, by a test): « anglais »,
« espagnol », « français »; "English", "Spanish", "French"; « inglés », « español », « francés ».
Prototyped (scratchpad) on the committed messages and the programme's pair lists — stage 2's as
#810 and #814 list them:

| Pairs listed | Committed summaries | With the drafts |
|---|---|---|
| en-fr, es-fr (today) | pass | — |
| + es-en (change 34); + en-es (change 35) | pass | the drafts fail: en names French, es names French, neither studied |
| + fr-en (change 52) | fail: en names Spanish, studies Spanish and French | pass with the English draft; the Spanish one fails (es names French, not studied) |
| + fr-en, fr-es | fail: en, and es names English, studies English and French | pass |

The pull request that lists fr-en therefore commits the English draft, and fr-es's the Spanish one —
change 52, whichever of 52 and 53 merges first: merged before, the check is on `main` when 52 lists
the pairs; merged after, this pull request's own check fails until it commits the drafts.

*Rejected — a description per set of studied languages, chosen by the build.* One sentence per
language and pair set, for a text written once per release that adds a language.

### D7 — The site says, per studied language, what the card names and which levels are estimated

`spanishCard` and `spanishLevels` become `genderCard` and `estimatedLevels`, filled with the studied
languages of the listed pairs that have the property, in the page's order (its readers' pairs first),
from one table in `lingua-text.ts` of what each studied language's packs are — `en`: neither; `es`,
`fr`: a card naming tense and gender, levels estimated (changes 45, 46; Spanish's since the Spanish
programme) — and a language the table does not describe fails the build, as a language it cannot name
does. The French sentence keeps its words for Spanish alone, so `/lingua/` with today's pairs moves
by the scale's name only:

- fr « En espagnol et en français, la carte nomme aussi le temps et le genre. » / « Pour l'espagnol et
  le français, les niveaux sont estimés d'après la fréquence des mots, faute de liste CEFR libre de
  droits. »
- en "In Spanish and French, the card also names the tense and the gender." / "For Spanish and French,
  the levels are estimated from word frequency, as no CEFR list can be shipped freely."
- es « En español y en francés, la tarjeta también indica el tiempo verbal y el género. » / « Para el
  español y el francés, los niveles se estiman a partir de la frecuencia de las palabras, a falta de
  una lista MCER de uso libre. » — on `/es/lingua/` with fr-es, « En francés y en español… »: its
  readers' pairs first.

**« CECR » or « CEFR »: « CEFR ».** M19 keeps the French interface's scale as it is, and it is
« CEFR » (`levelScale`, `src/i18n/fr/languages.ts`); the same page's card title already reads « Votre
niveau, en CEFR », so one page names one scale once. « CECR » was the sentence's own. The App Store
fr-FR keyword « CECRL » stays: Apple's keywords are search terms no reader sees, and the one French
learners type. Measured on the prototype (`linguaPageText` run on the committed tables and on the
programme's pair lists): with today's pairs, `/lingua/` differs in one word, `/en/lingua/` in none;
`test/fixtures/lingua/main.fr.html` is re-recorded by that word in this pull request, saying why.

### D8 — The coverage table turns once pairs of two native languages are listed

Change 30 keeps one column per pair, which today's two pairs fit. Measured with the site's CSS
(scratchpad page, built-in browser, 375 × 812, 335 px of text after the container's padding):

| Pairs | One column per pair (px) | One row per pair (px) |
|---|---|---|
| en-fr, es-fr (fr) | 335 — fits | not used |
| stage 2, four pairs (fr / en / es) | 439 / 431 / 421 | 335 / 335 / 335 |
| + fr-en (en) | 507 | 335 |
| all six (fr / en / es) | 612 / 588 / 583 | 335 / 335 / 335 |

The page scrolls sideways from stage 2 on. While every listed pair is glossed in one language the
table is today's; once two native languages are listed — the moment change 30's headers turn to
"<studied> → <native>" — it has one row per pair (the pair as the row's header, `scope="row"`) and
four columns (the corner header « Mots les plus courants », then the tops 5 000, 10 000, 20 000), the
readers' pairs first. At 360 px the English table is 327 px for 320, which the gutter would absorb; at
320 px the tables are 301–327 px for 280 and, unboxed, the page is 347 px wide — so the per-pair
table sits in a box that scrolls on its own (`overflow-x: auto`): measured, the page is then 320 px
wide at 320 px, and 360 at 360. The box is not used for the one-column-per-pair table, so today's
markup does not move; its rule in `global.css` renames the bundled stylesheet (*What does not
move*).

*Rejected — the scrolling box alone.* A phone would show the first pair and hide four or five.

### D9 — The English home's Lingua card follows the shipped pairs

Change 29b builds the Spanish home's card from the pairs and leaves the English one literal. From
change 34 an English speaker reads `/en/`'s « Read the English web… »; from change 52 they also read
French. The English home takes 29b's builder with the English table: once a pair glossed in English is
listed, the paragraph reads "Read the web in {Spanish or French} with the words you do not know yet
highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that
builds itself as you read." (`g.or` of the English-glossed pairs' studied languages; « in Spanish »
with es-en alone), the button still `/en/lingua`; until then the literal paragraph is rendered, byte
for byte. The French home's card is left as it is (Non-Goals). Change 34 then needs nothing of its
own for `/en/` (D12).

### D10 — Screenshots and the dashboards

One capture more where French is added, the fifth (the Chrome Web Store takes five; App Store
Connect ten per slot): `en` / en-US — a French article (a French Wikipedia page reads well), the card
open on « fut »: "third-person singular past historic (passé simple) of être", its English gloss
below; `es` / es-ES, if fr-es ships — the same page, « tercera persona del singular del pretérito
perfecto simple de indicativo de être ». Taken by the owner from a build of change 52, en-GB and es-MX
reusing their twins' sets.

No language is added to either dashboard: change 52's package carries `_locales/{fr,en,es}` as
change 35's did, with new `en` (and `es`) summaries. The submission is made by hand, as change 34's:
the package uploaded through the Chrome Web Store dashboard without publishing; the `fr` listing
checked unchanged; the `en` (and `es`) description and screenshot replaced; the single fields
replaced; submitted; addons.mozilla.org dispatched with `stores: firefox`, its English (US) and
Spanish descriptions pasted and its summaries checked. Both results recorded in a table of
`STORE-LISTING.md`.

### D11 — What this change does not decide

- **The share of definitions among fr-es's glosses** (change 49's open question 7): not on the site,
  this design's recommendation, the owner's to settle (open question 3). The table publishes one
  measure for every pair, "a gloss in the reader's language", and a translation-table entry written
  by a person is one (M5); a column for one pair would break like for like, and en-es's split (26.0 %
  of its glossed top 10,000 from a translation table) is in its README alone. fr-es's stays in its
  README too.
- **Withholding a pair's translation**: M15 is settled (translation opens with each pair); should
  change 52 not offer one, its clause goes from each text and every count only falls.
- **The host app's activation page** says « choose your level of Spanish » (change 28): the App Store
  texts add the French step after it, as the French description does for Spanish.

### D12 — What later changes take from here

| Change | Takes |
|---|---|
| 52 `enable-lingua-french` | commits D6's summaries in `_locales/en` (and `_locales/es` if fr-es ships) in the pull request that lists the pairs, which the check requires; lists fr-en after es-en and fr-es after en-es in `packs.json`, so Spanish and English stay the languages English and Spanish readers start in; its release carries these listings, pasted by the owner |
| 34 `enable-lingua-english-speakers` | the English home's card (D9) and the coverage table's rows (D8) come with its pairs; nothing to write |
| 56 `refine-lingua-matrix-wording` | the owner's corrections to the drafts after the release |

### D13 — The back office names French

Changes 39 and 52 leave the back office's name for `fr` to this change: from change 52 its Lingua
screen shows French's usage under the code « fr », as *The studied languages are named in the
console's language* allows for a language it has no name for. `lingua.languages.fr` is added —
« Français » in `fr.json`, "French" in `en.json` — and `test/lingua.spec.ts`'s `languageLabel` case
gains it. Inert until a device reports French; no requirement moves (that one already asks for a
name).

## Measured

How: the edits applied by a script (scratchpad, never committed) to the committed texts, counted as
the listing files count — every character inside the block, line breaks included; the site's text by
`linguaPageText` run on the committed tables; the table widths in the built-in browser.

| Field | Limit | Locale | Today | Draft | Count |
|---|---|---|---|---|---|
| Summary | 112 | en | Read Spanish on the web… (94) | Read Spanish and French on the web: unknown words highlighted, an honest percentage. Offline and private. | 105 |
| Summary | 112 | es (fr-es) | Lee inglés en la web… (99) | Lee inglés y francés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado. | 109 |
| Subtitle | 30 | en-US, en-GB | Learn Spanish while you read (28) | Spanish and French as you read | 30 |
| Subtitle | 30 | es-ES, es-MX | Aprende inglés mientras lees (28) | Inglés y francés mientras lees (fr-es; without it, unchanged) | 30 |
| Promotional text | 170 | en-US, en-GB | 120 | The Spanish or French words you don't know yet, highlighted on the page you're reading. The analysis runs on your device, offline. | 130 |
| Promotional text | 170 | es-ES, es-MX | 133 | Las palabras en inglés o en francés que todavía no conoces, resaltadas en la página que lees. El análisis se hace en tu dispositivo, sin conexión. (fr-es; without it, unchanged) | 146 |
| Keywords | 100 | en-US, en-GB | 99 | « review » replaced by « french » | 99 |
| Keywords | 100 | es-ES, es-MX | 98 | « repaso » replaced by « francés » (fr-es; without it, unchanged) | 99 |
| Description | 4,000 | en-US, en-GB | 2,682 | D2's claims | 2,902 |
| Description | 4,000 | es-ES, es-MX | 2,895 | D2's claims; without fr-es, the last line only | 3,279; 2,905 |
| Description | 4,000 | fr-FR | 3,021 | the last line (D3) | 3,036 |
| What's New | 4,000 | en-US, en-GB | — | the paragraph below | 380 |
| What's New | 4,000 | es-ES, es-MX | — | the paragraph below (fr-es) | 449 |
| Test instructions | 1,000 | Chrome Web Store, AMO | 996 | D4 | 998 |
| Review notes | 4,000 | every locale | 3,976 | D4 | 3,976; 3,935 |
| Single purpose | — | Chrome Web Store | 573 | D4 | 593; 583 |
| Description | 16,000 | Chrome Web Store, AMO en | 2,375 | D2's claims | 2,579 |
| Description | 16,000 | Chrome Web Store, AMO es | 2,576 | D2's claims; without fr-es, the last line only | 3,045; 2,586 |

The « What's New » drafts:

- en: "Cymbra Lingua now reads French too, for English speakers: check French in Settings › Language.
  French words come with their meaning in English, written by people; the card names the tense and
  the gender ("past historic (passé simple)"), and your French level is estimated from word
  frequency. Extended translation turns your French selection straight into English, on your device."
- es: « Cymbra Lingua ahora también lee francés, para los hispanohablantes: márcalo en Ajustes ›
  Idioma. Las palabras francesas llegan con su significado en español, escrito por personas; la
  tarjeta nombra el tiempo verbal y el género («pretérito perfecto simple de indicativo»), y tu nivel
  de francés se estima según la frecuencia de las palabras. La traducción ampliada traduce tu
  selección en francés al español, pasando por el inglés, en tu dispositivo. »

**What does not move.** No file under `crates/`, `scripts/lingua-data/`, `apps/lingua-extension/src/`
or `assets/` is edited: no table, pack, analyser version, engine or catalogue string. S0
(`english_baseline.rs`), the es-fr, es-en and en-es goldens and the French golden
(`french_baseline.rs`, `fr-en.golden`) read none of what this change edits and cannot move; the
French interface cannot move. `check_version.mjs` builds nothing: the packages are byte for byte. On
the site with today's pairs: `/lingua/` one word (« CECR » → « CEFR »), `/en/lingua/`, `/`, `/en/`,
`/es/…` and every other page byte for byte but one name — the box's rule in `global.css` changes the
bundled stylesheet, so its hashed name (`/_astro/Base.<hash>.css`) changes in every page's `<link>`.
The back office's new name shows only for a language no device reports before change 52.

## Appendix — the two fields at the edge, as measured

The test instructions (998 / 1,000), the same whether fr-es ships or not:

```
No account needed: signing in only syncs words across devices. The interface follows the browser: French, Spanish, else English (studying Spanish).

1. Pick a level, e.g. B1, at "What's your Spanish level?" (welcome tab) or "Choose your Spanish level" (popup). With none, every word is highlighted and the score reads 0%.
2. Open a Spanish article. On Chrome, click "Analyze this page" in the popup, or grant "Always highlight (every page)".
3. Words above that level are highlighted; the pill shows the share of the page you know.
4. Click one for its card: English translation, dictionary form, frequency, and "I know it", "+ Deck", "Ignore".
5. Alt+L captures a selected phrase; Alt+Shift+S or the popup's "Review" opens the review panel.
6. Popup gear ("Settings") > "Language": "Languages studied" > "French" adds French pages; "Español" to study English, "Français" English and Spanish. In Spanish, steps 1-5 on English pages: "Elige tu nivel de inglés", "Analizar esta página", "La conozco".
```

The App Store review notes (3,976 / 4,000) with fr-es; without it, « Spanish speakers English and
French, » reads « Spanish speakers English, » and « ; French to Spanish chains two » goes (3,935):

```
No account is needed to review this app: every feature works signed out. An optional Cymbra account saves the learner's progress - known words, deck, level and statistics - across reinstalls and devices (iPhone, iPad, Mac). To test it, create one from the extension (panel > Settings > Data > Account) with Sign in with Apple or Google - no invitation is needed.

IMPORTANT - this app is a Safari extension host: its own screen only explains. The extension must be enabled in Safari before anything happens.

1. Settings > Apps > Safari > Extensions (macOS: Safari > Settings > Extensions): enable "Cymbra Lingua", then set "Other Websites" to Allow.
2. In SAFARI (not in the app), open the extension from the address-bar menu, keep English as your language and pick a Spanish level, e.g. B1. With no level chosen the engine assumes zero known words: every word is highlighted and the pill reads 0%.
3. Open a Spanish page. Reload any tab opened BEFORE the extension was enabled or the level picked.
4. Words above your level are highlighted; the pill shows the share of the page you know. Tap a highlighted word: a card gives its English translation, dictionary form and frequency, with "I know it", "+ Deck", "Ignore".
5. Tap the pill to open the panel: Review, Stats (estimated vocabulary, A1 to C2), Settings (four tabs; the account is under Data).
6. French: Settings > Language > "Languages studied" > "French", then a French level, on a French page.

Account deletion: panel > Settings > Data, signed in > "Manage my data" opens the account page. "Erase my Lingua data…" erases that data on the server and every device, keeping the account. "Delete my Cymbra account" opens https://cymbra.app/en/delete-account/, where the account (shared by Cymbra's apps) is deleted after signing in.

OPTIONAL - "Extended translation", off by default (Settings > Language). The translation engine (Mozilla's Firefox Translations, WebAssembly) ships inside the app. Turning it on downloads data only - a model per pair studied (Spanish or French to English 26.2 MB each, English to Spanish 25.4 MB; French to Spanish chains two) from https://models.cymbra.app, checked against a pinned sha256 - and the selected sentence is then translated on the device. No page text, account or device identifier is sent. Turning it off deletes the models.

Purpose & audience: Cymbra Lingua helps people learn a language by reading real web pages and their own DRM-free EPUB books: unknown words are highlighted in place, looked up offline, captured into a deck reviewed with spaced repetition, and the reader's CEFR level (A1-C2) is estimated from the words they marked. English speakers learn Spanish and French, Spanish speakers English and French, French speakers English and Spanish.

External services: none by default - highlighting, lookup, the level estimate and translation run on the device. No AI/LLM API, no analytics, no ads. Only on the reader's action: https://models.cymbra.app serves the models above; https://api.cymbra.app (our backend) syncs the word list when signed in; Sign in with Apple / Google authenticate (the app never sees a password). No payment processor and no in-app purchase.

Regional differences: none. The interface follows the device's language (French, Spanish, else English); Settings > Language switches. A device in Spanish runs steps 2-5 on an English page: "Elige tu nivel de inglés", "La conozco", "+ Mazo", "Ignorar", "Repaso", "Ajustes".

Third-party material: the bundled dictionaries combine sources licensed for commercial use - ESDB (SCOWL) inflections, CEFR-J and Octanove level lists for English; the UD Spanish-GSD and French-GSD treebanks; wordfreq frequency lists; kaikki.org extracts of the English, French and Spanish Wiktionaries (definitions and translation tables) for the glosses - credited in the extension's "Sources & privacy" panel. The translation models are Mozilla's (MPL 2.0). The app does not operate in a regulated industry.
```

## Risks / Trade-offs

- **A listing that promises what is not shipped** → each French claim tied to its file (D2), the
  texts pasted with change 52's release; the Spanish ones in two variants (D1); the summary held by
  the check (D6).
- **Counts at the edge** (998 / 1,000, two subtitles at 30 / 30) → each count is the text's length
  in characters, a line break one, as changes 36 and 37 counted; recomputed by the task that writes
  it; an owner's rewording is recounted before pasting, and the dashboard's own counter read on
  paste (a field counting a line break as two would put the test instructions, seven breaks, at
  1,005).
- **Card lines quoted before change 51's snapshot exists** → quoted from its scenarios, checked by the
  owner against the merged snapshot (D2).
- **The table's orientation surprising a reader used to columns** → it turns at the moment its
  headers already change (two native languages), and only then.
- **A check that refuses a release over wording** → it fails with the language and the names it
  wanted, and the drafts are here.

## Migration Plan

Merged, nothing reaches a reader but `/lingua/`'s one word at the next site deploy. Change 52 commits
the summaries with its pairs; the owner deploys the site with its figures, then pastes the listings
and submits (M18). Reverting removes a check and a few sentences; no data moves.

## Open Questions

1. **The English home's card** (D9): its words are the owner's, as 29b says of the French and
   English cards — "Read the web in Spanish or French…", or keep the literal and write them by hand at
   change 34? **Settled by the owner on 2026-10-09: generated from 29b's builder, as D9 proposes.**
2. **The subtitles at 30 / 30** — "Spanish and French as you read", « Inglés y francés mientras
   lees » — or a shorter wording the owner prefers (M9)? **Settled on 2026-10-09: kept as drafted.**
3. **The share of definitions on the site** (change 49's open question 7, D11): in fr-es's README
   alone, as en-es's split is, or a column beside the coverage for every pair? **Settled on
   2026-10-09: in fr-es's README alone, as D11 recommends.**
