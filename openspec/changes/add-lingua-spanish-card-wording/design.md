# Design — add-lingua-spanish-card-wording

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `crates/lingua-wasm/tests/support/mod.rs` | `Scenario {pair, beside, test, pages, page_names, lemmas, phrases, grammar, reader_pages, statuses, exposures, cards, reader_phrase}`; the pack built from the committed tables (`<pair>/` and `<studied>/`); `golden_path` gives `<pair>.golden`; `LINGUA_BLESS=1` re-blesses; a golden's grammar probe is `### word-grammar <word> <lemma>` and one JSON line |
| `tests/english_baseline.rs`, `tests/spanish_baseline.rs` | en-fr's (alone) and es-fr's (en-fr beside it) goldens; `tests/support/english.rs` holds en-fr's probes; `beside: &[]` is how en-fr's runs alone |
| `.github/workflows/lingua-extension-check.yml` | its invariance step runs `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline` — the one baseline gate on a tables-only pull request, which does not start `rust` |
| `.github/workflows/lingua-pack-update.yml` | re-blesses with `LINGUA_BLESS=1 … --test english_baseline --test spanish_baseline` on the update's branch |
| `tests/cross_native.rs` | single-pack engines compared on their studied sections; `cross_native.rs`'s English check reads the real en-es pack in place of the synthetic `ENGLISH_IN_SPANISH` (change 22 leaves it to this change) |
| The pack's sense runs | the builder takes a noun's gender from its readings; the engine groups senses (`group_senses`) and falls back to one untagged group when the counts do not match — what the card shows is the engine's answer, not the table's line |
| `apps/lingua-extension/src/reading/{gloss-pages,selection-card,grammar-labels}.ts` | `glossPages`, `rowGloss` (first sense, ≤ 80 characters, cut at a word; `EMPTY_SENSE = /définition manquante/i`, matching no committed gloss; the cut's trailing set `[\s,;:(«[\-–—/]`), `senseHeading` — pure functions |
| Change 18 | the renderers and `word-grammar-es.spec.ts` over the French spec's made-up inputs |

## Goals / Non-Goals

**Goals:**
- en-es's card bounded by a golden of its own, from the engine's JSON to the rendered lines, run
  and re-blessed with the others.
- The Spanish wording corrected on real forms and real glosses.

**Non-Goals:**
- New pack data; the card's interface copy and the studied words' `lang` (14, 18); shipping (change 35).

## Decisions

### D1 — An en-es baseline, the reference's probes and 40 more

`en_es_baseline.rs` declares `Scenario { pair: "en-es", beside: &[], test: "en_es_baseline", cards: […], grammar: […], ..ENGLISH }`
— en-fr's corpus, lemmas, phrases and grammar probes, each card's shown gloss the en-es pack's
first page for its lemma — and 40 more lemmas asked as `word-grammar <lemma> <lemma>` probes after
the reference's: the 40 most frequent lemmas of `tables/en/freq.tsv` whose en-es gloss has two sense runs or
more, so that their glosses and sense runs are the engine's. On the probes they share, the two
goldens differ only in the native side, the lines naming the packs, `notice`, `licences` and the backup's
profile (en-fr's golden is rendered alone, as this one) — asserted by a test that compares the two committed goldens probe by
probe through `cross_native.rs`'s `studied_side`. `en_es_baseline` joins, beside change 23's
`es_en_baseline`, the
invariance step of `lingua-extension-check.yml` and the re-bless line of `lingua-pack-update.yml`, so
a re-reduction of en-es or of en-fr (which moves the studied tables) re-blesses it on the update's
branch and a tables-only pull request runs it.

### D2 — The wording pinned from the golden, never the tables

`test/word-card-en-es.spec.ts` reads every `word-grammar` probe of the committed golden — the
reference's and the 40 — renders each through `src/i18n/es/grammar.ts` and the card's layout functions
(`senseHeading`, `glossPages`, `rowGloss`) with the interface in Spanish, and pins the lines in a
committed snapshot (`toMatchFileSnapshot`), re-blessed with `yarn vitest -u
test/word-card-en-es.spec.ts`; `lingua-pack-update` re-blesses it beside the Rust goldens (with
Node). Nothing reads the tables, so nothing copies the engine's grouping into TypeScript.

### D3 — Corrections on real forms

The renderer's English-studied tables — tense names and their order (« pasado simple »), the -ing
form's name (« forma en -ing », M10), the past participle's — are read against the probes' real
readings and corrected where a line reads wrong against the names Spanish-language teaching of
English gives English forms: the Spanish Wiktionary's English form-of wording (« Pasado simple del
verbo … », « Participio pasado del verbo … ») and M10's « forma en -ing », with RAE/ASALE terms for
persons and numbers. An English form is never given a Spanish tense's name; each correction is a line of the snapshot, and change 18's `word-grammar-es.spec.ts` moves with it.
The owner reviews every line (M9), in this pull request.

### D4 — The row cut (change 23 owns its rules)

This change adds the Spanish edition's opening marks ¿ and ¡ (and “ ‘ if change 23 did not) to
change 23's trailing set, only if the committed en-es rows end on one — measured in the pull
request; « is in the set already, and a closing » or ” is never stripped (stripping » would move
en-fr's row of `viz`). Change 23's test over every en-fr and es-fr gloss holds the French rows.

### D5 — Dogfood

A development build with `packs.json` listing en-es beside en-fr and es-fr — local, never committed
(`check:variants` refuses it; restored before the gates) — so change 20's choice shows, and the
interface set to Spanish; twenty pages read on desktop Chrome, signed out or signed in to a test
account, never the owner's production account (rule *Server first*, risk 2). There is no translated
sentence before change 25's route. Every card that reads wrong is fixed here — wording (D3), the row
cut (D4), or, when the gloss itself is wrong, a rule of `reduce_edition_*.py` or the pair's reducer
that re-pins en-es alone (change 6 D2) — or listed for change 33 when the fix is the owner's wording
call, or for a follow-up named in the pull request.

## Risks / Trade-offs

- **A golden that moves with every re-reduction** → re-blessed on the update's branch, as en-fr's and
  es-fr's; a re-bless says why.
- **Translation-table glosses that read as bare words** (change 22's share) → shown in the pull
  request with the sample, definitions and table words marked; any display treatment is the owner's
  call (M9), a renderer option, not data.
- **French bytes** → `word-grammar.spec.ts` unchanged; change 23's snapshot holds every French row.

## Migration Plan

No release: tests, a renderer's tables and the row cut, all inert until change 35.
