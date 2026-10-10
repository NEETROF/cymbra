# leave-lingua-names-out-of-english-and-spanish-words — English's and Spanish's dictionary words hold no names, and English sets a document's names aside

## Why

Change 48b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`refine-lingua-fr-en-glosses`, implemented with #872) took the names out of French's dictionary
words: a lemma fr-en glosses by a proper noun's senses alone (`paris`, `durand` « a surname ») is no
word to learn. It stays glossed, but the vocabulary estimate no longer counts it and change 41's names
rule sets it aside. 48b's design (D2, *Settled by the owner*, Q1) left English and Spanish for later, in
a change of their own. It could not move en-fr's and es-fr's dictionary words, packs and goldens.

The owner decided on 2026-10-10 that the same rule applies to English and Spanish, in this change.
English counts 3,046 such names as words to learn and Spanish 1,693. Measured on the committed tables:

- **English** (en-fr, its reference pair, glosses 24,799 lemmas): 3,046 lemmas are glossed only by a
  proper noun's senses. 388, 993 and 1,868 of them are among the 5,000, 10,000 and 20,000 commonest
  lemmas: `york` (rank 348), `london` (442), `america`, `june` (502), `david`, `england`, `uk`,
  `james`, `paris`. The vocabulary estimate counts them in its universe of 25,372 words.
- **Spanish** (es-fr glosses 22,755 lemmas): 1,693 such lemmas, 261, 550 and 968 of them in the same
  three groups: `méxico` (149), `españa` (153), `madrid` (231), `juan`, `josé`, `venezuela`, `europa`.
  They are in the universe of 22,755 words, and Spanish's names rule keeps them as words. On the
  Spanish baseline's `nombres` page, `Madrid`, `Sevilla`, `María` and `Pedro` are underlined as
  unknown words.
- **What the data rule alone cannot do.** English has no names rule. A form that is no dictionary
  word is set aside only in a Spanish or a French document (`engine.rs` `analyse_page`, « English's
  analysis does not change »). Taking `london` out of English's dictionary words would shrink the
  vocabulary estimate, but `London` would still be underlined as an unknown word. That only changes if
  English gets the rule too, with two readings English needs: `I'll` and `Mr` are capitalised
  everywhere and are no names (design D4).

The programme's rule is that en-fr and es-fr output does not move. This change moves both on purpose,
which needs the owner's approval before the implementation (task 0.2).

## What Changes

- **One rule for the three studied languages** (design D1): a studied language's dictionary words are
  its reference pair's glossed lemmas, less those the pair glosses by a proper noun's senses alone.
  The reference pairs are en-fr for English and es-fr for Spanish (`tables/<lang>/studied.json`). « A
  name » is what the reference's edition files as a proper noun: every sense run of the lemma's row in
  `senses.tsv` is `PROPN`, as 48b reads fr-en's. A word with a common sense beside a name's stays a
  word (`bill`, `mark`, `luna`, `dios`). English's dictionary words go from 24,799 to 21,753 and
  Spanish's from 22,755 to 21,062. The lemmas keep their glosses, and a selection still opens their
  card.
- **Where the rule lives** (D2): `dictionary_words` in `reduce_edition_fr.py`, the French
  Wiktionary's module, which en-fr and es-fr load and no other pair does. Their reducers write
  `lexical.tsv` and 48b's `split` files it. 48b's `check_studied` already accepts a reference's glossed
  lemmas less its names, so there is no Rust change in lingua-pack. fr-en keeps its own copy and its
  pin. Only en-fr's and es-fr's rule digests move.
- **Levels** (D3): English's levels come from CEFR lists and are not touched. Eleven of its
  names-only lemmas carry one: eight months (`january`… `december`, A1, which the French Wiktionary
  files as proper nouns), `olympics`, `badminton` and `ph`. They keep their level and the vocabulary
  sizes still count them. Spanish's estimated levels already leave out exactly these lemmas (es-fr's
  `estimated_levels`: `madrid` takes none), and French's do since 48b. No `level.tsv` moves.
- **English sets a document's names aside** (D4): Spanish's rule as written. A form the document never
  writes in lowercase, capitalises at least once in mid-sentence, and whose lemma is no dictionary word
  is set aside. English adds two readings of its own:
  - the pronoun `I` and its contractions give no evidence (`I'm`, `I'll`, `I'd`, `I've`);
  - a lemma a CEFR list levels is a word (`June`, `Mr`, `Mrs`).

  English's analyser version is bumped (`1.2.0` → `1.3.0`). On the English baseline `Margaret`, `Sam`,
  `Ruiz` and `HTTP` are set aside. Over 10,000 quotations of the English Wiktionary (296,336 tokens)
  it sets aside 4,347 tokens (1.5 %): 2,587 are names this change takes out (`London`, `England`) and
  1,760 are words en-fr has never glossed (`Hong Kong`, `NASA`, `Trump's`). The two readings keep 233
  and 315 tokens a word.
- **The vocabulary figures** (D5): the estimate's universe goes from 25,372 to 22,337 words for English
  and from 22,755 to 21,062 for Spanish. English's own ladder gives typical vocabularies of 1,213 /
  3,074 / 7,155 / 14,433 / 18,123 at A2–C2 (was 1,292 / 3,359 / 7,988 / 16,326 / 20,556). The core's
  frozen copy, `ENGLISH_TYPICAL_VOCABULARY`, which the Spanish and French ladders borrow, is frozen
  again on these figures so the four ladders agree (open question 2).
