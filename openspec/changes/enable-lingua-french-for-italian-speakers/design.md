# Design — enable-lingua-french-for-italian-speakers

## Context

Change 50 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4: Italian speakers
learn French through fr-it. Read on main at `97c8aef7b`, where nothing Italian is implemented yet: every
fact about changes 20–49 below is their proposals'.

- **Change 45** makes Italian an interface language and lists en-it: an Italian reader studies English,
  and every Italian surface, the Italian listings (change 46) and the Italian home card (change 41) name
  English alone.
- **Changes 47–49** build fr-it: tables at or above a floor of about 79.3 / 68.1 / 55.8 (47), refined (48),
  its marks through fr-en then en-it measured and judged in Italian by two agent judges (49, PR #1027,
  whose trial put it on the first tier with a one-line strict margin).
- **The order rule** of the list (`src/analyzer/pairs.ts:48`, `:66`): the first pair glossed in a native
  language sets that reader's default studied language. en-it must stay ahead of fr-it.
- **What stays true** as written: change 45's *English is studied by readers of Italian* and *The Lingua
  page says what each studied language's card and levels are*; the stores' summary requirement; change
  39's ADDED manifest requirement (« en-it the only pair glossed in Italian » is its own condition);
  change 2's routes (« en-it and fr-it ship… en-it serving both routes »); change 32's native choice.

## Goals / Non-Goals

**Goals:**
- fr-it listed; French offered to Italian speakers after English; a French page read, glossed, recorded,
  read aloud and translated for them.
- Everything an Italian reader is told — the description, the activation page, the listings, the home
  card, the Lingua page — names French.
- Nothing moves for French-, English- and Spanish-native readers.

**Non-Goals:**
- es-it (changes 51–54).
- fr-it's tables, refinement and marks (changes 47–49).
- A separate listings change: the programme's row carries them here (D7).

## Decisions

### D1 — The list, fr-it at the end

`packs.json` gains `"fr-it"` at the end of the list, `SHIPPED_PAIRS` the same, its comment naming this
change. After changes 30 and 45 it reads `…, "it-en", "en-it", "it-es", "fr-it"` or `…, "it-en", "it-es",
"en-it", "fr-it"`; before change 30, `…, "it-en", "en-it", "fr-it"`. en-it is always ahead of fr-it: this
change follows change 45, which lists en-it.

Change 30's D1 put it-es « after en-it if change 45 listed it first »: with fr-it listed, that would insert
it-es before fr-it. This pull request amends change 30 to append it-es at the end of the list.

No testdata is added: change 47 writes `scripts/lingua-data/testdata/fr-it/` and its
`pipeline_testdata.rs` test. `committed_tables.rs`'s « listed in no package while the list does not name
it » for fr-it flips to « listed », as fr-en's did.

### D2 — fr-it by its floor, after its refinement

Change 47 commits fr-it at or above `FLOORS["fr-it"]`; `test_gloss_coverage.py`'s `ListedFloorsTest`
names it, and it is « published once listed ». fr-it is listed once change 48 has merged, as it-en and
it-es wait for their refinements (Open Question 1). `gloss_coverage.py --write` publishes its figures.

### D3 — The extension: no new type, one new offer

`StudiedLanguage` holds `fr`, and change 32 makes `it` a native; `shippedLanguages(pairsOf("it"))` now
gives English and French. Surfaces an Italian reader has never seen become visible: the « studied
languages » block (hidden while there was one language), the second level block, the statistics' note on
borrowed levels. Change 33 typed them in Italian; the reviewer agent and the dogfood read them. The level
texts come from change 33's `src/i18n/it/languages.ts`, filled from French's names; the card names French
tenses in Treccani's terms (change 34: « passato remoto », « condizionale presente »); read-aloud uses the
French voices; `GUARDED.fr` keeps the neighbours' pages out; online translation accepts French into
Italian since change 20's D5.

### D4 — Translation: two models, fr-en then en-it

fr-it is a route of two models, as change 2 declares it. Following change 49's figures:
- on the first tier, `MARKED_PAIRS` holds fr-it (change 49), live from this change;
- short of the tier but at least 75 % correct and at most 30 % withheld, it is offered without a mark by
  M15's rule;
- under 75 % or over 30 % withheld, the owner decides here: offered without a mark, or not offered. If not
  offered, `routes["fr-it"]` is dropped from `model-manifest.json` and the route texts are MODIFIED, each
  copied from its latest holder, as change 30's D4 does for it-es.

