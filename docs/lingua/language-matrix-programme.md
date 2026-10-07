# Cymbra Lingua — the language matrix programme (en · fr · es)

Every language of {English, French, Spanish} studied from each of the other two, and the interface
in the reader's own language. This file holds what the changes of the programme share: the product
owner's decisions, the architecture they build on, their order, and their status. Each change still
goes through OpenSpec; its proposal points here instead of re-arguing a decision.

- Template: [spanish-programme.md](spanish-programme.md). Its rules carry over unless this file
  says otherwise.
- Study: 2026-10-06, read-only. Nine dimensions, each mapped then challenged by an adversarial
  verifier, the synthesis reviewed again. Figures were measured on the real data (kaikki,
  wordfreq, UD treebanks, Mozilla's model registry, the pinned Bergamot engine, the 1.7.0
  package). They are orders of magnitude, not targets.
- Decisions: Guillaume Fortin (product owner), 2026-10-07.
- Status: as of 2026-10-07 (stage 0). Update the programme table in the pull request that moves a line.

## Scope

Six pairs, named `<studied>-<native>` as packs already are:

| Pair | Audience | State |
|---|---|---|
| en-fr | French speakers | shipped |
| es-fr | French speakers | shipped |
| es-en | English speakers | stage 2 |
| en-es | Spanish speakers | stage 2 |
| fr-en | English speakers | stage 3 |
| fr-es | Spanish speakers | stage 3 |

The interface (extension, Safari host app, agent plugin) speaks the reader's native language: fr,
en or es. Its listings, the site's Lingua pages and the account pages follow.

Out of scope: YouTube captions (an amendment only, as D7), remote translation (parked), a fourth
language (which would make a split pack format worth it, see Architecture).

## The product owner's decisions

The study numbered its questions M1–M25. Settled ones are binding; open ones carry the study's
recommendation and are settled before the stage named.

