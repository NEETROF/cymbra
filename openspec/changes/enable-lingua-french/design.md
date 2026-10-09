# Design — enable-lingua-french

## Context

See proposal.md (Why). What the list governs, and what each French change left for this one:

| Where | Today (`main` at `aabe17b3`, changes 34 and 35 as drafted in #810 and #814) |
|---|---|
| `packs.json`, `tool/check_variants.mjs` | `["en-fr", "es-fr"]` on `main`; `["en-fr", "es-fr", "es-en", "en-es"]` once 34 and 35 merge; `SHIPPED_PAIRS` equal to it, every package carrying exactly the listed packs |
| `tool/gen_pack.sh` | `--real` builds each listed pair from `tables/<studied>/` + `tables/<pair>/` against its pin; `yarn gen:pack` from `scripts/lingua-data/testdata/<pair>/`, refusing a listed pair without one — `testdata/fr-en/` exists (change 39), `testdata/fr-es/` does not, and change 49 adds none; `lingua-extension-check` and `lingua-apple-build` run `yarn gen:pack` |
| `src/analyzer/pairs.ts` | `pairsOf`, `defaultPair` (the first listed pair of a native), `readerPairs`, `shippedNatives`; `studied-languages-view.ts` offers `shippedLanguages(pairsOf(native))`, hidden under two |
| `src/analyzer/types.ts` | `StudiedLanguage = "en" \| "es"` — change 39's D8 and change 47's D4 leave its widening here, change 51's D1 widens the renderers' `StudiedLanguageCode` and leaves this one |
| `src/state/profile.ts` | `NAMES = { English, Spanish }`: a stored profile's `French` is dropped, English when none is known (change 39's D8: « nothing writes `French` before change 52 ») |
| `src/analyzer/language-labels.ts` | keyed by `NamedLanguage` (`en`, `es`, `fr`, change 47's D4): every label already has French's words in the three catalogues; the level messages take a `StudiedLanguage` and « gain French with it in change 52 » |
| Backup | schema 3 written only when the state names French (change 39's D7); read by every build since #803 (`9fde7bb1`, change 39's implementation), carried by no release yet |
| Model catalogue | four models, six routes, `fr-en` direct and `fr-es` through English (change 50, #837); `MARKED_PAIRS` lists both, « inert until change 52 »; `model-controller.ts` flags `pivot` when a needed route has two models, and the setting then says « environ 340 Mo », else « environ 200 Mo » (change 50's D3 hands the English-native case here) |
| `_locales/{fr,en,es}` | descriptions naming what each native can study — `en` « Read Spanish on the web… », `es` « Lee inglés en la web… »; change 27's D1: « it changes when a pair of that native is added (fr-en, fr-es) » |
| Safari host app | `apps/lingua-apple/tool/app_localizations.sh` declares the natives of the bundled packs (change 28); the activation page, `Shared (App)/Resources/copy.js` (French in place in `Base.lproj/Main.html`, held equal by `apps/lingua-extension/test/apple-activation-page.spec.ts`), names each native language's default studied language alone — « Lis le web en anglais », "Read the web in Spanish", « Lee la web en inglés » — in `lede` and `step3` |
| `gloss_coverage.py` | `--write` publishes the pairs of `packs.json`; `FLOORS` holds es-en and en-es, and fr-en (48) and fr-es (49) once they land; `test_gloss_coverage.py` holds the file to the list |
| `apps/site` | the Lingua pages built from `lingua-coverage.json` and the catalogue's routes (change 30); the Spanish home's Lingua card from the same list (change 29b, `spanishHomeLinguaCard`, #845) |
| Owner checks left here | change 40: whether two adjacent highlights need a hairline; change 44: an expression card on real pages; change 47's 6.2: the read-aloud checklist; change 50: the worker's memory on devices, fr-es on the iPhone; change 51's 6.2: the selection gestures |

## Goals / Non-Goals

**Goals:**
- fr-en in every package, and fr-es when its committed tables stand at or above its floor, shown to
  English and Spanish speakers; nothing moved for a French-native reader but the host app's
  activation page, by the owner's decision (D7).
- The extension's own French: the type, the stored profile, the offer, the labels, the memory
  sentence; measured on the real list before it ships.

**Non-Goals:**
- The listings, French's sentence on estimated levels on the site, « CECR » or « CEFR », the back
  office's name for `fr` (change 53); the agent plugin (M17).
- Any table, pack, rule, golden or analyser version; any card wording (51); the glosses' defects
  (`refine-lingua-fr-en-glosses` and `refine-lingua-fr-es-glosses`, each merged before this change
  lists its pair).
- A release: the owner's.

## Measured

On a scratch worktree (never committed): `origin/main` at `4222a013` with change 35's draft
implementation (`claude/enable-lingua-spanish-speakers-impl`, which holds 34's) merged locally,
conflicts resolved by keeping both sides; Node 22.22.2, the pinned engine, Rust stable, a MacBook
Pro (Apple M2 Max), 2026-10-09.

**The edits alone.** `packs.json` and `SHIPPED_PAIRS` → six pairs, `StudiedLanguage` →
`"en" | "es" | "fr"`, `NAMES` → `French: "fr"`:
- `tsc --noEmit`: one error, `wordpopup.ts:410`, the card handing its `StudiedLanguage` to a renderer
  typed `StudiedLanguageCode` (`en | es`) — change 51's D1 widens that code, after which the type
  check is clean. No `Record<StudiedLanguage, …>` table lacks French.
- `yarn vitest run`: 6 of 3,148 tests fail, in 5 files, each asserting the list or a French pair's
  absence — `manifests.spec.ts` (the list, `WITH_BOTH`), `packs.spec.ts`, `pairs.spec.ts` (French's
  two pairs, English's one, Spanish's one), `model-manifest.spec.ts` twice (change 50's « no
  reader's pairs need fr-en or fr-es », and change 35's « en-es is needed by no one else » — a
  Spanish-native reader of French needs it too, fr-es's second model), `model-controller.spec.ts`
  (change 50's « nothing of fr-en is downloaded », which already fails where change 35's list meets
  change 50's tests — 35's rebase repairs that). Every snapshot (`word-card-*.txt`,
  `selection-rows-fr.txt`, `voice-ranking.txt`), every lint and every surface spec passes.
- `cargo test -p lingua-pack --test committed_tables`: 1 of 15 fails, change 43's
  `spec_scenario_the_pack_builds_where_the_others_do`, « no package lists fr-en ». Re-run on `main`
  at `aabe17b3` (changes 42 and 46 implemented since; no file of `apps/lingua-extension` moved) with
  the six pairs listed: 1 of 20, the same test. Once change 49 is implemented, its « listed nowhere »
  assertion is a second (D11).
- `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline
  --test en_es_baseline --test french_baseline --test cross_native`: all pass, nothing re-blessed.
- `yarn check:variants`: « Variant check passed: chromium, firefox, safari ». `yarn check:version`:
  publishable, « in en, es, fr ».

**The labels French already has**, rendered through `language-labels.ts` with `fr` studied:

| | English interface | Spanish interface |
|---|---|---|
| name | French | Francés |
| level title, estimated | Estimated French level | Nivel de francés estimado |
| the note | Levels estimated from word frequency, as no freely licensed CEFR list exists for French. | Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el francés. |
| the ladder's borrowed sizes | taken from English, whose level sizes French borrows. | tomado del inglés, cuyos tamaños de nivel retoma el francés. |
| the prompt | Choose your French level | Elige tu nivel de francés |
| no text | No French text found on this page. | No se detectó texto en francés en esta página. |
| a level | B1 (estimated) | B1 (estimado) |

**The packages.** Built with `yarn gen:pack:real` for the four committed pairs and two stand-ins for
fr-en and fr-es (main's `tables/fr/` with the grammar of change 45's and the levels of change 46's
implementation branches, an empty native side: 1,459,990 B each), zipped as
`lingua-extension-release` zips them:

| | chromium.zip |
|---|---|
| change 35's list (four packs) | 9,077,438 B |
| six packs, stand-ins | 10,763,924 B |
| fr-en's native side, 959,616 B (change 48: 2,201,349 B with its glosses against 1,241,733 B without, on change 43's tables), deflating as es-en's native side does (990,688 → 932,754 B, 0.94) | + ≈ 0.90 MB |
| fr-es's native side, 451,649 B (change 49: 1,693,382 B against 1,241,733 B), deflating as en-es's (750,470 → 695,193 B, 0.93) | + ≈ 0.42 MB |
| **six pairs** | **≈ 12.1 MB** (12,085,150 B estimated) |
| **fr-en alone** (fr-es under its floor) | **≈ 10.8 MB** |

The programme estimated ≈ 11–13.5 MB zipped (5.8 MB in 1.7.0); six packs are ≈ 12.6 MB raw
(12,613,806 B), against its ≈ 11–15 MB. Firefox's and Safari's zips are 15 kB smaller. A pack's
native side hardly deflates, its studied side does (French's: 0.58). Every bundle grows by 12 to
26 B (`__LINGUA_PACKS__` and the `French` name), the manifest by 74 B (two
`web_accessible_resources`); the host app's bundles by the two packs, ≈ 4.3 MB raw.

**The reader's models and the memory sentence**, every native language and every set of accepted
languages, on the committed catalogue (`readerPairs`, `modelsFor`):

| Native | Accepted | Pairs | Models | Download | Today's sentence | Counting models |
|---|---|---|---|---|---|---|
| fr | en | en-fr | en-fr | 25,752,472 B | ≈ 200 | ≈ 200 |
| fr | en, es (any order) / es | en-fr, es-fr / es-fr | en-fr, es-en | 51,993,524 B | ≈ 340 | ≈ 340 |
| en | es | es-en | es-en | 26,241,052 B | ≈ 200 | ≈ 200 |
| en | fr | fr-en | fr-en | 26,234,715 B | ≈ 200 | ≈ 200 |
| **en** | **es, fr (any order)** | **es-en, fr-en** | **es-en, fr-en** | **52,475,767 B** | **≈ 200** | **≈ 340** |
| es | en | en-es | en-es | 25,373,354 B | ≈ 200 | ≈ 200 |
| es | fr / en, fr (any order) | fr-es / en-es, fr-es | en-es, fr-en | 51,608,069 B | ≈ 340 | ≈ 340 |

The two rules differ for one reader, the English-native reader of Spanish and French, the case
change 50's D3 names; every reader of the four pairs before this change reads the same sentence by
either rule. With fr-es under its floor, the Spanish rows but `en` disappear and nothing else moves.

**The site.** `lingua-coverage.json` written with change 48's and 49's prototype figures (fr-en
93.6 / 86.9 / 76.3, fr-es 83.2 / 70.8 / 56.8): `yarn test` 72 / 72, `yarn build`, `yarn
check:routes` 21 passed. `/en/lingua/` reads « A browser extension that highlights the Spanish or
French words you do not know yet », « Made for English speakers learning Spanish or French. », « Also
for Spanish speakers learning English or French… », the routes « French to English, … French to
Spanish (through English) »; `/lingua/` and `/es/lingua/` alike, in their language. Its level card
still says « Pour l'espagnol, les niveaux sont estimés… » and nothing of French: change 53's. The
Spanish home's Lingua card (change 29b, merged since as #845) reads the same list: by
`spanishHomeLinguaCard`'s rule — the languages of the pairs glossed in Spanish, in the list's order —
« Lee la web en inglés o en francés… » once fr-es is listed; its tests stand in their own pairs.

**The host app.** `apps/lingua-apple/tool/app_localizations.sh` over the six built packs: « now
declares ["fr","en","es"] with CFBundleDevelopmentRegion en (shipped natives: fr en es) » — change
35's.

**The model host.** `node tool/check_model_host.mjs` on 2026-10-09: « https://models.cymbra.app/
serves en-fr/base-memory/2.0, es-en/base-memory/2.0, en-es/base-memory/2.1, fr-en/base-memory/2.0 as
the extension expects ».

**The releases and backup v3.** `git tag --contains 9fde7bb1` (#803, backup v3 read and written)
names no `lingua-extension-v*` and no `lingua-apple-v*` tag: the store builds (1.7.0, the host app's
1.5.0, still the newest tags at `aabe17b3`) read versions 1 and 2 only, and refuse 3 by name
(« unsupported version 3 »). `fix-lingua-lemma-lookup` (41b) moves no stored format (its design,
*Migration*): a release reads 3 because it is cut after #803, whatever change it carries.

**The server.** `backend/lingua/src/language_core.rs` normalises a studied language and refuses
none (« The server decides nothing about which languages exist »): `fr` is stored on statuses,
levels, cards and days as `es` is.

## Decisions

### D1 — The list and the gate, French last

`packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"]` (D2 for fr-es) and
`SHIPPED_PAIRS` the same; the requirement's list follows. The French pairs go last: a native
language's default pair is its first listed one, so French natives keep en-fr, English natives
es-en, Spanish natives en-es — an installed reader's default, the onboarding's preset and change
36's and 37's listings stay true. French is then offered beside the default, in Réglages and at
onboarding, the reader ticking it (*The reader chooses the languages they study*).

`yarn gen:pack:real` builds both from their committed tables against their pins; `yarn gen:pack`
builds fr-en from change 39's fixture and fr-es from D10's. The tests that read the default list
take their list explicitly where they mean an earlier one, as changes 34 and 35 did (*Measured*: 6
vitest tests and 1 Rust test).

*Rejected — fr-en before es-en.* New English-native readers would default to French; the
onboarding, change 36's listings and every English-native reader's fallback would move, for an
order the reader sets with one box anyway.

### D2 — fr-es by its floor (M6)

Change 49 commits fr-es's tables only at or above 81.4 / 68.8 / 54.5 % (its D8), and fails any
later reduction under it. So this change is implemented once change 49's first committed measurement
is known, whatever it says:
- **At or above** (49 merged): fr-es is listed after fr-en once `refine-lingua-fr-es-glosses` has
  merged — settled by the owner on 2026-10-09 (Open Question 4): fr-es's known defects (change 51's
  *Known data defects*) are fixed before it ships, by a refinement of its own proposed after change
  49's implementation, as 24b was before 35, its reduce job holding the floor —; its figures are
  published, `_locales/es` and the activation page name French (D7), and Spanish-native readers are
  offered French. `refine-lingua-fr-es-glosses` joins `archiveAfter` once proposed, as
  `refine-lingua-fr-en-glosses` (change 48's D7, before fr-en ships) does.
- **Under** (49's pull request records the figures and does not merge): the list ends with fr-en;
  Spanish-native readers are offered English alone, as since change 35; `_locales/es`, the site's
  Spanish readers' line and change 53's Spanish listing name no French. A later change lists fr-es
  once a regeneration of the dumps measures at or above the same floor: one list entry, its figures
  and `_locales/es` — this design's D2, D7 and D10 again.

The list cannot hold fr-es without tables: `gen_pack.sh` refuses a pair with no committed tables
(*A pair the build cannot make*). And a listed pair is never published under its floor:
`test_gloss_coverage.py` gains a test that every pair of `packs.json` with a `FLOORS` entry measures
at or above it on the committed tables — es-en (93.0 / 86.5 / 76.5 against 87.6 / 77.2 / 63.7),
en-es (93.0 / 85.0 / 71.7 against 91.4 / 83.2 / 69.9), fr-en and fr-es pass it; a regression that
slipped past the reduce job fails it.

### D3 — The extension's French: one type, one name, no new surface

`StudiedLanguage` becomes `"en" | "es" | "fr"` and `profile.ts`'s `NAMES` gains `French: "fr"`.
Nothing else is written for French to be read: *Measured* shows the type check clean once change 51
has landed, no table keyed by `StudiedLanguage` lacking French, and every label already French's in
English and in Spanish. What follows from the two edits:
- **The offer.** `mountStudiedLanguages` offers `shippedLanguages(pairsOf(native))`: Spanish and
  French to an English speaker, English and French to a Spanish speaker (English alone without
  fr-es), English and Spanish to a French speaker — never French, which no French-native pair
  studies.
- **The stored profile.** A backup naming `French` is read as French by the surfaces that read it
  without an engine (the background's translation needs, the native-language choice), instead of
  being dropped.
- **The card.** `content.language` says `fr` for a French page, so the studied words carry
  `lang="fr"` inside the interface language's lines and the renderer names French's forms (change
  51's D1 and D11).
- **The page.** Detection among the reader's accepted languages (change 42's guard against Catalan
  and Occitan), the elided pieces each highlighted and clickable apart (change 40's D7), a selection
  inside one opening that piece (change 51's D7), the voice of France (change 47).

Tests: `profile.spec.ts` (a profile studying Spanish then French, English native, read back);
`studied-languages-view.spec.ts` (the three offers above, and Spanish speakers without fr-es);
`language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`,
`popup.spec.ts` and `stats-view.spec.ts` in the English and Spanish interfaces with French studied
(the labels of *Measured*, change 46's D6 hand-over);
`wordpopup.spec.ts` (a French card's studied words in `fr`). The extension's French interface does
not move: no French-native reader is offered French, and the French catalogue is untouched.

### D4 — Backup v3 reaches a store before French does

Once a reader studies French, their stored backup is schema 3 (change 39's D7), written only then;
an English or Spanish reader's stays 1 or 2, byte for byte. No store build reads 3 today
(*Measured*). Were this change's release withdrawn, the stores would serve the previous release
again, and a reader of French would find their extension refusing its own state (« unsupported
version 3 », the surfaces not starting) until the next one. What reads schema 3 is change 39's
implementation (#803, `9fde7bb1`, on `main` since 2026-10-08); 41b reads and writes no stored
format. So the first release cut after #803 reads it — change 34's, 35's or 41b's, whichever reaches
the stores first; 41b's (its task 4.3, before French ships) is the latest it can be — and that
release is live on the Chrome Web Store, addons.mozilla.org and, in the host app, the App Store
before this change's release. A rollback of this change's release then lands on a build that keeps
the reader's French records: its `profile.ts` drops `French` from what the surfaces read,
`readingLanguage` keeps a reader of French alone in their native language's default, the core keeps
the records in the backup, and this change's next release finds them. The dogfood checks it once
(D12, step 10).

*Rejected — ship French in the release that first reads schema 3.* Nothing older than it could be
served back without refusing a French reader's state.

### D5 — Translation as M15 settled it: offered, marked

M15 (settled 2026-10-10): translation opens with each pair, marks per pair under change 26's D2.
Change 50 measured fr-en 95 / 96 (4 % withheld) and fr-es 90 / 91 (9 % withheld), both on the first
tier, and listed both in `MARKED_PAIRS`; its routes and its model are in the catalogue, the model
host serves them (*Measured*). Nothing in `model-manifest.json` or `markup.ts` changes but the
comments that say « until change 52 ». Change 50's tests that held French's routes inert are
rewritten for the list: an English-native reader of French downloads fr-en alone (26,234,715 B), a
Spanish-native reader fr-en then en-es (51,608,069 B), a French-native reader what they downloaded
before; *A route of a pair studying French* keeps its name and its condition (« no shipped pair
studies French ») as a test of a list without them.

### D6 — The memory sentence counts the models

`pivots(needs)` asks whether a needed route has two models; the setting then says « environ 340 Mo »
(« about 340 MB », « unos 340 MB »), else « environ 200 Mo ». It counted what a reader holds as long
as no native language had two one-model routes; English does now. The cost's flag becomes
`twoModels`, set when the models the reader's pairs need are two (`needs.needed.length > 1`), and
the setting reads it where it read `pivot`. *Measured*: one reader's sentence moves, the
English-native reader of Spanish and French (« about 200 MB » → « about 340 MB », as es-en and
fr-en both loaded hold ≈ 322 MiB in the worker, es-fr's figure); every other row reads what it
reads today. The copy's keys (`memoryPivot`, `memorySingle`) and their text in the three catalogues
stay: no French string moves.

*Rejected — keep `pivot` and count a pivot.* The sentence would under-state by 140 MB the memory of
the one reader whose two languages are both direct.

### D7 — The manifest, the host app and what does not move

The French pairs add no native language: the built manifests keep `_locales/fr`, `en` and `es`,
`default_locale` English (change 27, M13), and the host app keeps `CFBundleLocalizations`
`[fr, en, es]` (change 28) — both measured. What moves is the description each native language reads,
which names what it can study (change 27's D1):

| | Today | Draft (M9) | Characters |
|---|---|---|---|
| `en` | Read Spanish on the web: unknown words highlighted, an honest percentage. Offline and private. | Read Spanish and French on the web: unknown words highlighted, an honest percentage. Offline and private. | 105 |
| `es` (with fr-es) | Lee inglés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado. | Lee inglés y francés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado. | 109 |
| `fr` | Lisez l'anglais et l'espagnol sur le web : … (109) | unchanged — a French reader studies no French | 109 |

Both within the 112 characters `check_version` holds (Apple's limit). They are change 53's drafts
(its D6), word for word: the Chrome Web Store and AMO read their summary from them, so the owner
reads them once, as listing text (M9). Change 53 adds to `check_version` a rule that each native
language's description names every language its shipped pairs study and no other: merged before this
change, it fails this pull request until the drafts are committed; merged after, its own — either
way they land here, in the pull request that lists the pairs (53's D12).

**The host app's activation page** — settled by the owner on 2026-10-09 (Open Question 2): each
language's `lede` and `step3` in `copy.js` name every language its native language's shipped pairs
study, in every language, the French page included — English and Spanish in French, Spanish and
French in English, English and French in Spanish with fr-es, English alone without it. Step 3
chooses the first language's level, as today, then says where another is ticked — Réglages' first
tab, « Langue » ("Language", « Idioma »: `tabLanguage`), which holds the studied languages' boxes and
a level block per ticked language (`settings-view.ts`) — so that it stays true for a reader of
several languages. The French page moves by this decision, the one exception to « the French
interface byte for byte » (M23): `copy.js`'s `fr` and `Main.html`'s text in place, two keys. Every
other key of every table, `ViewController.swift` and `Script.js` stay. Drafts (M9; the page's
register, tu and tú; `<strong>` as step 1's paths have it):

| | `lede` | `step3` |
|---|---|---|
| `fr` | `Lis le web en anglais ou en espagnol : les mots que tu ne connais pas encore sont surlignés, directement dans Safari.` | `Dans Safari, ouvre l’extension depuis le <strong>menu de la barre d’adresse</strong> et choisis ton niveau d’anglais. Pour l’espagnol, coche-le dans les <strong>Réglages</strong> de l’extension, onglet <strong>Langue</strong>, puis choisis ton niveau d’espagnol.` |
| `en` | `Read the web in Spanish or French: the words you don't know yet are highlighted, right in Safari.` | `In Safari, open the extension from the <strong>address bar menu</strong> and choose your level of Spanish. For French, check it in the extension's <strong>Settings</strong>, under <strong>Language</strong>, then choose your level of French.` |
| `es`, with fr-es | `Lee la web en inglés o en francés: las palabras que todavía no conoces aparecen resaltadas, directamente en Safari.` | `En Safari, abre la extensión desde el <strong>menú de la barra de direcciones</strong> y elige tu nivel de inglés. Para el francés, márcalo en los <strong>Ajustes</strong> de la extensión, pestaña <strong>Idioma</strong>, y luego elige tu nivel de francés.` |
| `es`, without fr-es | unchanged: « Lee la web en inglés: … » | unchanged: « … y elige tu nivel de inglés. » |

« de l’extension » keeps the extension's Réglages apart from step 1's, the system's. What moves in
`apps/lingua-extension/test/apple-activation-page.spec.ts`: *names, in English and Spanish, the
language their natives study* becomes « names every language a native language's pairs study »,
`lede` and `step3` holding « anglais » and « espagnol » in French, "Spanish" and "French" in English,
« inglés » and (with fr-es) « francés » in Spanish, and no language naming its own; *is the page's
French, byte for byte* keeps passing, `copy.js` and `Main.html` moving together. Change 53's App
Store descriptions quote step 3 (`apps/lingua-apple/STORE-LISTING.md`) and follow the page. The
requirement *Guided activation* said « the French copy SHALL stay what it was » (change 28): it is
MODIFIED to name every language each native language's pairs study, archived after
`add-lingua-apple` and `localise-lingua-apple-host`, which hold it (D14).

### D8 — The published coverage, and the site

`python3 scripts/lingua-data/gloss_coverage.py --write` rewrites `lingua-coverage.json` with fr-en's
figures (and fr-es's), measured on the committed tables as every pair's are (change 48's floor
91.9 / 85.1 / 74.4, its prototype 93.6 / 86.9 / 76.3; change 49's 83.2 / 70.8 / 56.8); the tests
that held them « published nowhere » (changes 48's and 49's) read « published once listed », as
changes 34 and 35 rewrote es-en's and en-es's. The site's three pages describe the French pairs from
the file and the routes with no code change (*Measured*), and so does the Spanish home's Lingua card
with fr-es (change 29b); its level card, French's « estimés » and « CECR »/« CEFR » are change 53's
(change 46's D11).

### D9 — What a French-native reader sees: nothing but the activation page

| | French native | English native | Spanish native |
|---|---|---|---|
| Pairs | en-fr, es-fr | es-en, fr-en | en-es, fr-es (en-es alone under the floor) |
| Default | English | Spanish | English |
| Offered in « Langues étudiées » | English, Spanish | Spanish, French | English, French (hidden under the floor) |
| Packs ever loaded | en-fr, es-fr | es-en, fr-en | en-es, fr-es |
| Translation downloads | as before | es-en, fr-en | en-es, fr-en |
| `_locales` read | `fr`, unchanged | `en`, names French | `es`, names French with fr-es |
| Backup | 1 or 2, unchanged | 3 once French is studied | 3 once French is studied |
| Host app's activation page | English and Spanish (D7) | Spanish and French | English and French with fr-es |

A French-native reader's package carries two packs more, never fetched (≈ 3.0 MB zipped with both,
≈ 1.75 MB with fr-en alone),
and every request, sentence, card and statistic is what it was. A French page is not read for them,
as today. On an account shared with an English-native device that studies French, their device
never pulls a French card: it pulls only the languages it accepts.

### D10 — A fixture for fr-es

`scripts/lingua-data/testdata/fr-es/`, hand-written for the tests as `testdata/fr-en/` is (change
39's D5) and in its layout: the same French forms, ranks and levels, the glosses in Spanish for its
glossed lemmas, `mwe.tsv` likewise, `manifest.json` studying `fr`, glossed in `es`, French's current
analyser version, a pack version that says « fixture », `NOTICE` saying the tables are hand-written.
`yarn gen:pack` and the checks need it the moment fr-es is listed (`gen_pack.sh`'s refusal); no
golden reads it. Not written when fr-es is not listed.

### D11 — Tests and documents

The tests that read the list or held a French pair inert (*Measured*) are rewritten for the list,
each keeping the earlier lists it named as explicit arguments; new ones are D3's, D5's and D6's
(`translation-setting.spec.ts` and `model-controller.spec.ts`: the English-native reader of Spanish
and French told « about 340 MB » for 52,475,767 B; the Spanish-native reader of French, 51,608,069 B
and « unos 340 MB »; the French-native rows unchanged), and D7's in `apple-activation-page.spec.ts`.
`committed_tables.rs`: change 43's *The pack builds where the others' do* asserts that `packs.json`
lists fr-en (D14); with fr-es, change 49's « listed nowhere » assertion goes, its scenario holding
only while the list does not name fr-es.
`test_gloss_coverage.py`: D2's floor test and D8's publication.

Documents: `README.md` and `REVIEWERS.md` (what ships, what a reviewer installs, French for English
speakers), `TRANSLATION.md` (the routes table's « not yet: change 52 »), `tool/marks/README.md`
(« marked once change 52 ships it »), the comments of `markup.ts`, `language-labels.ts` and
`soak_engine.mjs`, `scripts/lingua-data/SOURCES.md` and the French tables' READMEs that name change
52 as the future (on `main` today; the French changes still open add their own). The comments that
say which natives ship (`pairs.ts`, `native-language.ts`, `build.mjs`) stay true: no native is added.

### D12 — The dogfood pass, before merge

On Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), with a package built from
this branch with the real packs, on test accounts (risk 2):
1. A new install in a browser in English: native preset English, Spanish studied; « Studied
   languages » offers Spanish and French; ticking French shows « Estimated French level » and its
   note; the level chosen.
2. French pages — news, a novel (curly apostrophes and narrow no-break spaces), a page with « au »,
   « l'homme », « dit-il »: highlighted; each elided piece clickable apart; whether two adjacent
   highlights of one class need a hairline (change 40's question) noted for the owner.
3. The card in English: grammar lines (« past historic (passé simple) », one mood for « parle »),
   the gloss, the level labelled estimated; an expression card (« d'abord », « coup d'œil »)
   (change 44's check); change 51's gestures — a drag inside « l’homme » and « dit-il », a
   double-click and a long press on « l’homme » and « d’abord » (its 6.2).
4. Read-aloud, change 47's checklist (its 6.2): Chrome, Firefox and Safari on macOS — `Thomas`
   automatic, `Jacques` and the Eloquence voices under "Other voices" on Chrome; the iPhone; Firefox
   for Android — the switch, « — France »; the card of `l'` in « l'homme » heard as « l'homme »;
   Chrome on an English or Spanish Windows if one is at hand — the no-voice sentence, the tooltip,
   `Google français` once allowed.
5. A Catalan and an Occitan page: not read as French (change 42).
6. « Extended translation »: the sentence says 52.5 MB and « about 340 MB » with Spanish and French
   studied, 26.2 MB and « about 200 MB » with French alone; fr-en downloaded once; French
   selections translated and marked — literary French and a phrase among them, read by eye
   (change 50's risk); the worker's memory with es-en and fr-en loaded, read from the
   browser's task manager (change 50's D3); absent on Firefox for Android.
7. With fr-es: a new install in a browser in Spanish offers English and French; the card in
   Spanish (RAE terms) and an expression card (change 44's check, in fr-es); « Nivel de francés
   estimado »; fr-es's translation through English (51,6 MB, « unos 340 MB ») on the iPhone among
   others (change 50's risk). Without fr-es: French is not offered.
8. A French-native install of the last store release, updated in place by this build: nothing
   asked, nothing moved; French never offered; a French page not read.
9. Sync: a French card captured on the English-native device shows on a second English-native
   device; with fr-es, on a Spanish-native device of the same account with fr-es's gloss (M4); a
   French-native device of the account pulls no French card; the day's statistic stored under `fr`.
10. On Chrome (macOS), after step 1's reader has French cards: the previous store release loaded in
    place of this build (D4) starts and keeps reading Spanish — a reader of French alone too —, and
    this build reloaded finds the French cards.
11. The library: a French EPUB shelved under French for an English-native reader.
12. The host app's activation page on the iPhone and the Mac, the device in French, English and
    (with fr-es) Spanish: the languages D7 names, and step 3 followed as written to a second
    language's level.

The owner runs it on their devices, with Claude where a session can drive the browser; the findings
go to the pull request.

### D13 — The release, TestFlight first

Released on its own, after 34's and 35's, so that a regression is attributed to one audience —
settled by the owner on 2026-10-09 (Open Question 3). TestFlight first. The store submission that
first carries French carries change 53's listings —
French named in each store's listing, as *The store listings name each studied language* requires —
and the descriptions of D7; the site is deployed after the merge, with fr-en's (and fr-es's)
figures, before the listings are pasted. The previous release on every store reads backup v3 (D4).
Each submission is the owner's (M18).

### D14 — Requirements true of their time, rewritten here

Three open changes say of their pair what this change ends: change 43's *French's forms and
frequencies* (« fr-en's pack SHALL be built … and no package SHALL list it », and its scenario *The
pack builds where the others' do*), change 48's *fr-en is committed at its studied tables' snapshot,
and the French baseline runs on it* (« … and no package SHALL list it », and *The pack is built, not
shipped*) and change 49's *French is glossed in Spanish from the Spanish Wiktionary's French section
and the French Wiktionary's translation tables* (*The pack is built, not shipped*). Archived as
written, they would stand beside *The shipped pairs are one list* saying the opposite. So this change
MODIFIES the three, as change 41 rewrote changes 39's and 40's (its D8) rather than leave the spec
false between two archives, each archived after the change that holds it — `archiveAfter` names all
three, and no other open change holds them:
- the two sentences say the pack is carried by a package only once the list of shipped pairs names
  it;
- 43's scenario says the list names fr-en, which its test asserts (D11);
- 48's and 49's *The pack is built, not shipped* keep their names — `openspec archive` refuses a
  MODIFIED block that drops a scenario (change 41's D8) — and hold « while the list of shipped pairs
  does not name » their pair, as change 50's *A route of a pair studying French* holds « when no
  shipped pair studies French ».

Every other word and scenario is carried verbatim. D7's *Guided activation* is MODIFIED the same way,
after `add-lingua-apple` and `localise-lingua-apple-host`, which hold it. Checked with OpenSpec
1.13.2 on a scratch copy of `openspec/`: the 41 changes this one waits for, transitively, archived
each as soon as `openspec_archive_order.py` allowed, then this one — every archive succeeds, the five
requirements hold 12, 19, 6, 15 and 7 scenarios, no archived spec says the list does not name fr-en
or fr-es, and none that the French activation page stays what it was.

Two conditions bind the implementation. The blocks are the requirements as their changes hold them
when this change is implemented — 48's and 49's implementations may still reword theirs —, re-copied
with only these clauses differing (task 5.4). And under the floor, change 49 stays open and fr-es
unlisted, its scenario true as written: the implementation removes 49's block and
`add-lingua-pack-fr-es` from `archiveAfter` (Open Question 5).

## What moves, and what cannot

Nothing in `crates/` but the assertions of `committed_tables.rs` that read the list (D11), nothing in
`scripts/lingua-data/tables/`, and no existing `testdata/` folder changes: no pack, pin, table,
rule, analyser version or golden can move. `fr-en.golden` (and change 51's `fr-es.golden`) shows
nothing moving: it is written by the core over the committed tables, which this change does not
touch. en-fr, es-fr, es-en and en-es cannot move: their tables and pins are untouched, their goldens
pass as committed (*Measured*: the five baselines and `cross_native.rs` without re-blessing), their
readers' pairs, routes, downloads and marks are unchanged (`readerPairs` keeps the pairs of a native
language; D6's table), and the extension's snapshots pass as committed. The extension's French
interface is byte for byte: no French string moves, and no French-native reader is offered French.
The one French text that moves is the host app's activation page, `lede` and `step3`, by the owner's
decision (D7). The gate:
`git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables`
empty.

## Risks / Trade-offs

- **fr-es under its floor** → D2: French for English speakers alone, nothing published or listed
  for fr-es, a later change when a regeneration measures above it.
- **A rollback after French** → D4: the previous store release reads schema 3.
- **fr-en's or fr-es's glosses read wrong in places** → `refine-lingua-fr-en-glosses` (change 48's
  D7) and `refine-lingua-fr-es-glosses` (change 51's list; the owner, 2026-10-09) before this change
  lists each pair, the owner's sample (48's 6.1).
- **Package size** → ≈ 12.1 MB zipped, inside the programme's estimate and every store's limit;
  a reader loads only their native language's packs.
- **Memory on iOS with fr-es** → es-fr's size, already dogfooded on the iPhone; D12 step 7 again.
- **The descriptions read as listings** → the owner's wording (M9), within 112 characters, checked
  by `check_version`.
- **Two adjacent highlights read as one word** → each clickable apart (change 40); D12 step 2 asks
  the owner whether a hairline is wanted, a follow-up if it is.
- **Under the floor, the archive waits** → change 49 stays open until a regeneration measures fr-es
  at or above its floor, and changes 51 and 53 name it in `archiveAfter`: 51, this change (archived
  after 51) and 53 wait with it, their specs unfolded — Open Question 5.

## Migration Plan

Nothing to migrate: an installed reader keeps their native language, studied languages and backup
version; French is offered, never added. Merged with the owner's go-ahead; released by the owner,
TestFlight first, with change 53's listings, after the release that reads schema 3. Rollback: the
previous release, which reads a French reader's state (D4).

## Effort

Against 4–7.5 ideal days:
- the list, the gate, the fixture, the coverage and the tests that read the list: 0.75–1.25;
- the extension's French (type, name, offer) and its tests in English and Spanish: 1–1.75;
- the memory sentence and its tests: 0.25–0.5;
- the descriptions, the activation page and its test, the comments and the documents: 0.25–0.5;
- the checks (Rust, Python, site, host app): 0.25–0.5;
- the dogfood on five targets, two interfaces and the two owner checklists: 1.25–2.5;
- the spec and the programme: 0.25–0.5.

Total 4–7.5.

## Open Questions

For the owner:
1. **The descriptions** (D7): change 53's drafts, « Read Spanish and French on the web: … » and « Lee
   inglés y francés en la web: … » — or other words (M9), read once for both changes.
2. **The host app's activation page** (D7) — *settled on 2026-10-09*: every language the reader
   can study is named, in every language, the French page included; the drafts are read with the
   descriptions (M9, task 6.2).
3. **One release per audience** (D13) — *settled on 2026-10-09*: this change released on its own,
   after 34's and 35's.
4. **fr-es's known defects** (change 51's *Known data defects*: `être` opening on « Ser », `qui`
   « Quién. (Pronombre nominativo.) », `rien`, a run with no part of speech under `des`) — *settled
   on 2026-10-09*: fixed before fr-es ships, by `refine-lingua-fr-es-glosses`, proposed after change
   49's implementation; this change lists fr-es once it has merged (D2).
5. **The archive under the floor** (D14): change 49 stays open until fr-es measures at or above its
   floor, and changes 51 and 53, which name it in `archiveAfter`, wait with it — and this change,
   archived after 51. Recommended: under the floor, 51 and 53 drop 49 from `archiveAfter`, as this
   change does — none of the three MODIFIES a requirement of 49 then —, so that each archives once
   implemented; or all three wait for fr-es.
