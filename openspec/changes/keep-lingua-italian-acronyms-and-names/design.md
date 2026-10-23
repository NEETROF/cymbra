# Design

## Context

See proposal.md — Why. What constrains the shape:

- `resolve_lemmas` (`analysis/pipeline.rs`) runs `french::acronym` before `lemmatize` when the
  token stands alone in capitals and the studied language is French; `alone_in_capitals` answers
  for French alone today (every other language: no, without reading the text), judging each
  neighbour by its written word — `written(text, token)` reads the token's source span — so a
  neighbour written wholly in capitals is in capitals whatever its read text; `french::acronym`
  looks the token up as written and is language-neutral. The builder accepts a form or a lemma
  holding a capital only in a pack studying French, wholly in capitals, two letters or more, its
  own lemma (`lib.rs`, the `acronym` closure and the message naming the form); `word_grammar`
  keeps a dictionary form as asked when the pack holds it as a lemma; the extension keys a status
  and a card by the dictionary form as given (43c's language-neutral requirement).
- French's table is one row (`CA`) in `reduce_french_acronyms.py`, a rule file of fr-en and
  fr-es; `check_rows` refuses at reduction a row whose entry the section lacks or whose expansion
  the entry does not point at; fr-en writes `lexical.tsv` as `dictionary_words ∪ ACRONYMS`, which
  the builder's `check_studied` (`tables.rs`) accepts only because `CA` is glossed with noun runs:
  the check takes the reference's glossed lemmas, or those less the names (every run `PROPN`), and
  nothing else.
- `dictionary_words` in `reduce_edition_fr.py` (change 24c) leaves out a lemma every sense run of
  which is a proper noun's; en-fr and es-fr load it, and it-fr, glossed by the French Wiktionary's
  Italian section, loads the same module (change 16). Change 9's it-fr writes `tables/it/forms.tsv`
  and `freq.tsv`, an empty `lexical.tsv` and no gloss: the glosses and the dictionary words are
  change 16's, their defects change 17's.
- Italian's pre-pass (change 6) lowercases for its comparisons and splits articulated prepositions
  and elisions whatever their case, each piece taking the written word's capital on its own first
  letter; change 7's cascade runs after it; change 8's guard and change 10's apostrophe rule leave
  the lookup alone. Italian is at `1.2.0` after change 10.
- Measured on the English Wiktionary's Italian section (kaikki.org's extract, 770,859,636 B, the same
  623,702 entries as the `kaikki-Italian.jsonl` change 9 reduces, derived at 656,480,462 B): 211 headwords written wholly in capitals; 42 whose lowercase is a word or
  a form of the section; those whose lowercase is *another* word's form — the case a row repairs,
  43c's test's criterion —: `NATO` (*nascere*, *natare*), `USA` (*usare*, *uso*), `TARI` (*tarare*),
  `AGI` (*agio*), `FAI` (*fare*), `LUCE` (*lucere*), `EI` (*elli*), `FANS` (*fan*), `TUT` (*tutto*),
  and the two-letter `AI` (split by the pre-pass), `LE`, `MI`, `NE`, `SE`, `SO`, `SU`, `SUA`, `TI`.
  `ONU`, `RAI`, `UE`, `PIL`, `IVA`, `ISTAT`, `OMS`, `FIAT`, `UEFA`, `CIA`: their lowercase is no other
  word's form, so change 9's tables, which lowercase an all-capitals headword as any other, read
  them as their own entries with no row. `ANSA`, `CONI`, `SPA`, `ARPA`: no entry in the section.
- The entries: `NATO` is a proper noun whose only sense is « NATO » — no expansion, no synonym;
  `USA` points at the English « the United States of America » and names the synonyms `America` and
  `Stati Uniti`; `IVA`, `ONU`, `PIL`, `UE` point at Italian expansions (`imposta sul valore
  aggiunto`, `Organizzazione delle Nazioni Unite`). The French Wiktionary has no Italian section
  for `NATO` or `USA`.

## Goals / Non-Goals

**Goals:**

- « NATO » and « USA » in running Italian read as themselves, with a card of their own, and
  never as *nascere* or *usare*.
- Italian's names are out of its words to learn by the rule the other languages follow, the acronym
  rows in.

**Non-Goals:**

- Acronyms whose lowercase is no other word's form: read by the tables as their own entries, no row.
- Acronyms the section lacks (`ANSA`): no row until the section gains an entry.
- A headline in capitals: read as words, as French's rule reads it.
- Changing 24c's rule or module; writing it-fr's glosses and dictionary words (change 16).

## Decisions

### D1 — French's rule, for Italian: the same gates, opened

