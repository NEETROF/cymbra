# ignore-lingua-soft-hyphens — a soft hyphen is not part of a word

## Why

Change 42d of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside the 57. Change 42b's review (`add-lingua-spanish-occitan-guard`, its open question 5) found
it, and the owner made it a change of its own on 2026-10-10.

Some e-books hide a soft hyphen (U+00AD) inside their words so that a line can break there: « vida »
is stored as « vi‧da » (here and below, ‧ marks a soft hyphen, which is itself invisible). calibre's
*Add soft hyphens* puts one at every syllable break, and web pages write `&shy;` for the same
purpose. Nothing in Lingua strips it, and the core reads the word in two contradictory ways: the
tokeniser keeps « vi‧da » as one word but looks it up with the hyphen inside, which no pack holds;
the detector and the guards cut it into syllables. Measured on 580,268 sentences and paragraphs from the corpora of changes 42
and 42b and the English Wiktionary, hyphenated as calibre does (macOS's hyphenation for French, which
calibre lacks), against today's `main` (design, Measurement):

- **Spanish lines are set aside.** 37.0 % of the Spanish blocks kept today are no longer analysed
  once hyphenated: 34.7 % of the text, 26.5 % of the books' paragraphs, 31.5 % of Wikipedia's. Of the
  167,524 blocks lost, the guard refuses 130,301, because « to‧do », « vi‧da », « ca‧lle » give
  Galician `do`, `da`, `lle`, and « ma‧ña‧na », « co‧sas », « va‧lor » give Occitan `ma`, `sas`,
  `lor`. The detector refuses 37,223 more, 33,203 of them read as Portuguese.
- **No hyphenated word is recognised, in any language.** « gov‧ern‧ment », « tou‧jours » and
  « can‧tá‧ba‧mos » are unknown and unglossed, whatever the reader knows. Each is keyed by a dictionary
  form holding the invisible character: `tou‧jour`, `can‧tá‧ba‧mo`. « lors‧qu’il » is not split, and
  « could‧n't » becomes `could‧` + `not`. A reader who knows the 5,000 commonest words reads 92.0 %
  of Spanish pages as known today and 54.9 % once they are hyphenated; French 90.2 → 71.3 %;
  English 85.4 → 71.3 %. Half the Spanish tokens hold a soft hyphen, 30.5 % of the French ones and
  25.7 % of the English ones.
- **French and English blocks are lost too.** French loses 2.7 %, 957 of them to its guard
  (« chan‧té » gives Catalan `té`, « lo‧ge‧ment » Occitan `lo`). English loses 0.7 %, read as French,
  Portuguese or German. Among English and French, 63 of 17,950 English pages are given to French.
- **The extension adds its own readings.** A selection that stops inside « vi‧da » snaps to « vi »
  or « da » alone. A card the core does not key, a loose word or an expression, is saved under the raw
  selected text, invisible character included. The sentence a card keeps, translates and reads
  aloud carries the hyphens too.

A reader who opens such an e-book in Lingua's reader sees most words highlighted as unknown, a
meaningless percentage, and in Spanish whole paragraphs left unread.

## What Changes

