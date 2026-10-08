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
committed snapshot (`toMatchFileSnapshot`), re-blessed with `yarn vitest run
test/word-card-en-es.spec.ts -u` (the flag after the file: vitest's `-u [type]` would take a file
written after it as its value); `lingua-pack-update` re-blesses it beside the Rust goldens (with
Node). Nothing reads the tables, so nothing copies the engine's grouping into TypeScript. A word
the pre-pass split is rendered on its piece, as the card opens it (`openForToken`: the token's
`surface` is the piece, `written` the whole word): « don't » on « do ».

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

### Known data defects

What the snapshot pins that reads wrong and is not the renderer's wording (D3), named by the card,
not by a table line, each with the module its fix lives in — the input of an en-es reducer fix after
this change, which re-pins en-es alone (change 6 D2) and re-blesses the golden and the snapshot. The
counts are over the 5,000 commonest lemmas en-es glosses (`tables/en/freq.tsv`), on the tables of
2026-10-08.

The glosses (the native side: the Spanish Wiktionary's rules `scripts/lingua-data/reduce_edition_es.py`,
the pair's reducer `scripts/lingua-data/reduce-en-es.py`, or the rules every pair shares in
`reduce_common.py`):
- Surnames on a common word's card: « will » ends on « [nombre propio] Apellido; Hipocorístico de
  William », and 263 of the 5,000 end on a bare « Apellido » (« white », « small », « south »): the
  Spanish Wiktionary's English proper-noun sections — a surname spelled like the word — glossing the
  lower-case lemma — `reduce-en-es.py` (which entries gloss a lemma), or `reduce_edition_es.py` (a
  bare « Apellido » left out, as a letter's name is).
- Editorial notes kept: « stage », « unit »: « .^([cita requerida]) »; « favor »: « Hacer un favor
  [sentido del sustantivo] para »; « full-time »: « [a] tiempo completo » — `reduce_edition_es.py`
  (`_NOTES`).
- An English label in a Spanish gloss: « south »: « (region) Sur », a translation table's
  qualifier kept — `reduce-en-es.py` (`read_translated`).
- « etc » without its period: « oh » (« …desaprobación, etc »), « personal », « fair »
  (« Moderadamente bueno, grande, amplio, etc »), 29 of the 5,000: the period closing a sense is
  stripped with the sense's own — `reduce_common.py` (`rstrip(".:")`, every pair's: an edition rule
  keeps the en-fr and es-fr bytes), as es-en's « etc » (change 23).
- Usage notes inside a sense, shown on the row: « a »: « Un, una. A veces se omite en la
  traducción »; « get »: « (Seguido de un participio pasado) Ser » — `reduce_edition_es.py`.
- Senses shown with no label (usage, register and region are not in the packs): « do »'s
  « Timar, estafar » and « Follar », « mouse »'s « laucha » (Southern Cone) and « Timorato, apocado,
  flojo » (figurative), « or »'s « [sustantivo] Oro » (heraldry) — `reduce_edition_es.py`, whether
  such a sense is kept being the owner's call (change 33).
- Possessives, demonstratives and quantifiers headed « adjetivo »: « her » (« [adjetivo] Su (de
  ella) »), « its », « that » (« [adjetivo] Ese; Aquel »), « all », « no », « any », « much »,
  « other » — the Spanish Wiktionary's « adjetivo posesivo / demostrativo / indefinido », which the
  RAE's grammar calls determinantes, filed by kaikki as `adj`; the heading is the sense run's tag, not
  the renderer's wording — `reduce_edition_es.py` (those sections read as `DET`), through
  `reduce_common.py`'s map of kaikki's parts of speech.
- Rows opening on an unexpected group or sense: « up » on « Construido » (the adjective before the
  adverb « Arriba »), « well » on « Competentemente » (before « Bien »), « lead » on « Plomo » (the
  noun before the verb « Guiar, conducir ») — `reduce-en-es.py` (the runs' order, `reduce_common.py`)
  and `reduce_edition_es.py` (the sense kept first).
- The source's own wording: « huh »: « !Um¡, !uf¡ », the marks swapped; « who'd »: « Contracción de
  el pronombre who y el verbo had, ¿Quién tenía / tuvo / había / hubo....? »; « it's »: straight
  quotes, « ("ello") »; « a »: « Por, normalmente con sentido proporción » (« de » missing);
  « read »: « Consistir de un cierto texto »; « one »: « Uno, i, I o 1 », the Roman numerals — the
  entries upstream, or a `reduce_edition_es.py` rule for the quotes.

Not defects, said here so that they are not looked for again:
- The readings (`tables/en/`, en-fr's) read right on every probe; « goes » and « leaves » name the
  noun's plural beside the verb's form on one line, as the French card does.
- « lay », « saw » and « more » asked as their own dictionary form: the analyser reads them as forms
  of « lie », « see » and « many », so the engine answers with those readings and glosses while the
  line names the word asked (« también puede ser el pasado simple de lay ») — a card the reader never
  opens, the same in en-fr's golden.
- Single translation-table words (« expedition »: « Expedición », « lighthouse »: « Faro »,
  « dove »: « Paloma »), capitalised as the edition writes its senses: the Risks' owner's call.
- No row ends on an opening mark: the Spanish Wiktionary's ¿ and ¡ stand against their words, so
  a cut lands on the space before them (D4).

Wording the spec keeps, for the owner (M9, change 33), not data: the -ing form reads « forma en -ing »
(M10; the Spanish Wiktionary writes « Participio presente y gerundio del verbo (to) run », and the
spec forbids the gerundio); a line names the dictionary form alone (« de go »), not « del verbo (to)
go », and no persons' pronouns (« (he, she, it) »), as every renderer.

## Risks / Trade-offs

- **A golden that moves with every re-reduction** → re-blessed on the update's branch, as en-fr's and
  es-fr's; a re-bless says why.
- **Translation-table glosses that read as bare words** (change 22's share) → shown in the pull
  request with the sample, definitions and table words marked; any display treatment is the owner's
  call (M9), a renderer option, not data.
- **French bytes** → `word-grammar.spec.ts` unchanged; change 23's snapshot holds every French row.

## Migration Plan

No release: tests, a renderer's tables and the row cut, all inert until change 35.
