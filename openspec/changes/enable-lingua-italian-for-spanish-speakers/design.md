# Design — enable-lingua-italian-for-spanish-speakers

## Context

Change 30 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: Spanish speakers
learn Italian through it-es.

Change 20 (`enable-lingua-italian`) did what Italian needed once, and listed it-fr:
- the extension's studied-language type and the profile's name;
- `GUARDED.it`;
- online translation's backend;
- backup schema 4 in readers' hands.

Change 25 (`enable-lingua-italian-for-english-speakers`) listed it-en. What is left is it-es's part:
- the list;
- the Spanish description and activation page;
- the memory sentence's Spanish cases;
- the dogfood in the Spanish interface.

it-es differs from it-en in two ways:
- **Its glosses are thin and pivoted** (I4, I11). The Spanish Wiktionary's Italian section, the Italian
  Wiktionary's Spanish translations, the Spanish tables read backwards, then change 27a's pivot through
  seven Wiktionaries' translation tables and Wikidata Lexemes. The study puts it at about 84.7 / 76.1 /
  65.5, against en-es's 93.0 / 85.0 / 71.7.
- **Its translation is a route of two models**, it-en then en-es, as fr-es's is fr-en then en-es.

Each precondition below is a fact only once its change is implemented:
- it-es committed at or above its floor (change 27, the floor fixed by the owner at its task 0.1), its
  known defects fixed (change 28, `refine-lingua-it-es-glosses`, not yet proposed), its marks measured
  (change 29);
- change 20 implemented and released;
- change 25 implemented: this change appends after it-en and copies its *Guided activation* block;
- the Spanish catalogue names Italian (change 15: « Italiano », « Nivel de italiano estimado »), and the
  card names Italian's forms in Spanish (change 18: « pretérito perfecto simple de indicativo »);
