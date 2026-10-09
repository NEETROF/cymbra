# add-lingua-french-analysis — French's NFC, cascade, closed classes and names, analyser 1.0.0

## Why

Change 41 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the fourth of stage 3 (French studied: fr-en, fr-es). Change 39 (`add-lingua-french-baseline`)
made French a studied language served by the baseline analysis and gave this change « the
cascade, the function words, the names rule and NFC », with French's analyser leaving its `0.x`
versions here (its D4: `1.0.0`, as Spanish's `add-lingua-spanish-analysis` did). Change 40
(`add-lingua-french-tokenisation`, merged as #821) cut French into words at `0.2.0` — an elided
piece read as the word it stands for, `au` as `à` + `le`, an inversion as its words — and left
« pas » a function word to this change (its D1, M21). Change 43
(`add-lingua-french-forms-tables`, proposed) commits the forms and ranks the cascade reads, one
lemma per form (M8).

What French text still lacks on `main` (35faf774), measured on change 39's corpus, on UD
French-PUD and GSD with change 43's prototype tables, and on 864,079 tokens of raw French text
(design D6):

- **Decomposed text is not French's.** A word typed or pasted in NFD (`mémoire`) is not
  the pack's `mémoire`: change 39's corpus holds one such block on purpose, and its `mémoire`
  shows no gloss. A decomposed `ç'` escapes change 40's elision, a decomposed `peut-être` its
  listed-run check.
- **Every closed-class word is a row.** A word-by-word gloss lists `le`, `de`, `que`, `il` and
  « pas » beside the words worth reading: 46 of the 91 tokens of the golden's phrase glosses.
- **Names are words.** Change 43's tables rank `paris`, `lyon`, `lot`, `aube`, `durand` as
  wordfreq does, so the out-of-lexicon proper-noun rule leaves them counted: with the cascade's
  names rule, 31,582 tokens of the raw corpus (3.65 %; Wikipedia 6.0 %, novels 1.1 %) are set
  aside, Spanish's rule extended after an elided piece (`l'Europe`, `d'Espagne`) and to
  hyphenated names (`Saint-Étienne`, `Jean-Pierre`), which French writes far more than Spanish.
- **An unlisted plural is a second word.** `mégalithes` and `mégalithe` count twice; 1,400 tokens
  of the raw corpus, 18 PUD words.

## What Changes

- **NFC in French's pre-pass** (`analysis/tokenize.rs`, French's arm): every comparison the
  pre-pass makes (the elided forms, `au`/`aux`, a run the pack lists) and every token's text are
  composed; spans still point into the source.
- **French's cascade** (`analysis/french.rs`, new; `lemmatize` dispatches to it): the pack's
  forms → a lowercase plural the pack does not list whose singular it does not list either, read
  as that singular (`-s`, `-eaux` → `-eau`, `-aux` → `-al`; short words, `-us`/`-is`/`-ès`/`-os`
  singulars and passé simple endings left alone) → the form itself. The tables decide every
  lemma (M8): no rule reads a form as a word the pack lists, so change 43's `étés` stays out of
  *être*. Measured and rejected: a capital without its accent, `oe` for `œ`, a plural or feminine
  read through its listed singular, verb endings (D2). `lemmatize_baseline` is retired.
- **French's closed classes** (`analysis/function_words.rs`): determiners, pronouns,
  prepositions, conjunctions, auxiliaries and modals (`être`, `avoir`, `pouvoir`, `devoir`),
  negation (`ne`, `pas`, `non`) — « pas » as M21 decides; `personne`, `point`, `or`, `plus`,
  `jamais` left out, each measured on GSD (D3).
- **A French document's names are set aside** (`engine.rs`): Spanish's rule, a capital right
  after an elided piece counting as mid-sentence, and a hyphenated run the pack does not list
  read as one form (D4).
- **French's analyser version leaves `0.x`**: `1.0.0` (change 39's D4). The spec states the
  version without pinning a number a later bump would contradict (change 42's guard follows).
  The fixture's manifest follows; if change 43's tables are on `main`, fr-en is re-reduced (its
  manifest and pin alone move).
- **The French golden is re-blessed** against `main`'s: 39 of its 141 probes move, 2 phrase
  probes are added, 102 are byte for byte; the fixture gains 17 forms the real tables hold and 2
  glosses (D7).
- **English, Spanish and the four other goldens do not move** (en-fr, es-fr, es-en, en-es):
  measured on the prototype without re-blessing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *French text is read in NFC*, *French lemmatisation cascade*,
  *French closed classes* and *A French document's names are set aside*. No requirement is
  modified: *French is a studied language served by the baseline analysis* and *A French
  invariance baseline runs beside the English and Spanish ones*, added by change 39 and
  MODIFIED by change 40, both open, are left to them; their « until its lemmatisation rules are
  written » ends with this change's cascade, and the three scenarios that name `0.2.0` or « no
  function word yet » are handed on (design D8, open question 1). Archives after changes 39, 40
  and 43 (`archiveAfter`).

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: `analysis/french.rs`, French's tables in
    `function_words.rs`, French's arm of `document_names`, NFC in French's pre-pass,
    `FRENCH_ANALYZER_VERSION = "1.0.0"`; *consumed*: change 40's pre-pass, Spanish's
    `document_names` and `mid_sentence` (Spanish's behaviour unchanged), the dictionary words
    (`Pack::is_dictionary_word`), `unicode-normalization` (already a dependency).
  - `crates/lingua-wasm` — the French baseline re-blessed, two phrase probes added, its
    assertions moved to the analysis; `languages.rs` reads the new version.
  - `scripts/lingua-data/testdata/fr-en/` — the manifest at `1.0.0`, 17 forms and 2 glosses.
  - `scripts/lingua-data/tables/fr-en/` — re-reduced only if change 43 has merged (manifest and
    pin).
  - `apps/lingua-extension` — `test/packs.spec.ts` reads French's version; the selection card
    already leaves function-word rows out. Nothing a reader sees.

  ID, Music, Live, the back office, the site, the backend, the Apple host app and the agent
  plugin are untouched (the agent compiles the core and holds no French pack).
- **Release.** Silent. No listed pair studies French before change 52.
- **Compatibility.** No stored format, wire field or pin of another pair moves. A French pack must
  carry French's new version; only the fixture exists, and fr-en's tables if change 43 has
  merged.
- **Not here.** A capital `A` read as `à`, lowercase text without its accents (`apres`, `ca`),
  pre-1835 spellings (`étoit`) — measured or out of scope (D2); expression keys through the
  analyser (44); the readings and their moods (45, 51); the Catalan and Occitan guard (42, after
  this change); the cross-native test for French (48, 49); a one-word selection over several
  pieces (51).
- **Effort, against 4.5–7 ideal days.** NFC in the pre-pass and its span tests: 0.5–0.75. The
  cascade and its guards: 0.75–1. The closed classes and their tests: 0.75–1. The names rule's two
  extensions and their tests: 0.75–1.25. The French fixtures (at least 70 cases): 0.75–1. The
  golden, the fixture, the probes and the tests that flip: 0.5–0.75. The measurement on change
  43's tables: 0.25–0.5. Spec and programme: 0.5. Total 4.75–6.75.
