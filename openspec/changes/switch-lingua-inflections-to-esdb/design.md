## Context

`reduce-en-fr.py` reads inflections through one function, `parse_agid_relations`, which returns
`(pairs, relations)`: every `(form, lemma)` pair, and each inflected form's `{(lemma, kind)}` with
kind `N` (plural), `V` (verb form) or `A` (comparison). Everything downstream — which forms are
words of their own (`own_words`), which words are lemmas (`canonical_ranks`), `forms.tsv` — works
from that structure. So a new source is a new parser that returns the same structure.

The measurement behind this change (2026-09-25) used exactly that: the repository's reducer,
imported unchanged, with that one function replaced; then `lingua-pack-build`; then the extension's
engine (`LinguaEngine` through the wasm glue, under Node) on 160 Wikipedia articles, comparing each
token's lemma between packs. The harness is reproducible and is task 1.1.

What ESDB is: `scowl.txt`, exported from a SQLite database the repository builds with its own
`Makefile` (Python 3 and sqlite3, ~40 s). One group per sense; a line
`SIZE [tags]: [VARIANT-INFO: ] LEMMA <POS[/class]>: DERIVED, …`, derived forms in a fixed order per
POS, alternatives in parentheses, each prefixed by per-spelling variant levels
(`A B: focused`, `AV Bv: focussed`, `~: born`, `@: art`).

## Goals / Non-Goals

**Goals:**
- Inflections from a maintained source, pinned like every other.
- The measured gain (+353 glosses, +65 resolved tokens on the sample) with none of the measured
  regressions.

**Non-Goals:**
- Using ESDB's word list, sizes or spelling-variant tables for anything but inflections.
- Changing `own_words`, `canonical_ranks` or the lemma set's source (wordfreq).
- Migrating readers' statuses or deck cards to the lemmas forms move to: Lingua has no readers yet
  (product owner, 2026-09-25). The pack format, the builder and the core stay as they are.

## Decisions

### D1 — ESDB, pinned at a commit, built there

`pin.json` records the commit of `en-wl/wordlist` (`rel-2026.02.25`,
`7e99edab8e32f9f9ea2b15f249ca8d4d67237410`) and the sha256 of the exported `scowl.txt`. Re-reduce
and update build it from that commit (`make scowl.txt`); the export is deterministic (two builds,
one hash — measured). Keeping `scowl.txt` itself as a snapshot release (1.5 MB zstd) is the
fallback if the repository's history ever becomes unreachable.

### D2 — What counts as an inflection

- POS `n`, `v`, `m`, `n_v`, `aj`, `av`, `a`, `aj_av`; sizes ≤ 80 ("a valid word in current usage").
- An alternative counts when at least one of its spellings is primary or equal (`A B:`,
  `B Zv: learnt`, `A B= Z:`); it does not when every spelling is a lesser level (`AV Bv: focussed`,
  `~: born`, `@: art`). Measured: keeping lesser levels sends *born* to *bear* on 123 tokens.
- Possessives are never inflections (AGID has none; the tokenizer splits them).
- Today's reducer drops AGID's numbered variants (`born 1`, `art 2`) only because its token pattern
  refuses the digit. The same rule becomes explicit, named and tested — for ESDB, and so that no
  later cleanup restores it by accident.

### D3 — kaikki's `form_of`, regular inflections only

A kaikki entry whose sense is a form of another word (`Pluriel de …`, `Prétérit …`,
`Participe …`, `… personne du présent`, `Comparatif …`) adds a relation — only when
`regular_inflection(form, lemma, kind)` holds. Measured: without that condition it links *occupied*
to *nanny*, *stocks* to *mot*, *coats* to *coast* and *born* to *bear*; with it, none of those.

### D4 — The measured regressions, each fixed and tested

| Form | Today (AGID) | ESDB + kaikki, unfixed | Wanted |
|---|---|---|---|
| fewer | few | fewer | few |
| des, dis | des, dis | de, di (from kaikki) | themselves — never a 1–2-letter lemma |
| renowned | renowned | renown | renowned |
| vested | vested | vest | vested |
| roses | rose | ros | rose |

(*les* and *os*, which AGID sends to *le* and *o*, come out right: ESDB keeps them whole.)

The general rules behind them (a longer form is never sent to a lemma of one or two letters; an
adjective a dictionary lists as a word of its own stays one) go into the reducer with tests; the
update report (from `pin-lingua-pack-sources`) lists every remaining change for review.

### D5 — Notices

ESDB's copyright notice (Kevin Atkinson, permissive, notice required in copies) replaces AGID's in
the pack's NOTICE and on the attributions page; the licence list of **Licence hygiene** names ESDB.
ESDB's own notes say some of its words were selected with COCA frequency data used under licence;
the distributed database is under the permissive notice above.

## Risks / Trade-offs

- **[ESDB's format moves]** — its release notes call the architecture unstable. → Pinned at a
  commit; an update to a later ESDB is a reviewed update, and the parser's tests pin the grammar.
- **[A sample of 160 Wikipedia articles]** — a formal register. → The update report runs on every
  update; a blog or tech sample is added to the measurement (task 1.2).
- **[Readers arrive before this lands]** — their statuses on a lemma that moves would stop applying
  to the forms that moved. → Revisit then: a carry-over (a merges list in the pack, applied once by
  the core) is a change of its own.

## Migration Plan

Land the parser and the rules behind the re-reduce mode; run the re-reduce; review its
`forms.tsv` and `freq.tsv` diff; merge. Rollback: re-reduce with AGID, still pinned.

## Open Questions

None. (Carrying readers' data over was settled: not needed, there are no readers yet.)
