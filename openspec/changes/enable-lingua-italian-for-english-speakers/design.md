# Design — enable-lingua-italian-for-english-speakers

## Context

Change 25 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: English
speakers learn Italian through it-en. Change 20 (`enable-lingua-italian`) did what Italian needed once
— the extension's studied-language type, the profile's name, `GUARDED.it`, online translation's
backend, backup schema 4 in readers' hands — and listed it-fr. What is left is it-en's: the list, the
English description and activation page, the memory sentence's English cases, and the dogfood in the
English interface. Each precondition below is a fact only once its change is implemented:

- it-en's native side committed at or above its floor (change 22), its known defects fixed (change 23,
  `refine-lingua-it-en-glosses`), its marks measured (change 24);
- change 20 implemented and released — the release that ships it-fr reads backup schema 4 and carries
  the extension's Italian;
- the English catalogue names Italian (change 15: "Italian", "Estimated Italian level" from
  `levelTitleEstimated`), the card names Italian's forms in English (change 18: "past historic
  (passato remoto)");
- the site describes Italian (change 21: `names.it`, `STUDIED.it`), so its build takes an it-en row.

## Goals / Non-Goals

**Goals:**
- it-en listed; Italian offered to English speakers after Spanish and French; an Italian page read,
  glossed, recorded, read aloud and translated for them.
- The English description and the host app's English activation page name Italian.
- Nothing moves for French- and Spanish-native readers, nor for it-fr.

**Non-Goals:**
- it-es (change 30), the Italian interface (stage 3).
- The English listings and the site's English home card (change 26,
  `add-lingua-italian-english-listings`).
- Anything change 20 already did.

## Decisions

### D1 — The list, it-en last

`packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es", "it-fr", "it-en"]` and
`SHIPPED_PAIRS` the same. An English native's default pair stays es-en; Italian is offered after
French. `yarn gen:pack:real` builds it-en's layer against its pin, the Italian base already there;
`yarn gen:pack` needs `scripts/lingua-data/testdata/it-en/`, hand-written in `testdata/it-fr/`'s
layout and glossed in English, as `enable-lingua-french` wrote `testdata/fr-es/`.

### D2 — it-en by its floor, after its refinement

Change 22 commits it-en at or above `FLOORS["it-en"]`; `test_gloss_coverage.py` holds every listed
pair with a floor at or above it. it-en is listed once change 23 has merged, as change 20 lists it-fr
after change 17 (the owner's answer to change 20's Open Question 1 applies here too).

### D3 — The extension: no new type, one new offer

