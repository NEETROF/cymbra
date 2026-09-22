# add-lingua-expression-table — the pack learns expressions

## Why

The selection card answers a phrase word by word, and says so. For a free combination that
is honest and useful; for an expression it is the wrong answer. Selecting `put up with`
today shows nothing at all — the reader knows `put`, `up` and `with` are function words, so
no row qualifies and the card reads "Pas de traduction dans le pack pour cette expression."
The dictionary has the answer and we throw it away: `reduce-en-fr.py` keeps a Wiktionary
entry only when its headword is in the frequency lexicon, and that lexicon is built from a
single-word list, so all 33 404 multi-word entries of the French Wiktionary's English
section are dropped at build time.

Measured on the current source snapshot, 17 437 of them survive the reducer's filters, and
the builder keeps those whose every word the pack's lexicon holds. They carry exactly what the
word-by-word rows cannot say:

| Selection | Today | With the table |
|---|---|---|
| `put up with` | no translation | Supporter, subir |
| `gave up` | no translation | Abandonner |
| `raining cats and dogs` | rows for `cat` and `dog` | Pleuvoir à verse |
| `starting point` | rows for `start` and `point` | Point de départ |
| `in spite of` | row for `spite` | En dépit de |

This is the answer the word-by-word rows were written to step aside for: the phrase-gloss
requirement already says they show "only when the card has no better answer".

**Products.** Cymbra Lingua only: the data pipeline (`scripts/lingua-data`), the pack
builder (`crates/lingua-pack`), the core (`crates/lingua-core`) and the extension's card
(`apps/lingua-extension`, all three variants). No backend code, no `.proto`, no migration,
no flag, no permission and no store-listing change — but one committed backend file moves:
`backend/lingua/packs-manifest.json`, the registry the ops console serves, whose entry
records the testdata pack's size. Cymbra ID, Music, Live, the back office and the site are
untouched.

## What Changes

- **The pack gains an expression table.** A new optional section holds expressions keyed by
  their **dictionary-form sequence** (`gave up` and `give up` are one entry, `give up`) with
  one gloss each, built from the kaikki/Wiktionary source and licence already shipped.
  Measured on a real build: 17 437 entries, 345 KB of sections (a 167 KB key index and a
  178 KB compressed gloss blob); the pack goes from 1 199 437 B to 1 543 992 B, 29.4 % of
  the 5 MiB the builder enforces.
- **The core finds expressions in a selection.** The phrase gloss returns, beside its
  tokens, the expressions it recognises: the longest dictionary-form sequence that matches,
  over a bounded window, reported with the tokens it covers.
- **An expression becomes the card's answer.** When one covers the whole selection it is
  shown as the answer, ahead of the word-by-word rows, and — unlike a machine translation or
  a row — it is **stored on the card**, because it is dictionary data, as a word's gloss is.
  The card and its status are keyed by the expression's dictionary form, so `gave up` and
  `give up` become one card instead of two, and the card offers the actions of a word,
  "Je connais" included: a keyed expression is a lexical unit the reader can settle.
- **An expression inside a longer selection** replaces the rows of the words it covers, and
  is not stored.

Not in this change: machine translation of any kind, the "Traduction étendue" setting,
contexts on the review card, discontinuous expressions (`turned the offer down`), and
expression-aware page highlighting — the percentage and the paint keep counting single
words, so `analyse_page` is untouched.

## Capabilities

### Modified Capabilities

- `lingua-data-packs`: an ADDED requirement for the expression table (its source, its keys,
  its licence, its optionality), and two MODIFIED — the container's contents, whose present
  wording also claims that adding the level table bumped `analyzer_version`, which it did
  not; and the size budget, whose arbitration order the table joins at the head.
- `lingua-analysis`: an ADDED requirement for expression lookup inside a phrase gloss.
- `lingua-browser-extension`: an ADDED requirement for an expression as the card's answer
  and as its key, and one MODIFIED — the word-by-word requirement, whose `put up with`
  scenario this change answers.

### New Capabilities

_None._

**Archive order.** `add-lingua-phrase-gloss` is a prerequisite and SHALL be archived first:
this change extends the phrase gloss and the card that change introduces, and modifies one
of its requirements, which reaches `openspec/specs/` only when it archives.

## Impact

- **Pipeline**: `scripts/lingua-data/reduce-en-fr.py` emits `mwe.tsv` from the multi-word
  entries its existing form-of filter already cleans; `build.sh` needs no new source, the
  entries coming from the kaikki file it already downloads.
- **Builder**: `crates/lingua-pack` reads the optional `mwe.tsv` as it reads `level.tsv`,
  and **normalises every key through `lingua_core::analysis::lemmatize::lemmatize`** — the
  crate already depends on the core — so a key is exactly what the reader's own cascade
  produces at runtime.
- **Core**: `crates/lingua-core` gains the section reader and the longest-match lookup, and
  the phrase gloss reports the spans it finds. `crates/lingua-wasm` carries them through;
  the **phrase-gloss golden** gains the new field, while the page-analysis golden must not
  move.
- **Extension**: the selection card prefers an expression, keys the card by it, stores its
  gloss, and drops the rows it covers.
- **Pack version, not analyser version**: the section is additive and only a new interface
  reads it, so `ANALYZER_VERSION` does not move and `analyse_page` output is byte-identical
  — the same rule the level table followed. `pack_version` moves.
- **Committed artefacts that follow the testdata**: the vitest fixture pack
  (`apps/lingua-extension/test/fixtures/en-fr.testdata.lingua`) and
  `backend/lingua/packs-manifest.json`, both regenerated.
- **Dogfooding**: needs `yarn gen:pack:real`.
