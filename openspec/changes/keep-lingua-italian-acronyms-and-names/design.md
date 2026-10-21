# Design

## Context

See proposal.md — Why. What constrains the shape:

- `pipeline::alone_in_capitals` answers for French alone today (every other language: no, without
  reading the text); `french::acronym` looks the token up as written before the cascade; the
  builder accepts a form holding a capital only in a pack studying French, wholly in capitals, two
  letters or more, its own lemma; `word_grammar` keeps a dictionary form as asked when the pack
  holds it as a lemma; the extension keys a status and a card by the dictionary form as given
  (43c's language-neutral requirement).
- French's table is one row (`CA`) in `reduce_french_acronyms.py`, a rule file of fr-en and
  fr-es; a row is needed only where the lowercase reading would be another word's; the gloss comes
  from the pair's glosses of the expansions.
- `dictionary_words` in `reduce_edition_fr.py` (change 24c) files a name as a lemma every sense
  run of which is `PROPN`; en-fr and es-fr load it, and it-fr, glossed by the French Wiktionary,
  loads the same module (change 16).
- Italian's pre-pass (change 6) lowercases for its comparisons and splits articulated prepositions
  and elisions whatever their case; change 7's cascade runs after it; change 10's apostrophe rule
  in the seam. Italian is at `1.2.0` after change 10.
- Italian acronyms whose lowercase is a word: `NATO` (*nato*, born), `USA` (*usa*, he uses),
  `ANSA` (*ansa*, a handle), and candidates to measure (`SPA`, *spa*; `ERA`? not an acronym; `AI`,
  split by the pre-pass; `DOC`, `TAR`, `ASL`: their lowercase is no Italian word). `ONU`, `RAI`,
  `UE`, `PIL`, `IVA`, `ISTAT`, `FIAT`: no word, no row.

## Goals / Non-Goals

**Goals:**

- « NATO », « USA », « ANSA » in running Italian read as themselves, with a card of their own, and
  never as *nascere*, *usare*, *ansa*.
- Italian's names are out of its words to learn by the rule the other languages follow.

**Non-Goals:**

- Acronyms whose lowercase is no word: unknown words, as today, until a pair glosses them.
- A headline in capitals: read as words, as French's rule reads it.
- Changing 24c's rule or module.

## Decisions

### D1 — French's rule, for Italian: before the cascade, after the pre-pass

`alone_in_capitals` answers for Italian as for French (one match arm); `italian::lemmatize` runs
the acronym step first, as `french::acronym` does: the token as written, NFC, not lowercased, in
the pack's forms; a hit is the acronym, capitals kept. Italian's pre-pass runs before any of it and
lowercases for its own comparisons: « AI » is `a` + `i`, « L'ONU » is `lo` + `ONU` (the piece
read by the elision rule, the rest a token the acronym rule then reads), « DELL'ANSA » is `dello`
+ `ANSA` — the pieces' casing is French's (the read word's own first letter), so a piece never
stands wholly in capitals and never reaches the rule.

### D2 — A reviewed table, in an Italian rule module

`ACRONYMS` in `scripts/lingua-data/reduce_italian_acronyms.py`, a rule file of it-fr's reducer and
of every pair glossing Italian (`pack_sources.rule_files`): rows `NATO`, `USA`, `ANSA` — each the
acronym as written, the expansions or the sense the English Wiktionary's Italian section's entry
gives, its part of speech (`PROPN` for the three), its reason — measured on the corpora (Italian
Wikipedia, Tatoeba, the two books) for how often the capitalised form stands alone against a
shouted lowercase word, as 43c measured « CA », and settled by the owner (task 5.1). Its test
holds that every expansion or sense is the entry's own, the acronym wholly in capitals, and its
lowercase a form of the tables. it-fr's reducer writes each row into `tables/it/forms.tsv` as a
form in capitals mapping to itself and into `lexical.tsv`; the builder's acceptance widens to a
pack studying French or Italian (`StudiedLanguage::French | Italian`), the stray-capital refusal
unchanged elsewhere.

### D3 — The gloss: the entry's when the pair glosses it, else the expansions'

Italian's commonest acronyms are proper names: no pair glosses « Organizzazione del Trattato
dell'Atlantico del Nord » as an expression, while the French Wiktionary's Italian section glosses
`NATO` and `USA` as entries. A pair's gloss of a row is therefore its own gloss of the acronym's
entry, cut to its first sense, when its edition has one; else 43c's D4 over the expansions; else
none. it-fr measured at implementation for each row; it-en and it-es follow with their editions.

### D4 — No rank, no level; a dictionary word; the capitals kept

As 43c's D5 and D6: wordfreq lowercases, so a row has no rank and no level, is never presumed
known and never seeded; it is a dictionary word; `word_grammar` keeps the form as asked; the
extension's keys keep the capitals already.

### D5 — Names leave the words to learn: 24c's rule, through the module it-fr loads

it-fr is glossed by the French Wiktionary's Italian section through `reduce_edition_fr.py`
(change 16), whose `dictionary_words` already leaves out a lemma every sense run of which is a
proper noun's. No new rule: this change states the requirement for Italian, has
`committed_tables.rs` hold `tables/it/lexical.tsv` to it as it holds English's and Spanish's, and
checks the names rule of change 7 against it (a dictionary word is never set aside). Measured at
implementation: how many of it-fr's glossed lemmas are names alone, as 24c counted English's
3,046 and Spanish's.

### D6 — Version, fixture, golden

`ITALIAN_ANALYZER_VERSION` `1.2.0` → `1.3.0`; the fixture's manifest follows, its `forms.tsv`
gains `NATO` → `NATO` and `nato` → `nascere`; the golden re-blessed: the version line, and the
`notizie` page where a row's acronym stands (`USA`, if the page writes it; else the version line
alone). The six goldens do not move.

## Risks / Trade-offs

- [A shouted word read as the acronym] → the rule reads the neighbours, as French's: « LA NATO HA
  VOTATO » keeps *nato*.
- [A row whose entry the Italian section lacks] → no row: the table chooses among the
  dictionary's entries, never writes one.
- [`dictionary_words` moved by 24c] → read as 24c leaves it; this change archives after 24c.

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. Rollback is a revert with the
tables' and the golden's.
