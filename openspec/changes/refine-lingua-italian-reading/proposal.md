# refine-lingua-italian-reading — an Italian specialist's review of the test pages, the pre-pass and the closed classes

## Why

Change 11a of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Changes
5, 6 and 7 each left the owner a reading task — the test pages and their attributions (change 5's
5.1), the elision, articulated-preposition and truncation tables (change 6's 4.1), the closed
classes, the names rule and the cascade's choices (change 7's 9.1). No one on the team reads Italian
(I2), so an Italian specialist read them on the owner's behalf; the owner, Guillaume Fortin,
approved every recommendation on 2026-10-24 (« Tout appliquer »). This change gathers them:

- **Four test sentences read as no Italian writes**: « Le lettere che ho scritto sono arrivate
  ieri » (one's own letters arriving), a *frittata* made with flour and milk, « l'attività del
  servizio è più stabile », « che sia sparita nell'armadio ».
- **Elisions the closed list leaves out**, each read today as one unknown, unglossed word: the
  committed Italian tables (`tables/it/`) rank `trent'anni` 3,522nd and `qual'è` 4,273rd, and 22 more such words
  (`quarant'anni`, `tant'è`, `gliel'ho`, `fin'ora`…).
- **Literary truncations read as nothing**: `de'` (« Lorenzo de' Medici », 613 times in change 10's
  corpus) is read as `de`, `que'` (267) as `que`, and the truncated articulated prepositions `a'`
  (99), `co'` (81), `ne'` (106), `su'` (13) as `a`, `co`, `ne`, `su`; `da'` (68, Manzoni's « da'
  primi suoi anni ») is read as the verb `dai` where books write the preposition.
- **Two words read one way whatever their context**: `mo'` always as `modo` (« a mo' di esempio »,
  right; « mo' arrivo », *now*, wrong), and `dai` always as the preposition `da` + `i`, which hides
  the verb *dare* in « Dai, andiamo! » and « mi dai una mano? ».
- **Fifteen closed-class words missing** from change 7's tables (`davanti`, `dietro`, `entrambi`,
  `parecchio`, `anziché`, `bensì`, `nonché`, `ossia`, `pertanto`, `qualora`…).

## What Changes

- **The test pages** (`crates/lingua-wasm/tests/baseline/pages-it.txt`, design D1): four sentences
  rewritten as Italian writes them, every probe word kept — `grammatica` « Le lettere che mi hai
  scritto sono arrivate ieri; le ho lette subito. », `ricetta` « Sbattete 6 uova con 50 g di
  parmigiano e sale q.b., poi cuocete per 10 minuti a fuoco basso. » (the *frittata* kept),
  `tecnico`'s NFD block « l'affidabilità del servizio è più stabile » (the elision and the
  decomposed accent kept), `elisioni` « Com'è possibile che non sia più nell'armadio? » —; and the
  specialist's optional ones: `elisioni` « E sarà tutt'altro che noiosa. », `notizie`'s percentages
  without a space (« 0,3% », « 3,5% »), `informale` « aspetta un pò, qual'è il problema? », and a
  courtesy sentence in `grammatica`, « La ringraziamo, Le risponderemo al più presto. ».
- **Eighteen elisions and two elided prepositions** (`ITALIAN_ELISIONS`,
  `ITALIAN_ELIDED_PREPOSITIONS`; D2): `cinquant'`, `diciott'`, `fin'`, `gliel'`, `neanch'`,
  `novant'`, `null'`, `ogn'`, `ottant'`, `prim'`, `qual'`, `quarant'`, `sessant'`, `settant'`,
  `tant'`, `trent'`, `ultim'`, `ventiquattr'`; `coll'` → `con` + `lo`, `degl'` → `degli` (the `di`
  family whole). `oltr'` is left out (« Oltralpe » is one word).
- **Truncations** (D3): `ITALIAN_TRUNCATIONS` gains `ca'` → `casa`, `de'` → `dei`, `que'` → `quei`;
  a new table reads a truncated articulated preposition as two words sharing the span, as an
  articulated preposition is read: `a'` → `a` + `i`, `co'` → `con` + `i`, `ne'` → `in` + `i`, `su'`
  → `su` + `i`, and `da'` → `da` + `i`. Change 10's apocope list (`ITALIAN_APOCOPES`), which held
  seven of these only to keep its accent rule off them, keeps the nine the pre-pass still does not
  read. `to'` stays out: the interjection « tiens ! », which `togli` would gloss « enlever ».
- **Two context rules** (D4): `mo'` is `modo` before `di` and `mò` otherwise; `dai` and `da'` are
  the verb *dare* after `non` or a clitic pronoun, or before `,`, `!`, `?` or a sentence's end, and
  the preposition otherwise. A word card, which has no context, reads the word the way its
  dictionary form asks. Both rules are measured before they ship, by criteria written in this
  design, on corpora that exclude UD Italian-PUD and MarkIT's test section (D5).
- **The closed classes** (`analysis/function_words.rs`, D6): the prepositions `davanti`, `dietro`;
  the determiners `ambedue`, `entrambe`, `entrambi`, `parecchi`, `parecchia`, `parecchie`,
  `parecchio`; the conjunctions `anziché`, `bensì`, `nonché`, `ossia`, `pertanto`, `qualora`.
  `stare`, `volere`, `secondo`, `lungo`, `mai`, `più`, `anche` and `salvo` stay out, with the
  specialist's reasons.
- **The reducer follows** (D7): `reduce-it-fr.py` mirrors change 6's tables (its test holds them to
  `tokenize.rs`), and the committed tables must hold every word the pre-pass writes as a form, each
  elided piece and truncation a form of its word. The new pieces therefore need it-fr reduced again
  — the owner's `lingua-pack-update` dispatch in mode `reduce` —: 24 ranked words beginning with a
  new piece (`trent'anni` …) and 7 ranked stems (`trent`, `tant`, `coll` …) leave the ranks, `prim'`
  becomes a form of *primo* (the section files it under *prima*), `que'`, `degl'`, `fin'` and the
  other new pieces become forms, `da'` stops being one.
