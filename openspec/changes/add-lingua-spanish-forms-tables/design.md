# Design — add-lingua-spanish-forms-tables

## Context

See proposal.md (Why). The pipeline (`scripts/lingua-data/`, pin-lingua-pack-sources and
generalise-lingua-pack-reducer) works in two stages:
1. **Reduce**: `reduce-<pair>.py`, with the shared rules of `reduce_common.py`, turns the raw sources
   into the tables of `tables/<pair>/`, which are committed with `pin.json`.
2. **Build**: `lingua-pack-build` turns the committed tables into the pack, offline, checked against
   `pin.json`.

`forms.tsv`, `freq.tsv`, `gloss.tsv`, `NOTICE` and `manifest.json` are required; `level`, `mwe`,
`grammar` and `senses` are optional. The extension check builds the pack of every pair that has
tables. The builder refuses a pack whose `analyzer_version` is not its language's (`Pack::load`).

kaikki's extract of the English Wiktionary, Spanish section (`kaikki.org/dictionary/Spanish`) holds
811,049 entries. A lemma's entry lists its inflections with tags (`doy` = first person, present,
singular). Combined clitic forms carry the `combined-form` tag (`dámelo`, `darlo`), and their own
entries' senses name the pronoun (`object-third-person`, `object-plural`). A form's own entry points
at its lemma through `form_of` (`luces` → *luz*, and → *lucir*).

## Goals / Non-Goals

**Goals:**
- An es-fr forms table and frequency list good enough to pass the programme's gates on a treebank
  the reduction never saw.
- One lemma per form, by evidence, with a reviewed override for the rest.
- Tables committed, reproducible from pinned sources, and checked in CI like en-fr's.

**Non-Goals:**
- French glosses and expressions (`add-lingua-spanish-gloss-tables`), grammar readings
  (`add-lingua-spanish-grammar-tables`), estimated levels (`add-lingua-spanish-levels`). `gloss.tsv`
  is empty until then.
- A multi-lemma format for homographs: the programme costs it as a branch if lemma agreement stays
  under 91 %, and S1 measured 93.9 %.
- Shipping the pack (`enable-lingua-spanish`).

## Decisions

### D1 — Forms: tagged tables and form-of senses, no combined forms

A form is lowercased and in NFC, and must match `[a-záéíóúüñ]+(-[a-záéíóúüñ]+)*`. Its candidate
lemmas come from two places:
- **the forms a lemma's entry lists**, except those tagged `combined-form` and the table's own
  bookkeeping (`table-tags`, `inflection-template`, `class`, `romanization`);
- **the `form_of` targets** of a form's own entry, when the target is itself a single Spanish word
  (`llamar al pan, pan, y al vino, vino` is not), except from a sense naming the pronoun
  (`object-…` tags), which is a combined form's.

An entry with any sense that is not a form-of is a lemma, and maps to itself.

Combined forms are left out: the enclitic rule (`add-lingua-spanish-analysis` D3) resolves them,
and they are 519,030 of kaikki's forms. A combination gives a string no lemma; it does not take
away the lemmas the string has otherwise. 16,375 strings are both a combined form and a plain form:
- `principales` is *principar* + `les`, and the plural of *principal*;
- `estarse` is *estar* + `se`, which kaikki also lists plainly, as *estar*'s reflexive infinitive.

They keep their plain lemma. A combined form that is also a word of its own (an entry with a
meaning, `vete`) stays as that word. One that is neither never becomes a lemma (D3).

### D2 — One lemma per form

When a form has several candidates, the first rule that decides wins:
1. **The override list** (`OVERRIDES` in `reduce-es-fr.py`: form, lemma, reason), reviewed by a
   person. It is part of the rules: editing it moves the rules' sha256, so the tables are reduced
   again, as for any rule.
2. **GSD's counts** of the form under each candidate lemma, from the training and development
   sections of UD Spanish-GSD.
3. **The form's own entry**, when it is a lemma: *casa* over *casar* with no evidence either way.
4. **The candidate lemma's wordfreq frequency.**
5. **Alphabetical order**, so the result never depends on the order of the source.

S1 on common homographs: `fue` → *ser* (GSD 1,458 against 0), `casa` → *casa*, `como` → *como*,
`luces` → *luz*, `río` → *río*, `cuenta` → *contar*, `vino` → *vino*. `vino` is the noun by GSD
(20 against 4 for *venir*). Narrative text meets *venir* more often, and the override list is where
that judgement goes, with its reason: the list starts with that one row, `vino` → *venir*. It costs
one agreement on PUD (two nouns against one preterite), a news treebank like GSD. The card shows
the other reading either way, from the grammar tables.

*Rejected — keep every reading.* One form, one lemma is the pack's contract, which the analyser,
the knowledge model and the sync all read. The programme keeps the multi-lemma format as a costed
branch for an agreement under 91 %.

