# add-lingua-italian-analysis — Italian's NFC, cascade with its enclitic rule, closed classes and names, analyser 1.0.0

## Why

Change 7 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Change 5
made Italian a studied language served by the baseline analysis; change 6 gave it its tokenisation
pre-pass (I7). What remains between the tokens and the pack is still the baseline's: a form is
looked up as the pack lists it, else it is its own lemma; a decomposed accent is a different word;
no word is a function word, so a phrase gloss glosses `di`, `il` and `non` as if they carried the
sentence; a capitalised word the pack does not list is a name only by Spanish's rule inherited
through an unguarded default; and every verb with a clitic pronoun on its end — « dirmelo »,
« vattene », « eccolo », « parlandogli » — is an unknown word, where Spanish's reader has had its
enclitic rule since the Spanish programme and Italian's clitics are more frequent still (an
infinitive, a gerund or an imperative takes them in nearly every dialogue line).

This change gives Italian its own analysis, as change 41 gave French its own: NFC in the
pre-pass, a lemmatisation cascade whose second step is the enclitic rule modelled on Spanish's,
closed classes, a names rule with French's reading after an elided piece, and the analyser
version `1.0.0`. The tables decide every lemma (M8); the rules only reach what the tables do not
list. Change 8 (the detection guard), 9 (the forms tables, whose measurement on the Italian
treebanks sets the gates this cascade is held to), 10 (the apostrophe-accent spellings) and 11
(acronyms and names out of the words to learn) follow.

## What Changes

- **NFC in Italian's pre-pass** (`analysis/tokenize.rs`, Italian's arm): every comparison the
  pre-pass makes (the elided forms, the articulated prepositions, the truncations, a run the pack
  lists) and every token's text are composed; spans still point into the source.
- **Italian's cascade** (`analysis/italian.rs`, new; `lemmatize` dispatches to it): the pack's
  forms → **the enclitic rule** → an unlisted plural the pack does not list whose singular it does
  not list either, read as that singular → the form itself (design D2). The enclitic rule reads
  off a form the pack does not list one clitic (`mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`,
  `le`, `ne`, `gli`) or one combined clitic (`melo`… `mene`, `telo`…, `selo`…, `celo`…, `velo`…,
  `glielo`… `gliene`), and accepts the rest only as a base the pack lists shaped to take clitics:
  an infinitive with its ending restored (`dir` → `dire`; `por` → `porre`, `-re` tried where `-e` lists nothing), a gerund, an
  imperative the pack files under a verb, one of the five monosyllabic imperatives whose doubled
  consonant is undone (`dammi`, `fallo`, `vattene`, `dimmi`, `stacci`), or `ecco`. Italian's
  clitics move no written accent, so Spanish's stress check has no counterpart.
- **Italian's closed classes** (`analysis/function_words.rs`): determiners, pronouns,
  prepositions, conjunctions, the quantifiers among the determiners, the auxiliaries and modals `essere`, `avere`, `potere`,
  `dovere`, and the negation `non`, `no`, `né`; `mai`, `più`, `neanche`, `nemmeno` and `neppure`
  left out as adverbs that carry meaning, `volere` as a content verb, `secondo` and `lungo` as
  adjectives, with the sibling tables (D3).
- **An Italian document's names are set aside** (`engine.rs`): Spanish's rule — a form never
  written in lowercase, capitalised at least once mid-sentence, not a dictionary word —, a capital
  right after an elided piece counting as mid-sentence (`l'Italia`, `dell'Europa`), a hyphenated run the
  pack does not list read as one form (D4) — an explicit arm, where change 5 returned early.
- **Italian's analyser version leaves `0.x`**: `1.0.0`. The spec states the version without
  pinning a number a later bump would contradict (change 8's guard follows). The fixture's
  manifest follows.
- **The Italian golden is re-blessed**: the `clitici` page's forms resolve to their verbs, the
  `tecnico` page's NFD block to its words, the phrase probes drop their function words, the names
  of the `nomi` page are set aside.
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Italian text is read in NFC*, *Italian lemmatisation cascade*
  (with its enclitic rule), *Italian closed classes*, *An Italian document's names are set aside*,
  and *Italian is a studied language served by its own analysis*, which replaces the REMOVED
  *Italian is a studied language served by the baseline analysis* (its name and its scenarios *No
  Italian word is a function word yet*, *An enclitic form is one token* and `0.2.0` would read
  false; OpenSpec refuses a MODIFIED block that drops a scenario); MODIFIED — *An Italian
  invariance baseline runs beside the English, Spanish and French ones*, whose *What the baseline
  shows today* names Italian's own version. The removed and the modified requirement were added
  by change 5 and modified by change 6, both open: this change archives after them.
- `lingua-data-packs`: MODIFIED — *A pack names the language it studies*, whose *An Italian pack
  at Italian's analyser version* no longer has a `0.1.0` pack load; held by `add-lingua-french-baseline`,
  `add-lingua-french-analysis`, `ignore-lingua-soft-hyphens` and `add-lingua-italian-baseline`,
  after which this change archives.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/{tokenize.rs, italian.rs (new), lemmatize.rs,
  function_words.rs, mod.rs}` and `engine.rs` (`document_names`'s Italian arm);
  `crates/lingua-core/tests/italian_fixtures.rs` (at least 70 cases more).
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs, languages.rs}` and
  `baseline/it-fr.golden` (re-blessed once); `scripts/lingua-data/testdata/it-fr/` (the manifest at
  `1.0.0`, the forms the probes need).
- `apps/lingua-extension/test/packs.spec.ts` (`coreAnalyzerVersion("it", modRs)` reads a version at or above `1.0.0`, by a pattern as French's).
- `scripts/lingua-data/measure/it-ud.sh` (new): the cascade's lemmas against UD Italian-PUD and
  MarkIT's test section, run once change 9's tables exist.
- Untouched: every committed table and pin, the six goldens, the English, Spanish and French arms,
  the extension's bundle, the agent, the server, the site.