| # | Decision | Status |
|---|---|---|
| M1 | **Order: es-en and en-es first, French as a studied language after.** The two new audiences are reached without the French analyser on the critical path. es-en may ship before en-es: it needs no Spanish site pages and does not load the en-es model. | Settled |
| M2 | **The interface language is the native language: one choice.** It is stored under its own key, so a separate override can be added later without a migration. | Settled |
| M3 | **The native language stays on the device** (profile and backup), like D9. A new device asks for it at onboarding, preset from the browser's language. A full reset keeps it. | Settled |
| M4 | **Cards record the language of their gloss, server first.** Every stored gloss is French today, so a default of `fr` labels existing rows exactly; the label is in production before any client that can create a non-French gloss syncs. Review shows the current pack's gloss when the card's gloss is in another language (words and table expressions); a free translation keeps its text. | Settled |
| M5 | **D4, restated:** a gloss is in the reader's native language and written by a person — never the studied language, never a third language, never machine-translated or pivoted. Words people wrote into Wiktionary translation tables qualify. | Settled |
| M6 | fr-es ships with its coverage published (es-fr precedent). A floor, and what happens below it, are fixed before the committed measurement. | Open, before stage 3 |
| M7 | French levels are estimated, as D1 (FLELex is CC BY-NC-SA 4.0); a licence request goes with ELELex's. | Open, before stage 3 |
| M8 | Lemma alternatives for noun/verb homographs (*porte*/*porter*, *cuenta*/*contar*), in every language — or keep « one form, one lemma ». | Open, before stage 3 |
| M9 | **The owner reviews the English and the Spanish wording** (interface, card, listings). Drafts land in each pull request. | Settled |
| M10 | **Spanish: tú, neutral, no vosotros. English: US.** Terms: « forma en -ing », RAE tense names for French forms, "past historic (passé simple)", RAE numbers (20 000; 96 %). | Settled |
| M11 | Spanish site pages: at least privacy, account deletion, support and `/es/lingua`. | Open, before stage 2 (en-es) |
| M12 | **Account e-mails and the deletion link follow the interface language for fr, en and es; another language Cymbra speaks (it) keeps the browser's**, so the shared account locale Music adopts never moves from Italian to English. | Settled |
| M13 | **Fallback for any other browser language: English** (`default_locale`, the host app's development region). The English listing ships in the same submission as the first `_locales`. | Settled |
| M14 | **Synced daily statistics carry the native language, server first** (default `fr`); the back office can break usage down by pair. It is new collected data: the privacy policy (fr, en, es) and the App Store privacy answers change before the release that sends it. | Settled |
| M15 | Translation opens with each pair, everywhere at once, once the engine hardening has merged; marks per pair under D2. | Open, before stage 2 |
| M16 | App Store: fr-FR stays primary; en-US, en-GB, es-ES and es-MX are added. | Open, before stage 2 |
| M17 | The agent plugin is in scope, last, first cut. | Open, before stage 4 |
| M18 | Nothing is released, deployed or published without the owner's go-ahead. | Settled |
| M19 | **Level acronym: CEFR in English, MCER in Spanish, French unchanged.** | Settled |
| M20 | English glosses drop long explanatory parentheses (≥ 40 characters). | Open, before stage 2 |
| M21 | French tokenisation: au/aux split, du/des whole; one highlight span per elision piece; « pas » a function word; moods merged on five-reading forms. | Open, before stage 3 |
| M22 | **Existing installs stay French without being asked**; the choice is in Réglages. | Settled |
| M23 | **The French interface is extracted byte for byte.** A typography pass, if any, is separate. | Settled |
| M24 | **The studied side of the tables is kept once per studied language** (`tables/<studied>/`), the native side per pair (`tables/<pair>/`). | Settled |
| M25 | The real translation engine does not run in CI; per-route soak tests are a manual tool. | Open, before stage 2 |

## Architecture

Decided by the study; a change that departs from one of these says so in its design.

- **One native language per engine**, read from each pack's `meta.native` and validated at load.
  An engine refuses a pack of another native. The extension, the agent and the sync choose a pack
  by (studied, native); `pairFor` no longer takes the first listed pair.
- **The studied side does not depend on the native side.** Forms, frequencies, grammar and levels
  belong to the studied language and are reused byte for byte across natives. Glosses, senses and
  expressions belong to the pair. Two measured leaks are removed:
  - « a dictionary word » becomes a pack section of its own instead of « has a gloss » (it drives
    the vocabulary estimate and the Spanish names rule);
  - the tag pool is pinned, so a new native's sense tags cannot reshuffle `paradigms`.
- **Estimated ladders use English's typical vocabularies as a frozen constant**, not whatever
  English pack an engine happens to hold (an English-native reader never holds one).
- **What the server learns** (M4, M14): an optional gloss language on card ops and an optional
  native language on daily stats, both absent = `fr`, a capability flag, deployed and checked from
  outside before any client sends them. `client_id` is the lemma, so two devices of one account
  with different natives write one card row; the label travels with it.
- **Interface:** a typed catalogue `src/i18n/{fr,en,es}` (en and es typed `typeof fr`), plurals
  through `Intl.PluralRules`, slot messages for sentences built from fragments, a lint against
  French literals outside the catalogue. French keeps its current formatting byte for byte.
  `chrome.i18n` follows the browser's language and cannot follow a reader's choice, so `_locales`
  serves only the manifest and the per-locale listings, and ships with the first non-French pair.
- **Word card:** a language-neutral description of readings, then one renderer per native
  language. Only tense names are keyed by pair (`Tense=Past` is « prétérit » in en-fr, « passé
  simple » in es-fr, "preterite" in es-en, "past historic" in fr-en, « pasado simple » in en-es,
  « pretérito perfecto simple » in fr-es). Goldens lock the order of tenses.
- **Translation:** catalogue routes keyed by pair. Mozilla publishes no fr↔es model: en-es 2.1 is
  pinned in stage 2, fr-en 2.0 in stage 3, fr-es pivots through English. A reader needs at most two
  models (≈ 322 MiB, as es-fr). The engine worker respawns after a WebAssembly trap and evicts
  models no current pair needs; measured: en-es poisons the engine on some inputs, and models of a
  previous language linger to 463 MiB.
- **Packs stay embedded, one package per store.** Six packs are ≈ 11–15 MB raw; the extension
  package would be ≈ 11–13.5 MB zipped (5.8 MB in 1.7.0). A split format (a core per studied
  language plus a layer per native) is worth it only from a fourth language.
- **French as a studied language** gets its own analyser version, elision as pieces with their own
  spans, and backup schema v3, written only when French is studied, so released builds refuse such
  a backup as unsupported rather than malformed.

## Measured figures (study, 2026-10-06)

Gloss coverage is the share of the 5,000 / 10,000 / 20,000 commonest lemmas that carry a gloss,
the metric of `scripts/lingua-data/gloss_coverage.py`. Marks are right marks over shown marks on
100 PUD selections, one judge (the study agent, ± 6 %): indicative until each pair's committed
measurement.

| Pair | Glosses | Source | Translation | Marks |
|---|---|---|---|---|
| en-fr | 95.1 / 90.1 / 78.9 % | French Wiktionary | en-fr | 96/97 |
| es-fr | 87.6 / 77.2 / 63.7 % | French Wiktionary + fallbacks | es-en → en-fr | 89/90 |
| es-en | 93.6 / 87.0 / 77.1 % | English Wiktionary, Spanish section | es-en (pinned already) | 94/96 |
| en-es | 93.4 / 85.2 / 71.9 % | Spanish Wiktionary + English Wiktionary translation tables | en-es 2.1 | 96/96 |
| fr-en | 93.9 / 87.1 / 76.4 % (prototype ranks, ± 1–2) | English Wiktionary, French section | fr-en 2.0 | 94/96 |
| fr-es | 83.4 / 70.8 / 56.5 %, 24 % of them definitions | Spanish Wiktionary + French Wiktionary translations | fr-en → en-es | 90/91 |

French as a studied language: the English Wiktionary's French section has 403,269 entries and
≈ 7,380 fully tagged verbs, and a gender on 99.5 % of nouns. A prototype reducer scores 99.04 %
resolved, 95.93 % content lemmas and 99.90 % AUX on UD French-PUD, past Spanish's gates (98.5 /
93.5 / 97). PUD does not exercise tokenisation (UD splits elision already), while elided words are
6.5 % of literary French: the French analyser needs raw-text fixtures.

