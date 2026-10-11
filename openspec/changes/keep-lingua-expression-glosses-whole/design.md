# Design — keep-lingua-expression-glosses-whole

## Context

See proposal.md (Why). What exists, on `main` at `61430101`:

| Where | What |
|---|---|
| `reduce_common.reduce_expressions(path, maxlen, per_sense=42, max_senses=3)` | every pair's expressions from its Wiktionary section: each sense through `clean_gloss(gloss, 42)` — the raw cut, `g[:42].rstrip()`, `whole_words` false —, up to three senses one per entry in turn (`_join_senses`), the joined string cut raw at `maxlen` (`joined[:maxlen]`), `EXPRESSION_GLOSS_LEN = 80` |
| `native_tables(…, expression_len=EXPRESSION_GLOSS_LEN)` | en-fr, es-fr, es-en and fr-en read the section's expressions through it; en-es and fr-es call `reduce_expressions` themselves; en-fr passes `--max-gloss-len` (default 80) |
| `reduce-fr-en.py` | `split_words(…, maxlen=EXPRESSION_GLOSS_LEN, per_sense=42)`: the words French's tokenisation splits, glossed as expressions; `lent_senses`: « de l' » opens on « de la »'s first sense, the whole cut again at 80; `with_etc_period` may run a gloss one character past 80 |
| `reduce-fr-es.py` | `EXPRESSION_SENSE = 42`; `label_expressions` rebuilds the shared rules' senses to open each on its labels after the cut (49b D2), so a labelled gloss runs past 80 (the longest 160) |
| Word glosses | `WORD_GLOSS = {"maxlen": 800, "per_sense": 300, "max_senses": 8}`, `clean_gloss(…, whole_words=True)`: a sense past 300 characters cut after a whole word with « … » (`cut_at_word`), whole senses while they fit 800 (`_join_senses_by_pos`) |
| Fallbacks | `fallback_expressions` / `translation_gloss`: up to `FALLBACK_WORDS = 3` native words a part of speech, from translation tables — a count of words, never a character cut |
| The card | `reading/gloss-pages.ts`: a gloss in pages of whole senses, `PAGE_CHARS = 160`, at least one sense a page however long; `wordpopup.ts` `renderPackAnswer` renders an expression's gloss through the same `renderGloss` as a word's. `selection-card.ts` `rowGloss`: a row shows a gloss's first sense within `ROW_SENSE_CHARS = 80`, cut after a whole word with « … ». The card has no maximum height: a paged card keeps the size of its largest page |
| The core | hands the gloss through as the pack holds it (`gloss_phrase`'s `expressions`); nothing truncates a gloss in `lingua-core` or `lingua-wasm` |
| The deck | a card created from a paged gloss stores its first page (`storedGloss`); the server's `gloss` is `TEXT` |
| The goldens | `crates/lingua-wasm/tests/baseline/{en-fr,es-fr,es-en,en-es,fr-en,fr-es}.golden`, built in-process from the committed tables; `apps/lingua-extension/test/baseline/word-card-*.txt` render their probes; `test/row-gloss-tables.spec.ts` pins the rows of every committed **word** gloss (`gloss.tsv`), not of the expressions |
| The owner's decision (2026-10-11) | from dogfooding #810: an expression's gloss is no longer cut — not cut at a word with an ellipsis either |

## Goals / Non-Goals

**Goals:**
- An expression's card shows its senses as the dictionary writes them.
- The same expressions, the same senses, in the same order: only the cut text comes back.
- Every pair's word glosses, sense runs, studied tables and coverage byte for byte.

**Non-Goals:**
- A word's gloss: its senses are whole already, within 300 characters each and 800 together; the
  owner's decision is about expressions.
- The number of senses an expression keeps (three), and the fallbacks' three words a part of speech:
  counts, not cuts. Raising them is another decision.
- The selection card's row (80 characters, cut after a whole word with « … »): a row is a word-by-word
  hint, one sense, a word's and an expression's alike; the expression's own card shows it whole.
- The defects the whole text reveals (*Found while measuring*): each pair's refinement change's.

## Measured

A prototype in the scratchpad (never committed): `main` at `61430101` with the rules of D1–D3 applied to
`reduce_common.py`, `reduce-en-fr.py`, `reduce-fr-en.py` and `reduce-fr-es.py`, each pair reduced again
with `build.sh --reduce <pair>` from its pinned release assets (Python 3.12, the pinned
`requirements-reduce.txt`), in the order en-fr, es-fr, fr-en (the three references), es-en, en-es,
fr-es. Only `mwe.tsv`, `manifest.json` and `pin.json` moved in each pair: `gloss.tsv`, `senses.tsv`,
`NOTICE`, `tables/en/`, `tables/es/` and `tables/fr/` came out byte for byte — the reduction is
reproducible and the rule touches nothing else. The packs of `main` were rebuilt from the committed
tables with the same builder and matched their pins' sha256.

**Rows.** No row is added or left out in any pair. Every changed row holds the old row's senses, each
old sense a prefix of its new one, but for 87. In 86 of them the old senses are still found whole, in
order: the cut had fallen inside a sense holding its own `;` (« chaque jour une pomme conserve son
homme ; », es-en « in the other direction; in the wrong way; »). One, fr-en « saint-cyrien », picks
another sense: cut at 42 characters, two of its senses read alike (« of or relating to Saint-Cyr, in the
depart ») and the second was dropped as a repeat; whole, they differ and the round-robin keeps both.

| Pair | Rows | Changed | Longest gloss, `main` → whole | Over 80 / 160 / 300 | Longest sense | `mwe.tsv` |
|---|---|---|---|---|---|---|
| en-fr | 17,416 | 2,878 | 80 → 537 (« situation awareness ») | 1,352 / 312 / 30 | 300 (bound) | 744,353 → 883,153 B |
| es-fr | 15,133 | 525 | 95 → 476 (« palo bobo ») | 273 / 89 / 12 | 300 (bound) | 574,640 → 607,181 B |
| es-en | 15,515 | 1,793 | 80 → 337 (« lo barato sale caro ») | 519 / 73 / 4 | 260 (« pupusa loca ») | 623,946 → 673,438 B |
| en-es | 17,094 | 132 | 154 → 294 (« to go ») | 193 / 13 / 0 | 217 (« foster care ») | 684,073 → 689,150 B |
| fr-en | 43,424 | 5,178 | 142 → 408 (« mardi gras ») | 2,215 / 315 / 9 | 299 (« maladie du sommeil », bound) | 1,904,546 → 2,099,112 B |
| fr-es | 12,108 | 109 | 160 → 275 (« fils de pute ») | 87 / 6 / 0 | 242 (« maïs éclaté ») | 487,122 → 490,922 B |

The median gloss is 16 to 20 characters in every pair: most expressions were never cut.

**The bound (D2).** Without it, 26 senses run past 300 characters, in 29 rows: en-fr 19 (« machete
order » 458, « lumped element » 412, « penis fencing » 411, « comic strip » 410, « machine learning »
357…), es-fr 7 (« sistema cardiovascular » 1,132, « violencia sexual » 421, five « palo … » trees of 322
to 362), fr-en 3 (« maladie du sommeil » 363, « arc-boutement » 332, « légende noire » 328); none in
es-en, en-es or fr-es. Without the bound the longest gloss is en-fr's « lumped element », 637
characters.

**The packs and the package.**

| Pair | Pack, `main` → whole | Of 5 MiB | Zipped (`zip -9`) |
|---|---|---|---|
| en-fr | 1,840,736 → 1,876,816 B (+36,080) | 35.8 % | 1,395,062 → 1,431,160 B |
| es-fr | 2,234,330 → 2,242,425 B (+8,095) | 42.8 % | 1,538,978 → 1,547,067 B |
| es-en | 2,612,075 → 2,626,743 B (+14,668) | 50.1 % | 1,912,416 → 1,927,024 B |
| en-es | 1,688,931 → 1,690,521 B (+1,590) | 32.2 % | 1,237,299 → 1,238,883 B |
| fr-en | 3,003,601 → 3,052,887 B (+49,286) | 58.2 % | 2,238,266 → 2,287,541 B |
| fr-es | 1,983,054 → 1,984,066 B (+1,012) | 37.8 % | 1,314,460 → 1,315,495 B |

The packs are zstd sections already, so a zip gains what the pack gains: the extension's package, which
lists en-fr and es-fr today, grows by about 44 KB zipped; with all six pairs, about 111 KB. The budget is
far: the largest pack, fr-en, is at 58.2 %.

**The card and the rows.** With 160 characters a page, an expression card now pages for 438
expressions — en-fr 175, es-fr 25, es-en 53, en-es 11, fr-en 171, fr-es 3 —, three pages at most; on
`main` none did (fr-es's longest gloss was exactly 160). The bound keeps a page within what a word's
longest sense already gives one. The rows: the first sense of 10,321 expression rows changes text
(en-fr 2,522, es-fr 496, es-en 1,300, en-es 109, fr-en 4,393, fr-es 101), most of them now whole where
they were cut mid-word; 3,092 end on « … » (their first sense past 80 characters, cut after a whole word
by the card, as a word's row is). None ends on an opening mark before its « … », the rule
`row-gloss-tables.spec.ts` holds for the words.

**The goldens.** Re-blessed on the prototype's tables (`LINGUA_BLESS=1`): the `pack` lines of all six
(each golden names its pair's pack, and es-fr's, fr-en's and fr-es's name a second pair's), and the
expression glosses of these probe lines —

- en-fr (2): « look forward to », « make up » (« Rattraper, se faire pardonner pour, regagner ce qu'on
  avait perdu »);
- es-fr (0): no probe meets an expression whose gloss was cut;
- es-en (8): « tener en cuenta », « sin embargo » ("however, notwithstanding, nevertheless, in spite of,
  nonetheless"), « por lo tanto », « darse cuenta », « de vez en cuando », « un poco », « pese a »,
  « al día »;
- en-es (1): « turn off » (gains « Desagradar (a alguno) », its third sense, which the 80 cut dropped);
- fr-en (19 lines, 16 expressions): « jusqu'à », « y a-t-il », « au fur et à mesure », « de l' » (twice:
  « …the singular prevocalic partitive article »), « ça va », « ça ira », « quel que soit », « comme ça »,
  « c'est ça », « c'est » (three lines), « très bien » (two), « façon de parler », « ça suffit »,
  « faire du bien », « il était une fois », « champ de bataille », « venir de »;
- fr-es (5): « parce que » (« Porque (como causa, explicación o motivación) »), « ça va » (twice),
  « à la fois », « ce que ».

`word-card-fr-en.txt` moves 11 lines and `word-card-fr-es.txt` 5 — the same expressions' pages and rows;
`word-card-es-en.txt` and `word-card-en-es.txt` render no expression whose gloss was cut and do not move.
`selection-rows-fr.txt` reads the word glosses alone and does not move.

**Found while measuring.** The whole text shows what the cut hid:
- es-fr « sistema cardiovascular »: its second sense is a Spanish quotation (« Las vitaminas son
  básicas para el cuerpo humano… »), a gloss in the studied language; the cut showed its first 42
  characters, the bound now shows 300.
- en-fr « look forward to »: « Note d'usage : Suivi par un nom ou un gérondif en -ing », a note to the
  edition's readers inside the sense.
- A `;` inside a sense — French typography's « ; » (« Attendre (…) ; se réjouir (d'avance) de »: 201
  en-fr, 9 es-fr and 1 fr-en expression glosses, against 138, 5 and 0 on `main`) and the English
  edition's own (es-en « a contramano » "in the other direction; in the wrong way; against the grain",
  one sense). The card pages and the row read `;` as a sense separator, so such a sense shows as
  several. A word's sense has its inner
  separator written « , » (`_INNER_SEPARATOR`) because its sense runs must count it; an expression has
  no sense runs, and the separator was there before. Left as it is (D4).

None is this change's to fix: each is a data defect of a shipped pair (en-fr, es-fr), for its own
refinement change, and listed in the pairs' *Known data defects* by task 3.3.

## Decisions

### D1 — No cut, the same pick

`reduce_expressions` cleans each sense without a character cut and `_join_senses` joins them without
one; `max_senses` stays 3 and the round-robin stays as it is, its repeat test now reading whole senses.
The owner's decision is about the cut, and keeping the pick makes the change « the text the cut took
comes back »: no row added or lost (measured), no expression keyed differently, nothing for the
matcher to do; one row (« saint-cyrien ») keeps a sense the cut had made a repeat.

*Alternatives.* A cut after a whole word with « … » at 42 / 80 — the owner chose against it. Taking
more senses now that each is whole — another decision (Non-goals).

### D2 — One safety bound: 300 characters a sense, cut as a word's

A sense past 300 characters is cut by `cut_at_word` — after a whole word, « … » at its end — as a
word's sense is (`WORD_GLOSS["per_sense"]`). The bound is technical, not a length policy: the card
has no maximum height and keeps the size of its largest page, so a 1,132-character sense would give a
card taller than a phone's screen, its actions out of reach. 300 is the longest sense a word's card
already shows, so the card's tested range does not grow. It touches 29 rows (26 senses), all of them
encyclopaedic definitions, and no row the owner met; the joined gloss is not bounded (three senses,
904 characters at most, three pages). The owner's question 1 asks whether to keep it; dropping it is
one constant, and the figures without it are in *Measured*.

### D3 — One rule, read everywhere an expression is glossed

The shared constant `EXPRESSION_GLOSS_LEN` goes; `EXPRESSION_SENSE_LEN = WORD_GLOSS["per_sense"]` (300)
is what an expression's sense is bounded by, read by:

- `reduce_expressions` (`clean_gloss(…, EXPRESSION_SENSE_LEN, whole_words=True)`), `_join_senses` with
  no length, `native_tables` without `expression_len`;
- `reduce-en-fr.py`: `--max-gloss-len` goes (its only use was `expression_len`);
- `reduce-fr-en.py`: `split_words` reads the same sense rule; `lent_senses` joins the lender's first
  sense to the headword's gloss with no cut; the comment of `with_etc_period` about « one character
  over the shared rules' 80 » goes;
- `reduce-fr-es.py`: `EXPRESSION_SENSE` is the shared constant and `label_expressions` rebuilds the
  senses with `whole_words`, so its check that the rebuilt gloss is the shared rules' still holds; the
  labels stay outside the bound, as they stayed outside the cut (49b D2).

The rule lives in `reduce_common.py`: every pair's rule digest moves and every pair re-pins, which is
the point — one rule, six pairs.

### D4 — What stays

A word's gloss, the fallbacks' count, the row's 80 characters and the inner « ; » (*Found while
measuring*). Writing an expression's inner « ; » as « , » would move rows the owner did not ask about,
in a shipped pair, for a separator the card has always read as one; it belongs with the pairs' own
refinements if wanted.

### D5 — The card: nothing to build, one test and one requirement

`renderPackAnswer` sends an expression's gloss through `renderGloss`, which pages it with
`glossPages` like a word's: the expression card already pages a long gloss, keeps one size, and stores
its first page in a deck card. Until now no expression gloss passed 160 characters, so nothing tested
it: a card test (`test/wordpopup.spec.ts` or its neighbour) opens an expression whose gloss holds three
senses of 120 characters and checks the pages, the paging control and the stored first page. The
lingua-browser-extension spec gains the requirement, since *The word card pages a long gloss* names the
word card. `row-gloss-tables.spec.ts` extends its two checks over every pair's `mwe.tsv` (no row empty,
none ending on an opening mark — measured: none).

