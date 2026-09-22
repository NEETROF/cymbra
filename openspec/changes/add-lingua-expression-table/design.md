## Context

`reduce-en-fr.py` builds the pack's glosses from the French Wiktionary's English section
(kaikki). It keeps an entry only when its headword is in the frequency lexicon, and that
lexicon comes from wordfreq's single-word list, so **every** multi-word entry is dropped:
33 404 of them in the current snapshot. Its form-of filter (`_FORM_OF`) is already thorough
— `pluriel`, `participe`, `prétérit`, `passé`, `imparfait`, `variante`, `autre graphie` and
more, plus kaikki's own `form-of` tag — and drops 9 970 senses of those multi-word entries
on its own.

The selection card, since `add-lingua-phrase-gloss`, answers a phrase word by word under a
label saying it is not a translation, and only "when it has no better answer". For
`put up with` it has nothing at all to say: `put` is known, `up` and `with` are function
words, no row qualifies, and the card states that the pack has no translation.

The pack container is a generic envelope — a section is a name and a blob, and `Pack::load`
looks sections up by name and ignores the rest — so the level table was added without moving
`ANALYZER_VERSION`. The requirement describing the container still claims otherwise.

`crates/lingua-pack` already depends on `lingua-core`, and its `build()` has the assembled
lexicon in hand before it writes the sections.

## Goals / Non-Goals

**Goals:**

- Give the card the dictionary's answer for an expression, keyed so that `gave up` and
  `give up` are one card.
- Keep a key and a runtime lookup that cannot drift apart.
- Stay additive: no `ANALYZER_VERSION` move, no change to `analyse_page`'s bytes, no
  permission, no listing, no backend code.

**Non-Goals:**

- Discontinuous expressions (`turned the offer down`). A selection is one contiguous range,
  and a gap-tolerant verb+particle rule is its own change.
- Expression-aware counting or highlighting. The percentage keeps counting single words.
- Machine translation, the download setting, review contexts: later changes.
- Touching the shared form-of filter. It already does its job (see D2).

## Decisions

### D1 — The key is the dictionary-form sequence, built by the core's own lemmatiser

An entry's key is its words, each lemmatised, joined by single spaces: `starting point`
becomes `start point`, `raining cats and dogs` becomes `rain cat and dog`. At runtime the
phrase gloss already holds each token's dictionary form in reading order, so a lookup is a
join and a compare.

The keys are built **in `lingua-pack`, by calling
`lingua_core::analysis::lemmatize::lemmatize` against the lexicon that build just
assembled** — not in Python. The pipeline can already mirror one step of the cascade (it
parses `IRREGULARS` straight out of `lemmatize.rs`), but not the two that follow: the
morphy-style suffix rules, which only accept a candidate the lexicon confirms, and the
out-of-lexicon plural fallback. A Python key would diverge wherever those decide, and such
an entry would sit in the pack unreachable for ever. Building the key where the cascade
lives makes that impossible, and costs the builder one call per word.

`lemmatize` never fails — its last resort is the lowercased form itself — so "a word the
analysis cannot produce" is not a failure to catch but a membership test: the key is kept
only when every one of its words is in the lexicon the build assembled.

_Alternative considered_ — key by the words as written, and lemmatise nothing. Rejected:
`gave up` would miss `give up`, which is most of the value, and the table would carry an
entry per inflected spelling.

### D2 — Two entries reaching one key: the dictionary spelling wins

Lemmatising the key makes distinct headwords collide: `breaking point` and `break point`,
`base jumping` and `base jump`, `breast feeding` and `breast feed`. 143 keys are reached by
two spellings, and their glosses differ in substance — "Point d'arrêt" against "Point de
rupture". Sorted arbitrarily, the gerund entry would win half the time.

The rule is the simple one: where two entries reach one key, the entry whose headword **is**
the key keeps it. No change to the shared form-of filter is needed — it already drops the
"Prétérit de give up" wordings, and extending it with the four it lacks (`Présent`, `Futur`,
`Conjugaison`, a bare `Graphie`) would remove 8 senses while risking the single-word tables,
since the same regex feeds them. Those four are filtered in the multi-word path only.

### D3 — Longest match, over a bounded window, non-overlapping