`resolve_lemmas`'s gate (`studied == French`) and `alone_in_capitals`'s (`if studied != French`)
open to Italian; `french::acronym` serves Italian as is — the token as written, NFC, not
lowercased, in the pack's forms; a hit is the acronym, capitals kept, and the cascade does not run.
Italian's pre-pass runs before any of it. A neighbour is judged by its written word, as 43c's: an
articulated preposition or an elided piece written in capitals beside the acronym is a neighbour
in capitals — « NEGLI USA » reads `in` + `gli` + `usa`, « DEGLI USA » `degli` + `usa`, as « CA HT »
reads `ça` — and in lowercase it is not: « negli USA » reads `in` + `gli` + the acronym `USA`. The
pieces of « AI » reach the rule as any token (a shared span in capitals, no neighbour among
themselves) and are never a row: a single letter is no acronym, and their texts (`A`, `I`) match
no all-capitals form.

### D2 — A reviewed table, in an Italian rule module

`ACRONYMS` in `scripts/lingua-data/reduce_italian_acronyms.py`, a rule file of it-fr's reducer and
of every pair glossing Italian (`pack_sources.rule_files`): rows `NATO` and `USA` — each the acronym
as written, the expansions and synonyms the English Wiktionary's Italian section's entry gives
(`NATO`: none; `USA`: `America`, `Stati Uniti`), its part of speech (`PROPN` for both), its reason
(its lowercase is *nascere*'s participle, *usare*'s third person) — measured on the Italian corpora
(Italian Wikipedia, Tatoeba, the two books, UD Italian-PUD, MarkIT) for how often the capitalised
form stands alone against a shouted lowercase word, as 43c measured « CA », and settled by the
owner (task 5.1) together with the candidates the section measurement adds: `TARI` (the waste tax;
`tari` *tarare*'s form), `AGI` (the press agency; *agio*'s plural), `FAI` (*fare*'s imperative, the
truncation `fa'`'s word). The two-letter province codes and function words in capitals (`MI`, `SO`,
`TI`, `LE`, `NE`, `SE`, `SU`) are left out: a shouted everyday word standing alone in capitals
(« lo SO ! », « NE voglio ») is likelier than the code, and 43c's two-letter floor is the table's
minimum, not a promise. `check_rows` widens: a row naming no expansion and no synonym is accepted
when its entry exists (`NATO`), and a row with no entry is refused naming it (`ANSA`). it-fr's
reducer writes each row into `tables/it/forms.tsv` as a form in capitals mapping to itself; the
builder's acceptance of a form or a lemma holding a capital widens to a pack studying French or
Italian (`StudiedLanguage::French | Italian`), the stray-capital refusal unchanged elsewhere — 43c's
requirement MODIFIED, its scenarios kept (English or Spanish still refused).

### D3 — The gloss: the expansions the entry points at, else its synonyms, else none

