# Design — add-lingua-spanish-gloss-tables

## Context

See proposal.md (Why). The rules every `<studied>->FR` pair shares turn the French Wiktionary's
entries for a language into a pack's glosses (`reduce_common.py`):
- `reduce_gloss` picks up to eight senses per word, groups them by part of speech and counts the
  runs (`add-lingua-word-grammar`);
- it keeps acronyms and the Wiktionary's notes to its readers out of a word's gloss;
- it lends a base's senses to a word that is only a form of it;
- `reduce_expressions` does the same for multi-word headwords.

They are driven by a `Studied`, which `reduce-es-fr.py` already defines for Spanish.

kaikki publishes the French Wiktionary's Spanish entries as a per-language file of 219,254
entries. It also publishes a dump of each whole Wiktionary edition:
- `frwiktionary/raw-wiktextract-data.jsonl.gz`, 703 MB;
- `eswiktionary/raw-wiktextract-data.jsonl.gz`, 98 MB.

kaikki has announced it will retire its per-language files (`docs/lingua/spanish-programme.md`).
The French Wiktionary's French section, which holds the translation tables, is a 3.2 GB
per-language file.

The French Wiktionary is thin on Spanish. Measured on the 2026-10-03 tables, 827 of the 857
lemmas of the top 5,000 left without a gloss have no Spanish entry at all: `través`, `sector`,
`decreto`. The other two editions fill part of that gap with translations people wrote.

## Goals / Non-Goals

**Goals:**
- French glosses and expressions for the es-fr pack, from the French Wiktionary first.
- The gaps filled where people wrote a French translation, never by a machine.
- The coverage published, as decision D4 asks.
- Sources that outlive kaikki's per-language files.

**Non-Goals:**
- Model-written glosses. The programme leaves that question open and recommends none at launch.
- A gloss read from the English Wiktionary's English definitions: a gloss is never English.
- Glossing a lemma with no source: such a card says the pack has no translation, as en-fr's does.
- The card's Spanish wording (`add-lingua-spanish-word-card`).

## Decisions

### D1 — Three files derived from two dumps, kept in the snapshot

`pack_sources.py` gains `DUMPS`: per pair, the dumps it reads and the files it derives from each.
`derive` makes one pass over a dump and writes two kinds of file:
- **`entries`**: one language's entries, each line as the dump writes it. The French
  Wiktionary's Spanish entries derived this way give byte-identical glosses, expressions and runs
  to the per-language file (measured).
- **`translations`**: each entry of one language that lists translations into another, cut down to
  its word, its part of speech and those translations (word, and sense when the table names one),
  as sorted JSON.

| File | From | Size |
|---|---|---|
| `kaikki-fr-Espagnol.jsonl` | the French Wiktionary's Spanish entries | 212 MB |
| `kaikki-fr-traductions.jsonl` | the French entries listing Spanish translations | 6.2 MB |
| `kaikki-es-traductions.jsonl` | the Spanish entries listing French translations | 1.5 MB |

A dump is never kept whole. Each derived file is kept as a zstd-compressed asset of the snapshot's
release, beside the extract, and checked by the sha256 of its decompressed bytes. That is how
kaikki's per-language files are kept today. `fetch-live` derives and records the files,
`fetch-pinned` fetches and checks them, and `assets` lists the release's files, which the update
workflow uploads.

*Rejected — the per-language files.* They are being retired, and the French section alone is
3.2 GB for a 6 MB table.

### D2 — The primary glosses: the shared rules, unchanged

`reduce_gloss` and `reduce_expressions` run over `kaikki-fr-Espagnol.jsonl` with Spanish's
`Studied`, at en-fr's limits:
- a word: eight senses, each within 300 characters and all within 800, with its runs;
- an expression: 80 characters.

Nothing in the shared module changes, so en-fr's rule set does not move.

### D3 — Two fallbacks, in order

A lemma the French Wiktionary does not gloss takes, in this order:
1. **The French translations its Spanish Wiktionary entry lists** (`sector` → *secteur*,
   `través` → *travers*).
2. **The French entries whose translation tables list it**, the commonest French word first by
   wordfreq `fr` (`decreto` → *arrêté*, `precisamente` → *juste, précisément*).

The gloss is one sense per part of speech, at most three French words each, its first letter in
capitals: `Matériau, matériel; Matériel`. Its runs count one sense per part of speech, so the card
groups it like any gloss.

A proper noun's entry glosses nothing. A place or a first name, translated, is itself
(`alcalá de la vega` « Alcalá de la Vega », `celina` « Céline »).

A fallback never replaces a gloss from the French Wiktionary, and the first fallback that says
something wins.

### D4 — Expressions

The French Wiktionary's Spanish multi-word entries come first. Then the same two fallbacks add the
multi-word headwords they translate (`tener en cuenta` « Prendre en compte, tenir compte de »,
`dar cuenta` « Rendre compte »). Their parts of speech are merged, since an expression's gloss has
no runs.

The builder keys an expression through the lexicon and drops one it cannot reach
(`add-lingua-expression-table`), so the many long headwords of the French tables cost the pack
nothing when their words are not in the lexicon.

### D5 — Curated locutions: the mechanism, no rows

`LOCUTIONS` in `reduce-es-fr.py` maps an expression to its gloss and wins over every source.
Decision D4 asks for such a list (« hay que », « tener en cuenta »). Measured here,
`tener en cuenta` already comes from both fallbacks, while `hay que` and `volver a` come from none.

Each row's gloss is written by a person and reviewed in the pull request, like an override. A
model-written gloss is the programme's open question, and its recommendation is none at launch. The
list ships empty: its rows are the owner's, and editing it is a rule change that reduces the tables
again.

### D6 — The coverage, published

The README and `SOURCES.md` give the share of the commonest lemmas glossed, French Wiktionary alone
and with the fallbacks, and the expression count by source, as D4 asks.

### D7 — Licences and credits

All three files are Wiktionary content, CC BY-SA 4.0 + GFDL, read through kaikki. `NOTICE` credits
the French and Spanish Wiktionaries beside the English one. The manifest's `kaikki` source covers
all three editions, so no new licence enters the pack.

## Risks / Trade-offs

- **A fallback is thinner than a definition**: a translation is a word, not a meaning. It is still
  a French word a person chose, and the card groups it by part of speech like any gloss.
- **An inverted table gives near-synonyms**: `casa` would read *foyer* beside *maison*. Only a
  lemma the other sources leave out takes one, and the commonest French words come first.
- **The dumps are large to fetch** (801 MB). Only an update reads them; a release and a pull
  request read the committed tables.
- **kaikki's schema drifts** → `derive` reads `lang_code`, `word`, `pos` and `translations` only.
  The monthly dry run reports a collapse.