- **Italian's analyser version** `1.3.0` → `1.4.0`; the fixture pack and `it-fr.golden` re-blessed
  once at implementation. **English, Spanish, French and the six goldens do not move.**
- **The three owner tasks are closed in this pull request** (changes 5's 5.1, 6's 4.1, 7's 9.1),
  each with the line the owner asked for; none of the three archives at merge (D9).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`:
  - MODIFIED — *Italian tokenisation pre-pass*, as change 10 (`read-lingua-italian-unmarked-spellings`)
    leaves it: the new elided forms, `coll'`, `degl'`, the truncations `ca'`, `de'`, `que'`, the
    truncated articulated prepositions, `mo'` and `dai` by their context, a word card's reading of
    them; every scenario kept, ten added.
  - MODIFIED — *The apostrophe that stands for an accent reads as the accented word* (change 10's):
    the apocopes it excludes are the nine the pre-pass does not read; the truncated articulated
    prepositions close no quote; its scenarios *A listed word with its apostrophe* and *An apocope*
    rewritten for what the pre-pass now reads, one added.
  - MODIFIED — *Italian closed classes* (change 7's, `add-lingua-italian-analysis`): the fifteen
    words; `anche` named among the words left out, as change 7's design already left it; one
    scenario added, one widened.
  - This change archives after changes 6, 7 and 10, which hold those requirements.
- `lingua-data-packs`: no requirement moves. Change 9's *Italian's forms and frequencies* (as change
  20 leaves it) already says the tables serve the pre-pass as change 6 writes it — every word it
  writes a form, each truncation a form of its word, the articulated prepositions it splits neither
  forms nor ranks —; the reducer follows the tables it mirrors (D7).

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them; no Italian pair ships before change 20, so no reader sees it before
then.

- `crates/lingua-core/src/analysis/tokenize.rs` (the tables, the truncated articulated prepositions,
  the two context rules, `ITALIAN_APOCOPES`), `analysis/function_words.rs`, `analysis/mod.rs`
  (`1.4.0`), `engine.rs` (`word_grammar` reads a context-dependent word by the card's dictionary
  form), `tests/italian_fixtures.rs`.
- `crates/lingua-wasm/tests/{baseline/pages-it.txt, baseline/it-fr.golden, italian_baseline.rs,
  support/italian.rs}`; `scripts/lingua-data/testdata/it-fr/` (the manifest at `1.4.0`, the new
  read words); `crates/lingua-pack/tests/{pipeline_testdata.rs, committed_tables.rs}`.
- `scripts/lingua-data/reduce-it-fr.py`, `test_reduce_it_fr.py`; `tables/it/{forms.tsv, freq.tsv}`
  and `tables/it-fr/{pin.json, manifest.json, README.md}` by the owner's dispatch.
- Untouched: the six pairs' tables, pins and goldens; English's, Spanish's and French's arms; the
  extension's bundle; the agent; the server; the site.
