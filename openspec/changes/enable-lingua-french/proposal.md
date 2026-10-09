# enable-lingua-french — French as a studied language: fr-en ships, and fr-es above its floor

## Why

Change 52 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the stage-3 release: English speakers learn French through fr-en, and Spanish speakers through
fr-es — only if fr-es's committed tables stand at or above the floor M6 fixed before they were
measured (81.4 / 68.8 / 54.5 %, settled by the owner on 2026-10-09, change 49's D8); below it,
French ships for English speakers alone. A French-native reader cannot study French (change 39), so
French readers see nothing new in the extension; the host app's activation page names, by the owner's
decision of 2026-10-09, every language its reader can study, the French page included (D7).

What French needs is built silently before it — a precondition, listed under Impact's
prerequisites, not a fact today: the variant and backup v3 (39), the tokenisation, analysis and
detection guard (40, 41, 42), the forms, expression keys, grammar and levels (43–46), the voice
(47), fr-en's and fr-es's glosses (48, 49, and their refinements `refine-lingua-fr-en-glosses`,
named by change 48's D7, and `refine-lingua-fr-es-glosses`, settled by the owner on 2026-10-09), the
routes and the marks (50), the card (51), and the core fix `fix-lingua-lemma-lookup` (41b)
in readers' hands. The English- and Spanish-native interfaces are changes 34's and 35's: French is
studied from them, so this change comes after both. Every one of those changes leaves the same
edits to this one: `packs.json` listing the French pairs, and the extension's own studied-language
type learning `fr` (change 39's D8, change 47's D4, change 51's D1).

Measured on a scratch copy of `main` with change 35's implementation (#814) merged (design,
*Measured*): with the list widened and the type widened, the extension compiles but for the one
call change 51 widens, 6 of its 3,148 tests fail — each asserting a list or a French pair « inert
until change 52 » — and one Rust test does the same (1 of 20, re-run on `main` at `aabe17b3`, after
changes 42 and 46 merged); every invariance golden and every snapshot passes unchanged; the variant
check passes; the site describes French on its three Lingua pages with no code change; the host app
declares the same three languages. The package grows to about
12.1 MB zipped with six packs (10.8 MB with fr-en alone), inside the programme's 11–13.5 MB.

## What Changes

- **The French pairs ship** (design D1, D2): `packs.json` becomes `["en-fr", "es-fr", "es-en",
  "en-es", "fr-en", "fr-es"]`, or ends with `fr-en` when change 49's first committed measurement
  fell under fr-es's floor (its tables are then not committed); fr-es is listed only once
  `refine-lingua-fr-es-glosses` has fixed its known defects and merged, at or above the floor;
  `check_variants`'s `SHIPPED_PAIRS` follows. French pairs come last, so every native language keeps
  its default pair: en-fr for French, es-en for English, en-es for Spanish.
  `scripts/lingua-data/testdata/fr-es/`, a hand-written fixture, lets `yarn gen:pack` build fr-es as
  it builds the others (D10).
- **The published coverage** (D8): `gloss_coverage.py --write` publishes fr-en's figures, and
  fr-es's when it ships, in `apps/site/src/data/lingua-coverage.json`; `test_gloss_coverage.py`
  holds every listed pair that has a floor at or above it. The site's Lingua pages name French for
  English (and Spanish) speakers from that file and the catalogue's routes, with no code change.
- **The extension studies French** (D3): its `StudiedLanguage` type gains `fr`; `state/profile.ts`
  reads a stored profile's `French`, which it drops today. Réglages and onboarding offer French to a
  reader whose native language has a French pair, after their other languages; the level labels
  already speak of French in English and Spanish (change 47's D4), the card names French's forms
  (change 51), the voice reads it (change 47).
- **Translation as M15 settled it** (D5): fr-en direct and fr-es through English, both marked
  (change 50: 95 / 96 and 90 / 91, on the first tier); nothing in the catalogue moves, and the model
  host already serves fr-en (checked 2026-10-09).
- **The memory sentence counts the models** (D6), handed over by change 50's D3: an English-native
  reader of Spanish and French holds es-en and fr-en, two models through no pivot, and is told
  « about 340 MB » rather than « about 200 MB ». Every reader before this change reads what they
  read today.
- **The manifest's descriptions** (D7): `_locales/en` names Spanish and French, `_locales/es` names
  English and French when fr-es ships — change 53's drafts (its D6), which its summary check
  requires of the pull request that lists the pairs, read by the owner (M9), within Apple's 112
  characters;
  `_locales/fr`, `default_locale` and the host app's languages do not move (the French pairs add no
  native language).
- **The host app's activation page names every studied language** (D7, settled by the owner on
  2026-10-09): `copy.js`'s `lede` and `step3` name every language a native language's shipped pairs
  study — Spanish and French in English, English and Spanish in French, English and French in
  Spanish with fr-es — step 3 choosing the first one's level, then saying where another is ticked;
  the French page, in `copy.js` and `Main.html`, moves by this decision.
- **Backup v3 in readers' hands** (D4): a reader who studies French writes backup schema 3
  (change 39). Every build since change 39's implementation (#803, `9fde7bb1`) reads it, but no
  release does yet — 1.7.0 and the host app's 1.5.0 predate it and refuse it by name. So the first
  release cut after #803 — change 34's, 35's or `fix-lingua-lemma-lookup`'s (41b, its task 4.3),
  whichever reaches the stores first; 41b itself moves no stored format — is live on every store
  before this change's release: a rollback then lands on a build that keeps a French reader's data.
- **Requirements true of their time** (D14): change 43's *French's forms and frequencies* and change
  48's *fr-en is committed at its studied tables' snapshot…* say « no package SHALL list it », and
  their scenarios and change 49's that the list does not name the pair. MODIFIED here, each archived
  after the change that holds it, so that no archived spec contradicts the list; 49's only when fr-es
  ships.
- **A dogfood pass on five targets** (D12) with the English and Spanish interfaces, carrying change
  47's read-aloud checklist and change 51's selection gestures; TestFlight first; the owner's
  go-ahead before the merge and before each submission (M18).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-spanish-speakers`, itself archived after `enable-lingua-english-speakers` and
  `enable-lingua-spanish`, so this change is archived after all three): the list holds en-fr,
  es-fr, es-en, en-es then fr-en, followed by fr-es once its tables are committed at or above its
  floor; a listed pair with a floor is held to it; every scenario kept, five added. MODIFIED too,
  each archived after the change that holds it, its other words and scenarios verbatim (D14):
  change 43's *French's forms and frequencies*, change 48's *fr-en is committed at its studied
  tables' snapshot, and the French baseline runs on it* and, with fr-es, change 49's *French is
  glossed in Spanish from the Spanish Wiktionary's French section and the French Wiktionary's
  translation tables* — a package carries the pair once the list names it.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by `localise-lingua-apple-host`, which
  modifies `add-lingua-apple`'s, so this change is archived after both): « the French copy SHALL
  stay what it was » becomes « each language's copy SHALL name every language the shipped pairs
  glossed in it study »; every scenario kept, three added.
- `lingua-browser-extension`: ADDED *French is studied by readers of English and Spanish* — the
  extension's own French: the stored profile, the offer per native language and never to a French
  speaker, the labels and the card's studied words.
- `lingua-translation`: ADDED *The setting's memory follows the models the reader's pairs need* —
  the sentence counts the models the reader's pairs need together, not the pivots. *The setting's
  cost comes from the catalogue* is held by no open change and is not modified.

Change 50's *A route of a pair studying French* stays true as written: it reads « when … no shipped
pair studies French ».

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension` — *changed*: `packs.json`, `tool/check_variants.mjs`,
    `src/analyzer/types.ts` (`StudiedLanguage`), `src/state/profile.ts` (`NAMES`),
    `src/translate/host/model-controller.ts` and `model-messages.ts` (the cost's flag),
    `src/reading/translation-setting.ts`, `_locales/en` and (with fr-es) `_locales/es`, the comments
    that say « until change 52 », `README.md`, `REVIEWERS.md`, `TRANSLATION.md`,
    `tool/marks/README.md`; the tests that read the list and those that hold a French pair inert;
    new tests of French in the English and Spanish interfaces; `test/apple-activation-page.spec.ts`
    (the activation page's languages).
  - `scripts/lingua-data/` — *new*: `testdata/fr-es/` (with fr-es); *changed*:
    `test_gloss_coverage.py`; *consumed*: `gloss_coverage.py`, every table, unchanged.
  - `crates/lingua-pack/tests/committed_tables.rs` — change 43's scenario asserts the list names
    fr-en; with fr-es, change 49's « listed nowhere » assertion goes (D11, D14).
  - `apps/site` — `src/data/lingua-coverage.json` (written by the script); no page or text changes:
    the three Lingua pages and the Spanish home's Lingua card (change 29b) follow the list (French's
    level sentence, « CECR » or « CEFR », and the listings are change 53's).
  - `apps/lingua-apple` — *changed*: `Shared (App)/Resources/copy.js` (`lede` and `step3` in fr, en
    and es, the last only with fr-es) and `Base.lproj/Main.html` (the French, in place); *consumed*:
    the host app declares fr, en and es as it does since change 35.

  ID, Music, Live, the back office (it shows the `fr` code; its name for it is change 53's, per
  change 39) and the backend are untouched: the server stores any studied language a client sends
  (`backend/lingua/src/language_core.rs` refuses none), so French cards, statuses and days sync with
  no server change. lingua-core, lingua-wasm, the packs, every table and every golden: untouched.
- **Prerequisites, all the owner's** (M18):
  - Implementations merged: changes 17 (#786), 34 (#810), 35 (#814) — 35 under the floor too: the
    list this change extends and the requirement it modifies are 35's, and M1 puts both stage-2
    audiences before French —, 45, 48, `refine-lingua-fr-en-glosses` (to be proposed, change 48's
    D7), 49's committed measurement (merged above the floor, or recorded under it), with fr-es
    `refine-lingua-fr-es-glosses` (to be proposed after change 49's implementation; the owner,
    2026-10-09), 51, and 41b; 39 to 44, 46, 47 and 50 are on `main`. The two refinements join
    `archiveAfter` once proposed.
  - `fix-lingua-lemma-lookup` released on the Chrome Web Store, addons.mozilla.org and in the host
    app (its task 4.3) — at the latest, the release that first reads backup v3, as every release cut
    after #803 does (D4).
  - The owner's readings: change 48's floor and sample (its 6.1), 51's tables and snapshots (its
    6.1), change 53's English and Spanish drafts, quoted here, and the activation page's drafts
    (M9).
  - Test accounts for the dogfood (risk 2).
  - Change 53's listings (French named in each store's listing, the requirement *The store listings
    name each studied language*) in the same submission (D13).
  - Nothing to deploy first: the model host serves fr-en (checked on 2026-10-09), and the privacy
    annex already names the languages generically, its sizes still true (change 31's D1; change 50).
- **Release.** The first packages in which French is studied, released on their own after 34's and
  35's (settled by the owner on 2026-10-09), TestFlight first (D13); the site
  deployed after the merge, with fr-en's (and fr-es's) figures, before the listings are pasted.
  Nothing is submitted by this pull request.
- **Order.** After 34 and 35 (stage 2); after every French change above; before 53 (listings and
  site) and 57 (YouTube captions amendment). Archived after the changes of `.openspec.yaml`.
- **Effort, against 4–7.5 ideal days**: 4–7.5 (design, *Effort*), the dogfood the uncertain part.