Runs are tried longest first from each token; a token belongs to at most one match. The
window is **five tokens**: 98.9 % of the table is five words or fewer (10 963 of two, 2 447
of three, 619 of four, 151 of five), and a selection is bounded at 120 characters anyway, so
a lookup is a few dozen probes. 406 keys extend a shorter key (`look forward` /
`look forward to`), which is why longest wins rather than first.

### D4 — What the card does with a match

- **Covering the whole selection** — the expression is the answer, shown where a word's
  gloss is shown, and **stored** on a card the reader creates. It is dictionary data, like a
  word's gloss; the rule that keeps the word-by-word rows off a card is written about the
  rows, and an expression is not one. The card and the status are keyed by the expression's
  dictionary form, so the deck holds one `give up`, and the card offers a word's actions,
  "Je connais" included — the knowledge model already treats an expression as a lemma of its
  own.
- **Covering part of a longer selection** — it takes the place of the rows of the words it
  covers, in the same list, and is not stored. An expression the reader has settled gives no
  row, exactly as a settled compound does.
- **No match** — today's behaviour.

Two mechanics this needs in the extension, neither of them free:

- `Gesture` carries no gloss today, and `cardGloss` asks the single-lemma pack port, which
  cannot answer `give up`. The expression's gloss therefore travels on the gesture.
- The card renders its rows **instead of** the gloss line and returns before reading it, so
  the whole-selection case must pass no rows for its answer to show.

### D5 — The section, and why `ANALYZER_VERSION` stays put

A new optional section, keys in an `fst::Map` (as the forms table is, sorted byte-wise, which
puts a space before every letter) pointing into a zstd-compressed gloss blob (as the gloss
table is). `Pack::load` parses it when present and leaves the reader's other tables alone; a
core that predates it ignores it. Only the new lookup reads it, `analyse_page`'s output is
unchanged, so `ANALYZER_VERSION` does not move and the page-analysis golden must match
untouched. `pack_version` moves, as it does for any data change.

Measured: 14 337 entries, 597 KB of TSV — **191 KB of gloss blob at zstd-19**, plus the key
FST, whose 210 KB of raw keys should fold to about 100 KB (the pack's forms FST folds 684 KB
of keys into 322 KB, 47 %). The pack goes from 1 199 437 B (22.9 % of the 5 MiB the builder
enforces) to roughly 1.49 MB, about 28 %. The
size budget is one requirement, not two: the expression table joins the head of its
arbitration order, and `BuildError::OverBudget`'s message — today "reduce glossed lemmas" —
has to name what is actually at fault.

### D6 — What is left out of the table

- Entries whose only part of speech is `name` (5 049): proper nouns, which the card already
  refuses to gloss.
- Entries holding a word the lexicon does not hold (3 045): the key would be unreachable.
- Entries with no sense left after filtering.
- Non-ASCII headwords, as the single-word reducer already does.

### D7 — Archive order

This change modifies "Word-by-word gloss is a labelled last resort", a requirement that
reaches `openspec/specs/` only when `add-lingua-phrase-gloss` archives, and both its ADDED
requirements are written as extensions of that change's phrase gloss and card. It therefore
archives **after** it. That is also what makes the `put up with` scenario consistent: this
change answers the selection the other one had no answer for.

## Risks / Trade-offs

- [A wrong expression gloss is stored on a card, where a wrong row never was] → it is
  dictionary data from the source already shipped, cut to its first sense like a row, and a
  card's gloss stays free text the reader sees in review.
- [The table grows the pack by a quarter] → still about 28 % of the budget, and the budget
  requirement names the table as what gives way first.
- [Build-time lemmatisation ties the pack build to the core's cascade] → it already is tied,
  through the forms FST; this makes the tie explicit and testable.
- [`analyse_page` must not move] → the section is read only by the new lookup, and the
  page-analysis golden is the proof, as in `add-lingua-phrase-gloss`.
- [Three committed artefacts follow the testdata pack] → the vitest fixture, the parity
  fixture and `backend/lingua/packs-manifest.json`; each has its own task, and
  `check-manifest` is the guard.

## Migration Plan

None. A rebuilt pack ships with the extension; nothing stored changes. Cards created before
this change under a surface key (`gave up`) are not migrated — they are few, and merging two
review histories is not worth it.

## Open Questions

None.
