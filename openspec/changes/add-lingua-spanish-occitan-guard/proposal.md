# add-lingua-spanish-occitan-guard — Occitan is not Spanish

## Why

Change 42b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside the 57. `add-lingua-french-detection-guard` (change 42) measured Spanish's own leak on
Occitan and left it to a Spanish change (its open question 2); the owner settled it that way on
2026-10-10. whichlang has no Occitan class and reads 27.2 % of Occitan blocks as Spanish; Spanish's
guard, written for Catalan and Galician (`add-lingua-spanish-detection-guard`), still lets 23.7 % of
the blocks and **19.2 % of the text** through as Spanish. It misses Occitan for a reason of its own:
Occitan's articles `lo`, `los`, `las` and its `sus` are in Spanish's own marker table, so an Occitan
sentence's articles vote for Spanish (« Los dròlles son totjorn dins lo jardin. » is Spanish today).

This is the shipped pair's leak. Today's readers of Spanish read es-fr and are French; Occitan is
the regional language of the south of France — Occitan Wikipedia, regional press, signs — and Aranese,
its Gascon variety, is official in Catalonia's Val d'Aran. For them an Occitan paragraph is highlighted
as broken Spanish, every word unknown, words that do not exist in Spanish proposed for the deck.

## What Changes

- **A third comparison in Spanish's guard.** A block whichlang reads as Spanish is also refused when
  its Occitan function words outnumber the Spanish markers Occitan does not write — every one of
  Spanish's but `lo`, `los`, `las` and `sus` (D1). Catalan's and Galician's comparisons are untouched:
  the guard only refuses more, and every block it refuses today stays refused.
- **121 Occitan function words and three elisions** (`qu'`, `m'`, `t'`), measured word by word (D2,
  D4): Languedocien, Provençal, Gascon and Aranese articles, pronouns, prepositions, adverbs, the
  copula and the modals (`pas`, `dins`, `aquò`, `çò`, `èra`, `siá`, `totjorn`, `fòrça`, `tanben`,
  `degun`, `eth`, `dera`, `damb`, `ua`…). Left out, each measured: every word a Spanish text writes as
  Spanish — `e` (« geografía e historia »: 517 Spanish blocks refused), `fa` (« fa mayor », « por
  fa »), `res` (« carne de res »), `cal`, `pus`, `ara`, `per` (« per cápita »), `cap.` — and every
  regional spelling met in the Spanish corpora: `mai` (« mi mai », Caribbean), `mos` (rural « Ya
  mos… »), `soi` (Chilean voseo), `ai` (an old *hay*), `ei`, `dei` and `aquelas` (*Martín Fierro*),
  `vos`, `sos`, `ta`. An Occitan word counts only as written in lowercase: « Pas de la Casa » is a name (D3).
- **Measured** on 597,086 blocks — change 42's corpus, with Tatoeba's whole Spanish export, UD Spanish
  (AnCora, GSD, PUD, COSER with its Canary and Colombian transcriptions), 58 Spanish Wikipedia articles
  (Spain, Catalan and Occitan subjects, Latin America and its varieties, the Canaries), five Argentine,
  Peruvian and Uruguayan books, and ten Aranese and Gascon articles:
  - Occitan: 23.7 → 15.8 % of the blocks read as Spanish (19.2 → 10.9 % of the text); UD 17.3 → 5.8 %
    of the text, Occitan Wikipedia 6.9 → 3.6 %, Aranese and Gascon Wikipedia 6.7 → 2.3 %;
  - Catalan, as a bonus: 15.5 → 13.7 % of the blocks (6.0 → 5.1 % of the text);
  - Spanish: 2 of the 452,971 Spanish blocks the guard keeps today are refused (0.0004 %), both in
    Spanish Wikipedia's article on Occitan, quoting Catalan and Occitan; none in UD, Tatoeba, the
    Latin American and Canary articles or the books.

  What still leaks is a short line without an Occitan function word (« Lo libre es sus la taula. »:
  `es`, `la`, `de` are Spanish too), and the spec says so.
