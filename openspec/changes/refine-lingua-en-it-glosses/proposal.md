# refine-lingua-en-it-glosses — en-it's glosses open on the Italian word of the English word's commonest use, before Italian speakers read English

## Why

Change 37 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3. Change 36
(`add-lingua-pack-en-it`, #1081) committed en-it's glosses on 2026-10-26 — 21,526 rows, coverage
**91.4 / 82.8 / 70.3 %** against the floor 89.4 / 80.8 / 68.3 — and handed this change its *Known
defects* (its D10). A specialist's review of the committed tables (2026-10-26) recommended merging them
as the base for this change, which must answer it before change 45 offers English to Italian speakers:
**23 of the 300 commonest rows wrong (7.7 %)** and 59 partly right, about 3 % wrong past them, and
issues A–I.

Measured on the committed tables and on en-it's pinned sources (design, *Measured*; criteria committed
before the figures):
- **A minor part of speech first** (A): the section orders its senses by part of speech, not by use —
  `but` « solo, solamente; ma, però », `make` « marca; fare… », `mean` « media; …; voler dire »,
  `country` « campagnolo; … », `go` « l'atto dell'andare; … », `very` « autentico, vero… ».
- **The main sense missing** (B): `can` « latta, lattina… » without « potere », `prove` « prove »,
  `course` « quotazione… », `down` « abbasso, a basso », `lot` without « molto ».
- **Descriptions in place of a translation** (C): `an` « articolo indeterminativo… », `me`
  « complemento oggetto di I… », `my` « forma possessiva di I usata come…, il mio… », `its`
  « possessivo di it »; given names' notes (`john`) and names on common words (`summer`, `buddy`).
- **Text that is no translation** (D–H): pointers (« vedi are »), heraldry in the middle of a row
  (`side`), English expansions (`uk` « United Kingdom : Regno Unito »), wiki text, a split form (`mx`
  « signorə, signor, a »), repeated senses (`national` « nazionale; nazionale »).
- **Wrong words** (F, I): misspellings (« squaglia », « sù », « grupo », « referenzia »), a page copied
  from another word (`some` « molti, tanti; la folla »), false friends (`disposable` « disponibile »,
  `editor` « editore/editrice », `fabric` « ordito »).
- **Contractions**: 57 of the 5,000 commonest unglossed (`you're`, `that's`), `doesn't` reading
  `don't`'s « contrazione di do not ».

## What Changes

Every rule is en-it's, in a new module `reduce_english_it.py` and `reduce-en-it.py`, which no other pair
loads; `reduce_common.py` and the Italian edition's rules do not move (design D1).

- **The order of a row** (D2): a row opens on the part of speech that the English Wiktionary lists first
  for the word *and* whose table words Italian writes most, a verb never yielding to a noun — 199 rows
  (`but` « ma, però; … », `make` « fare, creare, fabbricare; … », `mean` « voler dire, significare; … »,
  `country` « paese, terreno, campagna; … », `go` « andare; … »).
- **The missing main sense** (D3): the table's first sense opens a row that lacks that part of speech
  (`prove` « dimostrare, provare; … », `treat`, `rescue`) or that only describes (`its` « suo; … »,
  `should` « dovere; … », `trash` « spazzatura… »): 152 rows.
- **`FIRST_RUN`** (D4): 33 reviewed rows for the commonest grammatical words and the rows no rule tells
  apart (`that` « quello, quella; che… », `can` « potere; … », `an` « un, uno, una, un' », `me`
  « mi, me », `down` « giù; … », `still` « ancora, tuttora; … »), every word a source's.
- **Clean-up** (D5–D10, D13): frames out (`my` « il mio, la mia… »), names' notes off common words and
  given names short (`john` « Giovanni (nome) »), pointers, heraldry anywhere and tinctures, English
  expansions (`uk` « Regno Unito »), wiki text, a state's code beside another sense, whole repeated
  senses, split forms (`mx` « signor/a »).
- **`CORRECTIONS`** (D12): 21 reviewed rows — the false friends (`disposable` « monouso, usa e
  getta »), the copied page (`some` « qualche, alcuni, alcune; … »), English text (`media`), nine
  misspellings —, every word a source's.
- **Contractions** (D11, task 0.1): read as the pointers the section writes for those it glosses —
  `you're` « contrazione di you are », `doesn't` « contrazione di does not »: 58 lemmas gained.
- **The card in Italian** (D15): change 36's open task 4.3, `word-card-en-it.spec.ts` and its snapshot,
  now that changes 33 and 34 are on `main`, re-blessed with the golden.
- **One re-reduction** (D14): together, without D11, **1,005 rows change** (138 / 301 / 400 / 166), 587
  in their first sense, no lemma gained or lost, coverage **91.4 / 82.8 / 70.3 %** as now; with D11,
  1,064 rows, 58 lemmas gained, **92.0 / 83.2 / 70.6 %**. On the review's judged rows, the 300 commonest
  read wrong **0.7 %** (from 7.7 %) — `lot` and `doesn't` —, 0 with the owner's two answers, partly
  right about 11 % (from 19.7 %), as this design's reader reads them, for the specialist to confirm.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *en-it's glosses open on the Italian word of the English word's commonest use*, *en-it's
    glosses hold Italian words, without frames, notes, pointers, heraldry or repeats* and *en-it
    corrects, by name, what its sources write wrong*.
  - MODIFIED, copied whole from the open change that holds it, *English is glossed in Italian from the
    Italian Wiktionary's English section and the two Wiktionaries' translation tables*
    (`add-lingua-pack-en-it`): the first source glosses a lemma but where en-it's rules reorder or open
    its row; a gloss a reviewed table writes is a source's or marked as a person's; en-it's own module's
    digest scenario.
  - This change archives after `add-lingua-pack-en-it` (`archiveAfter`). Changes 47 and 51 hold their
    own copies of that requirement: whichever of the three is implemented later re-copies the other's
    block and names it in `archiveAfter` (D16).
- `lingua-analysis`: none.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *new*: `reduce_english_it.py`; *changed*: `reduce-en-it.py`,
    `test_reduce_en_it.py`, `test_reduce_editions.py` (en-it's rule files); `tables/en-it/`
    (`gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json`, `pin.json`, `README.md` by the dispatch);
    `SOURCES.md`; *consumed, unchanged*: `reduce_common.py`, `reduce_edition_it.py`, `pack_sources.py`,
    `gloss_coverage.py`.
  - `crates/lingua-wasm/tests/baseline/en-it.golden` re-blessed; `crates/lingua-pack/tests/committed_tables.rs`
    with en-it's rule files.
  - `apps/lingua-extension/test/word-card-en-it.spec.ts` and `test/baseline/word-card-en-it.txt` (new,
    change 36's task 4.3); `.github/workflows/lingua-pack-update.yml`'s bless step.
  - ID, Music, Live, the back office, the site, the backend, lingua-core and the engine, the extension's
    source, the Apple host app and the agent plugin are untouched.
- **What does not move.** `tables/en/` and every other pair — tables, pins, packs, goldens and
  snapshots — byte for byte; the shared and Italian edition rules; `FLOORS["en-it"]`; en-it's sources.
- **Release.** Silent: no package lists en-it before change 45.
- **Order.** After change 36 (on `main`, its requirement open); before change 45; either side of
  changes 47 and 51 (D16).
- **The owner's**: two questions (tasks 0.1–0.2: contractions, `lot`); the gloss diff after the
  specialist's review, the golden's and the card's moved lines (task 4.1).
- **Effort**: 4–5.5 ideal days (design, *Effort*), against the programme's 2–4.