Nothing is typed: `StudiedLanguage` holds `it` since change 20. `shippedLanguages(pairsOf("en"))`
now gives Spanish, French and Italian; the level block reads "Estimated Italian level" with its note;
the card names Italian's tenses in English; `GUARDED.it` keeps the neighbours' pages out as it does
for a French reader; online translation already accepts it-en (change 20's D5).

### D4 — Translation: one model

it-en is the it-en model alone (change 2), soaked by change 19; marked if change 24 put it on the
first tier, else offered without a mark unless the owner decided otherwise at change 24's figures
(M15). The model host serves it-en since change 20's release.

### D5 — The memory sentence's English cases

An English reader of Italian alone needs one model: "about 200 MB", the download it-en's served size.
An English reader of Spanish, French and Italian needs three (es-en, fr-en, it-en), held two at a time:
"about 340 MB", the download their sum (es-en 26,241,052 B and fr-en 26,234,715 B today; it-en's from
the catalogue, ≈ 26.3 MB by change 20's estimate, so ≈ 78.8 MB). Change 20's requirement already reads
« two models or more »; this change adds the two English scenarios.

### D6 — The English description and activation page

- **`_locales/en`'s `extensionDescription`** names Italian — `check_version`'s rule fails this pull
  request until it does. « Read Spanish, French and Italian on the web: … » is 114 characters, over the
  112 limit; drafts (M9):

  | | Draft | Characters |
  |---|---|---|
  | recommended, as change 20's French | "The web in Spanish, French or Italian: unknown words highlighted, an honest percentage. Offline and private." | 108 |
  | keeps "Read" | "Read Spanish, French and Italian on the web: unknown words highlighted, an honest score. Offline and private." | 109 |

- **The activation page's `en`** `lede` and `step3` (M9):

  | | Draft |
  |---|---|
  | `lede` | "Read the web in Spanish, French or Italian: the words you don't know yet are highlighted, right in Safari." |
  | `step3` | "In Safari, open the extension from the <strong>address bar menu</strong> and choose your level of Spanish. For French or Italian, check it in the extension's <strong>Settings</strong>, under <strong>Language</strong>, then choose its level." |

  The French and Spanish pages do not move; `test/apple-activation-page.spec.ts` follows (the English
  page names Spanish, French and Italian, in that order).

### D7 — The published coverage, and the site

`gloss_coverage.py --write` publishes it-en's figures; the site's tests require the file's pairs to be
the list's. The English Lingua page names Italian for English speakers with no code change (change
21's tables); the site's fixtures are refreshed from the built pages and
`test/astro/lingua-page.spec.ts` counts eight rows, as change 20 did for seven. The English home card's
requirement (`add-lingua-french-listings`' *The English home page describes Lingua for the readers it
serves*) is change 26's to modify, as change 21 modified the French one.

### D8 — What French- and Spanish-native readers see: nothing

Their pairs, offers, downloads and pages are what they were; the package carries it-en's layer, which
only an English-native reader who ticks Italian fetches.

### D9 — Tests and documents

The tests that read the list (change 20's seven: `packs.spec.ts`, `pairs.spec.ts`,
`manifests.spec.ts`, `model-manifest.spec.ts`, `studied-languages-view.spec.ts`,
`check-version.spec.ts`, `apple-activation-page.spec.ts`), each keeping the earlier lists as explicit
arguments; `engine.spec.ts` — an English reader fetches the Italian base and it-en's layer;
`translate-relay.spec.ts` — it-en marked if change 24 listed it; `translation-setting.spec.ts` and
`model-controller.spec.ts` for D5; `test_gloss_coverage.py`'s `ListedFloorsTest`;
`committed_tables.rs` — change 22's *Not shipped*, which holds only while the list does not name it-en.
Documents: `README.md`, `REVIEWERS.md`, `TRANSLATION.md`, `tool/marks/README.md`, the comments that
name `enable-lingua-italian-for-english-speakers` as the future.

### D10 — The dogfood pass, before merge

Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), in the English interface, on test
accounts: a new install in English offers Spanish, French and Italian; "Estimated Italian level";
Italian pages (news, a novel, « al mare », « dell'uomo », « un po' », « perche' »); the card in English
("past historic (passato remoto)"); an expression card; read-aloud with change 15's voices; "Extended
translation" with Italian alone ("about 200 MB") and with three languages ("about 340 MB"), the
worker's memory read; a French- and a Spanish-native install unchanged; sync across two English-native
devices; the library; the host app's English page. The owner runs it; findings to the pull request.

### D11 — The release

On its own, TestFlight first, with change 26's listings; the site deployed after the merge with it-en's
figures, before the listings are pasted; each submission the owner's (I12, M18).

### D12 — Requirements true of their time

Change 20 holds three requirements this change widens: *The shipped pairs are one list* (it-en last; *A
list widened too early*, *Building the listed packs* and *French for English speakers alone* name it;
*No Italian for English and Spanish speakers* becomes « while it-fr is the one listed pair studying
Italian »; *English speakers learning Italian* and *No Italian for Spanish speakers* added), *The
setting's memory follows the models…* (two English scenarios added) and *Guided activation* (*Every
language an English speaker can study* holds « while it-en is not listed »; *Italian for an English
speaker* added). Each MODIFIED block keeps every other word and scenario and is re-copied from change 20
at implementation. Change 22's *Not shipped* is worded « while the list does not name it-en » in change
22 itself, so it needs no block here.

## Risks / Trade-offs

- **it-en's glosses read wrong in places** → change 23 before the list (D2); the dogfood.
- **The English description's shape** → the owner's words (M9), within 112 characters.
- **Package size** → one layer more (it-en's), the Italian base already carried.

## Migration Plan

Nothing to migrate; Italian is offered, never added. Released by the owner after change 20's release.
Rollback: the previous release, which reads an Italian reader's state (change 20's D4).

## Effort

Against 4–7.5 ideal days: the list, the fixture and the tests that read it 0.75–1.25; the offer's and
the memory sentence's tests 0.5–1; the description, the activation page and the documents 0.25–0.5;
the checks 0.25–0.5; the dogfood 1.25–2.5; the spec and the programme 0.25. 3.25–6.

## Open Questions

For the owner:
1. **The description and the activation page** (D6): the recommended draft, the one that keeps "Read",
   or other words (M9).
2. **it-en below the marks' tier** (D4), at change 24's figures: offered without a mark (M15's
   default) or not offered.
