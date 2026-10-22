# Design — review-lingua-italian-wording

## Context

**The Italian surfaces, and who wrote each:**

| Surface | Change | Where |
|---|---|---|
| The extension's interface, 23 modules | 33 | `apps/lingua-extension/src/i18n/it/` |
| The card's grammar | 34 | `src/i18n/it/grammar.ts` |
| The manifest's description and commands | 39 | `_locales/it/messages.json` |
| The Safari app's activation page and sign-in sheet | 40 | `apps/lingua-apple/Shared (App)/Resources/copy.js`, `LinguaSignIn`'s `SignInCopy` |
| The site's `/it/` pages, islands and Lingua text | 41 | `apps/site/src/pages/it/`, `src/lib/i18n.ts`, `lingua-text.ts` |
| The Lingua page's captions | 45 | `lingua-text.ts`'s `LINGUA_TEXT.it.showcase` |
| The store listings | 46 | `apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-apple/STORE-LISTING.md` |

Each pull request had the reviewer agent read its own strings (I2).

**What the precedent did.** Matrix change 33 (`review-lingua-interface-translations`) was the owner reading
the English and Spanish interface on 2026-10-11, with no correction asked. It wrote no spec.

**What is known to drift.** Change 39 held `_locales/it`'s commands to Réglages' words by a test, the
only cross-surface check in place. Change 46's review found the Music app's own inconsistencies
(« Modalità Attesa » and « Modalità attesa », « Wait Mode » left in English).

## Goals / Non-Goals

**Goals:**
- One Italian term per product concept across every surface, kept so by a test where the text is code.
- Every Italian surface read together once, before an Italian reader sees it.

**Non-Goals:**
- Music's Italian app and listing, and Cymbra ID's Italian e-mails: other products, with their own text.
- Dogfood corrections from Italian readers (change 55).
- Rewriting what the French source says: the pass translates meaning; a finding that would change it is
  the owner's.

## Decisions

### D1 — The glossary

`docs/lingua/italian-glossary.md` lists, per concept:
- the French source term;
- the Italian term;
- the surfaces that use it;
- a note where Italian has a reason to differ. For example, « riquadro laterale » is Chrome's word for
  its side panel (change 39).

The agent builds it from the surfaces as they stand. Where they disagree, it chooses the term the
interface uses, since the reader meets that first, unless a platform's own word is better known (Chrome's,
Firefox's, Apple's). It records why. It holds about thirty concepts:
- the panels and the card;
- the review and its buttons;
- the settings' tabs and named settings;
- the level scale;
- the translation features;
- the account, sign-in and data words.

It is the reference for every later Italian string: changes 50 and 54, change 55, and every change that
adds Italian text.

### D2 — The pass

Change 33's reviewer agent (`.claude/agents/lingua-italian-reviewer.md`) reads every surface of the
*Context* table against the French source and the glossary, and checks:
- I6: « tu », never « Lei »; « QCER »; Treccani's tense names; Italian numbers (« 20.000 », « 96% »,
  « 5000 »);
- an address free of gender (no « connesso/a », no « Benvenuto »);
- Italian typography: « » without inner spaces, no space before « : ; ? ! »;
- elisions and contractions where a message is built from parts;
- each field's limit: the stores' character and byte caps, `_locales/it`'s 112 characters;
- the glossary's term for each concept;
- the claims the listings make, against the files the claims table names (change 46).

Each finding is applied in this pull request, or answered with a source.
`docs/lingua/italian-wording-review.md` records:
- each surface read;
- each finding, its line and its resolution;
- the count of strings read.

A finding that would change meaning rather than wording goes to the owner, with the French it departs
from.

### D3 — A check that keeps the vocabulary one

`apps/lingua-extension/test/italian-glossary.spec.ts` reads the glossary's concepts (French term, Italian
term). For each Italian message whose French counterpart, under the same key, contains the French term,
it checks that the Italian contains the Italian term, case-free and as a whole word, and fails naming the
key and the concept. It covers:
- the extension's catalogue (`src/i18n/it/` against `src/i18n/fr/`);
- `_locales/it` against `_locales/fr`;
- the Safari app's `copy.js` (its `it` table against its `fr` table).

A reviewed list of exceptions in the glossary holds the keys where the Italian rightly says it otherwise,
each with its reason. The site's pages and the listings are prose, not keyed messages, so the pass reads
them and the test does not.

### D4 — When

After changes 33, 34, 39, 40, 41 and 46 are implemented and change 45's captions written, and before
change 45's release (programme row 44). Corrections land in this change's pull request, so change 45
ships the reviewed text. An Italian reader's later corrections are change 55's.

## Risks / Trade-offs

- **An agent reviewing its own family's work** → the glossary and the report are committed and the
  owner reads the summary; the test makes the vocabulary mechanical where the text is code; change 55
  takes the corrections of readers.
- **A glossary too rigid for a term that must vary** → the exceptions list, each with its reason.
- **Text written after this pass** → the glossary is the reference, and the test fails a drifted term in
  the extension's catalogue.

## Migration Plan

Nothing deploys. The corrections ship with change 45. Rollback is a revert.

## Effort

2–3.5 ideal days, against the programme's 2–4:

| Part | Ideal days |
|---|---|
| The glossary | 0.5 |
| The pass over about 17,000 Italian words (the interface about 8,900, the site about 6,800, the listings about 1,070, the Safari app about 250), and its fixes | 1–2 |
| The test and its exceptions | 0.25–0.5 |
| The report | 0.25–0.5 |
