# Design — enable-lingua-spanish

## Context

See proposal.md (Why).

- **`packs.json`** (`generalise-lingua-pack-build`) lists the pairs a package ships; its first pair
  gives the default studied language.
- **`tool/gen_pack.sh`** builds each listed pair's pack: from the committed tables, checked against
  the pair's pin (`--real`, releases); or from `scripts/lingua-data/testdata/<pair>`
  (`yarn gen:pack`, dogfood builds and the checks).
- **`check_variants`** holds the list to `SHIPPED_PAIRS`, en-fr alone, so that widening it takes
  two edits in one pull request.
- **The R3 surfaces** read the shipped pairs:
  - the language choice shows when two pairs ship (`studied-languages-view`, `offered.length < 2`
    hides it);
  - the per-language controls show once the reader accepts two languages: review
    (`showLanguage`), statistics, reading session.

## Goals / Non-Goals

**Goals:**
- Packages that ship Spanish reading beside English, English the default.
- Nothing changes for a reader who does not choose Spanish.

**Non-Goals:**
- Translation in Spanish: change 26 hosts the pivot model, and change 27 measures it per platform.
- Store listings (change 29) and the site's pages (change 30).
- Model-written glosses or curated locutions (`add-lingua-spanish-gloss-tables` D5).

## Decisions

### D1 — The list and the gate, together

`packs.json` becomes `["en-fr", "es-fr"]`, and `SHIPPED_PAIRS` becomes the same. `check_variants`
keeps its two checks:
- the list equals `SHIPPED_PAIRS`;
- every package carries exactly the listed packs, byte-identical across Chromium, Firefox and
  Safari.

The gate's message no longer names a change to come.

### D2 — A testdata fixture for es-fr

`scripts/lingua-data/testdata/es-fr/` mirrors en-fr's: a few hand-written rows per table, never
source data.
- **Forms**: `casa`, `casas`, `hablar`, `hablo`, `habla`, `ser`, `es`, `fue`, `vino`, `de`, `el`,
  `la`.
- **Tables**: their ranks, French glosses, grammar readings and gendered sense runs, and an
  estimated level table.
- **Manifest**: Spanish's analyser version, and `levels_estimated`.

`gen_pack.sh` refuses a listed pair without testdata, so the fixture is what keeps the checks and
the dogfood builds working.

### D3 — The manifest summary

It is the store's short text and the browser's extension description, held to 112 characters.

**Settled** by the owner on 2026-10-05 (109 characters): « Lisez l'anglais et l'espagnol sur le web :
mots inconnus surlignés, pourcentage honnête. Hors ligne et privé. »

It keeps the current summary's voice, adding Spanish. The wording is the owner's, as the listings' is
(decision D10).

### D4 — The dogfood pass, before merge

On each of the five targets: Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS),
Safari (iOS). The run uses a package built from this branch with the real packs
(`yarn gen:pack:real`).
1. With English alone: reading, the card, the statistics and review are as before.
2. Accept Spanish in « Langues étudiées »:
   - a Spanish page is highlighted;
   - an English page still is;
   - a Catalan page is not.
3. A Spanish word's card shows:
   - its grammar (« 1re et 3e personnes du singulier de l'imparfait de l'indicatif ») and its
     gender heading;
   - its French gloss;
   - its level, labelled estimated.
4. The level in Réglages, the ladder and onboarding read « estimé ».
5. The review's language filter and the statistics' language selector work, and a Spanish card
   syncs to a second device.
6. Read-aloud speaks Spanish with a voice of Spain where one is installed.

The owner runs the pass on their devices, with Claude where a session can drive the browser.

### D5 — The release, a beta ring first

The first packages carrying Spanish go to the stores' beta channels before any public release. The
owner decides each submission, and this pull request submits nothing.

### D6 — The language choice says several languages are free for now

The owner decided that a reader may study several languages at once, free for now, and that the
choice says so. « Langues étudiées » carries the line « Plusieurs langues à la fois : gratuit pour
l'instant. » under its boxes, wherever it shows: Réglages in every host, and onboarding, both from
`mountStudiedLanguages`.

The line says « pour l'instant », not « bêta »: the Safari app is distributed through the App Store,
whose review guideline 2.2 keeps betas off it.

## Risks / Trade-offs

- **Package size** +2.19 MB per package → within the stores' limits. A reader of English alone never
  loads the Spanish pack.
- **A Spanish reader without translation** → the card's gloss and grammar work. Translation follows
  per platform (changes 26 and 27), as decision D3 of the programme plans.
- **A re-reduction needs the kaikki snapshot** → nothing in a package depends on it. The release
  named in `tables/es-fr/pin.json` is published (`lingua-pack-sources-es-fr-2026.10.03`, 2026-10-05).