### D5 — The memory sentence's Italian cases

The catalogue counts each needed model once (`modelsFor`). fr-en is 26,234,715 B; en-it's served size
comes from change 2's catalogue (change 49's trial measured 26,509,835 B of registry gzips).
- English alone: en-it, about 200 MB (change 45's scenario, now « English alone »).
- French alone, or English and French: fr-en and en-it, about 340 MB, the download their sum (52,744,550
  B by change 49's trial, PR #1027; « 52,7 MB » while en-it's served size is between 26,415,285 and
  26,515,285 B).

### D6 — The Italian description and activation page

**`_locales/it`'s `extensionDescription`** names French: `check_version`'s rule fails this pull request
until it does. Change 39 writes « Leggi il web in inglese: parole sconosciute evidenziate, percentuale
reale. Offline e senza tracciamento. » (105). Drafts (the reviewer agent's, I2; tu, never Lei):

| | Draft | Characters |
|---|---|---|
| recommended: changes 25's and 30's shape | « Il web in inglese o francese: parole sconosciute evidenziate, percentuale reale. Offline e senza tracciamento. » | 110 |
| keeps « Leggi », drops « Offline » | « Leggi il web in inglese e francese: parole sconosciute evidenziate, percentuale reale. Senza tracciamento. » | 106 |

Too long: « Leggi il web in inglese e francese: … Offline e senza tracciamento. » (116), « Il web in
inglese o in francese: … » (113). Change 54's three-language form will have to drop « Offline e » (110
without it, 120 with).

**The activation page's `it`** (change 40's copy), drafts:
- `lede`: « Leggi il web in inglese o in francese: le parole che non conosci ancora sono evidenziate,
  direttamente in Safari. »
- `step3`: « In Safari, apri l'estensione dal <strong>menu della barra degli indirizzi</strong> e scegli il
  tuo livello di inglese. Per il francese, selezionalo nelle <strong>{settings}</strong> dell'estensione,
  scheda <strong>{language tab}</strong>, poi scegli il tuo livello di francese. »

The two labels are change 33's `drawer.settings` and `settings.tabLanguage`, which
`apple-activation-page.spec.ts` holds equal to the page's; the brand « Lingua » beside a tab named
« Lingua » is the reviewer agent's point (change 46's D6). The page grows by about twenty words; the
dogfood reads it on an iPhone, where it must not scroll.

### D7 — The Italian listings name French

Change 46's Italian texts are drafts; these sentences are drafted by analogy with change 31's Spanish
ones, for the reviewer agent (I2), and counted when change 46's texts exist:
- Chrome Web Store and AMO: « …le parole in inglese o in francese che non conosci ancora… »; the card
  « …(«passato semplice di go»; in francese, «passato remoto»), e il genere dei sostantivi francesi »;
  the levels « per il francese, livelli stimati in base alla frequenza delle parole, in mancanza di un
  elenco QCER a uso libero »; translation « dal francese, passando per l'inglese… »; sizes « (26,5 MB per
  l'inglese, 52,7 MB con il francese) »; the dictionary « Per il francese, il dizionario è un po' meno
  completo che per l'inglese. » (fr-it is ten to thirteen points under en-it: 81.3 / 70.1 / 57.8 against
  91.4 / 82.7 / 70.3); a line on choosing
  languages, true only once a reader has two pairs: « Scegli nelle Impostazioni le lingue che studi: ogni
  pagina viene letta nella sua lingua. »
- App Store it-IT: subtitle « Inglese o francese, leggendo » (28 / 30); promotional text « Le parole in
  inglese o in francese che non conosci ancora, evidenziate nella pagina che leggi. L'analisi avviene sul
  tuo dispositivo, offline. » (142 / 170); keywords gain `francese`, the most general word giving way if
  over 100 bytes; the description as the Chrome text; « What's New » (about 490 characters with the voice
  clause, 432 without).
- The shared English fields: the single purpose « …; English or French for Italian speakers »; the
  remote code « …and reading French, fr-en then en-it ». The review notes and test instructions are
  expected not to move (change 21's audience sentence and change 31's « a pair through English chains
  two » cover fr-it), checked at implementation; the review notes have no room.