- **The core reads a word without its soft hyphens** (D1, D2): the tokeniser's text for every token
  — page analysis, phrase gloss, word card — is the word without U+00AD, in English, Spanish and
  French alike, before any rule reads it: the shared rules (edge apostrophes, digits, single letters,
  hyphenated compounds), English's `n't`, Spanish's NFC and `al`/`del`, French's elisions, `au`/`aux`,
  listed runs and inversions. Its span stays the source's, soft hyphens included, as a French token's
  composed text already keeps its source span: the extension's highlights need no change.
- **Detection reads the block without its soft hyphens** (D3): the gate, the vote, the minimum length
  and its weight, whichlang and the three guards. One place, `block_is_studied` and
  `detect_document_language`, and the guards' own word readings are untouched.
- **The extension stops cutting at the hyphen** (D5): its snapping takes U+00AD as part of a word.
  The selection's text, the word as written and the sentence with its selection span are read
  without soft hyphens, so the cards, the deck, translation and read-aloud are handed none.
  Highlights and hit-testing are unchanged, since the core's spans are the page's.
- **Measured with the change** (a prototype on a scratch checkout of `main`): every hyphenated
  block, page, vote, phrase gloss and word card answers exactly as the same text without soft
  hyphens. That holds for every block of the five corpora (605,753, the Galician and Catalan ones
  among them) and all 43,374 pages, spans aside, and each span of a hyphenated word covers it as
  written. On text without soft hyphens nothing moves: the gate's answers, the page analyses' JSON
  and the votes over every corpus have the same digests as on `main`, and the 689 tests of
  lingua-core, lingua-pack and lingua-wasm pass without re-blessing, the five goldens among them.
  The cost on such text is within 2 % of today's page analysis, native.
- **Every analyser version is bumped** (D6): English `1.1.0` → `1.2.0`, Spanish `1.3.0` → `1.4.0`
  (`1.4.0` → `1.5.0` as implemented, change 42c having bumped it first), French `1.1.0` → `1.2.0`. The change alters each language's output on text that holds a soft
  hyphen.
- **What moves** (D7), measured with the bump: in the five goldens, the version alone, on 16, 16,
  19, 18 and 19 lines (en-fr, en-es, es-fr, es-en, fr-en). Every committed pair's `manifest.json`
  and `pin.json` move (sha256 only, sizes unchanged), and so do the five fixtures' manifests and the
  pipeline test's recorded bytes, the parity fixture pack and its `golden.json`, the extension's
  and the agent's fixture packs, and the tests that write a version out. en-fr's and es-fr's output
  moving needs the owner's approval, in this change's pull request.
- **Requirements**: ADDED, *A soft hyphen is not part of a word* (`lingua-analysis`) and *A soft
  hyphen does not cut the reader's word* (`lingua-browser-extension`). Five requirements that name
  English's `1.1.0` are MODIFIED to name English's own version and nothing else (D9). Two of them
  are held by `add-lingua-french-analysis`, after which this change archives.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A soft hyphen is not part of a word*. MODIFIED, each on its version
  literal alone:
  - *An analyser version per studied language* — « English's analyser version SHALL remain `1.1.0`
    through this change » goes, a rule shared by every language bumps every version, and its
    scenario names English's own version;
  - *Analysis by studied language* — scenario *English output does not move*;
  - *A word's grammar, from the pack* — scenario *The page analysis does not move*;
  - *French is a studied language served by its own analysis* — scenario *Each language reports its
    own version*. This requirement is held by `add-lingua-french-analysis`, hence `archiveAfter`.
- `lingua-data-packs`: MODIFIED — *A pack names the language it studies*, scenarios *Loading the
  EN→FR pack names English* and *Versions are compared within a language*. Held by
  `add-lingua-french-analysis` too.
- `lingua-browser-extension`: ADDED — *A soft hyphen does not cut the reader's word*.

It also archives after `add-lingua-french-detection-guard` and `add-lingua-spanish-occitan-guard`,
whose guards it reads around. No other open change holds a requirement it touches.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core`:
    - `analysis/tokenize.rs`: `without_soft_hyphens`, read in `push_word`, `push_compound`,
      `french_lowercase`, the next word of an elision and an inversion's pieces;
    - `analysis/language.rs`: `block_is_studied`, `detect_document_language`;
    - `engine.rs`: `word_grammar`'s fallback;
    - `analysis/mod.rs`: the three versions, plus tests.
  - `crates/lingua-wasm`: the five goldens and the parity fixture re-blessed on the version, with
    their version literals.
  - `crates/lingua-pack`: the testdata digests.
  - `scripts/lingua-data`:
    - `tables/{en-fr,en-es,es-fr,es-en,fr-en}/` re-reduced, `manifest.json` and `pin.json` alone
      moving;
    - `testdata/*/manifest.json`.
  - `apps/lingua-agent/rust/tests/fixtures/`: both packs (test inputs; the agent is otherwise
    outside the programme, M17).
  - `apps/lingua-extension`:
    - `src/reading/selection.ts` (snapping, the captured text, the sentence) and
      `src/reading/session.ts` (the word as written);
    - `test/fixtures/en-fr.testdata.lingua`, plus tests.

  ID, Music, Live, the back office, the site, the backend and the Apple host app are untouched. No
  table row, wire field, proto or stored format changes, and no server change.
- **Release.** Every pack is rebuilt by the extension's next release. A reader of a hyphenated
  e-book or page then sees it read as its words. Nothing else a reader sees changes. A word saved
  before under a key holding the invisible character keeps it (open question 1).
- **Order.** After changes 41b, 42b and 44b (all merged), on `main`'s versions. Change 49
  (`add-lingua-pack-fr-es`) names French's version in its manifest: landing first, its fr-es is
  re-reduced here too; landing second, it reads the new version. Drafts #810 and #814 touch `packs.json` and
  `packs.spec.ts`, not the fixture this change rebuilds. #826 touches `session.ts`; a textual
  conflict there is resolved by hand, since the two behaviours are independent.
- **Not here.** Other invisible characters: the zero-width space (U+200B, which UAX #29 already
  treats as a break), the word joiner (U+2060) and U+FEFF (open question 2). A visible hyphen at a
  line's end (« vi-\nda ») is not touched either. Words already saved with a soft hyphen are not
  migrated (open question 1).
- **Effort, against 2–3.5 ideal days.**
  - The core's readings and doc comments: 0.25–0.5.
  - The core's unit tests: 0.5–0.75.
  - The extension and its tests: 0.5–0.75.
  - The bump, re-reductions, fixtures and goldens: 0.5–1.
  - Spec and programme: 0.25–0.5.
