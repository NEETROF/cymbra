# Cymbra Lingua — the Italian programme (it · fr · en · es)

Italian joins the matrix: every language of {English, French, Spanish, Italian} studied from each
of the other three, and the interface in the reader's own language, Italian included. This file
holds what the changes of the programme share: the product owner's decisions, the architecture they
build on, their order, and their status. Each change still goes through OpenSpec; its proposal
points here instead of re-arguing a decision.

- Template: [language-matrix-programme.md](language-matrix-programme.md), itself after
  [spanish-programme.md](spanish-programme.md). Their rules carry over unless this file says
  otherwise; the matrix's decisions M1–M25 hold for Italian unless a decision below restates one.
- Measurement: 2026-10-21, read-only, from this repository (the code graphs, the OpenSpec graph,
  the pack tables and the workflows) and from the day's upstream figures (kaikki.org's sense
  counts per edition and section, Mozilla's model records, the treebanks' licence lines). They are
  orders of magnitude, not targets; each pair's committed measurement comes with its change.
- Decisions: Guillaume Fortin (product owner), 2026-10-21.
- Status: each row's last cell (proposals in flight and merged are linked). Update the programme table in the pull request
  that moves a line.

## Scope

Six new pairs, named `<studied>-<native>` as packs already are, beside the six of the matrix:

| Pair | Audience | Stage |
|---|---|---|
| it-fr | French speakers | 1 |
| it-en | English speakers | 2 |
| it-es | Spanish speakers | 2 |
| en-it | Italian speakers | 3 |
| fr-it | Italian speakers | 4 |
| es-it | Italian speakers | 4 |

The interface (extension, Safari host app) speaks Italian too; its listings, the site's Lingua
pages and the account pages follow. The agent plugin stays out (M17).

Out of scope: remote translation (parked), the agent plugin (M17), YouTube captions beyond
change 56's amendment.

## The product owner's decisions

