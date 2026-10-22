# Design — enable-lingua-italian

## Context

Change 20 of the [Italian programme](../../../docs/lingua/italian-programme.md), the stage-1 release:
French speakers learn Italian through it-fr. Its model is matrix change 52, `enable-lingua-french`,
which listed fr-en and fr-es once every French change had landed; this change lists it-fr once every
stage-1 Italian change has. Where things will stand when it is implemented (each a precondition,
listed in the proposal's Impact, not a fact today — on `main` today are only the proposals):

- **The pack format** (changes 3 and 4): a base per studied language and a layer per pair; the
  package carries each base once (`assets/bases/`) and each listed pair's layer; the engine fetches a
  base and a layer on first need. `packs.json` still lists pairs; `check_variants`'s `SHIPPED_PAIRS`
  mirrors it; the Apple host derives its native languages from it.
- **Italian in the core** (changes 5–15): `StudiedLanguage::Italian` and backup schema 4 (5), the
  tokenisation (6) and analysis (7), the detection guard (8), the forms (9), the unmarked spellings
  (10), the acronyms and names (11), the expression keys (12), the grammar (13), the estimated levels
  (14), the voice (15) — each inert while no package lists an Italian pair.
- **it-fr's glosses** (16, and 17 `refine-lingua-it-fr-glosses`): committed at or above the floor
  task 0.1 of change 16 fixes; the known defects of its design fixed by change 17.
- **The card** (18): the renderers name Italian's forms in French; `CARD_NAMES.it`.
- **Translation** (2, 19): it-fr routed through it-en then en-fr; the it-en model on the model host
  after the owner's redeploy; it-fr's marks measured and, on the first tier, listed in
  `MARKED_PAIRS`, inert until this change.
- **The extension's own studied languages**: `StudiedLanguage = "en" | "es" | "fr"` in
  `src/analyzer/types.ts`, `profile.ts`'s `NAMES` without Italian — what changes 14, 15 and 18 left
  to this one, as matrix changes 46, 47 and 51 left French's to `enable-lingua-french`. Change 15 named Italian in the
  language catalogue apart (`NamedLanguage = StudiedLanguage | NativeLanguage | "it"`).
- **The listings, the site's description of Italian and the back office's name for it** are change
  21's, `add-lingua-italian-listings`, implemented before this change (D8).

## Goals / Non-Goals

**Goals:**
- it-fr listed; Italian offered to French speakers, after English and Spanish; an Italian page read,
  glossed, recorded, read aloud and translated for them.
- The French description and the host app's French activation page name Italian.
- Nothing moves for English- and Spanish-native readers, and nothing of the six pairs.
- Released on its own, after a release that reads backup schema 4 is live on every store.

**Non-Goals:**
- it-en and it-es (stage 2: changes 25 and 30), the Italian interface and en-it (stage 3), fr-it and
  es-it (stage 4).
- The store listings, the site's Lingua pages' Italian text, the back office's name for `it`
  (change 21).
- Any table, pin, golden or rule: all are the earlier changes'.

## Measured

On a scratch copy of `main` at `66695c937` (2026-10-22), nothing Italian implemented, so each failure
is sorted by the change that removes it. Unmodified, `yarn typecheck` is clean and vitest passes
4,347 tests in 164 files.

**The type widened alone** (`StudiedLanguage` with `"it"`, `NAMES` with `Italian`): four type
errors. `language-labels.ts:26` and `:40` (`NAMED` and `words()` lack `it`) are change 15's, whose D3
adds them; `wordpopup.ts:552` (`grammarBlocks` takes the card's `StudiedLanguageCode`) is change 18's,
whose D1 widens it; `test/native-language-view.spec.ts:358` (a test-local `NAMES` map) is this
change's. Change 14 fills none and needs none. The type checker cannot see one gap: `GUARDED`
(`src/reading/session.ts:273`) is a `Partial` record and has no Italian row (D3). vitest still passes
4,347 of 4,347: nothing offers Italian until a pair is listed.

**it-fr listed** (`packs.json`): 100 of 4,347 tests fail in 14 files.
- 93 throw in `languageName` (`language-labels.ts:65`), Réglages now naming Italian: change 15's
  catalogue entry removes them (and its D7 adds the Italian preview to `SAME_EVERYWHERE`).
- 1, `packs.spec.ts:155`, reads no analyser version for `it` (`tool/packs.mjs` `ANALYZER_CONSTANTS`):
  change 5's.
- 7 are this change's: `packs.spec.ts:28`, `pairs.spec.ts:141` and `:144`, `manifests.spec.ts:253`,
  `model-manifest.spec.ts:348`, `studied-languages-view.spec.ts:162` and `:165`,
  `check-version.spec.ts:263` (D7) and `apple-activation-page.spec.ts:130` (D7); once change 2 is
  implemented, its « no reader fetches the it-en model » test is one more, as change 50's was for
  French.
- `yarn gen:pack` needs `testdata/it-fr/` (change 5), `yarn gen:pack:real` the committed tables
  (changes 9 and 16), `yarn build` the pack; `tool/check_variants.mjs:335` holds the list as a second
  literal, by design.
- Python: `test_every_shipped_pair_is_published` fails until `gloss_coverage.py --write` runs, which
  needs `tables/it/` (change 9). Rust: `committed_tables.rs` passes; its one read of the list asserts
  fr-en. `README.md:210` quotes the list.

**The offer.** `pairsOf` filters `packs.json` in order, so a French speaker's pairs are en-fr, es-fr,
it-fr and the boxes read English, Spanish, Italian; Réglages (`settings-view.ts:282`) and the
onboarding (`onboarding.ts:87`) mount the same view. The default pair stays en-fr; the shipped natives
stay fr, en, es.

**The memory sentence.** `needsTwoModels` is `needed.length > 1` (`model-controller.ts:94`); the
worker holds `MODEL_BOUND = 2` (`model-residency.ts:21`). Served sizes: en-fr 25,752,472 B, es-en
26,241,052 B. A French speaker reading English and Spanish downloads 51,993,524 B (« 52,0 Mo »,
« environ 340 Mo »). it-en's served size is not in the catalogue before change 2's task 1.1; at es-en's
and fr-en's gzip ratios (0.709, 0.705) over its 37,088,127 B of files it is ≈ 26.2–26.3 MB, so Italian
alone ≈ 52.0 Mo and English, Spanish and Italian ≈ 78,2 Mo, « environ 340 Mo » both.

**The description.** `_locales/fr` (and `manifest.json`'s literal, French being the literal native)
reads « Lisez l'anglais et l'espagnol sur le web : mots inconnus surlignés, pourcentage honnête. Hors
ligne et privé. », 109 characters of the 112 `check_version` allows. With it-fr listed, `yarn
check:version` fails twice — « the fr description … does not name "it" » —, the raw `"it"` because
`LANGUAGE_NAMES` (`check_version.mjs:12`) has no Italian. The direct wording, « Lisez l'anglais,
l'espagnol ou l'italien sur le web : … », is 120 characters. `_locales/en` (105) and `es` (109) are not
flagged.

**The activation page.** `copy.js` `fr` `lede` (line 23) and `step3` (line 26), repeated in
`Main.html` (lines 16 and 22) and held equal by `apple-activation-page.spec.ts:109`; the test at
`:124` asks each page to name its natives' languages in the pairs' order. `app_localizations.sh`
derives the same natives from seven packs as from six: `CFBundleLocalizations` stays `[fr, en, es]`.

**The site.** With an it-fr row in `lingua-coverage.json`, `yarn test` passes but `yarn build` fails
on every Lingua page: the fr, en and es tables of `lingua-text.ts` have no name for `it`, and
`STUDIED` does not describe it. With the names and `STUDIED.it` given, the build passes (35 pages),
`yarn check:routes` passes, and three tests of `test/lingua-text.spec.ts` that take Italian as their
example of an unnamed language need another; the French page then reads « … l'anglais, l'espagnol ou
l'italien », its table gains « Italien → français », the French home card « Lisez le web en anglais,
en espagnol ou en italien ». The site's tests require the file's pairs to be `packs.json`'s.

**The back office** shows « it » raw: `lingua.languages` holds en, es and fr.

**Backup v4.** The released builds (extension 1.10.1, host app 1.8.1) write and read schema 3; a
schema-4 backup is refused by name — « backup schema v4 is not supported (this build reads v1 to v3) »
(`decks/backup.rs`). In the extension, a restored file shows « Fichier de sauvegarde non reconnu. »; a
stored one stops the surfaces from starting (`hydrateEngine`); sync leaves aside the records of a
language the device does not accept.

**The package**, six pairs, zipped as the release workflow zips it: 13,037,241 B for Chromium,
13,021,083 B for Firefox, 13,023,646 B for Safari. it-fr's own base and layer cannot be measured
before its tables exist.

**« change 20 » in the code** already means other changes — the matrix programme's
`add-lingua-native-language-choice` (`TRANSLATION.md`, `account/locale.ts`, `account/page.ts`) and the
Spanish programme's (`measure.rs`). Comments name this change `enable-lingua-italian`, never a bare
number.

## Decisions

### D1 — The list and the gate, Italian last

`packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es", "it-fr"]` and
`SHIPPED_PAIRS` the same; the requirement *The shipped pairs are one list* follows. Italian goes last:
a native language's default pair is its first listed one, so French natives keep en-fr, English
natives es-en, Spanish natives en-es — an installed reader's default and the onboarding's preset stay
true. Italian is then offered after Spanish, in Réglages and at onboarding, the reader ticking it.

`yarn gen:pack:real` builds it-fr's layer and the Italian base from the committed tables against
their pin; `yarn gen:pack` builds them from change 5's fixture (`scripts/lingua-data/testdata/it-fr/`).
The tests that read the default list take their list explicitly where they mean an earlier one, as
matrix changes 34, 35 and 52 (`enable-lingua-french`) did.

*Rejected — Italian before Spanish.* A French reader's order in Réglages would move for no reason, and
change 21's listings would name Italian before the language every French reader already reads.

### D2 — it-fr by its floor, after its refinement

Change 16 commits it-fr's tables only at or above `FLOORS["it-fr"]`, fixed by the owner at its task
0.1, and fails any later reduction under it; `test_gloss_coverage.py` holds every listed pair with a
floor at or above it (`enable-lingua-french`'s D2), so the list cannot carry it-fr under it. it-fr is listed once
change 17 (`refine-lingua-it-fr-glosses`) has merged — the defects change 16's design lists (the
species names among the expressions, `della`, `del`'s note, the letter `te`, the glosses repeating a
word across parts of speech) fixed before French readers see them, as fr-es's were by
`refine-lingua-fr-es-glosses` (the owner, 2026-10-09). Open Question 1 asks the owner to confirm the
same rule for it-fr.

### D3 — The extension's Italian: one type, one name, no new surface

`StudiedLanguage` becomes `"en" | "es" | "fr" | "it"` and `profile.ts`'s `NAMES` gains
`Italian: "it"`; `NamedLanguage`'s explicit `"it"` (change 15) is then redundant and goes. What
follows from the two edits:
- **The offer.** `mountStudiedLanguages` offers `shippedLanguages(pairsOf(native))`: English, Spanish
  and Italian to a French speaker; nothing new to an English or a Spanish speaker, whose native
  language has no pair of Italian.
- **The stored profile.** A backup naming `Italian` is read as Italian by the surfaces that read it
  without an engine, instead of being dropped.
- **The card.** `content.language` says `it` for an Italian page: the studied words carry `lang="it"`
  inside the French lines, and change 18's renderer names Italian's forms.
- **The page.** Detection among the reader's accepted languages with change 8's guard; the pieces of
  change 6 (« l' », « dell' », « al », « po' ») highlighted and clickable apart; change 10's
  unmarked spellings (« perche' », « E' »); change 15's voice of Italy; the level block titled
  « Niveau d'italien estimé », its note saying the levels are estimated (change 14's flag).

What the type check asks once the type is widened, and which tests fail, is in *Measured*. Two
things come with the type, because nothing else can write them:
- **`GUARDED.it`** (`src/reading/session.ts`): the languages change 8's guard keeps out of Italian, as
  its implementation settles them — by default Catalan, Galician, Venetian, Sicilian, Neapolitan,
  Sardinian, Corsican and Friulian (`ca`, `gl`, `vec`, `scn`, `nap`, `sc`, `co`, `fur`) —, with
  `THREE_LETTER` gaining `srd` and `cos` (and `ita` for Italian itself): a word card on a page
  declaring one of them is never read in Italian. The row is a `StudiedLanguage` key, so change 8
  could not write it, as change 42's French row came with French's type; the requirement *No word is
  read in a language whose guard keeps the document's language out*, which names each language's
  list, is MODIFIED (D13).
- **The test-local `NAMES`** of `test/native-language-view.spec.ts` gains Italian.

### D4 — Backup v4 reaches a store before Italian does

Once a reader studies Italian, their stored backup is schema 4 (change 5's D7), written only then; a
reader who studies no Italian keeps 1, 2 or 3, byte for byte. A build released before change 5's
implementation refuses schema 4 by name. Were this change's release withdrawn, the stores would serve
the previous release again: if that one predated change 5, a reader of Italian would find their
extension refusing its own state. So the first release cut after change 5's implementation — which
reads schema 4 — is live on the Chrome Web Store, addons.mozilla.org and, in the host app, the App
Store before this change's release. A rollback then lands on a build that keeps the reader's Italian
records: its `profile.ts` drops `Italian` from what the surfaces read, the core keeps the records in
the backup, and this change's next release finds them. The dogfood checks it once (D11, step 10).

### D5 — Translation as M15 settled it

M15: translation opens with each pair, marked per pair under matrix change 26's D2. it-fr goes through it-en
then en-fr (change 2), two models in one engine as es-fr and fr-es; the model host serves it-en once
the owner has redeployed it after change 2 (its task 4.1), which `check_model_host` verifies before
any delivery. Marks: if change 19 measured it-fr on the first tier, `MARKED_PAIRS` already lists it,
inert until now; below the tier, M15 offers it-fr's translation without a mark unless the owner
decided otherwise at change 19's figures (Open Question 2). Nothing in `model-manifest.json` or
`markup.ts` changes but the comments that say « until change 20 ».

### D6 — The memory sentence counts two models at most

The requirement *The setting's memory follows the models the reader's pairs need* (`enable-lingua-french`) says
« two models when the reader's pairs need two models together ». A French-native reader of English,
Spanish and Italian needs three (en-fr, es-en, it-en), which the engine holds two at a time, evicting
the one no current route needs (matrix change 9; change 2's D2): their memory is two models', and the
flag `twoModels` (`needs.needed.length > 1`) already says so. The requirement's wording widens to
« two models or more »; the download the setting states is the catalogue's sum over the three
models, as *The setting's cost comes from the catalogue* already requires. Figures in *Measured*.

### D7 — The French description and the host app's French activation page

it-fr adds no native language: the built manifests keep `_locales/fr`, `en` and `es`,
`default_locale` English, and the host app keeps `CFBundleLocalizations` `[fr, en, es]`. What moves is
what French readers are told they can study:
- **`_locales/fr`'s `extensionDescription`** names Italian, within the 112 characters
  `check_version` holds (Apple's limit) — and `check_version`'s rule (`add-lingua-french-listings`)
  that each native language's description names exactly the languages its shipped pairs study fails
  this pull request until it does. `LANGUAGE_NAMES` gains Italian in each native's words (« italien »,
  "Italian", « italiano »), as the interface's catalogue names it once change 15 is in. The direct
  wording is 120 characters, so the sentence's shape changes (M9, the owner's words):

  | | Draft | Characters |
  |---|---|---|
  | recommended | « Le web en anglais, espagnol ou italien : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé. » | 107 |
  | keeps « Lisez » | « Lisez l'anglais, l'espagnol et l'italien sur le web : mots inconnus surlignés, pourcentage honnête. Hors ligne. » | 111 |

  `manifest.json`'s French literal follows. `_locales/en` and `es` do not move.
- **The activation page** (`copy.js`'s `fr` `lede` and `step3`, and `Main.html`'s two French nodes,
  the same bytes): every language a French-native reader can study, as *Guided activation* already
  requires and `enable-lingua-french`'s drafts did for two — English, Spanish and Italian; step 3 chooses
  English's level, then says where Spanish or Italian is ticked. Drafts (M9; the page's tu;
  `<strong>` as today):

  | | Draft |
  |---|---|
  | `lede` | « Lis le web en anglais, en espagnol ou en italien : les mots que tu ne connais pas encore sont surlignés, directement dans Safari. » |
  | `step3` | « Dans Safari, ouvre l’extension depuis le <strong>menu de la barre d’adresse</strong> et choisis ton niveau d’anglais. Pour l’espagnol ou l’italien, coche-le dans les <strong>Réglages</strong> de l’extension, onglet <strong>Langue</strong>, puis choisis son niveau. » |

  The English and Spanish pages do not move. `test/apple-activation-page.spec.ts` follows: the French
  page names « anglais », « espagnol » and « italien », in that order.

### D8 — The published coverage, and the site

`python3 scripts/lingua-data/gloss_coverage.py --write` rewrites `apps/site/src/data/lingua-coverage.json`
with it-fr's figures once it is listed, as `enable-lingua-french` published fr-en's and fr-es's;
`test_gloss_coverage.py` (`test_every_shipped_pair_is_published`) requires it, and the site's tests
require the file's pairs to be the list's.

The site refuses a studied language it does not describe (*Measured*). Describing Italian — its name
in the fr, en and es tables of `lingua-text.ts`, `STUDIED.it` (the card names tense and gender, the
levels are estimated), the three tests that take Italian as their unnamed example moved to another
language — is change 21's (`add-lingua-italian-listings`), inert until the file lists it-fr, and
implemented before this change, as `add-lingua-french-listings` (#868, 2026-10-10) was before French
shipped (#880, 2026-10-11). This change then writes the file, and the French page and home card name
Italian with no code change. The back office's name for `it` is change 21's too.

### D9 — What English- and Spanish-native readers see: nothing

| | French native | English native | Spanish native |
|---|---|---|---|
| Pairs | en-fr, es-fr, it-fr | es-en, fr-en | en-es, fr-es |
| Default | English | Spanish | English |
| Offered | English, Spanish, Italian | Spanish, French | English, French |
| Base and layer fetched for Italian | on first need | never | never |
| Translation downloads | + it-en with Italian | as before | as before |
| `_locales` read | `fr`, names Italian | `en`, unchanged | `es`, unchanged |
| Backup | 4 once Italian is studied | unchanged | unchanged |
| Activation page | English, Spanish, Italian | unchanged | unchanged |

The package carries the Italian base and it-fr's layer for every reader; only a French-native reader
who ticks Italian fetches them (change 4).

### D10 — Tests and documents

The tests that read the list, or held an Italian pair inert, are rewritten for the list, each keeping
the earlier lists it named as explicit arguments; new ones are D3's (`profile.spec.ts`,
`studied-languages-view.spec.ts`, `language-labels.spec.ts`, `settings-language.spec.ts`,
`onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` with Italian studied in the
French interface; `wordpopup.spec.ts` with an Italian card's words in `it`), D6's
(`translation-setting.spec.ts`, `model-controller.spec.ts`), D7's (`check-version.spec.ts`,
`apple-activation-page.spec.ts`) and `GUARDED`'s (`session.spec.ts` or where the French row is
tested: a page declaring `scn`, `vec` or `srd` reads no card in Italian). `committed_tables.rs`: change 9's scenario asserts the list names
it-fr. `test_gloss_coverage.py`: D8's publication.

Documents: `README.md` and `REVIEWERS.md` (what ships: Italian for French speakers),
`TRANSLATION.md` (it-fr's route, offered), `tool/marks/README.md`, the comments of `markup.ts`,
`language-labels.ts` and `soak_engine.mjs`, `scripts/lingua-data/SOURCES.md` and the Italian tables'
READMEs that name this change as the future; the full list from `git grep -n "enable-lingua-italian"`
at implementation — not « change 20 », which the code already uses for two other programmes'
changes (*Measured*).

### D11 — The dogfood pass, before merge

On Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), with a package built from
this branch with the real packs, on test accounts, in the French interface:
1. A new install in a browser in French: native preset French, English studied; « Langues étudiées »
   offers English, Spanish and Italian; ticking Italian shows « Niveau d'italien estimé » and its
   note; the level chosen.
2. Italian pages — news, a novel (curly apostrophes), a page with « al mare », « dell'uomo »,
   « l'unico », « un po' », « perche' », « E' », « dirmelo »: highlighted; each piece clickable apart.
3. The card in French: the tense names of change 18 (passato remoto « passé simple »), `parli`'s
   moods, the gloss, the level labelled estimated; an expression card (« d'accordo », « essere
   d'accordo »); a drag and a long press inside « dell'uomo ».
4. Read-aloud, change 15's checklist (its task 4.1): Alice on macOS and iOS, Google's Italian voice
   on Chrome, Android's on Firefox for Android, an installed Windows Italian voice on Chrome for
   Windows if one is at hand; « l' » heard with its word, « è » heard as the verb.
5. A Catalan, a Galician, a Sicilian and a Venetian page: not read as Italian (change 8).
6. « Traduction étendue »: the download and « environ 340 Mo » with Italian alone and with English,
   Spanish and Italian; it-en downloaded once; Italian selections translated, marked if it-fr is
   listed in `MARKED_PAIRS`; absent on Firefox for Android.
7. An English- and a Spanish-native install: Italian never offered; no Italian base fetched (the
   network panel).
8. A French-native install of the last store release, updated in place by this build: nothing
   asked, nothing moved; Italian offered, not added.
9. Sync: an Italian card captured on one French-native device shows on a second of the same account;
   an English-native device of the account pulls no Italian card; the day's statistic stored under
   `it`.
10. On Chrome (macOS), after step 1's reader has Italian cards: the previous store release loaded in
    place of this build (D4) starts and keeps reading English, and this build reloaded finds the
    Italian cards.
11. The library: an Italian EPUB shelved under Italian.
12. The host app's activation page on the iPhone and the Mac, the device in French: Italian named,
    step 3 followed as written to Italian's level.

The owner runs it on their devices, with Claude where a session can drive the browser; the findings
go to the pull request.

### D12 — The release, TestFlight first

Released on its own, after the release that reads backup schema 4 (D4), so that a regression is
attributed to one audience, as `enable-lingua-french` was (the owner, 2026-10-09). TestFlight first. The store
submission that first carries Italian carries change 21's listings and D7's description; the site is
deployed after the merge, with it-fr's figures, before the listings are pasted. Each submission is
the owner's (I12, M18).

### D13 — Requirements true of their time, rewritten here

Three requirements of open changes say what this change ends:
- change 9's *Italian's forms and frequencies* — « no package SHALL list it-fr », and its scenario
  *The reference and the pipeline*, « no package lists it-fr »: MODIFIED, the pack carried once the
  list of shipped pairs names it;
- change 4's *The shipped pairs are one list* — the list ending with fr-es, and its scenario *No
  French for French speakers*, « their pairs are en-fr and es-fr as before »: MODIFIED, the list
  ending with it-fr, the scenario naming it-fr, two scenarios added;
- `read-lingua-card-in-another-studied-language`'s *No word is read in a language whose guard keeps
  the document's language out*, which names Spanish's and French's neighbours: MODIFIED, Italian's
  added (D3), one scenario added.

And two of `enable-lingua-french`'s widen without contradicting anything: *Guided activation* already asks each
language's page to name every language its pairs study — its scenario *The French page*, « names
English and Spanish », is rewritten and one is added; *The setting's memory…* reads « two models or
more » (D6), two scenarios added. Each MODIFIED block keeps every other word and scenario; each is
archived after the change that holds it — `archiveAfter` names `ship-lingua-pack-bases-once`,
`add-lingua-italian-forms-tables`, `read-lingua-card-in-another-studied-language` and
`enable-lingua-french`. The blocks are re-copied from those
changes as they stand when this one is implemented, only these clauses differing (task 5.4).

True as written, so not modified: change 2's *A route of an Italian pair* (« when … no shipped pair
studies Italian »), change 16's « no package SHALL list it before Italian is enabled », `enable-lingua-french`'s
scenario *A French speaker* (« English and Spanish are offered as before, French is not » — still
true, Italian offered beside them), and change 19's scenarios, each conditional on its pair's
figures.

## What moves, and what cannot

Nothing in `crates/` but the assertions of `committed_tables.rs` that read the list (D10), nothing in
`scripts/lingua-data/tables/`, and no `testdata/` folder: no pack, pin, table, rule, analyser version
or golden can move. The six pairs' readers keep their pairs, routes, downloads and marks; the
extension's snapshots pass as committed. The French interface moves in two places only, by
*Guided activation* and `add-lingua-french-listings`'s rule: the description and the activation page (D7). The gate:
`git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables`
empty but for READMEs.

## Risks / Trade-offs

- **it-fr's glosses read wrong in places** → change 17 before the list (D2), the owner's sample
  (change 16's 6.1), the dogfood (D11).
- **A rollback after Italian** → D4: the previous store release reads schema 4.
- **Package size** → one base and one layer more for every reader, on 13.0 MB zipped today
  (*Measured*); fetched only by a French-native reader of Italian.
- **Change 21 not yet in** → the site's build fails on the coverage file (D8): change 21 is a
  precondition, as the French listings were.
- **Memory with three models** → two held at a time (D6); the dogfood reads the worker's memory on
  the iPhone with Spanish and Italian studied.
- **A neighbour read as Italian** → change 8's guard; D11 step 5.
- **The description and the page read as listings** → the owner's wording (M9), within 112 characters.

## Migration Plan

Nothing to migrate: an installed reader keeps their native language, studied languages and backup
version; Italian is offered, never added. Merged with the owner's go-ahead; released by the owner,
TestFlight first, with change 21's listings, after the release that reads schema 4. Rollback: the
previous release, which reads an Italian reader's state (D4).

## Effort

Against 4–7.5 ideal days:
- the list, the gate, the coverage and the tests that read the list: 0.5–1;
- the extension's Italian (type, name, offer) and its tests in French: 0.75–1.5;
- the memory sentence and its tests: 0.25–0.5;
- the description, the activation page and its test, the comments and the documents: 0.25–0.5;
- the checks (Rust, Python, site, host app): 0.25–0.5;
- the dogfood on five targets with change 15's checklist: 1.5–3;
- the spec and the programme: 0.25–0.5.

Total 3.75–7.5.

## Open Questions

For the owner:
1. **it-fr after its refinement** (D2): it-fr listed only once change 17 has merged, as fr-es was.
   Recommended.
2. **it-fr below the marks' tier** (D5): if change 19 measures it-fr short of the first tier,
   translation offered without a mark (M15's default), or not offered for it-fr. Settled at change
   19's figures.
3. **The description and the activation page** (D7): the recommended draft « Le web en anglais,
   espagnol ou italien : … Hors ligne et privé. », the one that keeps « Lisez » and drops « et
   privé », or other words within 112 characters; and the page's drafts (M9).
4. **One release for it-fr** (D12): released on its own, as French was. Recommended.
