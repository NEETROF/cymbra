# keep-lingua-french-acronyms-in-capitals — « CA » in capitals stays the acronym

## Why

Row 43c of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes: the analyser change that change 43b (`read-lingua-french-ca-as-ca-cedilla`)
asked for and is waiting on.

Change 43b reads « ca » as `ça`, the web's « ça » without its cedilla (« ca va », « c'est ca »). The
pack keys its forms in lower case and French's analysis looks every token up lowercased (change 41),
so « CA », the acronym, reads as `ça` too: « Le CA a voté le budget. » shows « that », a function
word. The owner settled on 2026-10-10 that « CA » written in capitals stays the acronym, and that 43b
reaches readers only with this change or after it (43b's task 6.4). Of the three options 43b's
design sketched, the owner chose **A**: a small reviewed acronym table, looked up first — a word
written wholly in capitals, in a line that is not itself in capitals, is looked up in the table
before anything else, `CA` alone at first.

Measured on the French text at hand (design, *Measured*): 23.5 million tokens analysed by the real
engine — UD's seven French treebanks, Tatoeba's 727,867 French sentences, the French Wikipedia and
Gutenberg samples of the earlier changes, change 39's corpus, and the 614,357 French usage examples
of the French and English Wiktionaries. They write « CA » in capitals 35 times: 31 standing alone among
lowercase words, none of them « ça », and 4 in shouted text, every one « ça ». No line written
in capitals writes the acronym: 2 of their 232 such lines write « CA », both « ça ».

## What Changes

- **The rule** (design D1). Before French's cascade, a token written wholly in capitals — two letters
  or more, none of them lowercase — that stands alone in capitals — the nearest written word of two
  letters or more before it and after it, in the text being read, is not in capitals — is looked up
  in the pack's forms as written. A hit is a row of the acronym table: the token's dictionary form is
  the acronym, capitals kept, and the cascade does not run. Anything else is read as today, lowercased
  (43b's `ca` → `ça` included). The owner's « a line not itself in capitals » is read at the word's
  neighbours: on the corpora, reading it at the whole block would have taken 2 shouted « ça » for the
  acronym (« LES RÉSEAUX SOCIAUX CA RENDS LES GENS FOU… », « OUAH CA CHAUFFE…… je tartouille »), the
  neighbours none, and they lose nothing measured. One function reads the neighbours for a page's
  block, a selection, a card's written word and an expression's headword, so the four read alike.
- **The table and how the pack holds it** (D2). A reviewed table, `ACRONYMS`, in a French rule module
  of its own that fr-en's and fr-es's reducers both import (`reduce_french_acronyms.py`, as
  `reduce_french_treebank.py` is): one row, `CA`, its expansions as the English Wiktionary's French
  `CA` entry names them — `conseil d'administration`, `chiffre d'affaires`, `courant alternatif`,
  `comptable agréé` — and its reason. fr-en's reducer, French's reference, writes the row into
  `tables/fr/`: `forms.tsv` gains `CA` → `CA`, the one form holding a capital, its own lemma, and
  `lexical.tsv` gains `CA`, a dictionary word. In the pack it is a form in capitals of the existing
  forms map; no section, no format change. Every other lookup of the core lowercases first, so the
  form is reached only by the rule. The builder accepts a form holding a capital only when it is
  wholly in capitals, two letters or more, its own lemma, in a pack studying French.
- **The gloss: each pair glosses the acronym from its own glosses of the expansions** (D4), each
  expansion's first sense, in the row's order, an expansion the pair does not gloss skipped: fr-en
  « board of directors; revenue, turnover, net sales; alternating current », fr-es « Administración,
  consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna », both one
  noun run of three senses (`comptable agréé` has no gloss in either). Every sense is a gloss people
  wrote in a dictionary for the expansion; nothing is translated, and the French expansion is not
  shown.
- **No rank, no level** (D5): wordfreq reads `CA` as `ca`, whose frequency is `ça`'s. `CA` is never
  presumed known, carries no level, is never seeded into a deck, stays outside the vocabulary
  estimate, and its card says « Rare — beyond the 20,000 most frequent words ». It is the one glossed
  lemma of fr-en that is not ranked, which the requirement states (change 48's *A gloss is its
  lemma's* reads ranked lemmas).
- **The capitals are kept to the card, the status and the key** (D6): the core's word grammar keeps a
  dictionary form the pack holds as a lemma as it is asked (`CA`), and lowercases any other as before;
  the extension keys a word's status and card by the dictionary form the engine gave, lowercasing an
  expression's key alone, as before. Sync and the backup carry `CA` as any lemma.
- **What moves, measured on a prototype** stacked on 43b's prototype tables (D7):
  - French's analyser version is bumped (`1.2.0` → `1.3.0` on today's `main`); English's and
    Spanish's do not move;
  - `tables/fr/forms.tsv` and `lexical.tsv` one row each; `tables/fr-en/` and `tables/fr-es/`
    `gloss.tsv` and `senses.tsv` one row each; `freq.tsv`, `level.tsv`, `grammar.tsv`, `tags.tsv` and
    both `mwe.tsv` byte for byte; both pins and manifests (version, rule digest, pack); coverage
    unchanged;
  - the packs: fr-en 2,537,588 → 2,535,748 B, fr-es 1,971,114 → 1,969,484 B (`CA` takes lemma id 0,
    every other id moves by one, which the forms map and the paradigms encode slightly tighter);
  - `fr-en.golden` and `fr-es.golden`: 19 lines each — the pack line, the 17 page analyses on the
    version alone, and « Le CA a voté le budget. », whose `CA` turns from `ça` « that… », a function
    word, into `CA` and its gloss —; four probes added: « LE CA A VOTÉ LE BUDGET » (`ça`: a headline),
    « Il a crié : CA SUFFIT ! » (`ça`, `ça suffit` met: a shout), « Ca fait du bien. » (`ça`) and the
    card of `CA`; every other probe byte for byte, the vocabulary estimates and decks among them;
  - the word-card snapshots gain those four cards and nothing else.
- **On the corpora** (D3, D8): 31 tokens move, every `CA` standing alone, from `ça` to `CA`; no other
  token of the 43,515 written in capitals. The dictionary's meanings fit 6 of the 31 (board 3,
  turnover 2, chartered accountant 1), 4 are Geneva's Conseil administratif, 21 something else —
  sports clubs 6, army corps, Canadian dollars, certificate authorities, Latin syllables, a geometry
  segment, codes. Before 43b they read « board of directors », with 43b alone « that »: every one
  wrong but the three boards. The analysis costs +0.9 to +3.6 % of French analysis time (best and
  median of interleaved runs); a reader of English or Spanish pays nothing.
- **en-fr, es-fr, es-en and en-es cannot move** (D7): the rule runs for French alone, the form in
  capitals exists only in `tables/fr/`, the new module is imported by fr-en's and fr-es's reducers
  alone, the word-grammar and extension changes are no-ops for every lemma the engine gives today
  (all lowercase); on the prototype their goldens, `cross_native`, `parity`, `card_gloss_language`,
  `deck` and `languages` pass without re-blessing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A French word standing alone in capitals is read as the acronym its
  pack lists* (the rule, its neighbours, the text it reads them in, French's version bumped, no
  other language touched).
- `lingua-data-packs`: ADDED — *French's acronyms are a reviewed table, glossed through their
  expansions* (the row, the form in capitals its own lemma, the builder's guard, each pair's gloss, no
  rank or level, the one unranked glossed lemma, nothing else moving).
- `lingua-browser-extension`: ADDED — *A word's status and card keep its dictionary form's capitals*.

No requirement is modified. The new ones narrow what open changes state — change 41's *French
lemmatisation cascade* produces a lowercase lemma for each token, change 43's forms are lowercase,
change 48's *A gloss is its lemma's* reads ranked lemmas, and 43b's requirement reads « CA » as `ça`
« until a rule of French's analysis reads a word written in capitals apart », which this is — so they
say so rather than MODIFY them. Those changes, 46 (levels), 48b and 49b (each pair's glosses), 49
(fr-es) and 42d (which holds *A word's grammar, from the pack* and *An analyser version per studied
language*) are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core` — *new*: the rule (`analysis/french.rs` `acronym`, `analysis/pipeline.rs`
    `alone_in_capitals`, read by `analyse_document`, `headword_reading`, `engine::gloss_phrase` and
    `engine::word_grammar`), French's analyser version; *changed*: `word_grammar` keeps a lemma's
    capitals; *consumed*: the lexicon, the cascade, the names rule, unchanged.
  - `crates/lingua-pack` — the builder's guard on capitals; `tests/committed_tables.rs`.
  - `scripts/lingua-data` — *new*: `reduce_french_acronyms.py` and its tests; *changed*:
    `reduce-fr-en.py`, `reduce-fr-es.py`, their tests, `tables/fr/` (`forms.tsv`, `lexical.tsv`),
    `tables/fr-en/` and `tables/fr-es/` (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`,
    `README.md`), `testdata/fr-en/manifest.json`, `SOURCES.md`.
  - `crates/lingua-wasm` — four probes; `fr-en.golden` and `fr-es.golden` re-blessed;
    `french_baseline.rs`'s literals.
  - `apps/lingua-extension` — `reading/session.ts` and `reading/selection-card.ts` (the key); the
    French word-card snapshots gain four cards.

  ID, Music, Live, the back office, the site, the backend (it stores a lemma as it is sent), the
  Apple host app and the agent plugin are untouched; no proto, wire field or stored format changes.
- **Release.** Silent: no package lists a French pair. With 43b, before change 52 lists French; this
  change needs 43b's `ca` → `ça` row and 43b waits for this change (its 6.4).
- **Compatibility.** No reader holds a French status, card or level, so nothing is migrated.
- **Effort, against no programme estimate (outside the 57)**: 2–3.5 ideal days, as 43b's design
  sketched option A (design, *Effort*).
