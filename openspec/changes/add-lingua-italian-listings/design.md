# Design — add-lingua-italian-listings

## Context

Change 21 of the [Italian programme](../../../docs/lingua/italian-programme.md), the last of stage 1.
Change 20 (`enable-lingua-italian`) lists it-fr for French speakers; this change makes what a French
reader is told about Cymbra Lingua know it, as the matrix programme's `add-lingua-french-listings`
did for French. Where things stand on `main`:

- **The listing files**: `apps/lingua-extension/STORE-LISTING.md` (the Chrome Web Store and
  addons.mozilla.org: a description per native language, the summary, the test instructions, the
  single purpose, the permission justifications, the remote-code answer) and
  `apps/lingua-apple/STORE-LISTING.md` (the App Store: subtitle, promotional text, keywords,
  description, « What's New » per locale, the review notes). Each file states how it counts; no tool
  parses them.
- **The site** (`apps/site/src/lib/lingua-text.ts`): the Lingua pages and the French home card are
  built from the shipped pairs (`lingua-coverage.json`, whose pairs the tests hold to `packs.json`)
  and from tables — a name per language in each site language (`names`), and `STUDIED`, what each
  studied language's card and levels are. A shipped pair whose language a table lacks fails the
  build (*The Lingua page says what each studied language's card and levels are*).
- **The back office** names en, es and fr (`lingua.languages`), and shows any other code raw.
- Change 20 commits the French summary (`_locales/fr`) and `check_version`'s `LANGUAGE_NAMES.it`,
  in the pull request that lists it-fr, as `enable-lingua-french` committed the English and Spanish
  ones.

## Goals / Non-Goals

**Goals:**
- Every French text of the listings names Italian; every English and Spanish text that says what
  French speakers study says it; the fields one per store follow, within their limits.
- The site describes Italian, so that change 20's coverage file builds; the back office names it.
- Nothing a reader sees moves before change 20: the site's code is inert until it-fr is listed, and
  the listing texts are pasted with change 20's release.

**Non-Goals:**
- The French summary and `LANGUAGE_NAMES` (change 20).
- English and Spanish texts for readers of Italian (stage 2: it-en and it-es), Italian listings and
  the App Store's it-IT locale (stage 3, I9), the site in Italian (stage 3).
- The remote-code answer's « one model per pair » after the pack split: changes 3 and 4 make « a pack
  per pair » a base and a layer; their documents say so, not this change.

## Measured

On `main` at `9c5c36b38` (2026-10-22), counted by each file's own rules (scripts reproducing every
count the files record):

| Text | Today | With Italian (D3–D5) | Limit |
|---|---|---|---|
| Chrome Web Store and AMO, French description | 2,835 | 2,924 | 16,000 |
| App Store fr-FR subtitle | 29 | 28 | 30 |
| App Store fr-FR promotional text | 154 | 165 | 170 |
| App Store fr-FR keywords | 93 bytes | 91 bytes | 100 bytes |
| App Store fr-FR description | 3,186 | 3,299 | 4,000 |
| App Store fr-FR « What's New » (new) | — | 455 | 4,000 |
| Test instructions (CWS, AMO) | 998 | 993 | 1,000 |
| App Store review notes | 3,984 | 3,989 (3,978 without Morph-it!) | 4,000 |
| CWS single purpose | 593 | 602 | — |

- **Too long said directly**: the subtitle « … en lisant » with a third language (32); the keywords
  with `italien,` added (101 bytes); the test instructions with step 6 widened (1,007); the review
  notes with the audience, MarkIT and the chained route (4,030 before D5's cuts).
- **The site**, simulated by bundling the real `lingua-text.ts` with Italian's names, `STUDIED.it`
  and an it-fr row: the build passes; the French page reads « Conçue pour les francophones qui
  apprennent l'anglais, l'espagnol ou l'italien », « En espagnol, en italien et en français, la carte
  nomme aussi le temps et le genre », « Pour l'espagnol, l'italien et le français, les niveaux sont
  estimés », a row « Italien → français »; the French home card « Lisez le web en anglais, en espagnol
  ou en italien »; the English and Spanish pages name it in their sentences. Without change 2's
  it-fr route the note reads « Pas encore pour l'italien vers le français »; with it, « l'italien vers
  le français (en passant par l'anglais) ». Three tests of `test/lingua-text.spec.ts` (around lines
  263, 399 and 481) take Italian as their example of an unnamed language; `:474` already takes German.
- **The coverage table**, a row per pair at a phone's width: the new row headers measure 123.9 px
  (« Italien → français »), 115.1 px ("Italian → French") and 128.8 px (« Italiano → francés »),
  under each page's widest (147.9, 131.3 and 134.8 px) and its corner cell (162.1 to 178.0 px): the
  column does not widen. Measured as text, not in a browser.
- **The review notes**: two texts. The dashboard holds 1.7.0's (3,976, carried to 1.8.1), with step 7
  — the app reviews the deck — and « No analytics »; the file's newer block (3,984) names the
  anonymous counts but lacks step 7.
- **Screenshots**: the Chrome Web Store's French set is five captures, the store's cap
  (`tool/capture/plan.mjs`: card, second language, review, statistics, reader), English then Spanish;
  the App Store's, captured by hand, takes up to ten per slot.

## Decisions

### D1 — The site describes Italian, inert until it-fr is listed

`lingua-text.ts`: `names.it` in the fr, en and es tables (« italien », "Italian", « italiano »;
`frElides` already elides « italien », the Spanish conjunctions give « e italiano », « o italiano »),
and `STUDIED.it = { namesTenseAndGender: true, levelsEstimated: true }` — the card names Italian's
tense and gender (changes 13 and 18), its levels are estimated (I8, change 14). The three tests that
took Italian as their unnamed language take German, as `:474` does. `apps/site/README.md`'s note on
`STUDIED` names Italian. Nothing renders differently until `lingua-coverage.json` lists it-fr, which
change 20 writes; the byte-pinned fixtures (`test/fixtures/lingua/`) are built from stand-in lists
and do not move. A test feeds a seven-pair list with an it-fr route and asserts D1's sentences.

### D2 — The coverage table's seventh row

*The coverage table reads at a phone's width* already turns the table into one row per pair; it-fr
is the seventh, French readers' pairs first on the French page. *Measured* shows no column widening;
task 2.2 checks the page at 375 px in a browser, the table scrolling within its box if it must.

### D3 — The French texts name Italian

Drafts (M9: the owner's words; counts in *Measured*):

- **Chrome Web Store and AMO, French description**: « les mots d'anglais, d'espagnol ou d'italien que
  vous ne connaissez pas encore »; « En espagnol et en italien, la carte nomme aussi le temps et le
  genre, comme on les apprend en classe (« passé simple », « nom féminin ») » — change 18 names the
  passato remoto « passé simple »; « l'espagnol et l'italien passent par l'anglais »; the sizes
  (D6); « Pour l'espagnol et l'italien, le dictionnaire français est un peu moins complet que pour
  l'anglais » (D6). The text names no levels.
- **App Store fr-FR**:
  - subtitle `Anglais, espagnol ou italien` (« en lisant » no longer fits);
  - promotional text « Les mots d'anglais, d'espagnol ou d'italien que vous ne connaissez pas encore,
    surlignés sur la page que vous lisez. L'analyse tourne sur votre appareil, hors ligne. »;
  - keywords `anglais,espagnol,italien,vocabulaire,lecture,traduction,apprendre,mots,révision,CECRL,deck`
    — `extension` gives way, the app's category and name already saying it (Open Question 2);
  - description: the first sentence as the Chrome text's; step 3 « Pour l'espagnol ou l'italien,
    cochez-le dans les Réglages de l'extension, onglet Langue, puis choisissez son niveau. », as change
    20's activation page says it; LIRE as the Chrome text; RÉVISER « pour l'espagnol et l'italien, des
    niveaux estimés d'après la fréquence des mots » — the requirement asks a text that names levels to
    say Italian's are estimated; TRADUCTION and the dictionary sentence as the Chrome text; the last
    line « … qui apprennent l'anglais, l'espagnol et l'italien … ».
- **« What's New » fr-FR, for the release of change 20** — the first French notes for a studied
  language: « Cymbra Lingua lit maintenant l'italien : cochez-le dans les Réglages de l'extension,
  onglet Langue, puis choisissez votre niveau. Les mots italiens arrivent avec leur sens en français,
  écrit par des personnes ; la carte nomme le temps et le genre (« passé simple »), et votre niveau
  d'italien est estimé d'après la fréquence des mots. La traduction étendue traduit votre sélection en
  italien vers le français, en passant par l'anglais, sur votre appareil. »
- **The summary**: `## Summary` quotes change 20's (« Le web en anglais, espagnol ou italien : … »,
  or the owner's words).

### D4 — The English and Spanish texts say what French speakers study

One sentence each, nothing offered to their own readers: the Chrome text in English, « Cymbra Lingua
also teaches English, Spanish and Italian to French speakers, in French. »; in Spanish, « … enseña
inglés, español e italiano a los francohablantes, en francés … »; the App Store's en-US/en-GB
description « … who learn English, Spanish and Italian. » and es-ES/es-MX « … que aprenden inglés,
español e italiano; … ». The texts kept for a release without fr-es are history, not edited. The
files' notes that say what French speakers study (the listing languages, the claims table's « who
learns what », the graphics' French set, the permissions' introduction, the App Store's locales)
follow.

### D5 — The fields one per store, within their limits

- **Single purpose**: "English, Spanish or Italian for French speakers" (602).
- **Remote code**: the packs listed gain it-fr; the models a French speaker may download gain it-en
  for Italian, through English.
- **Test instructions** (993 / 1,000): step 6 « "Français" English, Spanish and Italian »; to make
  room, step 1 « Pick a level (B1) at … » and « With none, every word is highlighted, the score 0%. ».
  No step or label lost.
- **App Store review notes** (3,989 / 4,000): based on the file's newer block, which names the
  anonymous counts, with 1.7.0's step 7 restored (« 7. Open the app: it reviews the deck built in
  Safari (on the device). ») and the IMPORTANT line made true of it (« IMPORTANT - this app hosts a
  Safari extension: enable it in Safari first. »); Italian: « Readers study English, Spanish or
  French, other than their own; French speakers Italian too. », « French to Spanish and Italian to
  French chain two » (26.3 MB the range's top, D6), « the UD Spanish-GSD, French-GSD and
  Italian-MarkIT treebanks, Morph-it! » (MarkIT is CC BY 4.0, credited; Morph-it! only if change 9
  keeps it), « the English, French, Spanish and Italian Wiktionaries » (it-fr reads the Italian
  Wiktionary's translations). Cut to fit: the account's « (words, deck, level, statistics) », the
  engine's « WebAssembly », « in place ». Every step, the deletion path, every external service and
  every source stay.
- **The dashboard's « No analytics »**: 1.7.0's notes, still on the dashboard at 1.8.1, say it while
  the extension sends anonymous counts since #958 (2026-10-15). Not Italian's, but the next
  submission's notes must not say it; this change's text does not, and the owner is told (Open
  Question 4).

### D6 — Figures the texts quote, taken when they are known

- **Sizes**: it-en's served size enters the catalogue with change 2's task 1.1. The drafts use its
  estimate (≈ 26.3 MB: « 52,0 Mo avec l'espagnol ou l'italien, 78,2 Mo avec les deux »); the
  implementation writes the catalogue's sums, rounded as the texts round.
- **The dictionary sentence** names Italian if it-fr's committed coverage is under en-fr's at each
  top (the study: 89.2 / 82.2 / 73.4 against 95.1 / 90.1 / 78.9; change 16), and keeps Spanish alone
  otherwise.
- **Translation** is said offered with change 2's route and change 19's figures: if the owner chose
  not to offer it-fr's translation below the marks' tier (change 20's Open Question 2), the TRADUCTION
  sentences name English and Spanish alone.

### D7 — The back office names Italian

`apps/back-office/src/i18n/locales/{fr,en}.json` `lingua.languages.it`: « Italien », "Italian";
`test/lingua.spec.ts`'s names case gains it. Inert until a device reports Italian. *The studied
languages are named in the console's language* already asks for a name; no requirement moves.

### D8 — Screenshots and dashboards

- **App Store fr-FR**: one capture more, by hand, on an Italian page with the card open on a
  passato remoto form (« passé simple »), on each slot already sent — as `add-lingua-french-listings`
  added one on a French page. Taken from change 20's dogfood build.
- **Chrome Web Store French set**: at its five-capture cap, unchanged; swapping its second capture
  from Spanish to Italian would need Italian capture fixtures and moves the site's alt texts (Open
  Question 3).
- **Dashboards**: no language is added (Italian ships no `_locales/it`); the French listing and
  summary are pasted by `add-lingua-french-listings`'s procedure, and what each dashboard shows is
  recorded.

### D9 — Order

The site's part must be on `main` before change 20 lists it-fr, or change 20's coverage file fails
the site's build — the order `add-lingua-french-listings` (#868, 2026-10-10) kept before French
shipped (#880, 2026-10-11). This change therefore merges first; its site code and back-office name
are inert until then, and its listing texts are true of change 20's release, pasted with it after
the site deploy that publishes it-fr's figures (I12, M18).

## Risks / Trade-offs

- **Texts written before their figures** → D6: the implementation writes the catalogue's sizes and
  change 16's coverage; the drafts say which sentence depends on which.
- **Tight fields** → every cut listed (D5); the owner reads them (M9); the counters' rules are the
  files' own.
- **The CWS French screenshots show no Italian** → Open Question 3; the App Store's do.

## Migration Plan

Nothing to migrate. The site and the back office deploy with their next release, inert until it-fr
is listed; the listings are pasted by the owner with change 20's release. Rollback is a revert.

## Effort

Against 1.5–3 ideal days: the French texts and the English and Spanish sentences, recounted,
0.5–0.75; the one-per-store fields 0.25–0.5; the site (names, `STUDIED.it`, the tests, a seven-pair
test, the 375 px check) 0.5–1; the back office 0.1; docs 0.1. 1.45–2.45.

## Open Questions

For the owner:
1. **The drafts** (D3–D5): as written, or other words (M9).
2. **The keywords** (D3): `extension` gives way to `italien`, or `deck` does.
3. **The Chrome Web Store's French screenshots** (D8): unchanged (recommended), or the second
   capture moved to an Italian page.
4. **« No analytics » on the dashboard** (D5): the next submission's review notes name the anonymous
   counts; this change's text does.
