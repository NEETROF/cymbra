# Design — review-lingua-italian-wording

## Context

**The Italian surfaces, and who wrote each:**

| Surface | Change | Where |
|---|---|---|
| The extension's interface, 23 modules, the in-app review page among them (`app`) | 33 | `apps/lingua-extension/src/i18n/it/` |
| The card's grammar | 34 | `src/i18n/it/grammar.ts` |
| The manifest's description and commands | 39 | `apps/lingua-extension/_locales/it/messages.json` |
| The Safari app's activation page and sign-in sheets (the app's own sheet uses 7 of `SignInCopy`'s entries) | 40 | `apps/lingua-apple/Shared (App)/Resources/copy.js`, `LinguaSignIn`'s `SignInCopy` |
| The site's `/it/` pages, islands (the leaving page's six answers among them) and Lingua text | 41 | `apps/site/src/pages/it/`, `src/lib/{i18n,lingua-text,music-media,stores}.ts`, `src/layouts/{Base,Legal}.astro`, `src/components/{Community,Downloads}.astro` |
| The Lingua page's captions, and the captures they describe | 45 | `lingua-text.ts`'s `LINGUA_TEXT.it.showcase`; `apps/site/src/assets/lingua/it/*`, `public/media/lingua/it/demo.mp4`, `store-captures/it/` |
| The store listings, and the Italian labels quoted in their shared English fields | 46 | `apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-apple/STORE-LISTING.md` |

Each pull request had the reviewer agent read its own strings (I2). Together they hold about 11,900
words:

| Surface | Words |
|---|---|
| The interface's strings (change 33's measure) | 3,631, or 3,772 with `grammar.ts` |
| The site | 6,800 |
| The listings | 1,070 |
| The Safari app | 250 |

**What the precedent did.** Matrix change 33 (`review-lingua-interface-translations`) was the owner reading
the English and Spanish interface on 2026-10-11, with no correction asked. It wrote no spec.

**Cross-surface checks already in place:**
- change 39 holds `_locales/it`'s commands to Réglages' words;
- `apple-activation-page.spec.ts` holds `copy.js`'s step 3 to `drawer.settings` and `settings.tabLanguage`;
- the site's `lingua-text.spec.ts` holds the Lingua page to `card.ts`'s buttons and the captures' labels,
  which change 45 extends to Italian.

**Known inconsistencies.** Change 41's review found Music's: « Modalità Attesa » in its store copy against
« Modalità attesa » in its app, and « Wait Mode » left in English. Only the site's `/it/music/` meets them.

## Goals / Non-Goals

**Goals:**
- One Italian term per product concept across every surface, kept so by a test where the text is keyed
  messages.
- Every Italian surface read together once, and corrected before the build the owner dogfoods and
  photographs.

**Non-Goals:**
- Music's Italian app and listing, and Cymbra ID's Italian e-mails (about 60 words, shared with Music):
  the pass reads the e-mails, under I10, but does not edit them.
- Dogfood corrections from Italian readers (change 55).
- Rewriting what the French source says: a finding that would change meaning is the owner's.

## Decisions

### D1 — The glossary

The glossary's data is machine-readable, inside the extension's package so that its checks run whenever
it changes: `apps/lingua-extension/test/italian-glossary.json`. Each concept holds:
- `id`;
- `fr`, the French forms;
- `it`, the accepted Italian forms;
- `itRejected`, the variants that must not appear;
- a note where Italian has a reason to differ.

`docs/lingua/italian-glossary.md` presents it.

The agent builds it from the surfaces as they stand. Where they disagree, the order of precedence is:
1. the words change 39 pinned (« riquadro laterale », « pannello di ripasso », « cattura la selezione »),
   so that its test and this one never pull apart;
2. then Music's `app_it.arb` for Cymbra-wide terms (« Accedi », « Esci », « Elimina account »), as the
   programme and change 33's D1 take it;
3. then the term the interface uses;
4. then a platform's own word where better known.

It records why each choice was made. It holds about thirty concepts. It is the reference for every later
Italian string: changes 31, 50, 54, 55 and 56, and any change adding Italian text.

### D2 — The pass