### D6 — What moves

Per pair: `mwe.tsv`, `manifest.json` (`pack_version`: the rule digest moves), `pin.json` (`reducer`,
`pack`), `README.md` (the expressions' figures and rules). Byte for byte: `gloss.tsv`, `senses.tsv`,
`NOTICE`, the snapshot and every source record, `tables/en/`, `tables/es/`, `tables/fr/`, coverage.
The goldens and baselines of *Measured*. en-fr and es-fr move: the programme keeps them byte for byte
through every change of its stages, and this is the owner's decision to move them, their expressions
alone.

### D7 — Order with the changes in flight

Every change that re-pins a pair while this one is open touches the same `pin.json` and
`manifest.json`. Whichever lands second reduces its pairs again on the other's rules (`build.sh
--reduce`), re-blesses the goldens it moves and records the figures that differ; nothing in this change
depends on another's data. In the specs, `refine-lingua-fr-es-glosses` holds the fr-es requirement
this change modifies, and `add-lingua-expression-table` the expression card it builds on: both in
`archiveAfter`.

## Risks / Trade-offs

- **Longer cards.** 438 expression cards page, three pages at most; a reader who wanted the gist sees
  the first page first, the commonest sense first — as on a word's card.
- **Defects shown in full.** A sense that was nonsense at 42 characters is nonsense at 300 (*Found
  while measuring*); the rows are what the dictionary writes, and each pair's refinement list is where
  they go.
- **Shipped pairs move.** en-fr and es-fr readers see longer expression glosses at the next release;
  their words and highlights do not move.
- **Size.** +36 KB on en-fr's pack, +49 KB on fr-en's; the budget is far.

## Migration Plan

Data only: the packs are rebuilt and re-pinned; a reader's deck keeps the glosses it stored (a card
stores the gloss it was created with). No backup, profile or server change. Rollback: revert the rules
and reduce again; the pins move back.

## Effort

1–1.75 ideal days: the rules and their tests 0.5; the six re-pins and the figures 0.25–0.5; the goldens,
the card test and the row spec 0.25–0.5; the READMEs 0.25.

## Open Questions

1. **The 300-character bound (D2).** 29 rows of 120,690 hold a sense past 300 characters (« sistema
   cardiovascular » 1,132, « machete order » 458, « maladie du sommeil » 363): with the bound they end
   after a whole word with « … », without it the card grows past a phone's screen for them. Keep the
   bound, or show them whole too? The proposal keeps it.
