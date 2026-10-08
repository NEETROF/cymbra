# Design — add-lingua-english-card-wording

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `crates/lingua-wasm/tests/support/mod.rs` | `Scenario {pair, beside, test, pages, page_names, lemmas, phrases, grammar, reader_pages, statuses, exposures, cards, reader_phrase}`; the pack built from the committed tables (`<pair>/` and `<studied>/`); `LINGUA_BLESS=1` re-blesses |
| `tests/english_baseline.rs`, `tests/spanish_baseline.rs` | en-fr's and es-fr's goldens; es-fr holds en-fr beside it, as the extension does; an engine holds one native's packs, so a es-en engine holds nothing of French |
| `tests/support/english.rs`, `spanish.rs` | the probes: lemmas, phrases, (word, dictionary form) grammar pairs — 32 English, 28 Spanish |
| `tests/cross_native.rs` | the studied sections byte-equal across natives (change 21 replaces the Spanish synthetic pack with the real es-en one) |
| `apps/lingua-extension/test/word-grammar.spec.ts` | 108 assertions on French wording over synthetic `WordGrammar`; change 18 adds `word-grammar-en.spec.ts` and `-es.spec.ts` over the same inputs |
| `src/reading/wordpopup.ts`, `gloss-pages.ts`, `selection-card.ts` | the card: headword, seen form, rarity, listen, grammar lines, gloss (senses joined "; ", grouped by sense runs only when they rebuild the gloss, pages of 160 characters of whole senses), rows (first sense, ≤ 80 characters, cut at a word); `EMPTY_SENSE = /définition manquante/i`; the cut's trailing set `[\s,;:(«[\-–—/]` |
| The es-en tables (change 21) | the English Wiktionary's Spanish section, reduced by `reduce_edition_en.py`: its senses start in lower case (98.3 %, `capitalised=False`), its long explanatory parentheses are change 21's setting (M20), its translation-table glosses are words |
| The packs' sense runs | UD tags (`NOUN|Gender=Fem`, `VERB`…), the gender from the readings for any pair; no usage, register or regional label anywhere |

## Goals / Non-Goals

**Goals:**
- es-en's card bounded by a golden of its own, from the engine's JSON to the rendered lines.
- The English wording corrected on real forms and real glosses.

**Non-Goals:**
- New pack data (usage labels); the card's interface copy (14); en-fr's or es-fr's bytes.
- Shipping es-en (change 34).

## Decisions

### D1 — A es-en baseline, the reference's probes

`es_en_baseline.rs` declares a `Scenario` for es-en, no pair beside it (one native per engine), over
`baseline/pages-es.txt` (the Spanish corpus es-fr's golden reads) and es-fr's 28 Spanish word-grammar probes (`tests/support/spanish.rs`) — the same questions as the reference golden of Spanish, so that
a difference between the two goldens is a difference of native side alone. The studied sections are
byte-equal (`cross_native.rs`); the golden pins what differs: glosses, sense runs, expressions,
grammar JSON. It is blessed once, in this change's pull request; a es-en re-reduction re-blesses
it as the others are.

### D2 — The wording pinned from the golden's JSON

`test/word-card-es-en.spec.ts` reads the golden's `word-grammar` probes (parsed from the
committed file, so the spec moves when the golden does) and a fixed sample of 40 glossed lemmas
with their sense runs (read from the committed tables at test time), renders each through
`src/i18n/en/grammar.ts` and the card's layout functions with the interface in English, and pins the
lines: grammar lines, headings, pages and rows. A renderer change that moves an English line
fails here, as a French one fails `word-grammar.spec.ts`.

### D3 — Corrections on real forms

The renderer's tables for Spanish studied — tense names, their order, which moods are named — are
read against the probes' real readings and corrected where a line reads wrong in the English Wiktionary's form-of wording (change 18 D4);
each correction is a line of the spec. The owner reviews every line (M9), in this pull request.

### D4 — The card's French-only heuristics

`rowGloss`'s empty-sense pattern becomes one pattern per edition the packs read (the French
Wiktionary's « définition manquante »; none for an edition that drops undefined senses), and the
cut's trailing set gains “ ” and ‘ ’. A French gloss is cut and skipped exactly as before; a test
holds en-fr's and es-fr's rows byte for byte over their committed tables' first 500 lemmas.

### D5 — Dogfood

A development build with `packs.json` listing es-en — built locally, never committed — and the
interface set to English through change 20's choice; twenty pages of the corpus and of the
web read on desktop Chrome. Every card that reads wrong is either fixed here (D3, D4) or listed in
the pull request for change 33 when the fix is the owner's wording call.

## Risks / Trade-offs

- **A golden that moves with every re-reduction** → the same discipline as en-fr's and es-fr's: a
  re-bless says why.
- **Lower-case or translation-table glosses that read thin** → shown in the pull request with the
  sample; capitalising for display, or not, is the owner's call (M9), a renderer option, not data.
- **French bytes** → `word-grammar.spec.ts` unchanged; D4's test holds the French rows.

## Migration Plan

No release: tests, a renderer's tables and two heuristics, all inert until change 34.
