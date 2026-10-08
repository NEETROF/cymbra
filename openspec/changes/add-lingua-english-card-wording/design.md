# Design — add-lingua-english-card-wording

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `crates/lingua-wasm/tests/support/mod.rs` | `Scenario {pair, beside, test, pages, page_names, lemmas, phrases, grammar, reader_pages, statuses, exposures, cards, reader_phrase}`; the pack built from the committed tables (`<pair>/` and `<studied>/`); `golden_path` gives `<pair>.golden`; `LINGUA_BLESS=1` re-blesses; a golden's grammar probe is `### word-grammar <word> <lemma>` and one JSON line |
| `tests/english_baseline.rs`, `tests/spanish_baseline.rs` | en-fr's (alone) and es-fr's (en-fr beside it) goldens; `tests/support/spanish.rs` holds es-fr's probes; `beside: &[]` is how en-fr's runs alone |
| `.github/workflows/lingua-extension-check.yml` | its invariance step runs `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline` — the one baseline gate on a tables-only pull request, which does not start `rust` |
| `.github/workflows/lingua-pack-update.yml` | re-blesses with `LINGUA_BLESS=1 … --test english_baseline --test spanish_baseline` on the update's branch |
| `tests/cross_native.rs` | single-pack engines compared on their studied sections; `cross_native.rs` compares the real es-en pack's studied sections with es-fr's (change 21) |
| The pack's sense runs | the builder takes a noun's gender from its readings; the engine groups senses (`group_senses`) and falls back to one untagged group when the counts do not match — what the card shows is the engine's answer, not the table's line |
| `apps/lingua-extension/src/reading/{gloss-pages,selection-card,grammar-labels}.ts` | `glossPages`, `rowGloss` (first sense, ≤ 80 characters, cut at a word; `EMPTY_SENSE = /définition manquante/i`, matching no committed gloss; the cut's trailing set `[\s,;:(«[\-–—/]`), `senseHeading` — pure functions |
| Change 18 | the renderers and `word-grammar-en.spec.ts` over the French spec's made-up inputs |

## Goals / Non-Goals

**Goals:**
- es-en's card bounded by a golden of its own, from the engine's JSON to the rendered lines, run
  and re-blessed with the others.
- The English wording corrected on real forms and real glosses.

**Non-Goals:**
- New pack data; the card's interface copy and the studied words' `lang` (14, 18); shipping (change 34).

## Decisions

### D1 — An es-en baseline, the reference's probes and 40 more

`es_en_baseline.rs` declares `Scenario { pair: "es-en", beside: &[], test: "es_en_baseline", cards: […], grammar: […], ..SPANISH }`
— es-fr's corpus, lemmas, phrases and grammar probes, each card's shown gloss the es-en pack's
first page for its lemma — and 40 more lemmas asked as `word-grammar <lemma> <lemma>` probes after
the reference's: the 40 most frequent lemmas of `tables/es/freq.tsv` whose es-en gloss has two sense runs or
more, so that their glosses and sense runs, genders included, are the engine's. On the probes they
share, the two goldens differ only in the native side, the lines naming the packs, `notice`, `licences` and the backup's
profile (es-fr's golden is rendered with en-fr beside it; this one alone, so the lines naming the packs (`pack`, `beside en-fr`) differ too) — asserted by a test that compares the two committed goldens probe by
probe through `cross_native.rs`'s `studied_side`. `es_en_baseline` joins the
invariance step of `lingua-extension-check.yml` and the re-bless line of `lingua-pack-update.yml`, so
a re-reduction of es-en or of es-fr (which moves the studied tables) re-blesses it on the update's
branch and a tables-only pull request runs it.

### D2 — The wording pinned from the golden, never the tables

`test/word-card-es-en.spec.ts` reads every `word-grammar` probe of the committed golden — the
reference's and the 40 — renders each through `src/i18n/en/grammar.ts` and the card's layout functions
(`senseHeading`, `glossPages`, `rowGloss`) with the interface in English, and pins the lines in a
committed snapshot (`toMatchFileSnapshot`), re-blessed with `yarn vitest run
test/word-card-es-en.spec.ts -u` (the flag after the file: vitest's `-u [type]` would take a file
written after it as its value); `lingua-pack-update` re-blesses it beside the Rust goldens (with
Node). Nothing reads the tables, so nothing copies the engine's grouping into TypeScript. A word
the pre-pass split is rendered on its piece, as the card opens it (`openForToken`: the token's
`surface` is the piece, `written` the whole word): « al » on « a ».

### D3 — Corrections on real forms

The renderer's Spanish-studied tables — tense names and their order (the preterite, the imperfect,
the present subjunctive), the gerund's name, which moods are named — are read against the probes'
real readings and corrected where a line reads wrong in the English Wiktionary's form-of wording
(change 18 D4); each correction is a line of the snapshot, and change 18's `word-grammar-en.spec.ts` moves with it.
The owner reviews every line (M9), in this pull request.

### D4 — The row cut (this change owns its rules)

The empty-sense pattern stays: it matches no gloss of any edition the packs read, each reducer
leaving its edition's placeholders out. The cut's trailing set gains the opening marks “ and ‘ only
if the committed es-en rows end on one — measured in the pull request; a closing mark is never
stripped (’ is also the apostrophe). A test computes `rowGloss` over every gloss of
`tables/en-fr/gloss.tsv` and `tables/es-fr/gloss.tsv` and compares them with a snapshot recorded
from the code before this change, committed in the pull request's first commit; change 24 adds the
Spanish edition's opening marks to the same rule and test.

### D5 — Dogfood

A development build with `packs.json` listing es-en beside en-fr and es-fr — local, never committed
(`check:variants` refuses it; restored before the gates) — so change 20's choice shows, and the
interface set to English; twenty pages read on desktop Chrome, signed out or signed in to a test
account, never the owner's production account (rule *Server first*, risk 2). There is no translated
sentence before change 25's route. Every card that reads wrong is fixed here — wording (D3), the row
cut (D4), or, when the gloss itself is wrong, a rule of `reduce_edition_*.py` or the pair's reducer
that re-pins es-en alone (change 6 D2) — or listed for change 33 when the fix is the owner's wording
call, or for a follow-up named in the pull request.

### Known data defects

What the snapshot pins that reads wrong and is not the renderer's wording (D3), named by the card,
not by a table line, each with the module its fix lives in — the input of the es-en reducer fix
planned after this change, which re-pins es-en alone (change 6 D2) and re-blesses the snapshot.

The readings (the studied side, `tables/es/` as es-fr's reduction `reduce-es-fr.py` writes it, or
`lingua-core`):
- « al », « del »: the pieces « a » and « de » are read as nouns with a gender — the letters' names
  (« a » feminine and masculine, « de » feminine) — not as the preposition. The card names none of
  them (the piece is its own dictionary form), but the readings are the golden's: the Spanish
  tables, or `lingua-core`'s `word_grammar` taking the piece's readings under the preposition.
- « hecho »: « may also be the first-person singular present indicative of hechar », a misspelt
  lemma read as a verb (Wiktionary's misspelling entry): the Spanish tables.
- « dámelo », « levantarme », « fríelas »: an enclitic form gets no reading and no line — the
  pre-pass splits « al » and « del » only (`lingua-core` `analysis/tokenize.rs`), and the forms
  table lists « dámelo » without a reading, the others not at all.
- « gran »: its line names the noun « grande »'s genders (masculine and feminine singular); the
  adjective's reading carries no gender, so it is not named: the Spanish tables.

The glosses (the native side: the English Wiktionary's rules `scripts/lingua-data/reduce_edition_en.py`,
or the pair's reducer `scripts/lingua-data/reduce-es-en.py` with the order every pair shares in
`reduce_common.py`):
- « venir »: glossed by its sense-group headers, « Senses relating to literal movement; Figurative
  senses », not by a sense — `reduce_edition_en.py`.
- « su »: « apocopic form of suyo », a form-of line kept as a gloss — `reduce_edition_en.py`.
- « a » (and « al »): « Used to express indiference or sarcasm », Wiktionary's typo — `reduce_edition_en.py`
  or the entry upstream.
- « bueno » (« buen »): an IPA transcription inside a sense, « /bweˈno/, rather than /ˈbweno/ » —
  `reduce_edition_en.py`.
- « ya »: « (difference from sense 4 depends on context) », a sense number of the source —
  `reduce_edition_en.py`.
- « otro »: straight quotes and no ¡ ¿ in « "Not again!" or "What, again?" (also Otra vez! or Otra
  vez?) » — `reduce_edition_en.py`.
- « pero »: « well well, so, well (used for emphasis) » — `reduce_edition_en.py`.
- Ellipses written three ways: « not...anything » (« nada »), « either ... or » (« bien »),
  « either … or » (« o »), « sometimes...other times », « now...now, whether...or » (« ahora »),
  « both ... and » (« tanto ») — `reduce_edition_en.py`.
- « etc » without its period: « cómo, cuándo, etc » (« qué »), « often, etc » (« tanto ») —
  `reduce_edition_en.py`.
- Rows opening on an unexpected group or sense: « como » on the city of Como (the proper noun
  first), « primero » on « former (in contrast to the latter) » (the noun before the adjective
  « first »), « hasta » on « even » (the adverb before the preposition « until »), « estado » on
  « country, land » before « state », « yo » on « first-person singular pronoun in the nominative
  case, I » — `reduce-es-en.py` (the runs' order, `reduce_common.py`) and `reduce_edition_en.py`
  (the sense kept first).
- Senses shown with no label: « o » as the adverb « where », « ese » as the interjection « hello »,
  with nothing saying where or when they are used — `reduce_edition_en.py`.

Wording the spec keeps, for the owner (M9, change 33), not data: a noun's plural names its gender
(« feminine plural of casa », where the English Wiktionary writes "plural of casa") because every
renderer names the gender and number the French names (generalise-lingua-card-wording: « A noun's,
adjective's, determiner's or pronoun's form SHALL name its gender and number »); an apocope reads
« masculine singular of bueno » (« buen », « primer »), not "apocopic form of bueno", because no tag
of the engine's closed vocabulary tells an apocope from any masculine singular — naming it needs a
feature in `lingua-core`'s vocabulary and every renderer.

## Risks / Trade-offs

- **A golden that moves with every re-reduction** → re-blessed on the update's branch, as en-fr's and
  es-fr's; a re-bless says why.
- **Lower-case or translation-table glosses that read thin** → shown in the pull request with the
  sample; capitalising for display, or not, is the owner's call (M9), a renderer option, not data.
- **French bytes** → `word-grammar.spec.ts` unchanged; change 23's snapshot holds every French row.

## Migration Plan

No release: tests, a renderer's tables and the row cut, all inert until change 34.