- **What it costs** (D5): 5–7 µs more per KB of Spanish text, native and WebAssembly, on today's
  guard's 20–26 — about 2.5 % of a page analysis; one merged table answers the same in less than
  today's guard costs. It runs inside `iberian_neighbour`, which change 42's `detect` calls only when
  Spanish is asked about: a reader who does not study Spanish pays nothing.
- **Spanish's analyser version is bumped** (D6): `1.2.0` → `1.3.0`. English and French keep theirs.
- **What moves** (D7), measured on a scratch checkout of `main` with change 42 merged: the guard alone
  moves no probe of any golden (the corpora hold no Occitan block); the bump moves the version, and
  nothing else, on 18 lines of `es-fr.golden` and of `es-en.golden` (the pack line and the 17
  analyses) and on the `beside es-en` line of `fr-en.golden`; `tables/es-fr/` and `tables/es-en/` are
  re-reduced, their manifests and pins alone moving (sizes unchanged); the Spanish fixtures, their
  recorded digests and the agent's `es-fr.lingua` fixture follow. en-fr and en-es do not move, without
  re-blessing. es-fr's output moving needs the owner's approval (the programme's rule), in this
  change's pull request.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Occitan is not read as Spanish*. No requirement is rewritten:
  *Catalan and Galician are not read as Spanish* holds as written, and change 42's *Catalan, Occitan
  and Romanian are not read as French* is change 42's. It archives after
  `add-lingua-french-detection-guard`, whose `detect` it builds on.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core` — *new*: Occitan's table, the Spanish markers Occitan writes and the third
    comparison in `analysis/language.rs`, Spanish's analyser version; *consumed*: whichlang, change
    42's `detect`, unchanged.
  - `crates/lingua-wasm` — the three goldens re-blessed on the version; `tests/languages.rs`'s literal.
  - `scripts/lingua-data/tables/es-fr/`, `tables/es-en/` — re-reduced: `manifest.json` and `pin.json`;
    `testdata/es-fr/`, `testdata/es-en/` — `manifest.json`; `crates/lingua-pack/tests/pipeline_testdata.rs`
    — the two Spanish fixtures' digests.
  - `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` — rebuilt (a test input; the agent is
    otherwise outside the programme, M17).
  - `apps/lingua-extension` — nothing: it reads Spanish's version from the core and builds its es-fr
    pack from the committed tables.

  ID, Music, Live, the back office, the site, the backend and the Apple host app are untouched. No
  table row, wire field, proto or stored format changes.
- **Release.** es-fr ships: the next extension release carries it, and a reader of Spanish stops
  seeing Occitan highlighted as Spanish. Nothing else a reader sees changes; nothing is migrated.
- **Order.** After change 42 (merged, `a656dbcb`), on its `detect(trimmed, languages)`. Independent of
  change 41b (`fix-lingua-lemma-lookup`) in code and spec; both move `es-fr.golden` and `es-en.golden`,
  on different lines, and whichever merges second re-blesses on top of the other.
- **Not here.** Recognising Occitan as a language; Asturian (70 % of its blocks still read as Spanish)
  and Aragonese; the vote, which still gives a page to Spanish when any of its blocks leaks; the
  existing guard's reading of words, and its own refusals of real Spanish — 850 of the Spanish blocks
  measured, 795 of them by Galician's `da` and `das`, Spanish's *gives* (« ¿Cuánto se da de propina en
  España? »): the design's open question 4 asks whether to fold that fix into this bump.
- **Effort, against 1.5–3 ideal days.** The comparison, its tables and doc comments: 0.5–0.75. Unit
  tests: 0.5–1. The bump, the re-reductions, the fixtures and the goldens: 0.25–0.75. Spec, programme:
  0.25–0.5.