| # | Decision | Status |
|---|---|---|
| I1 | **The full matrix: the four languages interchangeable.** Twelve pairs and an Italian interface, not one audience. | Settled (2026-10-21) |
| I2 | **A dedicated agent reviews the Italian wording** (interface, card, listings, site), where the owner reviewed English and Spanish himself (M9). Drafts still land in each pull request. Dogfood of the Italian interface on the five targets is the owner's, or stays unverified. | Settled (2026-10-21) |
| I3 | **The pack format is split before any Italian pack ships: one base per studied language, one layer per pair, assembled at load.** The owner first accepted a package that grows (twelve embedded packs, ≈ 27–30 MB raw against 13.5 MB today), then reversed it the same day once the saving was measured: the studied side is 59 % of the six packs and byte-identical across natives, so a split carries each language's base once — ≈ 9.5 MB today instead of 13.5, ≈ 15–18 MB with twelve pairs instead of 27–30. The matrix programme's note that a fourth language makes a split worth it is taken up (changes 3 and 4). | Settled (2026-10-21) |
| I4 | **Thin pairs ship with their coverage published, under a floor fixed before the committed measurement** (fr-es precedent, M6). Expected thin: it-es and es-it. | Settled (2026-10-22): new sources were researched — none written directly for the pair reaches fr-es's floor (Wikidata Lexemes and the Italian Wiktionary's Spanish section read backwards: 65.3 / 53.2 / 40.5) —, and the owner chose the full pivot: it-es also reads the translation tables of seven other Wiktionaries (I11), every pair kept, which measures 84.8 / 76.3 / 65.8 on the study's ranking. Its floor is fixed at change 27's task 0.1 on the committed ranks less two points, and its coverage published. Apertium's GPL dictionary is not used. **es-it, settled 2026-10-22**: direct sources reach 64.0 / 49.9 / 36.5 on Spanish's committed ranks (a quarter of the commonest dictionary words unglossed, `a`, `para`, `bien` among them); the owner chose the same full pivot, read the other way from change 27a's files (86.0 / 75.8 / 64.0, level with es-fr's 87.6 / 77.2 / 63.7), with a reviewed list of the commonest function words; floor fixed at change 51's task 0.1 |
| I5 | **Order: it-fr first** (today's readers, no Italian text to write), then it-en and it-es, then the Italian interface with en-it, then fr-it and es-it. Stage 3 does not depend on stage 2 and may interleave with it, but for change 45, which appends en-it after it-en and extends changes 25's and 26's blocks: changes 20 and 22–26 land first. | Settled (2026-10-21) |
| I6 | **Italian: tu, never Lei; QCER for the level acronym (M19's rule); Treccani's names for tenses and moods; Italian numbers (20.000; 96%).** | Settled (2026-10-22) |
| I7 | **Articulated prepositions, read as M21 reads French:** the forms that are also partitives stay whole (del, dello, della, dei, degli, delle, as du/des), the others split into their two pieces with their own spans (al, allo, alla, ai, agli, alle; dal, dallo, dalla, dai, dagli, dalle; nel, nello, nella, nei, negli, nelle; sul, sullo, sulla, sui, sugli, sulle; col, coi, as au/aux). Elision pieces (l', dell', un', c', quest') get their own spans, as French's. Truncations (po', fa', va', di') read as their word. | Settled (2026-10-21) |
| I8 | **Italian levels are estimated, as French's and Spanish's** (M7, D1): no licence-clean Italian CEFR list exists (the Profilo della lingua italiana is not free). | Settled (2026-10-21) |
| I9 | **App Store: it-IT joins fr-FR, en-US, en-GB, es-ES and es-MX** (M16 carried); the dashboards' Italian listings are pushed from the repository like the others. The Italian listing ships with the first build that declares Italian (change 45), as the English listing shipped with the first localised build (M13): change 45's Apple submission goes with change 46's listing. | Settled (2026-10-22) |
| I10 | **Account e-mails and the deletion link follow the Italian interface language**, like fr, en and es: M12's special case for Italian ends the day Italian is an interface language (change 45). Until then it holds. | Settled (2026-10-22) |
| I11 | **Sources.** Studied side: the English Wiktionary's Italian section (forms, readings, it-en glosses), wordfreq's Italian list, UD Italian-MarkIT (CC BY 4.0) for parts of speech, Morph-it! (CC BY-SA 2.0 / LGPL) as a candidate completing the forms, verified in change 9. Native side: the Italian Wiktionary's English, French and Spanish sections, then the English, French and Spanish Wiktionaries' Italian translation tables, then the Italian Wiktionary's own tables read backwards, as en-es reads the Spanish Wiktionary's (M5: written by people). UD Italian-ISDT, VIT, ParTUT and PoSTWITA are non-commercial and never enter a pack. **For it-es and es-it only** (the owner, 2026-10-22, I4): Wikidata Lexemes (CC0, an Italian and a Spanish sense of the same item), the Italian Wiktionary's Spanish section read backwards, and the translation tables of the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries read as a pivot — an Italian and a Spanish word under the same sense of one table, or the same entry when the table names none, each word written by a person — every pair kept, names left out; an exception to M5's « never a pivot through another language », for these two pairs, because no direct source reaches a shippable coverage; es-it reads the same sources the other way — Wikidata's lexemes, the Spanish Wiktionary's Italian section read backwards, the seven Wiktionaries' tables as a pivot —, after the Italian Wiktionary's Spanish section and both tables, and its reviewed function-word list only chooses among the words its sources pair. | Settled (2026-10-21; amended 2026-10-22 for it-es, then for es-it) |
| I12 | **Merges and releases as the matrix programme (M18): merged at green CI; nothing released, deployed or submitted without the owner.** | Carried; the owner confirms |
| I13 | **The site's Italian legal pages are courtesy translations; the French prevails.** The Italian privacy policy, terms and account deletion page (change 41) open with « Traduzione di cortesia: fa fede la versione francese », linking the French page, and are reviewed by the Italian reviewer agent (I2), with no lawyer. No one on the team reads Italian, and an Italian legal text that says more than the French could bind the publisher. | Settled (2026-10-22) |

## Architecture

The matrix programme's architecture holds. What Italian adds or stresses:

- **Variants are appended, never inserted.** `StudiedLanguage::Italian` after French and
  `NativeLanguage::Italian` after Spanish: the derived order keys serialised maps, so every stored
  state stays byte-identical. The extension's `StudiedLanguage` and `NativeLanguage` unions widen
  to `it`; `INTERFACE_LANGUAGES` and the site's `LANGS` gain `it` in last position.
- **Backup schema v4**, written only when Italian is studied, as v3 is for French: a build
  released with v3 refuses such a backup as unsupported, never reads it as malformed. An Italian
  native alone is a version 2 file already.
- **Detection.** whichlang knows Italian and none of its neighbours: probed with whichlang itself,
  it reads Catalan, Galician, Venetian, Sicilian, Neapolitan, Sardinian, Corsican and Friulian
  sentences as Italian, and Latin ones seldom (`crates/lingua-core/src/analysis/language.rs`, the
  Galician probe; `crates/lingua-wasm/tests/french_baseline.rs`, the Catalan one). Change 8 adds
  French's guard shape for Italian — function words pooled and compared, measured word by word —
  with those eight neighbours kept by measurement and Latin, Occitan and Romanian measured, out
  by default; a recall gate on Italian corpora.
- **Tokenisation.** Italian stacks what French and Spanish had apart: elision pieces (as French),
  articulated prepositions (I7, where Spanish has al and del only), truncation, the web's apostrophe
  for an accent (E', perche', piu', citta'; the precedent is French's unmarked spellings) and
  enclitic pronouns stacked on infinitives, gerunds and imperatives (dirmelo, vattene, eccolo;
  Spanish's enclitic rule is the model, without its stress shift).
- **Pack format (I3).** A pack becomes two files: a *base* per studied language — `forms`,
  `lemmas`, `freq`, `levels`, `lexical`, `paradigms.zst` and a `tags` pool of the pinned pool and
  the readings' tags, with metadata naming the language, its analyser version, a `base_version`
  digest of the base's own bytes and whether its levels are estimated — and a *layer* per pair —
  `gloss.zst`, `senses.zst` and the tags only those senses carry, the `expr*` sections, the pair's
  whole NOTICE and licences, with metadata naming the pair, `pack_version` and the `base_version` it
  was built against. Assembled, the two are today's pack, every named section byte for byte (the
  pool being the base's followed by the layer's sense tags, as a whole pack lays it out), so
  `PackSet`, the engine and every golden are untouched; a whole pack keeps loading, so the agent
  plugin and a test fixture still read one file, and every whole-pack reader refuses a part by
  name. No table, pin or pinned pool moves. The extension fetches a base once per studied language
  and a layer per pair (change 4); the builder writes both and `pin.json` records both; the 5 MiB
  budget applies per file.
- **Packs.** `tables/it/` holds the studied side once (M24); `tables/it-fr`, `it-en`, `it-es`,
  `en-it`, `fr-it` and `es-it` the native side. The `lingua-pack-update` workflow gains the six
  pairs and an `it` group (the reference pair writes `tables/it/` and brings its readers along).
  A `reduce_edition_it.py` carries the Italian Wiktionary's rules beside the French, English and
  Spanish editions'. Six new goldens, one per pair over its studied language's corpus (`pages-it.txt` for it-*), run
  beside the six of the matrix; `lingua-extension-check` gates them as it gates the others.
- **Translation.** Mozilla publishes it-en 2.0 and en-it 2.1 (Remote Settings, 2026-10-21);
  nothing between Italian and French or Spanish. Routes: it-en and en-it direct; it-fr through
  it-en then en-fr; it-es through it-en then en-es; fr-it through fr-en then en-it; es-it through
  es-en then en-it. Two models at most per route, as es-fr; the engine's hardening (matrix change
  9) already evicts what a route no longer needs. The owner redeploys the model host before the
  first release whose catalogue names them.
- **Interface.** A fourth module set `src/i18n/it/`, typed `typeof fr` over the 23 text surfaces (`grammar` with change 34);
  Italian plurals through `Intl.PluralRules`; `formatNumber` and `formatPercent` learn Italian
  (20.000; 96%). `_locales/it` serves the manifest and the listings. The Apple host's `copy.js`
  and `SignInCopy` gain Italian; `app_localizations.sh` accepts `it` beside fr, en and es.
- **Word card.** The language-neutral readings get an Italian renderer (native side) and tense
  names per pair (studied side): an Italian verb's `Tense=Past` is the passato remoto named in
  French for it-fr, in English for it-en, in Spanish for it-es; an English, French or Spanish past
  is named in Italian for en-it, fr-it and es-it (the terms: I6). Goldens lock the order of tenses.
- **Server.** Nothing to deploy: card gloss languages and daily native languages are stored as
  given (`backend/lingua/src/language_core.rs`, `normalise_or` refuses nothing; migration 0006
  adds no check), and Cymbra ID already sends its e-mails in Italian (`SupportedLocale::It`).
  Only the legal links move, from the English pages to the Italian ones once they exist
  (change 43).
- **Site.** `/it/` mirrors `/es/`: home, Music, Lingua, privacy, terms, support, account deletion,
  404, plus the islands' copy; `hreflang` lists four languages.
- **Package.** 13.5 MB of packs today: 8.0 MB of studied sides, 5.5 MB of native sides. Split,
  the six pairs are 4.0 MB of bases and 5.5 MB of layers, 9.5 MB; with twelve pairs, four bases
  (≈ 5.5 MB) and twelve layers (≈ 10–12 MB), ≈ 15–18 MB — against ≈ 27–30 MB embedded. No store
  limit is near either way (AMO's is 200 MB).

## Measured figures (2026-10-21)

Sense counts are kaikki.org's on the day, per edition and section; they size a source, they are
not coverage. Coverage (the share of the 5,000 / 10,000 / 20,000 commonest lemmas with a gloss)
is measured by each pair's change, as before.

| Pair | Gloss source | Section size | Reading |
|---|---|---|---|
| it-fr | French Wiktionary, Italian section | 1,310,041 senses | the richest section any pair has read (en-fr reads 225,028 and covers 95.1 / 90.1 / 78.9) |
| it-en | English Wiktionary, Italian section | 719,680 senses, 589,335 distinct forms | larger than its French section (459,894), which gave fr-en 93.9 / 87.1 / 76.4 |
| it-es | Spanish Wiktionary, Italian section, then the Italian Wiktionary's Spanish translations, then the Spanish Wiktionary's tables read backwards | 7,810 senses (+ 13,018 translations of 7,029 Italian words; + 13,837 of 10,244) | measured on 2026-10-22 for change 27, all three sources, on change 16's study ranking: 57.8 / 46.8 / 35.0, about 25 points under fr-es's committed 83.2 / 70.9 / 57.1 — I4, settled on 2026-10-22, adds change 27a's sources (84.8 / 76.3 / 65.8 with all of them). The « + 8,665 » first written here was the Italian Wiktionary's Spanish section, es-it's source |
| en-it | Italian Wiktionary, English section, then the English Wiktionary's Italian translation tables, then the Italian Wiktionary's English tables read backwards | 41,754 senses, 25,700 headwords (dump of 2026-10-21) | measured for change 36 on the committed English ranks with change 35's rules: 84.8 / 72.2 / 53.0 from the definitions, 91.4 / 82.7 / 70.3 with both tables — against en-es's committed 93.0 / 85.0 / 71.7, level on English's dictionary words (96.8 % at the 5,000), the gap in contractions and names; floor about 89.4 / 80.7 / 68.3, the owner's |
| fr-it | Italian Wiktionary, French section, then the French Wiktionary's Italian translations, then the Italian Wiktionary's French translations read backwards | 35,026 senses, 26,108 headwords — 69.9 % of the senses French verb forms (75.5 % inflections), 6,579 headwords with a meaning (dump of 2026-10-21) | measured for change 35: about 28.8 / 20.9 / 13.8 from the definitions (estimated; heraldry-only words go to the tables), about 81.4 / 70.2 / 57.8 with both tables — as fr-es (83.4 / 70.8 / 56.5) only once the tables are read; prototyped for change 47 on the committed ranks: 81.3 / 70.1 / 57.8, against fr-es's committed 83.2 / 70.9 / 57.1 |
| es-it | Italian Wiktionary, Spanish section, then the Spanish Wiktionary's Italian translations, then the Italian Wiktionary's Spanish translations read backwards; then (I11) Wikidata's lexemes, the Spanish Wiktionary's Italian section read backwards and the seven Wiktionaries' pivot | 8,660 senses, 6,861 headwords (dump of 2026-10-21) | measured for change 35: about 33.6 / 23.4 / 14.9 from the definitions (estimated), about 64.2 / 50.0 / 36.6 with both tables — thin, as I4 expects; a floor before measuring. Measured for change 51 on Spanish's committed ranks: 64.0 / 49.9 / 36.5 with both tables (the letters rule takes 11 one-letter lemmas, `o` among them); with Wikidata and the Spanish Wiktionary's Italian section read backwards 68.5 / 54.1 / 40.2; with the pivot 86.0 / 75.8 / 64.0 |

Other figures:

- Translation models: it-en 2.0 and en-it 2.1 in Mozilla's records; no fr↔it, no es↔it.
- Treebanks: UD Italian-ISDT and VIT (CC BY-NC-SA 3.0), ParTUT (CC BY-NC-SA 2.0) and PoSTWITA
  (CC BY-NC-SA 4.0) are denied by the licence rule; MarkIT (CC BY 4.0) is allowed; PUD serves the
  measurement as French-PUD and Spanish-PUD do. Morph-it! is dual CC BY-SA 2.0 / LGPL.
- The interface catalogue: 24 modules per language; the French set is 8,916 words of source, the
  English 6,657, the Spanish 7,416. `_locales` is ≈ 1.2 kB per locale.
- The site's Spanish locale: 8 files, 5,983 words, plus 762 words of island copy. The Apple host's
  copy: 951 words for three languages. The listings: ≈ 2,000 words per locale per store.
- Code that names the three languages: `StudiedLanguage` is matched in 35 Rust files,
  `NativeLanguage` in 6; 34 extension files name a pair literally; 25 test files enumerate fr, en
  and es; the pack-update workflow and the Apple localisation script list them.
- Packs, built from the committed tables on 2026-10-21 (the bytes `pin.json` records): en-fr
  1,877,205 B; es-fr 2,242,425; es-en 2,626,743; en-es 1,690,521; fr-en 3,052,887; fr-es
  1,984,066 — 13,473,847 B for six. Of these, the studied sections (`forms`, `lemmas`, `freq`,
  `levels`, `tags`, `paradigms.zst`, `lexical`) are 7,975,182 B (59 %) and the native sections
  (`gloss.zst`, `senses.zst`, `expr*`) 5,485,694 B. A studied side is byte-identical across its
  natives (es-fr = es-en, fr-en = fr-es; en-fr = en-es but for one tag in the pool, the sense tag
  `NUM` a whole pack's pool ends with and a base's never holds): one base per language is
  935,029 B for English, 1,575,403 for Spanish, 1,477,161 for French.
- Precedents, in ideal days after the verifiers: the matrix's stage 1 (platform, 19 changes)
  64–110; stage 2 (two audiences, 22 changes) 54–98; stage 3 (French studied, 40 changes with its
  follow-ups) 76–134. Stage 3 went from its first proposal ([#817](https://github.com/NEETROF/cymbra/pull/817),
  2026-10-09) to its listings merged ([#868](https://github.com/NEETROF/cymbra/pull/868), 2026-10-10).

## Programme

In merge order within a stage (1 change = 1 pull request, none above ≈ 10 ideal days), but for one
inversion: change 21's description of Italian on the site (its names, `STUDIED.it`) and in the back
office is implemented before change 20 lists it-fr, which the site's build would otherwise refuse —
as the matrix's change 53 (#868) was before 52 (#880). Effort is
in ideal days, min–max, by analogy with the measured change of the matrix programme named in the
*Precedent* column; it measures size, not calendar. Stage 3 does not depend on stage 2, but for change 45 (I5).

| Stage | # | Change | Effort | Precedent | Status |
|---|---|---|---|---|---|
| 0 Prerequisites | 1 | This programme and its decisions | 0.25 | — | Done (this file) |
| 0 | 2 | `add-lingua-italian-translation-models`: it-en 2.0 and en-it 2.1 pinned, six routes; the owner redeploys the model host right after | 1–2 | 25 | Proposal merged ([#973](https://github.com/NEETROF/cymbra/pull/973)); implemented ([#1034](https://github.com/NEETROF/cymbra/pull/1034)), the model host deployed on 2026-10-22 |
| 0 | 3 | `split-lingua-pack-format` (I3): a base per studied language and a layer per pair, `Pack::assemble`, the builder writing both, the wasm engine loading both; no table, pin or golden moves | 4–6.5 | 5, 7 | Proposal merged ([#972](https://github.com/NEETROF/cymbra/pull/972)); implemented ([#1035](https://github.com/NEETROF/cymbra/pull/1035)) |
| 0 | 4 | `ship-lingua-pack-bases-once`: the package carries each base once (`assets/bases/`) and each listed pair's layer at the pair's path, so the Apple natives derivation and `packs.json` read what they read; the engine fetches a base and a layer on first need; `pin.json` records both parts; `build.sh --parts`, `gen_pack.sh`, the build's checks, the variant check, the manifests, the CI comparison and the reviewer's source archive follow; the test fixtures gain their pool | 2.5–4 | 7, 8 | Proposal merged ([#974](https://github.com/NEETROF/cymbra/pull/974)); implemented ([#1037](https://github.com/NEETROF/cymbra/pull/1037)), the dogfood before the next release |
| 1 Italian studied: it-fr | 5 | `add-lingua-italian-baseline`: the variant, backup v4, the Italian corpus and golden beside the six | 3–5 | 39 | Proposal merged ([#975](https://github.com/NEETROF/cymbra/pull/975)); implemented ([#1038](https://github.com/NEETROF/cymbra/pull/1038)) |
| 1 | 6 | `add-lingua-italian-tokenisation` (I7): elision pieces, articulated prepositions, truncation (the apostrophe for an accent is change 10's) | 5–8 | 40 | Proposal merged ([#976](https://github.com/NEETROF/cymbra/pull/976)); implemented ([#1040](https://github.com/NEETROF/cymbra/pull/1040)), analyser 0.2.0 |
| 1 | 7 | `add-lingua-italian-analysis`: NFC, cascade, closed classes, names rule, enclitic pronouns; analyser 1.0.0 | 5–8 | 41 | Proposal merged ([#977](https://github.com/NEETROF/cymbra/pull/977)); implemented ([#1041](https://github.com/NEETROF/cymbra/pull/1041)), analyser 1.0.0 |
| 1 | 8 | `add-lingua-italian-detection-guard`: Catalan, Galician, Venetian, Sicilian, Neapolitan, Sardinian, Corsican, Friulian pooled and measured; Latin, Occitan, Romanian measured, out by default | 3–6 | 42, 42b | Proposal merged ([#980](https://github.com/NEETROF/cymbra/pull/980)) |
| 1 | 9 | `add-lingua-italian-forms-tables` (I11): the English Wiktionary's Italian section, MarkIT's parts of speech, Morph-it! verified | 8.5–14 | 43 | Proposal merged ([#978](https://github.com/NEETROF/cymbra/pull/978)) |
| 1 | 10 | `read-lingua-italian-unmarked-spellings`: E', perche', piu', citta', and the dictionary's own accent-less spellings | 2–4 | 43d, 43e | Proposal merged ([#981](https://github.com/NEETROF/cymbra/pull/981)) |
| 1 | 11 | `keep-lingua-italian-acronyms-and-names`: acronyms in capitals stay acronyms; names leave the words to learn (the rows' gloss and `lexical.tsv` land with change 16) | 1.5–3 | 43c, 24c | Proposal merged ([#979](https://github.com/NEETROF/cymbra/pull/979)) |
| 1 | 12 | `add-lingua-italian-expression-keys`: expressions keyed as Italian is read; the expression written as the page | 2–4 | 44, 44c–44e | Proposal merged ([#982](https://github.com/NEETROF/cymbra/pull/982)) |
| 1 | 13 | `add-lingua-italian-grammar-tables`: UD tags (passato remoto `Tense=Past`, imperfetto `Tense=Imp`, checked against it_isdt already) | 3–5 | 45 | Proposal merged ([#984](https://github.com/NEETROF/cymbra/pull/984)) |
| 1 | 14 | `add-lingua-italian-levels` (I8): the estimated ladder | 1.5–3 | 46 | Proposal merged ([#985](https://github.com/NEETROF/cymbra/pull/985)) |
| 1 | 15 | `add-lingua-italian-read-aloud`: it-IT and it-CH voices | 1–2 | 47 | Proposal merged ([#986](https://github.com/NEETROF/cymbra/pull/986)) |
| 1 | 16 | `add-lingua-pack-it-fr`: the French Wiktionary's Italian section; coverage committed; the dictionary words, which narrow change 14's levels | 3–5 | 48 | Proposal merged ([#987](https://github.com/NEETROF/cymbra/pull/987)) |
| 1 | 17 | `refine-lingua-it-fr-glosses`: change 16's *Known data defects* | 2–3.5 | 48b | Waits for change 16's committed it-fr tables: its defects are measured on them |
| 1 | 18 | `add-lingua-italian-word-card`: readings and tense names of an Italian word, in French, English and Spanish | 3.5–6 | 51 | Proposal merged ([#988](https://github.com/NEETROF/cymbra/pull/988)) |
| 1 | 19 | `measure-lingua-it-fr-translation-marks` | 1–2 | 26, 50 | Proposal merged ([#990](https://github.com/NEETROF/cymbra/pull/990)); implemented ([#1039](https://github.com/NEETROF/cymbra/pull/1039)): 93 of 97 correct, 3 % withheld, 89 of 97 strict — on the first tier, listed in `MARKED_PAIRS`, inert until row 20 |
| 1 | 20 | `enable-lingua-italian` (it-fr): packs.json, the five targets, the owner's go-ahead | 4–7.5 | 52 | Proposal merged ([#992](https://github.com/NEETROF/cymbra/pull/992)) |
| 1 | 21 | `add-lingua-italian-listings`: the French listings and the site's French Lingua pages name Italian | 1.5–3 | 53 | Proposal merged ([#993](https://github.com/NEETROF/cymbra/pull/993)) |
| 2 Italian studied: it-en, it-es | 22 | `add-lingua-pack-it-en`: the English Wiktionary's Italian section | 3–5 | 48 | Proposal merged ([#995](https://github.com/NEETROF/cymbra/pull/995)) |
| 2 | 23 | `refine-lingua-it-en-glosses` | 2–3.5 | 48b | Waits for change 22's committed it-en tables: its defects are measured on them |
| 2 | 24 | `measure-lingua-it-en-translation-marks` | 1–2 | 26 | Proposal merged ([#996](https://github.com/NEETROF/cymbra/pull/996)) |
| 2 | 25 | `enable-lingua-italian-for-english-speakers` (it-en) | 4–7.5 | 34 | Proposal merged ([#999](https://github.com/NEETROF/cymbra/pull/999)) |
| 2 | 26 | `add-lingua-italian-english-listings` | 1.5–3 | 36 | Proposal merged ([#1000](https://github.com/NEETROF/cymbra/pull/1000)) |
| 2 | 27a | `add-lingua-it-es-pivot-sources` (I4, I11): seven Wiktionaries' translation tables kept by sense for a pivot, the Italian Wiktionary's Spanish section, Wikidata Lexemes — derived and pinned | 2.5–4 | — | Proposal merged ([#1005](https://github.com/NEETROF/cymbra/pull/1005)) |
| 2 | 27 | `add-lingua-pack-it-es` (I4): the floor fixed first; the Spanish Wiktionary's Italian section, the Italian Wiktionary's Spanish translations, the Spanish tables read backwards, then change 27a's sources | 4–7 | 49 | Proposal merged ([#1006](https://github.com/NEETROF/cymbra/pull/1006)) |
| 2 | 28 | `refine-lingua-it-es-glosses` | 2–3.5 | 49b | Waits for change 27's committed it-es tables: its defects are measured on them |
| 2 | 29 | `measure-lingua-it-es-translation-marks`: modifies *A pair's marks are measured before they are shown* as change 38 leaves it, and archives after it | 1–2 | 26 | Proposal merged ([#1022](https://github.com/NEETROF/cymbra/pull/1022)) |
| 2 | 30 | `enable-lingua-italian-for-spanish-speakers` (it-es): two models even for Italian alone; listed by change 27's floor, after change 28; whichever of 30 and 45 is implemented second re-copies the first's blocks and names it in `archiveAfter`, the list naming en-it and it-es in the order `packs.json` first listed them; renames changes 22's and 27's « …, and not shipped » requirements; change 31 implemented before it; it-es appended at the end of the list, after any Italian-native pair (amended by change 50) | 4–7.5 | 35 | Proposal merged ([#1025](https://github.com/NEETROF/cymbra/pull/1025)) |
| 2 | 31 | `add-lingua-italian-spanish-listings`: archives after changes 45 and 46, carries change 46's scenario *A listing in Italian*, and updates the Italian texts' line on what Spanish speakers study (whichever of 31 and 46 is implemented second writes it); adds the table's scenarios for it-es as the ninth or tenth pair on change 45's block, which already narrows *Eight* and *Nine pairs on a phone* | 1.5–3 | 37 | Proposal merged ([#1024](https://github.com/NEETROF/cymbra/pull/1024)) |
| 3 Italian speakers: en-it | 32 | `add-lingua-italian-native-language`: the variant, the native list and names, every enumeration of fr/en/es, inert until a pair is glossed in Italian | 1–2 | 4, 20 | Proposal merged ([#1003](https://github.com/NEETROF/cymbra/pull/1003)) |
| 3 | 33 | `add-lingua-italian-interface`: `src/i18n/it`, 23 text modules typed after the French (`grammar.ts` is change 34's); the Italian reviewer agent defined; Italian numbers, percentages, dates and plurals; the lint; `InterfaceLanguage = NativeLanguage` restored and change 32's it→en mapping dropped (`INTERFACE_LANGUAGES` gains `it` with change 45) | 4–7 | 13–17 | Proposal merged ([#1007](https://github.com/NEETROF/cymbra/pull/1007)) |
| 3 | 34 | `add-lingua-italian-card-wording`: the Italian renderer of readings; English, French and Spanish tense names for an Italian reader | 3.5–4.5 | 23, 24 | Proposal merged ([#1010](https://github.com/NEETROF/cymbra/pull/1010)) |
| 3 | 35 | `add-lingua-italian-edition-rules`: the Italian Wiktionary's rules in the gloss reducer | 2–3 | 6 | Proposal merged ([#1011](https://github.com/NEETROF/cymbra/pull/1011)) |
| 3 | 36 | `add-lingua-pack-en-it` (I11): the Italian Wiktionary's English section, then both Wiktionaries' tables; the floor fixed first | 3.5–6 | 22 | Proposal merged ([#1016](https://github.com/NEETROF/cymbra/pull/1016)) |
| 3 | 37 | `refine-lingua-en-it-glosses`: modifies change 36's *English is glossed in Italian…* as change 47 leaves it if 47 is implemented first (whichever of the two is second re-copies the other's block and names it in `archiveAfter`), and as change 51 leaves it if 51 is implemented first, by the same rule | 2–4 | 24b | Waits for change 36's committed en-it tables: its defects are measured on them |
| 3 | 38 | `measure-lingua-en-it-translation-marks`: the English selections through en-it, judged in Italian by two agent judges (I2) | 1–2 | 26 | Proposal merged ([#1020](https://github.com/NEETROF/cymbra/pull/1020)) |
| 3 | 39 | `localise-lingua-manifest-italian`: `_locales/it` | 0.5–1 | 27 | Proposal merged ([#1009](https://github.com/NEETROF/cymbra/pull/1009)) |
| 3 | 40 | `localise-lingua-apple-host-italian`: `copy.js`, `SignInCopy`, `app_localizations.sh` | 1.5–3 | 28 | Proposal merged ([#1013](https://github.com/NEETROF/cymbra/pull/1013)) |
| 3 | 41 | `add-site-italian-locale`: `/it/` home, Music, Lingua's Italian text, privacy, terms, support, account deletion, 404, and Lingua's Italian leaving page, pinned before the extension build that compiles it into `LEAVING_PAGES.it` (change 45) ships; the islands | 6–10 | 29, 29b | Proposal merged ([#1014](https://github.com/NEETROF/cymbra/pull/1014)) |
| 3 | 42 | `extend-site-lingua-matrix-pages-italian`: folded into change 45 (2026-10-22). Once changes 21 and 41 are in, the Lingua pages take Italian's pairs, `/it/lingua/`'s table and its `hreflang` from the data (measured on a build); what remained — a scenario and five tests — is triggered by change 45's own coverage write | 0 | 30 | Folded into 45 |
| 3 | 43 | `localise-cymbra-id-email-legal-links-italian` (I10): Italian e-mails link `/it/` pages, after the site; the privacy annex names English only as the pivot, stays true, and is translated with `/it/privacy/` (change 41); merged only after change 41's site is deployed | 0.5–1 | 31, 32 | Proposal merged ([#1015](https://github.com/NEETROF/cymbra/pull/1015)) |
| 3 | 44 | `review-lingua-italian-wording` (I2): the dedicated agent's pass over interface, card, manifest (`_locales/it`), listings and site — and the Apple host's `copy.js` and `SignInCopy`; implemented after changes 45 and 46 are merged, so that its pass covers the listings and the captures, and before the owner's dogfood build and change 45's release | 2.5–3.75 | 33 | Proposal merged ([#1021](https://github.com/NEETROF/cymbra/pull/1021)) |
| 3 | 45 | `enable-lingua-italian-speakers` (en-it): the Italian interface shown, en-it ships; the extension's privacy, leaving and deletion links moved to the `/it/` pages (after 41); its dogfood checks the shortcuts page and about:addons in Italian (change 39's D3); its Apple submission goes with change 46's Italian listing (I9); its dogfood opens the host app on a device in Italian before the extension has run (change 40's D8); it touches `apps/lingua-apple` or carries `Release-As`, so that release-please cuts the host app's release with en-it (change 40's D7); lists en-it together with `/it/lingua/`'s screenshots and demo video, rewriting `LINGUA_TEXT.it`'s captions against its captures (change 41's D6); modifies the Lingua page's `hreflang` scenario (site-discoverability) and its level-scale names (site-lingua-page: QCER for Italian); modifies *The account's e-mails follow the interface language* (« `fr`, `en` or `es` », its scenario *An Italian browser*, `account-locale.spec.ts`, `locale.ts`'s comment) and *Cymbra account deletion is reachable from Lingua* (`archiveAfter: add-site-spanish-locale`), `deleteAccountUrl` gaining Italian's arm and the Spanish one change 29 left undone, `/en/delete-account/`'s `pinnedBy` updated (I10, change 43's D3); gives the capture harness a one-language mode (an Italian reader studies English alone; `plan.mjs` assumes a second language), whose run makes the site's and the stores' Italian captures (change 46's D4); carries change 42's leftovers: *Eight pairs on a phone* narrowed (« and no other pair ») and a ninth pair's scenario at 375 px, unless it-es ships first (row 31); `alternates.spec`'s Lingua test generalised; `/it/lingua/`'s render and post-build tests, `head.spec`'s share preview and `lingua-text.spec`'s card buttons for Italian; pins `/it/lingua/`, the App Store's Italian marketing URL (change 46); lands after changes 20 and 22–26, whose list and blocks it extends | 5–9.25 | 35 | Proposal merged ([#1019](https://github.com/NEETROF/cymbra/pull/1019)) |
| 3 | 46 | `add-lingua-italian-audience-listings` (I9): Chrome Web Store, AMO, App Store it-IT | 2–5.5 | 37 | Proposal merged ([#1017](https://github.com/NEETROF/cymbra/pull/1017)) |
| 4 Italian speakers: fr-it, es-it | 47 | `add-lingua-pack-fr-it`: fr-es's order and add-pack rules, a borrowed gloss giving way to the tables; prototyped at 81.3 / 70.1 / 57.8, floor about 79.3 / 68.1 / 55.8; widens change 36's « Italian edition read by en-it alone » | 3.5–6 | 22, 49 | Proposal merged ([#1026](https://github.com/NEETROF/cymbra/pull/1026)) |
| 4 | 48 | `refine-lingua-fr-it-glosses` | 2–4 | 49b | Waits for change 47's committed fr-it tables: its defects are measured on them |
| 4 | 49 | `measure-lingua-fr-it-translation-marks`: the 100 French selections already committed, through fr-en then en-it, two agent judges as change 38 (Italian judged); a trial outside the repository found 90 of 93 correct, 7 withheld; MODIFIES the requirement as change 29 leaves it, and archives after it; the gloss columns stay empty if changes 47 and 48 are not committed, change 48 then rerunning the experiment | 1–2 | 26 | Proposal merged ([#1027](https://github.com/NEETROF/cymbra/pull/1027)) |
| 4 | 50 | `enable-lingua-french-for-italian-speakers` (fr-it), with its listings; rewrites `_locales/it`'s description and the activation page's Italian copy to name French; fr-it appended at the end, en-it ahead of it (change 30 amended to append it-es at the end too); the Italian captures taken again with French; whichever of 30, 50 and 54 is implemented later re-copies the earlier's blocks | 4.75–9.5 | 52, 53 | Proposal merged ([#1029](https://github.com/NEETROF/cymbra/pull/1029)) |
| 4 | 51 | `add-lingua-pack-es-it` (I4, I11): the floor fixed first; the full pivot read the other way, with a reviewed function-word list (the owner, 2026-10-22); modifies change 36's *English is glossed in Italian…* as change 47 leaves it (the Italian edition read by en-it, fr-it and es-it), change 27's *A gloss is in the reader's language…* and change 27a's sources requirement, and archives after them | 4.75–8.75 | 49 | Proposal merged ([#1028](https://github.com/NEETROF/cymbra/pull/1028)) |
| 4 | 52 | `refine-lingua-es-it-glosses` | 2–4 | 49b | Waits for change 51's committed es-it tables: its defects are measured on them |
| 4 | 53 | `measure-lingua-es-it-translation-marks`: the 100 Spanish selections already committed, through es-en then en-it, two agent judges as changes 38 and 49, no trial before the criteria; modifies the requirement as change 49 leaves it, and archives after it; the gloss columns stay empty if changes 51 and 52 are not committed, change 52 then rerunning the experiment | 1–2 | 26 | Proposal merged ([#1030](https://github.com/NEETROF/cymbra/pull/1030)) |
| 4 | 54 | `enable-lingua-spanish-for-italian-speakers` (es-it), with its listings; rewrites `_locales/it`'s description and the activation page's Italian copy to name Spanish (« Offline e » giving way with three languages); es-it appended at the end; follows change 50 step for step and copies its blocks; changes 30, 31 and 50 are hard prerequisites, so the archive order cannot cycle | 4.5–9 | 52, 53 | Proposal merged ([#1032](https://github.com/NEETROF/cymbra/pull/1032)) |
| 5 Wording | 55 | `refine-lingua-italian-wording`: the corrections dogfood brings, in every language | 1.5–3 | 54 | Waits for the dogfood passes' findings |
| 5 | 56 | `amend-lingua-youtube-captions-italian`: an in-place amendment of the two open YouTube changes (`add-lingua-youtube-captions`, `add-lingua-youtube-caption-translation`), as the matrix's #904 and the Spanish programme's #674 did — Italian captions kept out (Scope), the mode offered to an Italian speaker who studies English, YouTube's translation into Italian, Italian copy and privacy annex where they exist; if YouTube ships before change 45, change 45's dogfood checks the Italian caption mode, and change 46's Italian description names it; the main spec's `lingua-privacy` three-language sentence is unassigned, left to the next change that touches it | 0.5–1 | 57 | Done ([#1031](https://github.com/NEETROF/cymbra/pull/1031)) |

**Size.** 57 changes, 145.25–261 ideal days: stage 0 7.75–12.75, stage 1 50.5–89, stage 2 26.5–48,
stage 3 35–62, stage 4 23.5–45.25, stage 5 2–4. Stage 0 and 1 together — the split, then Italian
for today's French-speaking readers — are 58–102. The matrix programme's 57 changes were estimated at 175–311; its French stage
merged in two calendar days of bursts. The calendar here is set the same way: by reviews, dogfood,
the model host's deploys and the store submissions, not by the volume of code.

## Rules every change of the programme follows

- **The six pairs of the matrix do not move, nor the French, English and Spanish interfaces.**
  Their goldens run on every pull request that can move them; a re-bless is the owner's approval,
  in that pull request.
- **Variants are appended** (Architecture); a backup of a state without Italian keeps its version.
- **Server first is already met**: the server stores any language label, so no deploy gates a
  client. The model host is the gate instead: no release names it-en or en-it before the owner has
  redeployed it (change 2).
- **OpenSpec: never MODIFY a requirement an open change holds.** Use ADDED or `archiveAfter`. A
  requirement quoting Italian copy quotes the French interface's words and says Italian carries the
  same message in its own words (the matrix's change 13 umbrella rule).
- Never stack a pull request on another without retargeting it.
- **Italian wording lands as drafts in its pull request and is reviewed by the dedicated agent
  before merge (I2)**; cited conventions: Treccani and the Accademia della Crusca for grammar
  terms, Cymbra Music's Italian strings (`app_it.arb`) for product terms.
- App Store listings, in every locale, say nothing of betas or prices (guidelines 2.2 and 2.3.7).
- Nothing is released, deployed or published without the owner's go-ahead (I12).

## Main risks

1. **Thin Italian-glossed pairs** (es-it, it-es) read as a lesser product. Mitigated by I4: a floor
   before measuring, coverage published, the pair held back if it falls under.
2. **No licence-clean large Italian treebank.** The French rank corrections leaned on GSD; Italian
   has MarkIT only. Mitigated by Morph-it! as a second inflection source (verified in change 9) and
   by the committed measurement on PUD.
3. **Catalan and Galician read as Italian today.** Mitigated by change 8 merging before any Italian
   pack is enabled.
4. **The web's Italian without its accents** (E', perche') is commoner than French's. Mitigated by
   change 10 before enabling it-fr.
5. **Italian wording no human reviews** (I2). Mitigated by the agent's pass, the goldens per pair
   that bound it, and the owner's dogfood where he can.
6. **Owner workload**: three dashboards × an Italian locale, screenshots, legal pages, five
   targets, a model host deploy. Mitigated by the stages: Italian for French speakers first costs
   him no Italian text at all.
7. **The split touches every loader and tool that names `<pair>.lingua`, and change 4 records
   two files per pair in every `pin.json`.** Mitigated by the six goldens and S0 running unchanged
   on changes 3 and 4, by a whole pack still loading and every whole-pack reader refusing a part
   (the agent plugin, the fixtures), and by the two changes landing before any Italian pack.
8. kaikki's per-language extracts are deprecated (the matrix's risk 6); the raw-dump migration
   (matrix change 38) already covers new pairs.