- the site describes Italian (change 21: Italian's name in the Spanish table, `STUDIED.it`), so its build
  takes an it-es row;
- change 31 (`add-lingua-italian-spanish-listings`) implemented: the requirements listing it-es would
  make false.

## Goals / Non-Goals

**Goals:**
- it-es listed, and Italian offered to Spanish speakers after English and French.
- For them, an Italian page read, glossed, recorded, read aloud and translated.
- The Spanish description and the host app's Spanish activation page name Italian.
- Nothing moves for French-, English- and Italian-native readers, nor for it-fr, it-en and en-it.

**Non-Goals:**
- The Spanish listings, the Spanish home card and the table's scenarios (change 31).
- it-es's tables, refinement and marks (changes 27, 28, 29).
- fr-it and es-it (stage 4).
- Anything change 20 or 25 already did.

## Decisions

### D1 — The list, it-es after it-en

`packs.json` gains `"it-es"` after `"it-en"`, or after `"en-it"` if change 45 has listed it first.
`SHIPPED_PAIRS` follows, and its comment names this change. A Spanish native's default pair stays
en-es, and Italian is offered after French.

`yarn gen:pack:real` builds it-es's layer against its pin, with the Italian base already there.
`yarn gen:pack` needs `scripts/lingua-data/testdata/it-es/`: the it-fr fixture glossed in Spanish, as
change 25 made it-en's.
- Its Italian files (`forms.tsv`, `freq.tsv`, `level.tsv`, `tags.tsv`, and `grammar.tsv` and
  `lexical.tsv` where it-fr's carries them) are byte for byte `testdata/it-fr/`'s.
- `manifest.json`'s analyser version and `levels_estimated` are the same as it-fr's.
- Only `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and the manifest's native and pack version are its
  own.

`gen_pack.sh` refuses two pairs of one studied language whose bases differ by a byte
(`ship-lingua-pack-bases-once`). `crates/lingua-pack/tests/pipeline_testdata.rs` gains
`the_it_es_fixture_is_the_italian_fixture_glossed_in_spanish` and it-es's row in the fixtures' pins.
`committed_tables.rs` asserts that `packs.json` lists it-es, and change 27's « listed in no package
while the list does not name it » flips.

### D2 — it-es by its floor, after its refinement

Change 27 commits it-es at or above `FLOORS["it-es"]`, fixed by the owner at its task 0.1, before the
first committed measurement: the study re-run on change 9's committed Italian ranks, less two points
(about 82.7 / 74.1 / 63.5 on the study's ranking). `test_gloss_coverage.py`'s
`ListedFloorsTest` holds every listed pair with a floor at or above it, and names it-es.

it-es is listed once change 28 has merged, as it-en waits for change 23 and fr-en waited for
`refine-lingua-fr-en-glosses` (Open Question 3). A pivot gloss reads right only where the senses line up
across three languages: change 27's known defects (`sparato`, `posto`) are what change 28 and this
change's dogfood read for. When change 28 is proposed, `.openspec.yaml` names it already.

### D3 — The extension: no new type, one new offer

Nothing is typed: `StudiedLanguage` holds `it` since change 20. Everything follows the data:
- Settings and onboarding offer `shippedLanguages(pairsOf("es"))`: inglés, francés, italiano.
- The level texts come from change 15's `src/i18n/es/languages.ts`: « Nivel de italiano estimado », its
  note, « Elige tu nivel de italiano ». The ladder reads them.
- The card says « Palabra italiana » and names Italian's tenses in Spanish (change 18). Read-aloud uses
  « Italiano (Italia) » (change 15).
- `GUARDED.it` keeps the neighbours' pages out, as it does for a French or English reader. Online
  translation already accepts Italian into Spanish (change 20's D5).

### D4 — Translation: two models, it-en then en-es

it-es is a route of two models, it-en then en-es, as change 2 declares it and change 19 soaked it.
Following change 29's figures (its D4, as change 38's):
- **on the first tier**, `MARKED_PAIRS` holds it-es (change 29), live from this change;
- **short of the tier, but at least 75 % correct and at most 30 % withheld**, it is offered without a
  mark, by M15's rule;
- **under 75 % correct or over 30 % withheld**, the owner decides here: offered without a mark, or not
  offered.

If it is not offered:
- this change drops `routes["it-es"]` from `model-manifest.json`;
- the site says « Pendiente: del italiano al español »;
- change 31's « withheld » variants are pasted;
- three texts that name the it-es route are MODIFIED here, each copied from its latest holder at
  implementation (change 45, where its own withheld branch modified them), `add-lingua-italian-translation-models`
  being in `archiveAfter` already:
  - change 2's *Italian is translated to and from English directly, and to and from French and Spanish
    through English* (« it-es's it-en then en-es »);
  - change 2's *French is translated into English directly, and into Spanish through English*, whose
    *Every native language's pairs* names it-en among a Spanish reader's models;
  - *Translation models are listed in one catalogue, with a route per pair*, held by change 20, whose
    *Every reader today* lists the it-es route.

No model host redeploy gates this change: it-en and en-es are served already.

### D5 — The memory sentence's Spanish cases

Unlike an English reader of Italian, a Spanish reader of Italian alone needs two models, because it-es
goes through English:
- **Italian alone**: it-en and en-es, « unos 340 MB », the download their sum. en-es is 25,373,354 B, and
  it-en's served size comes from the catalogue (about 26.2–26.3 MB by change 20's estimate), so the sum
  is about 51.6–51.7 MB.
- **English, French and Italian**: en-es, fr-en and it-en, held two at a time, « unos 340 MB », the
  download their sum (fr-en 26,234,715 B; about 77.8–77.9 MB). en-es serves both routes through English.

Change 20's requirement already reads « two models or more »; this change adds the two Spanish
scenarios. If it-es's translation is not offered (D4), a Spanish reader's accepted Italian adds no
model, and the scenarios' condition (« extended translation serves it ») is false.

### D6 — The Spanish description and activation page

**`_locales/es`'s `extensionDescription`** must name Italian: `check_version`'s rule fails this pull
request until it does. Today it reads « Lee inglés y francés en la web: palabras desconocidas
resaltadas, porcentaje honesto. Sin conexión y privado. » (109). Said the same way with Italian it is
119 characters, over the 112 limit, and change 25's shape (« The web in … ») is 115 in Spanish. Drafts
(M9):

| | Draft | Characters |
|---|---|---|
| recommended: change 25's shape, « nuevas » for « desconocidas » | « La web en inglés, francés o italiano: palabras nuevas resaltadas, porcentaje honesto. Sin conexión y privado. » | 109 |
| keeps « Lee », drops « y privado », as change 25's alternative | « Lee inglés, francés e italiano en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión. » | 109 |
| change 25's shape, drops « y privado » | « La web en inglés, francés o italiano: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión. » | 105 |

**The activation page's `es`** `lede` and `step3` (M9), its header comment and the `es` table's comment
(« who study English and French ») following:

| | Draft |
|---|---|
| `lede` | « Lee la web en inglés, en francés o en italiano: las palabras que todavía no conoces aparecen resaltadas, directamente en Safari. » |
| `step3` | « En Safari, abre la extensión desde el <strong>menú de la barra de direcciones</strong> y elige tu nivel de inglés. Para el francés o el italiano, márcalo en los <strong>Ajustes</strong> de la extensión, pestaña <strong>Idioma</strong>, y luego elige su nivel. » |

The page grows by three words (the lede two, step 3 one). Change 40 watched the iOS page's scroll, so the dogfood reads it on
an iPhone. The French and English pages do not move; `test/apple-activation-page.spec.ts` follows (the
Spanish page names inglés, francés and italiano, in that order).

### D7 — The published coverage, and the site

`gloss_coverage.py --write` publishes it-es's figures, and the site's tests require the file's pairs to
be the list's. The site needs no code change (change 21's tables):
- the Spanish Lingua page names Italian for Spanish speakers;
- the French and English Lingua pages gain it-es's row;
- the site's fixtures are refreshed from the built pages, and `test/astro/lingua-page.spec.ts` counts
  nine rows, or ten if change 45 has listed en-it.

The Spanish home card, which is data-driven, reads « Lee la web en inglés, en francés o en italiano… »
from this pull request's coverage file. Change 31 is implemented first, as change 26 is before change
25. It modifies the card's requirement, the listings' and the table's, so none of them reads false once
it-es is listed.

### D8 — What French-, English- and Italian-native readers see

Their extension's pairs, offers, downloads and surfaces are what they were. The package carries it-es's
layer, which only a Spanish-native reader who ticks Italian fetches. Three things move:
- the site's French and English Lingua pages gain it-es's row (D7);
- a French- or English-native reader of Italian who switches their native language to Spanish keeps
  Italian, now through it-es (`studiedForNative`), where before it was dropped;
- the Safari app's own review (#925) serves an Italian card glossed in Spanish to a Spanish-native
  reader.

### D9 — Tests and documents

The tests that read the list, each keeping the earlier lists as explicit arguments:
- `packs.spec.ts`, `pairs.spec.ts`, `manifests.spec.ts`, `model-manifest.spec.ts`;
- `studied-languages-view.spec.ts`: `languages("es")` gives `["en", "fr", "it"]`, and change 25's
  « Italian to no Spanish speaker » case flips, run on change 25's list passed explicitly;
- `check-version.spec.ts`: `ES_DRAFT` and its pinned length;
- `apple-activation-page.spec.ts`: `named("es")` gains italiano.

New cases, as change 25's tasks 2.1–2.2, in the Spanish interface:
- `profile.spec.ts`: English then Italian, Spanish native;
- `language-labels`, `settings-language`, `onboarding-level-row`, `popup`, `stats-view`: « Nivel de
  italiano estimado »;
- `wordpopup.spec.ts`: an Italian card in Spanish;
- `engine.spec.ts`: a Spanish reader fetches the Italian base and it-es's layer. Change 25's « Spanish
  reader refused » keeps running on change 25's eight-pair list, passed explicitly, since with it-fr,
  it-en and it-es listed no native language lacks an Italian pair;
- `native-language.spec.ts`: a reader of Italian switching to Spanish keeps it;
- `model-residency.spec.ts`: a Spanish reader's fr-es and it-es routes share en-es;
- `translation-setting.spec.ts` and `model-controller.spec.ts`: D5's two cases;
- `translate-relay.spec.ts`: it-es marked if change 29 put it on the tier.

Others:
- `test_gloss_coverage.py`'s `ListedFloorsTest`, and it-es « published once listed »;
- `committed_tables.rs` (D1).

Documents:
- `README.md`, `REVIEWERS.md`, `TRANSLATION.md`, `tool/marks/README.md`;
- the comments that name `enable-lingua-italian-for-spanish-speakers` as the future.

### D10 — The dogfood pass, before merge

In the Spanish interface, on test accounts, on Chrome and Firefox (macOS), Firefox for Android, and
Safari (macOS, iOS). The owner runs it, and findings go to the pull request.
- A new install in Spanish offers inglés, francés and italiano, with « Nivel de italiano estimado ».
- Italian pages: news, a novel, « al mare », « dell'uomo », « un po' », « perche' ».
- The card in Spanish (« pretérito perfecto simple de indicativo »), and an expression card.
- A sample of pivot glosses read against their Italian, starting from change 27's known defects
  (Open Question 6).
- Read-aloud with change 15's voices.
- « Traducción ampliada » with Italian alone (« unos 340 MB », two models) and with three languages,
  reading the worker's memory.
- « Traducción en línea » on an Italian sentence, on a Spanish-native account granted it.
- A Spanish-native install of the last store release, updated in place: nothing asked, nothing moved,
  Italian offered but not added.
- A French-, an English- and an Italian-native install, unchanged.
- Sync across two Spanish-native devices.
- The rollback: a Spanish-native reader with Italian cards loads the previous release in place of this
  build. It knows `it` but has no it-es, so its accepted languages leave Italian out and sync sets the
  records aside. Reloading this build finds the cards.
- The library.
- The host app's Spanish page, read on an iPhone (D6), and an Italian card glossed in Spanish reviewed
  in the app (#925).

### D11 — The release

it-es is released on its own, TestFlight first, with change 31's listings. The site is deployed after
the merge with it-es's figures, before the listings are pasted. Each submission is the owner's (I12,
M18).

### D12 — Requirements true of their time

This change MODIFIES three requirements:
- ***The shipped pairs are one list***, held by change 45, whose block lists en-it:
  - the list gains it-es, by its floor: after en-it, in the order `packs.json` first listed them, if
    change 45 is implemented first, else after it-en alone (D13);
  - *Building the listed packs*, *A list widened too early* and *French for English speakers alone* name
    it, the last offering a Spanish reader Italian once it-es is listed;
  - *No Italian for Spanish speakers* holds while no listed pair studies Italian glossed in Spanish;
  - *Spanish speakers learning Italian* is added.
- ***The setting's memory follows the models…***, held by change 45: the two Spanish scenarios are
  added.
- ***Guided activation***, held by change 25, which change 45 does not touch: *Spanish speakers without
  fr-es* holds while neither fr-es nor it-es is listed, and *Italian for a Spanish speaker* is added.

Each MODIFIED block keeps every other word and scenario, and is re-copied at implementation from its
holder as it then stands.

True as written, so not modified:
- change 20's *Translation models are listed in one catalogue, with a route per pair*: its *A route of
  an Italian pair* is conditional, and its *Every reader today* lists the it-es route, true while the
  route is kept; D4 modifies it if translation is withheld;
- change 20's ADDED requirement and its *Readers with no pair of Italian* (« to no other reader » and the
  condition still hold);
- change 27's « No package SHALL list it-es before Italian is enabled for Spanish speakers » and *Not
  shipped*. Its title, « …, and not shipped », would read false once it-es is listed: it is RENAMED
  « …, and shipped once Italian is enabled for Spanish speakers », body and scenarios unchanged. This
  pull request renames change 22's it-en twin the same way, in change 25's delta;
- change 29's scenario, which is conditional on its figures.

Change 31's to modify, implemented first:
- the listings' *fr-es held below its floor*, *Spanish speakers learning French* and the « no Spanish
  text offers Italian » clauses;
- the table's ninth and tenth rows;
- the Spanish home card's *Spanish speakers learning English*.

### D13 — Against change 45

Stage 3 does not depend on stage 2 (I5), so either of it-es and en-it may be listed first. This
proposal copies change 45's blocks, today's latest holders. The rule, for both changes: **the one
implemented second re-copies the first's blocks and names it in `archiveAfter`**, so that each archives
right after its own implementation and the published specs are never left behind.
- **If change 45 is implemented first** (en-it listed first), this change re-copies its blocks from
  change 45 as it then stands (or from `openspec/specs/` if archived), and `packs.json` reads
  `…, "it-en", "en-it", "it-es"`.
- **If this change is implemented first** (it-es listed first), it re-copies its two shared blocks from
  change 25 instead, drops `enable-lingua-italian-speakers` from `archiveAfter`, and `packs.json` reads
  `…, "it-en", "it-es"`. Change 45 then appends en-it, re-copies this change's blocks and names it in its
  own `archiveAfter` (its D1, amended by this pull request). Any edit change 45 makes to a shared block
  is made in its own delta, on this change's text.
- **If change 45 is withdrawn** (en-it under its floor, say), the second bullet applies.

The list's requirement follows the branch:
- en-it first: « …then it-fr, then it-en, then en-it and it-es in the order `packs.json` first listed
  them, it-es once its tables are committed at or above its coverage floor », as this proposal's block
  reads;
- it-es first: « …then it-fr, then it-en, then it-es once its tables are committed at or above its
  coverage floor », with *Building the listed packs*, *A list widened too early* and *French for English
  speakers alone* naming it-es without en-it. Change 45 then writes « en-it and it-es in the order
  `packs.json` first listed them » when it re-copies.

*Guided activation* is not shared: change 45 does not touch it.

## Risks / Trade-offs

- **it-es's glosses read wrong in places**, since a pivot pairs senses across three languages. → Its
  floor, change 28 before the list (D2), and the dogfood's sample (D10).
- **Two models for Italian alone** → the memory sentence says so (D5); the download is given.
- **The Spanish description's shape** → the owner's words (M9), within 112 characters.
- **Translation short of the tier** → M15, decided here at change 29's figures (D4).
- **Package size** → one layer more (it-es's); the Italian base is already carried.

## Migration Plan

Nothing to migrate: Italian is offered, never added. The owner releases it after changes 20's and 25's
releases. Rollback is the previous release, which reads schema 4 and knows `it` but lists no it-es. A
Spanish reader's Italian is left out of what it accepts, and kept in the backup, until this release
returns (D10).

## Effort

Against 4–7.5 ideal days: 3.25–6.5.

| Part | Ideal days |
|---|---|
| The list, the fixture, the tests that read the list, and the coverage write | 0.75–1.25 |
| The Spanish offer, memory and residency tests | 0.5–1 |
| The description, the activation page and the documents | 0.25–0.5 |
| The checks | 0.25–0.5 |
| The M15 branch (D4) | 0–0.25 |
| The dogfood | 1.25–2.5 |
| The spec and the programme | 0.25–0.5 |

## Open Questions

For the owner:
1. **The description and the activation page** (D6): the recommended draft, one of the others, or other
   words (M9). « palabras nuevas » keeps « privado » but shifts « unknown » towards « new » (it could read
   as new coinages); the other two keep « desconocidas ».
2. **it-es under 75 % correct or over 30 % withheld** (D4), at change 29's figures: offered without a
   mark, or not offered. Short of the tier but within that band, it is offered without a mark by rule.
3. **it-es after its refinement** (D2): listed only once change 28 has merged. Recommended.
4. **Which of it-es and en-it is listed first** (D13): no technical preference once the second re-copies
   the first's blocks; the one whose data is ready first.
5. **The release** (D11): on its own, TestFlight first. Recommended.
6. **Reading pivot glosses before listing** (D10): a sample of about fifty, read by the owner or by an
   agent reading Italian and Spanish, its findings fixed in change 28 or here. Recommended.