The interface holds ≈ 453 unique texts (≈ 2,490 words) of French copy in 22 TypeScript files and
6 HTML pages, with no i18n layer.

## Programme

In merge order within a stage (1 change = 1 pull request, none above ≈ 10 ideal days). Effort is
in ideal days, min–max, after the verifiers' corrections; it measures size, not calendar.

| Stage | # | Change | Effort | Status |
|---|---|---|---|---|
| 0 Prerequisites | 1 | Re-bless the English baseline after #726 | 0.25–0.5 | Done ([#727](https://github.com/NEETROF/cymbra/pull/727)) |
| 0 | 2 | Submit to the Chrome Web Store through API V2 (V1.1 stops on 2026-10-15) | 1–2 | Done ([#728](https://github.com/NEETROF/cymbra/pull/728)); the owner sets `CWS_PUBLISHER_ID` before the next store dispatch |
| 0 | 3 | `add-lingua-spanish-baseline`: an es-fr invariance golden beside S0 | 1–2 | Done: `crates/lingua-wasm/tests/spanish_baseline.rs`, 133 probes over a 13-page Spanish corpus with es-fr beside en-fr; one harness (`tests/support`) for both baselines, the English golden unchanged |
| 1 Native-language platform (silent release) | 4 | `generalise-lingua-native-language` | 6–10 | Done (proposal [#732](https://github.com/NEETROF/cymbra/pull/732)) |
| 1 | 5 | `add-lingua-pack-lexical-layer` | 4–7 | Done (proposal [#735](https://github.com/NEETROF/cymbra/pull/735)) |
| 1 | 6 | `generalise-lingua-gloss-reducer`: the fr, en and es Wiktionary editions' rules at once | 4–7 | Done (proposal [#737](https://github.com/NEETROF/cymbra/pull/737)); en-fr and es-fr re-pinned, tables byte-identical |
| 1 | 7 | `split-lingua-pack-tables-by-language` (M24) | 4–7 | Done (proposal [#739](https://github.com/NEETROF/cymbra/pull/739)); tables/en and tables/es, nothing moved |
| 1 | 8 | `generalise-lingua-translation-routes-by-pair` | 5–8.5 | Not started |
| 1 | 9 | `harden-lingua-translation-engine` | 2–3.5 | Not started |
| 1 | 10 | `add-lingua-native-language-server` (M4, M14), server first | 4–6.5 | Done (proposal [#748](https://github.com/NEETROF/cymbra/pull/748)); the owner deploys it and checks it from outside before change 12 |
| 1 | 11 | `add-lingua-card-gloss-language` | 3–5.5 | Not started |
| 1 | 12 | `add-lingua-native-language-sync-client` | 3.5–6 | Not started |
| 1 | 13 | `add-lingua-interface-language`: the catalogue, no visible change | 5–8 | Not started |
| 1 | 14 | `localise-lingua-reading-surfaces` | 3.5–6 | Not started |
| 1 | 15 | `localise-lingua-settings` | 4–7 | Not started |
| 1 | 16 | `localise-lingua-review-stats` | 3.5–6 | Not started |
| 1 | 17 | `localise-lingua-account-onboarding` (M12) | 3.5–5.5 | Not started |
| 1 | 18 | `generalise-lingua-card-wording` | 2.5–4.5 | Not started |
| 1 | 19 | `add-lingua-native-language-labels` (M19) | 2–3.5 | Not started |
| 1 | 20 | `add-lingua-native-language-choice`: hidden while only French-native pairs ship | 4–7 | Not started |
| 2 New audiences: es-en, en-es | 21 | `add-lingua-pack-es-en` | 3.5–6 | Not started |
| 2 | 22 | `add-lingua-pack-en-es` | 5–8 | Not started |
| 2 | 23 | `add-lingua-english-card-wording` | 3–5 | Not started |
| 2 | 24 | `add-lingua-spanish-card-wording` | 3–5.5 | Not started |
| 2 | 25 | `add-lingua-translation-matrix-models`: en-es 2.1; the owner redeploys the model host right after | 1–2 | Not started |
| 2 | 26 | `measure-lingua-translation-matrix-marks`: es-en, en-es | 2.5–4 | Not started |
| 2 | 27 | `localise-lingua-manifest`: `_locales`, with the first non-French pair | 1–2 | Not started |
| 2 | 28 | `localise-lingua-apple-host` | 3.5–6 | Not started |
| 2 | 29 | `add-site-spanish-locale` (M11) | 4.5–8 | Not started |
| 2 | 30 | `add-site-lingua-matrix-pages` | 1.5–3 | Not started |
| 2 | 31 | `update-lingua-privacy-annex-languages` | 0.5–1.5 | Not started |
| 2 | 32 | `localise-cymbra-id-email-legal-links` | 0.5–1 | Not started |
| 2 | 33 | `review-lingua-interface-translations`: the owner's corrections, dogfood in en and es | 3–5.5 | Not started |
| 2 | 34 | `enable-lingua-english-speakers` (es-en) | 4–7.5 | Not started |
| 2 | 35 | `enable-lingua-spanish-speakers` (en-es), after change 9 | 4–7.5 | Not started |
| 2 | 36 | `add-lingua-english-listings` | 3–5.5 | Not started |
| 2 | 37 | `add-lingua-spanish-audience-listings` | 2–3.5 | Not started |
| 3 French studied: fr-en, fr-es | 38 | `migrate-lingua-pack-sources-to-raw-dumps` | 2.5–5 | Not started |
| 3 | 39 | `add-lingua-french-baseline`: the variant, backup v3 | 3–5 | Not started |
| 3 | 40 | `add-lingua-french-tokenisation` (M21) | 4.5–7.5 | Not started |
| 3 | 41 | `add-lingua-french-analysis` | 4.5–7 | Not started |
| 3 | 42 | `add-lingua-french-detection-guard`: Catalan and Occitan | 2–4 | Not started |
| 3 | 43 | `add-lingua-french-forms-tables` | 8.5–13.5 | Not started |
| 3 | 44 | `add-lingua-french-expression-keys` | 1–2.5 | Not started |
| 3 | 45 | `add-lingua-french-grammar-tables` | 3.5–6 | Not started |
| 3 | 46 | `add-lingua-french-levels` (M7) | 1.5–3 | Not started |
| 3 | 47 | `add-lingua-french-read-aloud` | 1.5–3 | Not started |
| 3 | 48 | `add-lingua-pack-fr-en` | 2–3.5 | Not started |
| 3 | 49 | `add-lingua-pack-fr-es` (M6) | 2.5–5 | Not started |
| 3 | 50 | `add-lingua-french-translation`: fr-en 2.0, marks for fr-en and fr-es | 1.5–3 | Not started |
| 3 | 51 | `add-lingua-french-word-card` | 3.5–6 | Not started |
| 3 | 52 | `enable-lingua-french` | 4–7.5 | Not started |
| 3 | 53 | Listings and site: French | 1.5–3 | Not started |
| 4 Agent and wording | 54 | `refine-lingua-language-wording` (Spanish change 33, carried) | 1.5–3 | Not started; waits on archives |
| 4 | 55 | `add-lingua-agent-native-language` (M17) | 2–4 | Not started |
| 4 | 56 | `refine-lingua-matrix-wording` | 0.5–4 | Not started |
| 4 | 57 | `amend-lingua-youtube-captions-matrix` | 0.5–1 | Not started |

Optional, outside the counts: `guard-lingua-translation-source-language` (1–2),
`add-admin-lingua-pair-usage` (3–5.5), `add-lingua-new-device-language-offer` (1.5–3),
`add-lingua-asc-localisations-upload` (2–4), `add-lingua-lemma-alternatives` (4–10, M8),
`add-lingua-curated-word-glosses` (1–2), `add-lingua-gloss-triangulation` (2–3.5, only under a
relaxed D4). A dogfood reserve of 12–20 days follows the Spanish experience (about twelve
unplanned changes after its enable).

**Size.** 57 changes, 175–311 ideal days; stages 0–2 alone are 111–194.5. The Spanish programme's
32 delivered changes were estimated at 124.5–218 and merged in about six calendar days, in bursts
paced by the owner's sessions. The calendar here is set the same way: by reviews, dogfood, deploys
and submissions, not by the volume of code.

## Rules every change of the programme follows

- **en-fr and es-fr output does not move**, nor the French interface. S0 and the es-fr golden
  (change 3) run on every pull request that can move them. Their bytes move once, in change 6
  (the rule digest moves `pack_version` and the pinned sha256), re-blessed with the owner's
  approval in that pull request.
- **Server first.** No build that can create a non-French gloss or send a native language syncs
  to production before the server that stores them is deployed and checked from outside.
- **`refine-lingua-review-session` (#696) merges before changes 13, 15, 16 and 18**, or is rebased
  onto the catalogue; its requirements name answers by meaning before it merges.
- **OpenSpec: never MODIFY a requirement an open change holds.** Use ADDED or `archiveAfter`.
  Change 13 ADDS an umbrella rule: copy quoted in a Lingua requirement is the French interface's,
  and every interface language carries the same message in its own words.
- Never stack a pull request on another without retargeting it.
- App Store listings, in every locale, say nothing of betas or prices (App Store Review
  Guidelines 2.2 and 2.3.7).

## Main risks

1. A generalisation moves en-fr, es-fr or the French interface. Mitigated by S0, the es-fr golden,
   the French literal inventory and silent releases. S0 does not run on a tables-only pull request
   except in `lingua-extension-check`'s `check`, which is not a required check (#726).
2. A non-French gloss or a native language reaches the server before it can label them. Mitigated
   by server first, a capability flag the client waits for, and test accounts for dogfood.
3. The en-es model traps the engine. Mitigated by change 9 before change 35.
4. English and Spanish wording that does not read naturally. Mitigated by the owner's review,
   goldens per pair that bound it, and cited conventions (English Wiktionary form-of wording,
   RAE/ASALE terms, Cymbra Music's Spanish terminology).
5. Thin Spanish glosses (en-es, fr-es) read as a lesser product. Mitigated by published,
   like-for-like coverage and a floor fixed before measuring (M6).
6. kaikki removes its per-language files (marked deprecated, still served on 2026-10-06).
   Mitigated by en-es deriving from the raw English dump, a snapshot of the French section before
   stage 3, and change 38.
7. Owner workload: three dashboards × two new locales, screenshots, legal pages, dogfood on five
   targets × two interfaces. Mitigated by two waves (English speakers with es-en, Spanish speakers
   with en-es), copy kept in the repository, AMO listings pushed from it.
