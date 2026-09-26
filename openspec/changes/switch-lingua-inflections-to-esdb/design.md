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
- Migrating readers' statuses or deck cards to the lemmas forms move to: Lingua has no readers as
  of 2026-09-25 (product owner), though it will certainly have some after that date. The pack
  format, the builder and the core stay as they are.

## Decisions

### D1 — ESDB, pinned at a commit, built there

`pin.json` records the commit of `en-wl/wordlist` (`rel-2026.02.25`,
`7e99edab8e32f9f9ea2b15f249ca8d4d67237410`) and the sha256 of the exported `scowl.txt`. Re-reduce
and update build it from that commit; the export is deterministic (two builds, one hash — measured,
and the same hash, `3da811b8…`, from the pinned Python 3.12 as from the measurement's build).
*Found while implementing:* the repository's `Makefile` runs `combine.py`, whose first line names
`/usr/bin/python3` — Python 3.9 on a Mac, 3.12 on the runners — so `pack_sources.py` runs its two
steps (`combine.py create-db`, `scowl export`) with the pinned interpreter instead. AGID leaves
`pin.json` (the first re-reduce drops a source the code no longer reads); a rollback reverts this
change, AGID's pin with it. Keeping `scowl.txt` itself as a snapshot release (1.5 MB zstd) is the
fallback if the repository's history ever becomes unreachable.

### D2 — What counts as an inflection

- POS `n`, `v`, `m`, `n_v`, `aj`, `av`, `a`, `aj_av`; sizes ≤ 80 ("a valid word in current usage").
  And `d`, a determiner, for its comparisons only (`few`: `fewer`, `fewest`) — its other derived
  forms are words of their own (`that`: `those`).
- An alternative counts when at least one of its spellings is primary or equal (`A B:`,
  `B Zv: learnt`, `A B= Z:`) in the American, British or Canadian spelling — not `D`, Australian,
  which carries a copyright of its own in ESDB's `Copyright`; it does not when every spelling is a lesser level (`AV Bv: focussed`,
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

| Form | Today (AGID) | ESDB + kaikki, unfixed | Wanted | With this change |
|---|---|---|---|---|
| fewer | few | fewer | few | **few** — `d` comparisons (D2) |
| des, dis | des, dis | de, di (from kaikki) | themselves | **themselves** — no kaikki link to a 1–2-letter target (D3) |
| renowned | renowned | renown | renowned | **renowned** — an adjective of its own (D6) |
| vested | — (out of the lexicon) | vest | vested | **vest**, accepted: `vest` now enters the lexicon, and a participle reads as its verb, as `tired` reads as `tire` |
| roses | — (out of the lexicon) | ros | rose | **ros**, 2 tokens, accepted: `rose` is no lemma today either (taken for the past of `rise`); fixing that is `own_words`' business, not this change's |

(*les* and *os*, which AGID sends to *le* and *o*, come out right: ESDB keeps them whole.
*more*, *most*, *less*, *least* now point at *many* and *little* in `forms.tsv`; the analyser's own
irregular table resolves them to those lemmas before it reads the pack, so nothing changes for a
reader.)

The general rules behind them — no kaikki link to a one- or two-letter target (the irregular forms
of such words, `goes`, `is`, come from ESDB), an adjective a dictionary lists as a word of its own
stays one — are in the reducer with tests; the update report lists every remaining change.

### D6 — An adjective of its own (found while implementing)

A form ESDB lists as an adjective at a smaller size — commoner — than the line that derives it is
not taken as that derivation: `renowned` (an adjective at 35) is no verb form of `renown` (a verb
only at 80). 190 derivations are dropped this way, the commonest of them rightly: `sophisticated`,
`outstanding`, `situated`, `antiquated`, `jagged` (no form of `jag`), `packed` (none of `pac`),
`wasted` (none of `wast`). At an equal size the derivation stands: `tired` stays a form of `tire`.

### D7 — A form with no lemma behind it (found while implementing)

The first re-reduce lost words readers meet: ESDB knows rare bases AGID did not (`grandkid`,
`policymaker`, `uprise`, `gree`, `crowdfund`), wordfreq ranks their forms far above them, and a form
is left out of the lemmas because its base stands for it — which, unranked, stands for nothing.
`grandkids`, `policymakers`, `uprising`, `greed` and `crowdfunding` dropped out of the pack. Rule: a
form none of whose bases, nor any base of a base, is a kept lemma is read as a word of its own. The
second level matters: without it, `buildings` (of `building`, of `build`) and every plural of an
`-ing` noun became lemmas of their own — 1,225 lemmas in and out, 137 glosses fewer — where with it
they read as they always did. The lemmas the rule adds push the rarest ones past the 40,000 cut:
those are mostly acronyms and names (`shimbun`, `ghb`, `gsk`), with `initialize` and `tiktok` the
losses worth naming.

### D5 — Notices

ESDB's copyright notice (Kevin Atkinson, permissive, notice required in copies) replaces AGID's in
the pack's NOTICE and on the attributions page; the licence list of **Licence hygiene** names ESDB.
ESDB's `Copyright` adds that WordNet's licence may apply to results drawn from its database (it
assigned the initial parts of speech): WordNet's notice and disclaimer go into the NOTICE too.
ESDB's own notes say some of its words were selected with COCA frequency data used under licence;
the distributed database is under the permissive notice above.

## Measured with this change (tasks 1.2, 4.2)

`scripts/lingua-data/measure/` (committed): the extension's engine under Node reads the corpus with
a pack from `main`'s tables (AGID) and one from this change's (ESDB + kaikki), token by token.
Articles are fetched as they are on the day, so the Wikipedia figures differ a little from the
2026-09-25 exploration's.

| Sample | Tokens | Glosses shown | Tokens resolved | Tokens changing lemma |
|---|---|---|---|---|
| Wikipedia, 10 recent articles | 63,906 | +310 | +9 (+49/−40) | 414 (0.65 %) |
| Wikipedia, 150 random articles | 107,857 | +26 | +46 (+116/−70) | 294 (0.27 %) |
| Wikinews, 20 articles | 5,076 | +8 | +1 (+3/−2) | 16 (0.32 %) |
| MDN, 20 guide pages | 53,486 | +67 | +78 (+89/−11) | 214 (0.40 %) |
| **Total** | **230,325** | **+411** | **+134** (+257/−123) | **938 (0.41 %)** |

The largest changes: `header` read as itself, not as a comparison of `head` (AGID's; 101 tokens,
most of them in MDN), recent plurals resolved (`smartphones`, `cryptocurrencies`, `influencers`,
`apps`, `apis`, `urls`), UK spellings joined to their lemma (`travelled`, `cancelled`), `les`, `os`,
`vs`, `mrs` kept whole. The losses are named in D7.

## Risks / Trade-offs

- **[ESDB's format moves]** — its release notes call the architecture unstable. → Pinned at a
  commit; an update to a later ESDB is a reviewed update, and the parser's tests pin the grammar.
- **[A sample of 160 Wikipedia articles]** — a formal register. → The update report runs on every
  update; a blog or tech sample is added to the measurement (task 1.2).
- **[Readers arrive before this lands]** — expected: readers are anticipated right after
  2026-09-25. Their statuses on a lemma that moves would stop applying to the forms that moved. →
  Checked when this lands: the update report lists every move; if readers exist, a carry-over (a
  merges list in the pack, applied once by the core) is a change of its own, landed first.

## Migration Plan

Land the parser and the rules behind the re-reduce mode; run the re-reduce; review its
`forms.tsv` and `freq.tsv` diff; merge. Rollback: re-reduce with AGID, still pinned.

## Open Questions

- Whether readers exist when this lands, and so whether a carry-over change must land first
  (none as of 2026-09-25; some are expected soon after).