### D3 — Lemmas and cuts

- **Lemmas**: the 60,000 commonest by wordfreq `es` 3.1.1, skipping words that are only inflected
  forms (en-fr's `canonical_ranks`). The combined forms that are no word of their own are skipped
  too. Ranked, `hacerlo` would be a lemma that stands in the table for itself, ahead of the
  enclitic rule. Dense ranks go to `freq.tsv`.
- **Words kaikki does not know** (names, loans, abbreviations) are ranked as en-fr's are. A
  lowercase word outside the lexicon reads as unknown, so leaving them out would mark `etc` new.
- **Forms**: those whose chosen lemma is kept, and attested in wordfreq (a Zipf frequency above
  zero), plus each lemma's identity form. An unattested form of a kept lemma is a form nobody
  writes. The analyser's rules (old spellings, enclitics, plurals) cover the long tail.

S1, with the dictionary lookup alone:

| Cut | Forms | Size | PUD resolved | content lemmas | AUX |
|---|---|---|---|---|---|
| every form | 677,239 | 13.5 MB | 99.08 % | 94.99 % | 97.95 % |
| 40k lemmas | 166,993 | 3.3 MB | 98.14 % | 93.40 % | 97.95 % |
| 60k lemmas | 209,051 | 4.2 MB | 98.42 % | 93.96 % | 97.95 % |
| **60k lemmas, attested** | **107,647** | **2.0 MB** | **98.39 %** | **93.90 %** | **97.95 %** |

The harness (D5) measures with the analyser's rules too, which resolve the combined forms the table
leaves out. That is the figure the gates hold to. The reducer as built (D1 and D3, 144,804 rows,
2.5 MB) measures 99.38 % resolved, 95.91 % of content lemmas and 97.95 % of auxiliaries.

### D4 — The tables, in git

`tables/es-fr/` holds `forms.tsv`, `freq.tsv`, `gloss.tsv` (empty), `NOTICE`, `manifest.json`,
`pin.json` and a `README.md`, as en-fr's do. `manifest.json` names Spanish and the Spanish
analyser version read from `analysis/mod.rs`, so `Pack::load` accepts it. About 3.3 MB of text is
committed, far under the ≈10 MB the programme set as the threshold for a snapshot outside git.

### D5 — The measurement harness

`lingua-pack-measure <pack> <conllu>` loads a built pack and lemmatises every token of a CoNLL-U
file through lingua-core, as a page token is lemmatised, skipping punctuation, numbers, symbols,
proper nouns and tokens with no letter. It reports three figures:
- **tokens resolved**: the lemma is in the lexicon, through the table or a rule the lexicon
  validates;
- **content-word lemma agreement**: NOUN, VERB, ADJ, ADV;
- **auxiliary agreement**.

It exits non-zero under 98.5 %, 93.5 % and 97 %. `measure/es-pud.sh` fetches UD Spanish-PUD at its
pinned commit, builds the pack from the committed tables and runs the harness. PUD is never
committed, and the reduction never reads it.

### D6 — Pinned sources

`pin.json` records:
- **kaikki's snapshot**: the dump of 2026-09-28, fetched 2026-10-03, by sha256 and size, as the
  release asset `lingua-pack-sources-es-fr-<snapshot>` that `lingua-pack-update` publishes;
- **UD Spanish-GSD** at its commit, by sha256;
- **wordfreq** by version;
- **the reducer's sha256**: `reduce-es-fr.py` and `reduce_common.py`.

A release and a pull request build from the tables and read none of it. This change publishes no
release, so the owner publishes the snapshot asset by uploading the pinned bytes as that release.
Dispatching `lingua-pack-update` in update mode instead keeps a new snapshot and proposes tables
reduced from it. Until one of them is done, `build.sh --reduce es-fr` cannot fetch the pinned
kaikki bytes. `--update` and every build work.

### D7 — The pipeline knows es-fr

- `build.sh` already dispatches on `reduce-<pair>.py`.
- `lingua-pack-update` gains `es-fr` among its pairs, and its monthly dry run checks every pair
  instead of en-fr alone. Its release notes name the kaikki extract a pair reads, which for es-fr is
  the English Wiktionary.
- The extension check already loops over every pair with tables.
- `packs.json`, the list the extension ships, is unchanged: en-fr alone.

## Risks / Trade-offs

- **A homograph GSD gets wrong for narrative text** (`vino`) → the override list, reviewed in the
  pull request, with a reason per row.
- **GSD's counts are sparse beyond common words** → steps 3 to 5 of D2 decide. They err towards the
  word's own entry, the reading a learner looks up.
- **kaikki regenerates daily** → the snapshot pins the bytes, the tables are what releases read, and
  the monthly dry run reports drift.
