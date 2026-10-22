# Design — enable-lingua-italian-speakers

## Context

Measured on main at `4a7e0ca32`, where no Italian change is implemented yet. A prototype listed en-it in
`packs.json` and added `it` to `INTERFACE_LANGUAGES`, with stand-ins for what earlier changes write.

**The extension.** Typecheck failed once: `it` is not a `NativeLanguage` (change 32's). vitest gave
17 failures in 11 files:

| Prototype | Failures | Whose |
|---|---|---|
| `INTERFACE_LANGUAGES` gains `it` alone | 8 | `account-locale.spec.ts` ×4 and `account-page.spec.ts` ×1: **this change's**; the labels lint ×3: change 33's |
| `packs.json` lists en-it alone | 7 | six tests reading the list: **this change's**; `check-version.spec`: change 39's |
| Both | +2 | `native-language.spec`, `native-language-view.spec` (« Italiano » offered with no name): the list is this change's, the name change 32's |

**The build chain:**

| Gate | Fails on | Whose |
|---|---|---|
| `gen:pack` | no `testdata/en-it` | change 36 |
| the builder | refuses a native `it` | change 32 |
| `yarn build` | no `_locales/it` | change 39 |
| `check:variants` | `SHIPPED_PAIRS` | **this change** |
| `check:version` | `LANGUAGE_NAMES.it` | change 39 |
| `app_localizations.sh` | warns | change 40 |

**The site.** With changes 21's and 41's names stood in, the build passes, and `check:routes` gives 59
passed and 2 skipped. The two skips are the Lingua fixtures' byte pins, which the coverage write
refreshes.

**What nobody else covers:**
- the interface-language requirement;
- the site's one-language shot set;
- the three « not yet » tests;
- the stale `pinnedBy` source;
- dropping en-it's route if the owner withholds its translation under M15.

## Goals / Non-Goals

**Goals:**
- An Italian speaker installs Cymbra Lingua and reads English with Italian glosses, the interface, card,
  links, site page and store listings in Italian.
- Nothing moves for a French, English or Spanish reader but what names the new pair, the Spanish and
  Italian deletion links, and I10's account locale for a reader whose browser is in Italian.

**Non-Goals:**
- fr-it and es-it (changes 47–54).
- The wording review (change 44, before this change's release, D9).
- The macOS menu bar (change 40's open question).

## Decisions

### D1 — The switch

`packs.json` appends en-it after it-en, so changes 20 and 22–26 land first: this change's list and
its blocks are changes 25's and 26's (or after it-es, if change 30 lists it first, this change then
re-copying change 30's blocks). `src/i18n/language.ts`'s `INTERFACE_LANGUAGES` gains `it`, and its comment
naming « `fr`, `en` or `es` » follows. `tool/check_variants.mjs`'s `SHIPPED_PAIRS` and its comment follow;
it is the only `check:variants` failure.

The tests that read the list keep their earlier lists as explicit arguments, as change 25's D9 did:
- `packs.spec.ts`, `pairs.spec.ts` (two), `manifests.spec.ts`, `model-manifest.spec.ts`;
- `studied-languages-view.spec.ts`, `native-language.spec.ts`, `native-language-view.spec.ts`;
- by precedent, `engine.spec.ts` (an Italian reader fetches the English base and en-it's layer),
  `model-residency.spec.ts`, and `translation-setting.spec.ts` with `model-controller.spec.ts` (an
  Italian reader of English alone: one model, en-it's).

`GUARDED` needs nothing: en-it studies English, which has no guard.

### D2 — The links, and the account's locale

| Link | File | From | To |
|---|---|---|---|
| The leaving page | `src/usage/counts.ts` (`LEAVING_PAGES.it`) | the English page (change 33) | `/it/lingua/arrivederci/` |
| The privacy policy | `src/translate/online.ts` (`PRIVACY_POLICY_URLS.it`) | the English page (change 33) | `/it/privacy/` |
| Account deletion | `src/account/locale.ts` (`deleteAccountUrl`) | French alone | an Italian arm (`/it/elimina-account/`), and the Spanish arm (`/es/eliminar-cuenta/`) that change 29's task 3.2 never landed |

All three pages are change 41's, pinned and deployed before this change's release (D9). The comments of
`src/account/locale.ts` follow: `deleteAccountUrl`'s, and `accountLocale`'s and `accountLanguage`'s,
which cite `it` as the language the extension does not speak. Change 29's task 3.2, the Spanish arm, is
done here and ticked pointing to this change, so that change 29 can archive.

The account's locale needs no code: `accountLocale` keeps the interface language unless the browser's
is one Cymbra speaks and the extension does not. With `it` an interface language, an Italian reader sends
`it`, and a French reader with an Italian browser sends `fr` (I10, ending M12's special case).
`account-locale.spec.ts` and `account-page.spec.ts` follow, five cases.

`apps/site/src/lib/pinned-routes.ts`'s `/en/delete-account/` names « every interface language but
French, Spanish and Italian ». Its source and `/suppression-compte/`'s name `src/account/locale.ts`, where
the function lives, beside `src/account/flow.ts`, which re-exports it.

### D3 — The tests written for « not yet »

These flip:
- change 33's « a stored `it` reads French »;
- change 40's « the teller sends no `it` » (`native-signin.spec.ts`).

Change 32's « an Italian browser preset to English » holds only while no pair is glossed in Italian and
stays as written; this change adds « preset to Italian ».

### D4 — Models

en-it's route is change 2's (`model-manifest.json`). Change 38 measures en-it's marks; the owner
decides under M15, at this change, what follows:
- **on the first tier**: listed in `MARKED_PAIRS` by change 38, by the tier rule; nothing here;
- **offered without a mark**: nothing changes, M15's default;
- **withheld**: this change drops en-it's route, or the site would say « direct »
  (`apps/site/src/lib/lingua-pairs.ts`), modifies change 2's route requirements on that branch (hence
  `archiveAfter: add-lingua-italian-translation-models`), and change 46's « withheld » variant is
  pasted. The translation scenario holds « while extended translation serves en-it ».

### D5 — Captures: one language studied

The capture harness assumes two studied languages:
- `tool/capture/capture.mjs` destructures `[first, second]` in two places;
- its store set includes a `second` shot;
- `test/capture-tool.spec.ts` loops over the interface languages' recorded states
  (`fixtures/state/<lang>.json`).

An Italian reader studies English alone, so the harness gains a one-language mode.
`tool/capture/plan.mjs` gains `it`, `it-IT` and `STUDIED.it = ["en"]`, and its overrides and reader levels
are checked. `fixtures/state/it.json` is recorded with `--record-seed`. The Italian « second » shot is taken on a
second English fixture article, with its own `WORDS` entry and a line in the fixtures' README (« one
article … in each studied language »), with a noun's card, so that the site's shot keys keep their
shape: `SHOT_KEYS` in `lingua-text.ts`, `linguaMedia` and
`LinguaPage.astro` render every key. Only the alt-text test changes (`lingua-text.spec.ts`, which
destructures the plan's languages). `test/capture-tool.spec.ts` follows.

One run makes the site's media (`src/assets/lingua/it/*`, `public/media/lingua/it/demo.mp4`) and the
stores' Italian set (`store-captures/it/`), as change 46's D4 expects.

### D6 — The site

- **The coverage write**: `python3 scripts/lingua-data/gloss_coverage.py --write` adds en-it's row to
  `apps/site/src/data/lingua-coverage.json`, from change 36's committed tables at or above
  `FLOORS["en-it"]`. `test_gloss_coverage.py`'s listed floors name en-it. `committed_tables.rs` flips
  change 36's « listed in no package » to « listed ». The Lingua fixtures are refreshed
  (`node test/support/refresh-lingua-fixtures.mjs`), which lifts the two skipped byte pins.
- **`/it/lingua/`** builds itself from the data (change 41's trap 1), with this change's media (D5). Its
  `LINGUA_TEXT.it.showcase` captions, change 41's stubs, are rewritten against the captures, and the
  capture tests gain Italian.
- **Change 42's leftovers**, folded in here:
  - *Eight pairs on a phone* narrowed with « and no other pair », and *Nine pairs on a phone* added, the
    Italian page at 375 px, English → Italian first (unless it-es ships first, row 31);
  - `alternates.spec`'s Lingua test generalised (it hard-codes Spanish);
  - `/it/lingua/`'s render test (`test/astro/lingua-page.spec.ts`, its row counts up by one) and its
    post-build twin (`lingua.spec.ts`);
  - `head.spec`'s share preview and `lingua-text.spec`'s card buttons for Italian.
- **Pins**: `/it/lingua/`, the App Store's Italian marketing URL (change 46), joins `PINNED_ROUTES`.
  The English and Spanish Lingua pages, also marketing URLs, are not pinned today: reported to the
  owner, not changed here.
- **Verified, not changed**: change 41's Italian home card now names English and links `/it/lingua/`;
  the site's media stay within change 41's 50 MB.

### D7 — The requirements

Twelve MODIFIED, each copied whole from its latest holder, every scenario kept:

| Requirement | Holder | Edit |
|---|---|---|
| *The shipped pairs are one list* | change 25 | en-it in the list; *Italian speakers learning English* |
| *The setting's memory follows the models…* | change 25 | an Italian speaker reading English: one model |
| *The interface language is the reader's native language…* | main spec | `it` among the interface languages; an Italian reader's key |
| *The account's e-mails follow the interface language* | `send-lingua-browser-locale-on-account-emails` | `it` an interface language; *An Italian browser* now sends the interface language; an Italian-native reader |
| *Cymbra account deletion is reachable from Lingua* | `add-site-spanish-locale` | the Italian page for Italian |
| *Every indexable page names its canonical address* | `add-site-lingua-showcase` | the Lingua page's `hreflang` names `/it/lingua/` |
| *The Lingua page says what each studied language's card and levels are* | change 21 | QCER in Italian |
| *The coverage table reads at a phone's width* | change 26 | the ninth pair |
| *Store and site captures come from the real extension, by one command* | `add-site-lingua-showcase` | a second article for a reader who studies one language; Italian's set |
| *The Lingua page shows the extension at work* | `add-site-lingua-showcase` | a second page in the one language studied; *The Italian page* |
| *Each browser's consent rule decides when counting starts…* | `add-lingua-usage-counts` | the note in Italian |
| *The app signs in with an email and a password…* | `add-lingua-app-email-signin` | failures in Italian |

One ADDED, *English is studied by readers of Italian*, as change 25 added *Italian is studied by readers of
English*.

These stay true with no block:
- change 32's ADDED requirement;
- change 33's *Not shown yet*, which ends here;
- change 36's « No package SHALL list en-it before Italian is an interface language »;
- the routes of the Italian pairs;
- change 40's « once en-it ships » scenarios.

*Guided activation* needs no Italian scenario: change 40's requirement covers the page.

### D8 — The Apple host

No code: `app_localizations.sh` writes `["fr","en","es","it"]` from the packs once en-it is listed, and
change 40 already holds the Italian tables. Because release-please assigns a release by path, this
change's squashed commit, a `feat`, touches `apps/lingua-apple` (its README's languages line names
Italian as offered from this release), so that the host app's release carries en-it. A `Release-As` footer would
impose one version on every component the commit touches, so it is not used. The it-IT App Store
screenshots are taken by hand from this change's build (change 46's D4).

### D9 — The order of release

**Before this change's pull request**, on main:
- changes 2, 3, 4, 21, 32, 33, 34, 35, 36 (tables at or above their floor), 37 (refined before listing, as
  change 25 did), 38, 39, 40 and 41;
- changes 20 and 22–26, whose list and blocks this change extends (I5's « stage 3 does not depend on
  stage 2 » holds for every stage-3 change but this one);
- if change 30 lists it-es first, this change re-copies its blocks.

**Before this change's release**, released or deployed:
- the extension's and the host app's releases carrying change 32, live on the three stores (change 32's
  D4, change 40's D7);
- the model host with en-it 2.1 (change 2's model gate);
- the backend with change 20's online translation for `it` and change 41's `thanks_path`;
- change 41's site, with the `/it/` pages pinned and `/elimina-account` registered on Apple's Services
  ID; and Spanish's `/eliminar-cuenta`, registered with change 29's site deploy (its task 4.3), since this
  change is the first build to send readers to `/es/eliminar-cuenta/`;
- change 43's backend (I10, settled: Italian e-mails link the Italian pages);
- change 44's pass over every Italian surface (I2), which runs after change 46 and reads this change's
  captions and captures;
- change 46 on main, its listings ready to paste.

**The owner's steps:**
1. Go-ahead before merge (I12).
2. Dogfood on the five targets, on a build carrying change 44's corrections: Chrome, Firefox (desktop and Android) and Safari (macOS and iOS). Check:
   - a new install in Italian;
   - the card on « went » and the QCER;
   - the three links;
   - the account locale;
   - extended and online translation;
   - French, English and Spanish installs updated in place, with nothing moving;
   - `chrome://extensions/shortcuts` and `about:addons` (change 39's D3);
   - the host app on a device in Italian before the extension has run (change 40's D8);
   - a rollback onto change 32's release, which is why change 32's D4 exists.
3. TestFlight first; the extension's packages uploaded without publishing.
4. The site deployed after the merge, publishing `/it/lingua/` and en-it's figures.
5. Change 46's listings pasted.
6. Each store submitted.

### D10 — The Italian this change writes

This change rewrites `LINGUA_TEXT.it.showcase`'s captions and alt texts and the capture fixtures' Italian.
They are reviewed in its pull request by change 33's Italian reviewer agent, its report pasted before
merge, as changes 33, 39, 40 and 46 do (I2).

## Risks / Trade-offs

- **Many changes must land first** → D9's list is the checklist; a missing one fails a named gate
  (*Context*).
- **The site claims a translation en-it lacks** → D4: the route dropped when the owner withholds it under M15.
- **A French reader with an Italian browser now sends `fr`** → I10's decision; an account that has a
  language keeps it (Cymbra ID records a locale only on an account with none).
- **Italian shown without a native reader** → change 44's pass, the dogfood, and change 55.

## Migration Plan

Readers update through the stores. An installed reader is unaffected unless their native language is
Italian, when Italian becomes offered and their interface Italian, but for the Spanish and Italian
deletion links and I10's account locale (D2). Rollback is the previous build re-published, through each
store's own means (the Chrome Web Store's rollback, AMO's disabled version, a new App Store build), the
repository untouched: `/it/lingua/` stays built, since it is pinned and change 46's
it-IT listing points at it.

## Open Questions

For the owner, all recorded elsewhere and gathered here:
1. en-it's floor (change 36's task 0.1).
2. en-it's translation, under M15, on change 38's figures.
3. Whether en-it is listed only after change 37's refinement (recommended, as change 25).
4. The macOS menu bar (change 40).
5. The English reviewer fields (change 46).
6. The Italian App Store support link (change 46).

The dogfood is the owner's, or left unverified (I2).

## Effort

5–9.25 ideal days, against the programme's 4.25–8:

| Part | Ideal days |
|---|---|
| The switch, its tests, the links, the account tests | 1.25–2 |
| The capture harness's one-language mode | 0.5–1 |
| The site: the coverage write, media and captions, the one-language set, change 42's leftovers, the pins | 1.1–2.25 |
| The requirements (twelve MODIFIED, one ADDED) and the reviewer agent's pass | 0.5–0.75 |
| The checks | 0.25–0.5 |
| The dogfood | 1.25–2.5 |
| The Apple touch and the release notes | 0.1–0.25 |

The row did not price the one-language shot set or the interface-language requirement.
