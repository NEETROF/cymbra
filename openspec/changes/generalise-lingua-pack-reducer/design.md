## Context

`scripts/lingua-data/reduce-en-fr.py` turns the pinned raw sources of the en-fr pair into seven
committed tables (`tables/en-fr/`). `build.sh` dispatches a pair to `reduce-<pair>.py`;
`pack_sources.py record-build` stores the sha256 of that one file in `pin.json`, `check-reducer`
compares it in the check lane, and a re-reduction's pack version is `<snapshot>+<digest[:7]>`.

A dependency graph of the file's top-level definitions (by AST) shows the French-side rules —
gloss cleaning, sense grouping, expressions, form resolution, ranks, level extras — depend on
English only through four module globals: `_TOKEN` (`[A-Za-z][A-Za-z'\-]*`), `_FORM_OF_TARGET`
(which skips an English `to`), `_COORDINATORS` (seven English conjunctions) and the literal
`"en"` given to wordfreq.

## Goals / Non-Goals

**Goals:**

- One copy of the rules every `<studied>->FR` pair shares, with the gate covering it.
- Proof that the split changes no English table.
- A second pair can be registered and built without editing en-fr's code paths.

**Non-Goals:**

- Writing the Spanish reducer or registering Spanish sources (`add-lingua-spanish-forms-tables`,
  `add-lingua-spanish-gloss-tables`).
- Several kaikki extracts per pair, or reading kaikki's raw dump instead of the per-language file
  it announces as deprecated: the en-fr pin already keeps its own snapshot as a release asset, so
  reproducibility is not at stake; the next `update` is, and the Spanish tables change (which
  needs two extracts) generalises the kaikki entry.
- Passing the pack's language to lingua-core's lemmatiser in the builder
  (`generalise-lingua-pack-build`).
- A per-pair size budget.

## Decisions

### D1 — Move whole definitions, parameterise four values

The shared module is built by moving top-level definitions intact (with their comments), then
replacing exactly the four English values by fields of a frozen `Studied` passed as a
keyword-only `studied` argument. The functions that received it are those that read one of the
four values, and the two that call them (`_read_entries`, `reduce_gloss`).

`reduce-en-fr.py` declares `EN = Studied(...)` from the same globals it already had and binds the
shared functions to it with `functools.partial` under their existing names. Its `main` and its
tests call them exactly as before.

*Rejected — a class per pair with overridden hooks.* More indirection for four values, and the
tests would have to construct it.

*Rejected — copy the file for Spanish.* Two copies of ~500 lines that must stay identical, and a
gate that can only see one of them.

### D2 — The rule set is the pair's reducer and every `reduce_*.py`

A pair's rules are `reduce-<pair>.py` (hyphen) and the shared modules (underscore) beside it.
`rules_sha256` hashes each file's name and sha256 in order; `pin.json` keeps that digest under
`reducer.sha256` and the list under `reducer.files`. A change to a shared module therefore moves
every pair's digest and fails every pair's check until each is reduced again — which is the point:
their tables were made by the old rules. A new pair's reducer never enters another pair's set.

`build.sh --reduce` names the pack version after the same digest (`pack_sources.py rules`), so
the version and the gate cannot disagree.

### D3 — Prove the split on the pinned sources, before and after

Reference run: `build.sh --reduce en-fr` with the unchanged reducer reproduces the committed
tables and pack exactly (sha256 `fd3d9351…`, 1,835,497 bytes), so the pipeline is reproducible on
the machine used. Split run: the same command with this change gives the seven data tables
byte-identical; only `NOTICE` (D4), `manifest.json`'s `pack_version` (D2) and `pin.json` differ.
The check lane then holds that state.

### D4 — Fix the attribution in the same change

wordfreq's NOTICE makes crediting Robyn Speer by that name a condition of the permission. The
committed NOTICE named no author. Since this change moves the pack's sha256 anyway (the version
carries the rule digest), the attribution lands here, so the pack changes once rather than twice.
A unit test keeps the name in the NOTICE.

### D5 — The analyser version comes from the core

The manifest's `analyzer_version` was a literal duplicating lingua-core's `ANALYZER_VERSION`; a
bump in the core without a matching edit would build a pack the core refuses at load. The reducer
now reads the constant from `crates/lingua-core/src/analysis/mod.rs`, as it already reads the
analyser's irregulars. The value is unchanged (1.1.0).

## Risks / Trade-offs

- [A shared-module edit asks every pair to be reduced again] → intended; with two pairs it is two
  dispatches of `lingua-pack-update` (mode `reduce`), each opening its own branch.
- [`functools.partial` hides the bound argument from a reader of `reduce-en-fr.py`] → the
  bindings sit together next to `EN`, with a comment naming the shared module.
- [The monthly dry run covers en-fr only] → it is the only pair; the Spanish tables change adds its
  pair to the workflow's choices and to the schedule.

## Migration Plan

1. Merge: the check lane builds en-fr from the new tables and checks the new rule digest.
2. The next extension and Apple releases ship the new pack (attribution and version only).
3. Rollback: revert; the previous tables, NOTICE and pin come back together.

## Open Questions

None.
