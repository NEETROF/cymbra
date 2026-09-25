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
- No reader loses a status because a form now belongs to another lemma.

**Non-Goals:**
- Using ESDB's word list, sizes or spelling-variant tables for anything but inflections.
- Changing `own_words`, `canonical_ranks` or the lemma set's source (wordfreq).
- Migrating deck cards: a card keeps the key it was created with.

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

The general rules behind them (a lemma of one or two letters is never the target of a merge; an
adjective a dictionary lists as a word of its own stays one) go into the reducer with tests; the
update report (from `pin-lingua-pack-sources`) lists every remaining change for review.

### D5 — Statuses follow a merged form

The update computes, from the old and new tables, the **merges**: a form that was its own lemma
(or another word's form) and now belongs to a different lemma. The pack carries them as an optional
section (`old lemma → new lemma`), built by `lingua-pack-build` from a `merges.tsv` table the reducer
writes.

The core, loading a pack whose `pack_version` it has not applied merges for, sets for each merge
the new lemma's status to the old lemma's — only when the reader gave the old one an explicit status
and the new one has none — as an ordinary status change at that moment, then records the version.
It synchronises like any status; a device still on the previous pack receives a status for a lemma
it also knows, which changes nothing there.

*Rejected — no carry-over:* the reader re-marks the words, and the percentage counts the merged
forms as unknown until they do: small (0.38 % of tokens) but visible exactly to the readers who
used the product most. *Rejected — rewriting the old keys:* a status keyed by the old lemma may be
the other device's truth; adding to the new lemma never contradicts anything.

### D6 — Notices

ESDB's copyright notice (Kevin Atkinson, permissive, notice required in copies) replaces AGID's in
the pack's NOTICE and on the attributions page; the licence list of **Licence hygiene** names ESDB.
ESDB's own notes say some of its words were selected with COCA frequency data used under licence;
the distributed database is under the permissive notice above.

## Risks / Trade-offs

- **[ESDB's format moves]** — its release notes call the architecture unstable. → Pinned at a
  commit; an update to a later ESDB is a reviewed update, and the parser's tests pin the grammar.
- **[A sample of 160 Wikipedia articles]** — a formal register. → The update report runs on every
  update; a blog or tech sample is added to the measurement (task 1.2).
- **[Merges misfire]** — a wrong merge carries a status to the wrong word. → Only an explicit status
  moves, only onto a lemma without one; the merges list is part of the reviewed diff.

## Migration Plan

Land the parser, the rules and the merges section behind the re-reduce mode; run the re-reduce;
review its `forms.tsv` and `merges.tsv` diff; merge. Rollback: re-reduce with AGID (still pinned),
whose merges back are applied the same way.

## Open Questions

- Should the merges also move deck cards (re-key *smartphones*' card to *smartphone*) when the new
  lemma has no card? Proposed: no (non-goal); to revisit with readers' decks.
