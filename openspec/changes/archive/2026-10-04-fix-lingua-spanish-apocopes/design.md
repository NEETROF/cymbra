# Design — fix-lingua-spanish-apocopes

## Context

See proposal.md (Why). In `reduce-es-fr.py`:

| Where | What it does |
|---|---|
| `read_entry` | A form's candidates are the lemma entries listing it in their table, bookkeeping aside, plus its own entry's `form_of` targets. An entry with a sense that is not a form-of is a lemma. |
| `read_readings` | The same rows give the grammar readings. `_NOT_A_READING` lists the rows that give none. |
| `choose_lemma` | One lemma per form, in this order: `OVERRIDES`, GSD's counts, the form's own entry, then the commoner lemma. |

kaikki's apocope entries look alike:

| Entry | Sense | Table |
|---|---|---|
| `buen` adj | `apocopic`, `alt-of`; `alt_of: bueno` | `bueno` tagged `standard` |
| `gran` adj | `apocopic`; `alt_of: grande` | `grande` tagged `standard` |
| `algún` det | `apocopic`; `alt_of: alguno` | `alguno` tagged `standard` |
| `mal` adj | `apocopic`; `alt_of: malo bad` | `malo` tagged `standard` |
| `muy` adv | `apocopic`; `alt_of: mucho` | — |
| `un` num | `apocopic`; `alt_of: uno` (the article is another entry) | — |

GSD lemmatizes `bueno` as `buen` (26 of 27), `muy` as *mucho* (547 of 548) and `un` as *uno* (3,967
of 3,968).

## Goals / Non-Goals

**Goals:**
- `bueno`, `buen` and the other adjective and determiner apocopes reach the full word's gloss.
- `muy` and `un` keep their own entries and glosses.

**Non-Goals:**
- The other words without a gloss: dictionary words no source glosses (`replicar`, `señorito`),
  character names, old spellings, classical Spanish and derived words. The owner chose to treat
  character names next, in a change of their own.
- Changing GSD's role elsewhere.

## Decisions

### D1 — A `standard` row is no inflection

kaikki's `standard` tag names the standard word an entry is a variant of. It is never an inflection
of the entry. `read_entry` skips the row as it skips bookkeeping, and `_NOT_A_READING` lists it, so it
gives no reading either. Alone, this rule gives `bueno` back to *bueno* and `malo` to *malo*. `buen`
remains a lemma of its own, with no gloss.

### D2 — An adjective's or a determiner's apocope is a form of its full word

A sense tagged `apocopic` of an `adj` or `det` entry makes the entry a form of its first `alt_of`
target. Only the target's first word counts: kaikki sometimes splits the gloss into the target
(`malo bad`), then into more targets (`whatever`). Such an entry is no lemma of its own when all its
senses are apocopic, so `buen`, `gran`, `primer`, `tercer`, `postrer`, `algún`, `ningún`, `cualquier`
and `cualesquier` map to their full words. `san`, which also has noun entries, stays a lemma. GSD's
counts then choose *santo*.

*Rejected — every apocopic sense.* GSD lemmatizes `muy` and `un` to *mucho* and *uno*, so its counts
would pull them there. The card of `muy` would read « beaucoup » where the reader needs « très ».

*Rejected — one override per word.* An override takes the other lemma out of the pack, which would be
right here. But the rule covers the whole class, and kaikki adds new apocopes without anyone writing
a row.

### D3 — Measured, then reduced again

The tables are reduced again from the pinned snapshot:
- `forms.tsv`: `bueno`, `malo` and the apocopes' rows move to their full words (+27, −12 lines);
- `freq.tsv`'s ranks shift: lemmas left and lemmas entered at the tail of the 60,000;
- `gloss.tsv` gains `bueno` and `malo`, and loses six apocopes' own glosses, since the full words
  carry theirs;
- `grammar.tsv`, `level.tsv`, `senses.tsv` and the pin follow.

The PUD gates and the owner's books are measured before and after (proposal.md, Impact).

## Risks / Trade-offs

- **`gran` reads as *grande*** where PUD keeps `gran`. The card shows « Grand », which is what the
  reader needs.
- **An apocope with a meaning of its own** would lose it. None of the adjective or determiner
  apocopes in the snapshot has one.