Change 33's reviewer agent reads every surface of the *Context* table and checks:
- I6: « tu », never « Lei »; « QCER »; Treccani's tense names; Italian numbers (« 20.000 », « 96% »), and
  « 5000 » under five digits (change 33's Open Question 1);
- an address free of gender (no « connesso/a », no « Benvenuto »);
- Italian typography: « » without inner spaces, no space before « : ; ? ! »;
- elisions and contractions where a message is built from parts;
- each field's limit: the stores' character and byte caps, and `_locales/it`'s 112 characters, the
  listings' English fields included where they quote Italian labels;
- the glossary's term for each concept;
- the listings' claims, against the files change 46's claims table names.

The reference differs by kind of text:
- a translation is read against its French;
- a text written per audience is read against what it says to whom: the listings, `_locales/it`'s
  description, `copy.js`'s `lede` and `step3`, `LINGUA_TEXT.it` and the home card, all written for
  Italian speakers studying English. Its reference is change 46's D1, modelled on the Spanish text
  without fr-es, and its claims table.

Each finding is applied in this pull request, or answered with a source. A finding that would change
meaning goes to the owner, with the French it departs from. `docs/lingua/italian-wording-review.md`
records:
- each surface read;
- each finding, its line and its resolution;
- the count of strings read.

### D3 — A check that keeps the vocabulary one

`apps/lingua-extension/test/italian-glossary.spec.ts` reads the glossary's JSON.

**The texts compared:**
- **The catalogue's text surfaces**, through the parity test's `texts()`, extracted from
  `test/i18n.spec.ts` into `test/support/`. Each message is called with the French message's number of
  placeholder arguments, and keyed by its path (`review.remaining.other()`, the Italian `.many()` mapped to
  the French `.other()`). `grammar.ts` is left out: it renders tags, not text, and change 34's snapshot
  and the pass cover it.
- **`_locales/it`**: each `message` by key, never its `description`.
- **`copy.js`**: loaded as `apple-activation-page.spec.ts` loads it, its fragments' markup stripped.

Keys written per audience (D2's list) are excluded from rule 1 by design, not as exceptions; rule 2 still
reads them.

**Normalisation**: NFC; U+2019 read as `'`; no-break spaces and runs of whitespace read as one space; the
brand « Cymbra Lingua » masked, so that « Lingua » the settings tab and « Lingua » the brand never meet.

**Matching**: a form matches whole, case-free, by letters rather than by JavaScript's ASCII `\b`:
`(?<![\p{L}\p{M}\p{N}])(form|…)(?![\p{L}\p{M}\p{N}])`, flags `iu`.

**The rules**, each failing with its key and its concept:
1. For each key shared with the French, where the French matches a concept's French forms, the Italian
   matches one of its Italian forms.
2. No covered Italian text matches a rejected variant. This catches drift where the French has no
   anchor.
3. Every concept matches at least one French text, and an exception that no longer applies fails, as the
   labels lint's baseline does.

**Read by the pass, not the test**:
- the site's prose pages;
- its islands' dictionary and `LINGUA_TEXT`, keyed but in another package;
- `SignInCopy`, Swift.

### D4 — When

- **Merged after change 45's pull request**: no stacking, and change 45's captions and captures exist.
- **Before the build the owner dogfoods and takes the App Store shots from**, and before change 45's
  release. Its commit is a `fix` touching the extension, the Safari app and the site, landing before the
  release pull requests of change 45's release are merged, so the release carries it.
- **A correction to a string a capture shows** re-runs `yarn capture` in its Italian one-language mode
  (change 45's D5), and the refreshed site media and `store-captures/it/` are committed in this pull
  request.
- **Snapshots**: a correction to the card or its grammar re-blesses change 34's grammar snapshot and
  change 36's en-it card snapshot.
- **Deploy**: the site's corrections reach `/it/` with the owner's site deploy (I12). The slugs change 41
  pinned do not change.

## Risks / Trade-offs

- **An agent reviewing its own family's work** → the glossary and the report are committed and the owner
  reads the summary. The test makes the vocabulary mechanical where the text is keyed messages. Change 55
  takes readers' corrections.
- **A glossary too rigid for a term that must vary** → the accepted forms are lists, and a key written
  per audience is out by design.
- **Text written after this pass** → the glossary is the reference, and the test fails a drifted term in
  the extension's catalogue, `_locales/it` and `copy.js`.
- **Published captures showing uncorrected text** → D4's re-capture.

## Migration Plan

The corrections ship with change 45's release. The site's reach `/it/` with the owner's deploy. Rollback is
a revert, the pinned slugs unchanged.

## Effort

2.5–3.75 ideal days, against the programme's 2–4:

| Part | Ideal days |
|---|---|
| The glossary, its JSON and its page | 0.5 |
| The pass over about 11,900 words, and its fixes | 1–1.5 |
| The test, `texts()` extracted, its rules and exceptions | 0.5–0.75 |
| A re-capture where needed, the snapshots | 0.25–0.5 |
| The report | 0.25–0.5 |