43c's D4 glosses a row from the pair's glosses of the expansions the entry points at. Italian's
commonest rows are proper names: `NATO`'s entry points at nothing and `USA`'s at the English name,
which no pair glosses. So a pair glosses a row from its own glosses of the entry's expansions
(first sense each, in the row's order, an unglossed one passed over), and, when it glosses none,
from its own glosses of the entry's synonyms the same way — `USA` in it-fr: `america` « Amérique »
and `stati uniti` « États-Unis », where it-fr glosses them —; a row glossed by neither has no gloss
(`NATO` in it-fr): the card shows the dictionary form with no gloss, the status and the key work,
as 43c's *No expansion glossed* accepts. Never a gloss of the pair's own words, a translation, or
the entry's English. Measured at change 16 for each row; it-en and it-es follow with their editions.

### D4 — No rank, no level; a dictionary word whatever its runs; the capitals kept

As 43c's D5 and D6: wordfreq lowercases, so a row has no rank and no level, is never presumed
known and never seeded; `word_grammar` keeps the form as asked; the extension's keys keep the
capitals already. A row is a dictionary word whatever its sense runs (`USA` is glossed by a proper
noun's run; `NATO` by none) — the one case where a name or an unglossed lemma is a dictionary word.

### D5 — Names leave the words to learn: 24c's rule, through the module it-fr loads, the rows in

it-fr is glossed by the French Wiktionary's Italian section through `reduce_edition_fr.py` (change
16), whose `dictionary_words` leaves out a lemma every sense run of which is a proper noun's. it-fr
writes `lexical.tsv` as `dictionary_words ∪ ACRONYMS`, as fr-en does. The builder's `check_studied`
accepts today the glossed lemmas, or the glossed lemmas less the names, and nothing else: `NATO`
(unglossed) and `USA` (a name's run) would fail it. So `check_studied` and the committed-tables
checks (« every dictionary word is ranked », « a gloss is its lemma's ») exempt the reference's
acronym rows, as 43c's task 3.2 exempted `CA` from the ranked checks: the accepted sets become
`glossed ∪ rows` and `(glossed − names) ∪ rows`, a row being a form of the tables mapping to itself
in capitals. Measured at change 16: how many of it-fr's glossed lemmas are names alone, as 24c
counted English's 3,046 and Spanish's.

### D6 — What this change ships, and what change 16 takes

This change: the rule and its gates (D1), the table and `check_rows` (D2), the builder's acceptance
and the checks' exemption (D2, D5), the rows in `tables/it/forms.tsv`, the version, the fixture and
the golden (D7), and the rows in `tables/it/lexical.tsv` now: change 9's `lexical.tsv` is empty,
so the widened check (D5) accepts `{NATO, USA}` as `glossed ∪ rows`, and a page counts, keys and
cards the rows from this change on — without them in `lexical.tsv`, change 7's names rule would set
« NATO » aside as a name (a form capitalised mid-sentence, never in lowercase, no dictionary word).
Change 16, which writes it-fr's glosses and the dictionary words: the rows' gloss (D3) and
`lexical.tsv` as `dictionary_words ∪ ACRONYMS` (D5), by this change's requirements. The programme's row stays 11: the rule, the table and the
version chain (`1.2.0` → `1.3.0`) are stage 1's order, the data lands with change 16, and this
change archives after 16 and 17.

### D7 — Version, fixture, golden

`ITALIAN_ANALYZER_VERSION` `1.2.0` → `1.3.0`; the fixture's manifest follows, its `forms.tsv`
gains `NATO` → `NATO` and `nato` → `nascere`; the golden re-blessed on its version line and the two
probes added (the `notizie` page writes `ONU`, `RAI`, `PIL`, `UE`, none a row, so no page moves).
The six goldens do not move.

## As implemented (2026-10-23)

### Task 5.1's measurement, for the owner

The criteria were written before any figure was read. **Candidates**: every headword of the pinned
section (`kaikki-Italian.jsonl` of it-fr's pin, 665,207,473 B) written wholly in capitals — 212 by this
reading, letters only, two or more (the design counted 211) — whose lower case `tables/it/forms.tsv`
reads as another word, or as a lemma of its own spelling that another entry of the section heads; plus
`NATO`, `USA`, `TARI`, `AGI`, `FAI` whatever the computation said, and the two-letter ones, measured but
kept out. **Corpora**, each sentence or paragraph a block: UD Italian-PUD (19,530 words), MarkIT's three
sections (34,299), every Italian sentence of Tatoeba (6,016,976; the guard's 20,000 a subset), the
Italian Wikipedia sample of change 8 (542,868, markup stripped), *I promessi sposi* (239,278) and
*Pinocchio* (42,359), and the section's own usage examples (149,392): 7,044,702 words, 7,090 of them
in capitals. **Counted**, by D1's rule emulated over letter runs: each token written exactly as the
candidate, standing alone in capitals (what the rule reads as the acronym) or beside a word in
capitals (a headline or a shout, read as before), each alone occurrence then read one by one. **The
rule for a recommendation**: at least three alone, at most one in ten of them a shouted word, and a
majority meaning the section's entry gives; `TARI`, `AGI` or `FAI` implemented only if clearly so (five
or more alone, none shouted).

| Candidate | The tables read its lower case as | Alone in capitals | Of them the acronym | Shouted word read as it | In a line of capitals | Recommendation |
|---|---|---|---|---|---|---|
| `NATO` | *nascere* | 27 (PUD 1, Tatoeba 10, Wikipedia 16) | 27, the alliance | 0 | 0 | row (implemented) |
| `USA` | *uso* | 29 (MarkIT 2, Tatoeba 21, Wikipedia 4, examples 2) | 29, the country | 0 | 0 (1 in a hyphenated run) | row (implemented) |
| `RAI` | `rai` itself, a lemma ranked 1,214 that the section also heads as a poetic noun (« rays ») | 11 (Wikipedia 10, examples 1) | 11, the broadcaster | 0 | 0 | passes the rule; `check_lowercase` refuses it as written (its lower case is its own lemma), so a row needs that check widened — the owner's call |
| `FAI` | *fare* | 2 (MarkIT) | 2, the *Fondo per l'Ambiente Italiano* | 0 | 0 | too few to recommend |
| `AGI` | *agio* | 1 (Wikipedia) | 1, the press agency | 0 | 0 | too few to recommend |
| `TARI` | `tari` itself, the tax's own entry lowercased (ranked 15,602), not *tarare* | 0 | — | 0 | 0 | none written: no row; the tables already read it as the tax |
| `LUCE`, `FANS`, `TUT`, `SUA`, `ANCA`, `IDEM`, `GIP` | *luce*, `fans`, `tut`, *suo*, *anca*, *idem*, `gip` | 0 | — | 0 | 0 | none written: no row |
| `MI` (two letters) | `mi` | 58 (Tatoeba) | 0 | 58: `mi` typed in capitals at a sentence's head | 1 | stays out, confirmed |
| `AI` (two letters) | split by the pre-pass | 5 (Tatoeba 4, examples 1) | 5, *intelligenza artificiale* | 0 | 1 | stays out: the pre-pass reads it as `a` + `i` whatever its case (D1) |
| `NE`, `SE`, `LE`, `AD` (two letters) | `ne`, `se`, *il*, `ad` | 2, 1, 1, 2 | `NE`/`SE` compass points, `AD` once an *amministratore delegato* | `NE` and `AD` once each a word cited in capitals | `SE` 3, `SO` 3, `SU` 2, `NO` 1, `TI` 1 | stay out |

The corpora are edited text with little news: `TARI`, a municipal tax, is written nowhere in them, and
`FAI` and `AGI` once or twice. Written as a word (« la Nato », « gli Usa ») an acronym is never reached
by a row, and these corpora rarely write it so: a capital initial after a lowercase word, 5 `Nato`
(the participle among them, after a full stop), 1 `Usa`, 44 `Rai`.

### Departures

- **`usa` reads as *uso*.** The committed tables read `usa` as *uso*'s feminine, not *usare*'s third
  person: the scenario *The rows* says « keep the readings the tables gave them, *nascere* and *uso* »,
  and `USA`'s reason names it. Nothing else depends on which.
- **Italian's version string is English's and French's.** `1.3.0` (D7) is the string English and French
  have; the core compares a pack with its studied language's own constant, so nothing is ambiguous, but
  two core tests assumed four distinct strings: `language.rs`'s *each language reports its own version*
  now holds each language to its own constant, and `pack.rs`'s Italian refusals pass over the versions
  equal to Italian's and add `1.2.0` and `1.1.0` (*A pack built before is refused*).
- **`USA`'s row names synonyms and no expansion.** Its entry's one pointer is the English name, never
  a row's (D3 never shows the entry's English): the row is `((), ("America", "Stati Uniti"), "PROPN",
  …)`. `check_rows` reads an entry's synonyms at its own level and at its senses'.
- **`check_lowercase`.** D2's « a row SHALL exist only where … the acronym's lowercase is a form of
  another word of the tables » is checked at reduction, read as a form whose lemma is not the lower case
  itself (`nato` → *nascere*): a row like `ONU` or `RAI`, whose lower case is a lemma of its own
  spelling, is refused until a person widens the check.
- **`check_studied` reads the rows from `forms.tsv`.** The accepted sets are `glossed ∪ rows` and
  `(glossed − (names − rows)) ∪ rows`, a row being a form of the studied folder's `forms.tsv` holding a
  capital and its own lemma; French's `CA`, glossed by a noun's run, leaves French's sets as they were.
- **`tables/it-fr/README.md`** and `SOURCES.md` describe the rows on this branch; the dispatch (2.3)
  writes the data alone.
- **`packs.spec.ts`** already matched Italian's version by its pattern: unchanged.

### What the reduce dispatch writes (task 2.3), reduced locally

From it-fr's pinned sources with the pinned interpreter and wordfreq, on this branch's rules:
`tables/it/forms.tsv` gains `NATO` and `USA` (129,599 → 129,601 rows), `tables/it/lexical.tsv` holds
them alone, `freq.tsv` and `NOTICE` are byte for byte; `manifest.json` reads `1.3.0` and the snapshot
with the new rules' digest; the pack grows 1,296,403 → 1,304,230 B (the lexical section, a bitset over
every lemma, and two forms), its base 1,294,975 → 1,302,802 B. Filed and recorded on a scratch copy of
the tables, every committed-tables check, `cross_native`, the reducer tests and `check-reducer` pass;
`measure/it-ud.sh` reads UD Italian-PUD 99.26 / 91.31 / 99.90 and MarkIT's test section 99.43 / 94.65 /
98.87, as before.

## Risks / Trade-offs

- [A shouted word read as the acronym] → the rule reads the neighbours, as French's: « LA NATO HA
  VOTATO » keeps *nato*; a word shouted alone (« NATO ! ») is the cost 43c accepted for « CA ».
- [A row the section lacks] → no row: the table chooses among the dictionary's entries, never
  writes one; `ANSA` and `CONI` read as the words until the section gains them.
- [An unglossed row] → `NATO` in it-fr shows no gloss (D3); the status, the key and the card work.
- [`dictionary_words` moved by 24c] → read as 24c leaves it; this change archives after 24c.
- [Change 16 forgets the rows] → its requirement is this change's; the checks hold `lexical.tsv`
  to `(glossed − names) ∪ rows`, so a `lexical.tsv` without the rows fails.

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. Rollback is a revert with the
tables' and the golden's.
