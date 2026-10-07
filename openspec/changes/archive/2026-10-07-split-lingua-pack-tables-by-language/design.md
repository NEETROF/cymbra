# Design — split-lingua-pack-tables-by-language

## Context

See proposal.md (Why). Measured in a scratch copy of origin/main 7ea6561d, with the full split
prototyped:
- en-fr and es-fr build to their pinned sha256 from the new layout, and from the reviewers' source
  archive;
- re-reducing both from their pinned sources (Python 3.12, hashed requirements) rewrites every
  committed byte in the new layout: both pair folders, both studied folders, manifests and pins;
- both goldens, `cross_native` and the committed-tables check pass;
- no `reduce*.py` file is touched;
- a simulated es-en pair builds from `tables/es/` with nothing copied, and its pack picks up the
  lexical section from `tables/es/lexical.tsv`.

## Goals / Non-Goals

**Goals:**
- One copy of each studied language's tables.
- Only the reference pair writes them.
- Every consumer reads them.
- Nothing shipped moves.

**Non-Goals:**
- A non-reference pair's pin record of the studied tables it read, its `pack_version` and its
  credits: change 21.
- Deriving the pair lists in `lingua-pack-update` (dispatch options, monthly matrix) from the
  tables: they stay hard-coded, and change 21 adds es-en.
- A per-language pin for the studied folder (D3).

## Decisions

### D1 — `tables/<studied>/`, as M24 words it

A folder is a pair when its name is `<studied>-<native>`. Otherwise it is a studied language, which
must hold `studied.json`. Build loops walk `tables/*-*/`. Reduce and update loops use
`pack_sources.py pairs`, which lists each language's reference first and, with `--after`, the pairs
reading a given reference. Checks in Python and Rust refuse any folder that is neither. A loop
written later with `tables/*/` fails loudly, because a studied folder has no `pin.json`.

Alternative: `scripts/lingua-data/studied/<lang>/`. No loop would change, but it departs from the
settled M24 wording.

### D2 — The split happens after reduction

`build.sh`'s copy step becomes `pack_sources.py split --work W --tables ROOT --pair P`:
- The pair's tables go to `tables/<pair>/`.
- The studied tables, plus a derived `lexical.tsv`, go to `tables/<studied>/`, and only when
  `studied.json` names P.
- A language with no folder yet makes its first pair the reference and writes `studied.json`.
- `tags.tsv` is kept, as a person's pin no reduction writes.

`lexical.tsv` is the reference's `gloss.tsv` keys: trimmed, non-empty, byte-sorted, each once. The
Rust check compares it with the builder's own reading of `gloss.tsv`. For the reference it equals
the glossed lemmas, so no lexical section is written and the bytes stay as they are, as the lexical
layer's D1 measured.

### D3 — One pin per pair

`studied.json` (`{"reference": "<pair>"}`) names the reference. Its `pin.json` — snapshot, sources,
rules — is the studied tables' provenance. The English forms come from en-fr's French Wiktionary
snapshot; Spanish's levels come from es-fr's French glosses.

`fetch-pinned`, `check-reducer` and `get` stay per pair. `record-build` looks for `tags.tsv` in the
pair's studied folder. en-fr's and es-fr's pins keep their exact bytes, which the reduce job
requires.

Alternative: a `pin.json` in each studied folder. The studied sources overlap the reference's
native ones, and every pin would be rewritten.

### D4 — The builder takes two folders

`lingua-pack-build --studied <studied> <pair> <out>` uses `inputs_from_dirs` /
`inputs_from_tables`. A studied table in the pair's folder, or a pair table in the studied folder,
is refused at its source, naming it. `testdata/<pair>/` fixtures stay single folders: `build.sh`
passes the same folder twice, and `inputs_from_dir` is unchanged. Measured: the pack bytes are the
same.

Alternative: assemble a scratch merged folder in `build.sh`. That costs up to 13 MB of copying per
build, and hides which side a file came from.

### D5 — A reference's change reaches its readers

When a reference pair's tables change, the same branch reduces every reader of that language again
(`pairs --after`), from each reader's own pin. A reader's glosses can depend on the studied lemma
set, so rebuilding alone could record a pack the reduce job will not reproduce. Readers do not
exist before change 21; the mechanism and its check land now, tested with fixtures.

### D6 — The consumers

| Consumer | Change |
|---|---|
| `lingua-extension-check` | build and `check-reducer` loops over `tables/*-*/`; the reduce job uses `pairs` |
| `lingua-pack-update` | `pair=all` uses `pairs` |
| `gen_pack.sh` | builds from both folders |
| `make_source_archive.sh` | copies all of `scripts/lingua-data` already; its README wording changes |
| `pack_report.py` | reports the studied folder's changes when the reference is updated |
| `gloss_coverage.py` | reads `freq.tsv` from the studied folder |
| `measure/` | reads the tables from both folders |
| `crates/lingua-wasm/tests/support` | `tables_dir` becomes two folders |
| `es_fr_grammar.rs`, `lib.rs` test helper | read the studied folder |
| `committed_tables.rs`, `cross_native.rs` | expect `lexical.tsv` in the studied folder |
| READMEs, `SOURCES.md`, `REVIEWERS.md` | describe the layout |

## Risks / Trade-offs

- **[A loop still treats `en` as a pair.]** → It fails loudly (no `pin.json`), and the folder check
  refuses an unknown folder.
- **[Python's `strip` and Rust's `trim` disagree on control characters]** when deriving
  `lexical.tsv`. → The Rust check compares it with its own reading of `gloss.tsv`. No such character
  exists today.
- **[About 400 KB of derived text in git.]** → It shows the dictionary words in diffs, and a future
  native-independent list is a one-file change.

## Migration Plan

`git mv` of the studied tables, plus the derived files, in the same pull request. Nothing shipped
moves. Rollback is a revert.
