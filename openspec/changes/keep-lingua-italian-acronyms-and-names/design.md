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
- Measured on the English Wiktionary's Italian section (`kaikki-Italian.jsonl`, 770,859,636 B, the
  file change 9 reduces): 211 headwords written wholly in capitals; 42 whose lowercase is a word or
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
