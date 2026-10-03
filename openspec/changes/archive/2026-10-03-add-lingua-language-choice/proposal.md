# add-lingua-language-choice — the reader chooses the languages they study

## Why

The reader's studied languages live in their profile since `add-lingua-studied-language-profile`,
and the sync and the reading session follow them since changes 11 and 12. But nothing lets a reader
choose them, and the surfaces still speak of English in hard-coded French: « Niveau d'anglais »,
« Aucune voix anglaise », « Pas de texte anglais détecté », « Mon niveau d'anglais ». Two settings
that depend on the language are also shared between languages:
- **The level.** `needsLevelChoice` reads a level decided in any language as decided, so a Spanish
  level would hide the English choice (a trap the study found).
- **The voice.** The chosen voice is one voice, whatever the language it should read.

This is change 13 of the Spanish programme (`docs/lingua/spanish-programme.md`), in R3, a silent
English release. The package ships en-fr alone, so the choice of languages has nothing to offer and
stays hidden. Every label reads « anglais », as it does today.

## What Changes

- **One module names the languages** (`src/analyzer/language-labels.ts`): « Anglais » / « Espagnol »,
  « d'anglais » / « d'espagnol », « anglaise » / « espagnole ». A lint spec refuses « anglais » in any
  source or page outside it.
- **« Langues étudiées »**, a block of the one settings builder (every Réglages host). It shows a box
  per language the package ships, ticked for the ones the reader studies:
  - ticking adds a language after the others, unticking removes one, and the last one cannot be
    unticked;
  - the change is saved in the profile, and every surface follows;
  - the block is hidden when the package ships a single language.
- **A level per language.** Réglages shows a level block for each language the reader accepts,
  titled « Niveau d'anglais », « Niveau d'espagnol ». Each holds its chips, its hint and its
  calibration. `needsLevelChoice` reads the decisions of its own language only.
- **A voice per language.** The chosen voice is kept per language. The Réglages voice block edits the
  voice of the language its speaker reads: a page's language in the drawer, the reader's first
  language elsewhere. A voice kept before this change becomes the English one.
- **The other labels follow the language they speak of:**
  - the popup's « Pas de texte … détecté » and level prompt, for the page's language;
  - the statistics' level heading;
  - the « Aucune voix … » note.
- **Onboarding** offers the choice of languages first when the package ships several, then a level
  for each chosen language.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`, ADDED:
  - *The reader chooses the languages they study*;
  - *A level per studied language*;
  - *A voice per studied language*;
  - *Languages are named in one place*.

  The open changes on this capability hold requirements under other names: `add-lingua-read-aloud`
  holds the read-aloud ones, which this change does not rewrite.

## Impact

- **Products.** Cymbra Lingua's browser extension: settings, statistics, popup, onboarding,
  speech preference, and the level-choice rule. No engine, server or proto change: the profile, its
  setter and the per-language level and calibration exist already.
- **Release.** R3, silent: a single shipped language hides the choice, every label reads « anglais »,
  and a voice kept today stays the English one.