- **What « a name » leaves out by mistake** (D1, measured): there are common words the French
  Wiktionary files only as names. English: the eight months (kept by their level), `easter`, `islam`,
  `hebrew`, `hindi`, `esperanto`, `renaissance`, and mis-tagged words such as `unseen` « Inédit;
  Inaperçu ». Spanish: `cristo`, `biblia`, `al` « Amérique latine » (rank 16; the contraction is split
  before lookup, so only the universe sees it) and `títere` « Marionnette ». They are listed for the
  owner (open question 3). Read in context, nearly every token set aside is a name: 133 of the 2,587
  English tokens and 379 of the 3,376 Spanish ones have a lemma that another edition glosses as a
  common word, and nearly all of those are `Australia`, `María`, `Luis`.
- **The re-pin** (D6): en-fr and es-fr are reduced again at their pins. Their tables stay byte for
  byte except `tables/en/lexical.tsv` and `tables/es/lexical.tsv`. Their packs now carry a lexical
  table (+5,098 and +7,512 bytes). es-en and en-es are recorded on the new studied tables: the same
  glosses, a new studied record and `pack_version`, the same sizes. fr-en, fr-es and `tables/fr/` stay
  byte for byte.
- **The goldens re-blessed, every move with its cause** (D7): `en-fr.golden` moves on 19 of 142
  probes and `en-es.golden` on 19 of 182: the pack line, the fifteen page analyses (on the version
  string; five of them on their tokens too), the ladder and the two estimates. `es-fr.golden` moves on
  7 of 137 and `es-en.golden` on 6 of 176: the pack lines, the two `nombres` analyses, the two
  estimates and the ladder. `fr-en.golden` and `fr-es.golden` move on 2 of 213 each: the `beside`
  line naming the reader pair's pack, and the ladder. The fixtures that carry English's version move
  too.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *A pack's dictionary words do not depend on its glosses*: every
  studied language's dictionary words leave out its reference's names-only lemmas, and the reference
  packs carry a lexical table. MODIFIED *A studied language's tables are kept once*: its scenarios
  follow. Both are held by `refine-lingua-fr-en-glosses`, which is in `archiveAfter`. MODIFIED *Only
  the reference pair's reduction writes its studied language's tables*: one sentence; no open change
  holds it.
- `lingua-analysis`: ADDED *An English document's names are set aside*. MODIFIED *A Spanish
  document's names are set aside*: it loses « The English analysis SHALL NOT change » and gains the
  scenario of a name es-fr glosses; no open change holds it. MODIFIED *A French document's names are
  set aside*: it loses « and English's analysis »; held by `add-lingua-french-analysis`, which is in
  `archiveAfter`.
- `lingua-knowledge-model`: MODIFIED *A vocabulary size counts dictionary words*: the measured
  universes and typical vocabularies; no open change holds it.
- `lingua-browser-extension`: MODIFIED *An estimated ladder shows English's typical vocabularies*: the
  frozen figures; no open change holds it.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/`: *changed*: `reduce_edition_fr.py` (`dictionary_words`),
    `reduce-en-fr.py` and `reduce-es-fr.py` (write `lexical.tsv`), their tests, `pack_sources.py` (doc
    only), `build.sh` (doc only), `tables/en/lexical.tsv`, `tables/es/lexical.tsv`, `tables/en-fr/`,
    `tables/es-fr/`, `tables/en-es/` and `tables/es-en/` (`manifest.json`, `pin.json`, `README.md`),
    the testdata manifests of en-fr and en-es, `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_en.py`, `reduce_edition_es.py`, `reduce-fr-en.py`, unchanged.
  - `crates/lingua-core`: *changed*: `engine.rs` (English's names rule and its two readings),
    `analysis/mod.rs` (English `1.3.0`), `knowledge/vocabulary.rs` (the frozen figures), the tests that
    write English's version.
  - `crates/lingua-pack`: *changed*: tests only (`committed_tables.rs`, `pipeline_testdata.rs`). The
    check of a studied folder is 48b's, unchanged.
  - `crates/lingua-wasm`: *changed*: the six goldens, `cross_native.rs`, `languages.rs`, the fixtures
    `pack.lingua` and `golden.json`.
  - `apps/lingua-extension`: *changed*: `test/fixtures/en-fr.testdata.lingua` only. No source and no
    snapshot (`word-card-*.txt` and `selection-rows-fr.txt` read glosses and word grammar, which do not
    move).
  - `apps/lingua-agent`: *changed*: its English test fixture `pack.lingua`, stamped `1.3.0`.
  - ID, Music, Live, the back office, the site (coverage does not move), the backend and the Apple
    host app are untouched.
- **What moves.** What en-fr and es-fr readers see: names are no longer underlined in a Spanish page,
  nor in an English one; vocabulary estimates go down (the English baseline's B1 reader from 3,363 to
  3,078 words, the Spanish one's from 2,285 to 2,176); every ladder's typical vocabularies change.
  Their exposures and « unknown seen » no longer count a name. A word the reader has marked keeps its
  status.
- **What cannot move.** Every gloss, sense run, expression, form, rank, level, reading and tag of the
  four English and Spanish pairs. fr-en, fr-es and `tables/fr/`, byte for byte. Spanish's and French's
  analyser versions. The French interface and the extension's sources.
- **Release.** en-fr and es-fr ship. The change reaches readers with the next extension release, and
  that release waits for the owner's go-ahead (M18). Nothing to migrate: statuses and cards are keyed
  by lemma. The agent plugin, once rebuilt, needs the new English pack; a `pack.lingua` built for
  `1.2.0` is refused, with an error that names both versions.
- **Order.** After 48b (required, on `main`), 41 and 42d. Either side of 34 and 35: they list es-en
  and en-es with whichever dictionary words are on `main` (design D8). No change waits for it.
- **Effort**: 2.5–4.5 ideal days (design, *Effort*).
