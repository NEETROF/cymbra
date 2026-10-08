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
committed snapshot (`toMatchFileSnapshot`), re-blessed with `yarn vitest -u
test/word-card-es-en.spec.ts`; `lingua-pack-update` re-blesses it beside the Rust goldens (with
Node). Nothing reads the tables, so nothing copies the engine's grouping into TypeScript.

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

## Risks / Trade-offs

- **A golden that moves with every re-reduction** → re-blessed on the update's branch, as en-fr's and
  es-fr's; a re-bless says why.
- **Lower-case or translation-table glosses that read thin** → shown in the pull request with the
  sample; capitalising for display, or not, is the owner's call (M9), a renderer option, not data.
- **French bytes** → `word-grammar.spec.ts` unchanged; change 23's snapshot holds every French row.

## Migration Plan

No release: tests, a renderer's tables and the row cut, all inert until change 34.