- The App Store file's *What the app itself says* quotes the Italian step 3 and follows.
- No French, English or Spanish description says what Italian speakers study (change 46's Non-Goals), so
  none moves; *A listing in Italian* (« neither fr-it nor es-it ») stays true.

Every new Italian sentence passes change 44's glossary test.

### D8 — The site

Data-driven, as change 41 built the Italian pages: the home card and the Lingua page name English and
French; the gender and levels sentences, which name every listed pair's studied language, already name
French, and move it first on the Italian page, the readers' own pairs coming first; the French, English and Spanish Lingua pages gain
fr-it's row, and their hero notes for Italian speakers name French. `test/astro/lingua-page.spec.ts`
counts the rows (ten, or eleven with it-es).

**The Italian set of captures is taken again** with French as the second studied language. This is a
decision, not a question: the showcase requirement asks for « a page in the readers' second studied
language », which Italian readers now have, and so do change 45's captures requirement and this change's
*The Italian page with French*. It takes:
- `STUDIED.it` becoming `["en", "fr"]` in the capture plan;
- `tool/capture/fixtures/state/it.json` recorded again with `yarn capture --record-seed`, change 45 having
  recorded it for English alone;
- `LINGUA_TEXT.it.shots.second.alt` and its caption in `apps/site/src/lib/lingua-text.ts` rewritten for a
  French page (« Una pagina in francese, con l'interfaccia in italiano… »), read by the reviewer agent;
- the Italian media staying within change 41's 50 MB budget, as change 45's D6 requires.

Change 45's one-language mode and its second English article stay: its *Store and site captures…*
requirement names them for a reader who studies English alone, and no interface language needs them
after this change but they cost nothing. One App Store it-IT capture on a French page is added by hand.

### D9 — Tests and documents

The tests that read the list, each keeping the earlier lists as explicit arguments: `packs.spec.ts`,
`pairs.spec.ts`, `manifests.spec.ts`, `model-manifest.spec.ts`, `studied-languages-view.spec.ts`
(`languages("it")` gives `["en", "fr"]`), `native-language.spec.ts` (a reader of French switching to
Italian keeps French), `native-language-view.spec.ts`, `check-version.spec.ts` (the Italian draft and its
pinned length; change 39's « naming francese fails » case keeps running on the en-it-only list), and
`apple-activation-page.spec.ts`. New cases, in the Italian interface: `profile.spec.ts`,
`language-labels`, `settings-language`, `onboarding-level-row`, `popup`, `stats-view`, `wordpopup.spec.ts`
(a French card in Italian), `engine.spec.ts` (the French base and fr-it's layer), `model-residency.spec.ts`
(en-it shared), `translation-setting.spec.ts` and `model-controller.spec.ts` (D5), `translate-relay.spec.ts`
(if change 49 marks fr-it), and the capture tool's tests. Site: `lingua-text.spec.ts`,
`astro/lingua-page.spec.ts`, the post-build check, change 41's Italian home-card test. Rust:
`committed_tables.rs`. Python: `test_gloss_coverage.py`.

### D10 — The dogfood pass, before merge

In the Italian interface, on test accounts, on Chrome and Firefox (macOS), Firefox for Android and Safari
(macOS, iOS): a new install in Italian offers English and French; French's level said estimated; French
pages; the card in Italian (« passato remoto »); read-aloud; extended translation with French alone and
with both; online translation from French into Italian; an Italian-native install of change 45's release
updated in place (French offered, not added); French-, English- and Spanish-native installs unchanged;
sync; the rollback to change 45's release; the host app's Italian page on an iPhone (no scroll); an
Italian review of a French card in the app (#925); the Italian Lingua page and home card. The owner runs
it; findings to the pull request.

### D11 — The release

On its own, TestFlight first, after change 45's. The site deployed after the merge with fr-it's figures,
then the listings pasted, the it-IT « What's New » and promotional text with them (I9). No model-host or
backend deploy. `copy.js` lives under `apps/lingua-apple`, so release-please cuts the host app's release.

### D12 — Requirements true of their time

MODIFIED, each copied whole from its latest holder:
- *The shipped pairs are one list* (change 30): fr-it in the list's clause, en-it before it, and in
  *Building the listed packs*, *A list widened too early* and *French for English speakers alone*;
  *Italian speakers learning French* added.
- *The setting's memory follows the models…* (change 30): *An Italian speaker reading English* for
  English alone; two Italian scenarios added.
- *Guided activation* (change 30): *French for an Italian speaker* added.
- Change 40's *The host app holds Italian before a pair glossed in Italian ships*: *A device in Italian
  once en-it ships* holds while no other pair is glossed in Italian.
- *The store listings name each studied language* (change 31): *Italian speakers learning French* added.
- *The coverage table reads at a phone's width* (change 31): ten and eleven pairs on the Italian page.
- *The Lingua page shows the extension at work* (change 45): *The Italian page* holds while no other pair
  is glossed in Italian; *The Italian page with French* added.
- *The Italian home page describes Lingua for the readers it serves* (change 41): narrowed and extended.

ADDED *French is studied by readers of Italian*, as change 30's *Italian is studied by readers of
Spanish*. RENAMED change 47's « …, and not shipped », as changes 25, 30 and 45 rename their twins.

### D13 — Against changes 30, 31 and 54

This change shares blocks with change 30 (the list, the memory sentence, *Guided activation*) and with
change 31 (the listings, the coverage table). The rule changes 30 and 45 set holds for every block shared
by two changes: **whichever is implemented later re-copies the earlier's block and names it in
`archiveAfter`**. This proposal copies changes 30's and 31's blocks, today's latest holders, and the
programme's order puts changes 30 and 31 first (recommended).

What decides is the order of **archiving**, not of implementing: change 31 archives after change 30
whatever order they are implemented in (change 31's D8). So:
- **Changes 30 and 31 implemented first** (recommended): this change copies their blocks as they then
  stand and archives after both.
- **This change implemented before change 30**, whether change 31 is implemented or not: this change
  archives before both. It re-copies the list and the memory sentence from change 45, *Guided
  activation* from change 25, the listings from change 46 and the coverage table from change 45, and
  drops both changes 30 and 31 from `archiveAfter`; its list reads without it-es. Its pull request
  amends change 31's delta to carry this change's listings and table scenarios and to name this change
  in `archiveAfter` (change 31 being implemented or not); change 30 re-copies this change's blocks at its
  own implementation and names it. The archive order is this change, then change 30, then change 31.
- **This change implemented after change 30 but before change 31**: change 30 archives first; this
  change copies change 30's blocks, takes the listings from change 46 and the table from change 45,
  drops change 31 from `archiveAfter`, and amends change 31's delta as above. The archive order is change
  30, this change, then change 31.

Change 54 (es-it) shares the same blocks and comes after this change in the programme's order
(recommended); if it is implemented first, this change re-copies its blocks by the same rule, and French
is offered after English and Spanish in the list's order.

## Risks / Trade-offs

- **fr-it reads thinner than en-it** → its floor, change 48 before the list, the dictionary sentence in the
  listings (D7).
- **Italian texts nobody on the team reads** → the reviewer agent (I2) and change 44's glossary.
- **The activation page scrolling on an iPhone** → the dogfood reads it (D6).
- **Translation short of the tier** → M15 at change 49's figures (D4).

## Migration Plan

Nothing to migrate: French is offered, never added. Released after change 45's release. Rollback is
change 45's release, which knows `fr` and `it` but lists no fr-it: an Italian reader's French is left out
of what it accepts, and kept in the backup, until this release returns.

## Effort

4.75–9.5 ideal days, against the row's 3–6 (raised: changes 30 and 31 together were designed at 4.6–9.25):

| Part | Ideal days |
|---|---|
| The list, the tests that read it, the coverage write, the fixtures | 0.75–1.25 |
| The Italian reader's offer, memory and residency tests | 0.5–1 |
| `_locales/it` and `copy.js` with their tests | 0.25–0.5 |
| The listings | 0.75–1.25 |
| The captures taken again | 0.25–0.75 |
| The site's scenarios and tests | 0.25–0.5 |
| The spec blocks | 0.25–0.5 |
| The reviewer agent's pass | 0.25–0.5 |
| The checks | 0.25–0.5 |
| The M15 branch | 0–0.25 |
| The dogfood | 1.25–2.5 |

## Open Questions

For the owner:
1. **fr-it after its refinement** (D2): listed only once change 48 has merged. Recommended.
2. **fr-it under 75 % correct or over 30 % withheld** (D4), at change 49's figures: offered without a mark,
   or not offered.
3. **The description** (D6): the recommended draft or the other; the reviewer agent settles the wording
   (I2).
4. **Keywords and « What's New »** (D7): the most general keyword gives way to `francese`; the voice
   clause kept. Recommended.
5. **The release** (D11): on its own, TestFlight first, after change 45's. Recommended.
