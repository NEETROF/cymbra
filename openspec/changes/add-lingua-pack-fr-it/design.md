# Design — add-lingua-pack-fr-it

## Context

**The data.**
- The Italian Wiktionary's dump of 2026-10-21 (43,849,715 B gzipped; 566,646,259 B decompressed,
  sha256 `09f22a16…`) and the French Wiktionary's of 2026-10-20 (736,596,809 B gzipped; 6,865,138,297 B
  decompressed, sha256 `6d4cfc2c…`), both as kaikki serves them today. fr-es pins the French dump of
  2026-10-02 (`968f7df0…`).
- French's committed ranks (`tables/fr/freq.tsv`, 60,000 lemmas), read as `reduce-fr-es.py`'s
  `read_studied` reads them.

**The method.**
- Nothing Italian is implemented on `main`: `reduce_edition_it.py` (change 35) and `EDITIONS["it"]`
  (change 16) are proposals. The pair was prototyped with change 35's rules as approximated for change
  36's review, every rule included (abbreviations, heraldry-only), and with `main`'s own
  `reduce_common.py` and `reduce-fr-es.py` readers. The files were derived with `main`'s
  `pack_sources.derive`, and match changes 16's and 35's figures byte for byte.
- Task 0.1 re-measures on the real `reduce_edition_it.py`.
- **Loose** coverage is any gloss, what `gloss_coverage.py` counts. **Strict** leaves out a lemma glossed
  only through a pointer.

**The sources:**

| Source | File | Size |
|---|---|---|
| Italian Wiktionary, French section | `kaikki-it-Francese.jsonl` | 13,314,042 B; 27,377 entries, 35,026 senses, 26,108 headwords, 6,579 with a sense that is not a form |
| French Wiktionary, Italian translations (direct) | `kaikki-fr-traductions-it.jsonl` | 7,469,549 B; 71,723 French entries |
| Italian Wiktionary, French translations (read backwards) | `kaikki-it-traductions-fr.jsonl` | 1,273,076 B; 10,086 Italian entries |

**Coverage after each source** (5,000 / 10,000 / 20,000):

| After | fr-it loose | fr-it strict | Lemmas glossed (of 60,000) |
|---|---|---|---|
| definitions, the Italian edition's rules | 28.8 / 20.9 / 13.8 | 28.4 / 20.5 / 13.5 | 3,542 |
| + the direct table | 79.8 / 68.7 / 56.7 | 79.4 / 68.3 / 56.4 | 19,930 |
| + the inverted table | **81.3 / 70.1 / 57.8** | 80.9 / 69.8 / 57.5 | 20,375 (34.0 %) |

The definitions match the programme's estimate exactly; with both tables the figures are 0.1 under it
at the 5,000 and 10,000. Against the committed French pairs:

| Pair | 5,000 / 10,000 / 20,000 | Floor |
|---|---|---|
| fr-en | 93.9 / 87.2 / 76.8 | 91.9 / 85.1 / 74.4 |
| fr-es | 83.2 / 70.9 / 57.1 | 81.4 / 68.8 / 54.5 |
| fr-it (prototype) | 81.3 / 70.1 / 57.8 | about 79.3 / 68.1 / 55.8 (D6) |

**Where the gap with fr-es lies.** On French's dictionary words (`tables/fr/lexical.tsv`) the two are
level: fr-it glosses 91.2 / 86.9 / 82.3 % of the 4,328 / 7,801 / 13,580 at the three tops, fr-es 92.2 /
86.6 / 80.1. At the 5,000, fr-es glosses 280 lemmas fr-it does not, function words first (`des`, `du`,
`ça`, `dont`, `cet`, `celui`); fr-it glosses 185 that fr-es does not (`part`, `afin`, `taux`,
`classement`, `réellement`).

**Where the top rows come from** (5,000 / 10,000 / 20,000):

| Source | Rows |
|---|---|
| Definitions | 1,420 / 2,053 / 2,695 |
| A pointer's borrowed gloss | 19 / 35 / 65 |
| The direct table | 2,550 / 4,782 / 8,576 |
| The inverted table | 77 / 142 / 227 |
| Unglossed | 934 / 2,988 / 8,437 |

The tables give 70.2 % of the 7,012 glossed among the 10,000 (4,782 + 142); fr-es's gave 59.8 % when
first committed, en-it's 12.8 %. A definition glosses 17.4 % of all glossed lemmas (fr-es: 23.9 %).

**What is particular to French in the Italian Wiktionary:**
- **Inflections.** 26,433 senses (75.5 %) are tagged forms, 24,483 of them in verb entries. Change 35
  keeps a tagged sense a pointer; on French its rules move 22 / 31 / 43 rows (its D11). A form matters
  only where it is spelled as a ranked lemma, and there the shared rules lend the verb's gloss (D4).
- **Empty senses.** 44 / 62 / 79 top lemmas have only senses kaikki extracted with no gloss (`savoir`,
  `jouer`, `mer`, `œil`, `siècle`, `feu`, `ciel`, `tenir`): the pages write the definition as a bullet
  (`* [[mare]]`), which wiktextract skips (checked on the raw pages of `mer` and `savoir`). The direct
  table glosses every one checked (30 of 30).
- **Heraldry.** Change 35's heraldry-only rule removes the definitions of 98 / 162 / 262 top lemmas (`le`
  « articolo che si usa per le pezze onorevoli… », `tout`, `contre`, `tête`, `chef`); the tables gloss
  97 / 158 / 251 of them again, at a cost of 0.0 / −0.1 / −0.1. The prototype's rule counts 98 / 97 at the
  5,000 where change 35's D11 estimated 95 / 94; task 0.1's real reader settles it. With change 36's
  amendment it takes about 350 heraldic expressions out of the definitions (712 → 362).
- **Elision.** 60 section headwords open on an elided piece (59 `s'…`, one `j'`); the direct table has
  1,671 French entries with a typographic apostrophe. `l'`, `d'` and `qu'` are read as `le`, `de` and
  `que` (M21) and need no rule.
- **Reflexive verbs.** 124 headwords: 65 `se …`, keyed as expressions, and 59 `s'…`, one token by
  French's pattern, which gloss nothing, as in fr-en and fr-es. The base verb has its own definition for
  103 of the 109 ranked bases.
- **Gender.** One meaning sense carries an Italian gender mark, which change 35's D9 removes; the card's
  gender comes from French's readings.
- **Headwords against French's tables.** Of the 6,579 headwords with a meaning: 3,872 ranked lemmas;
  1,183 not in French's forms (rare or technical); 671 multi-word, many heraldic; 507 a form of another
  lemma; 179 hyphenated and not in the forms (heraldic compounds, but also `abat-jour`, `avant-bras`, a
  studied-side gap); 122 reflexive; 43 capitalised or names. The other way round, 3,415 / 7,678 /
  16,879 top lemmas have no entry in the section: the tables carry them.

## Goals / Non-Goals

**Goals:**
- fr-it committed at or above a floor fixed before the first committed measurement, every gloss Italian
  written by a person.
- fr-es, fr-en, en-it and every other pair untouched.

**Non-Goals:**
- Shipping fr-it (change 50).
- fr-es's refinement rules and the defects of D11 (change 48).
- A word list composing glosses no source writes (M5); the owner's question for change 48.

## Decisions

### D1 — A reader pair

`reduce-fr-it.py` reads `tables/fr/` as committed and writes `tables/fr-it/{gloss.tsv, senses.tsv,
mwe.tsv, NOTICE, manifest.json}` with the Italian edition's rules (`reduce_edition_it.IT` and its
pre-pass), step by step as `reduce-fr-es.py`; nothing of the studied side is computed. Its pin records
the studied tables it read (fr-en's and the six French tables). Its rule digest is its own file,
`reduce_common.py`, `reduce_edition_it.py` and `reduce_french_acronyms.py`:
- `reduce_french_acronyms.py`, because `read_studied` leaves out the `CA` row and `acronym_glosses`
  glosses it from fr-it's own expansions (« consiglio di amministrazione », « fatturato », « corrente
  alternata » in the prototype's `mwe.tsv`), as in fr-en and fr-es; an edit of that table re-pins the
  three French-studied pairs. The open `keep-lingua-french-acronyms-in-capitals` says such an edit
  re-pins fr-en and fr-es: still true, now incomplete, and corrected when it is next touched;
- not `reduce_edition_fr.py`: the direct table's words are Italian, typeset by the Italian edition;
- not `reduce-fr-es.py` (D3).

### D2 — The sources, in fr-es's order

1. The section's definitions, with change 35's rules.
2. The French Wiktionary's Italian translations (the direct table), in the table's order.
3. The Italian Wiktionary's French translations read backwards (the inverted table), the commonest
   Italian word first by wordfreq's Italian list.

At most three words per part of speech from each table. These are the sources I11 names for Italian
natives; no pivot, no Wikidata (I11 keeps those for it-es and es-it). The inverted table adds 1.5 / 1.4 /
1.1 points: definitions and the direct table alone reach 79.8 / 68.7 / 56.7.

### D3 — fr-es's add-pack rules, copied

The readers of `reduce-fr-es.py` are copied into `reduce-fr-it.py`, as en-it copied en-es's (change 36's
D3); moving them to a shared module would add a rule file to fr-es and move its pin and
`pack_version`. Each is coverage-neutral here but the apostrophe:

| Rule | Effect |
|---|---|
| Letters out, in both tables | as fr-es |
| A word listed once across a row's parts of speech, read backwards | as fr-es |
| A typographic apostrophe read as `'` (`straight_apostrophes`) | +4 lemmas (`chef-d'œuvre`, `presqu'île`, `trompe-l'œil`, `qu'en-dira-t-on`), +1,177 expressions |
| The studied word is no definition: a definition repeating the headword gives way to the direct table | 4 rows (`charme`, `igloo` « iglù, iglò », `profane`, `tir`) |
| An adjective section tagged possessive or demonstrative read as a determiner (`french_entries`) | `ce` (rank 16), an adjective tagged demonstrative in the Italian section, reads DET and PRON as in fr-es instead of ADJ and PRON; kaikki's tags, no Spanish wording; its row count measured at task 2.1 |
| The translators' typography | the Italian edition's (`reduce_edition_it.typography`) where fr-es uses the Spanish |

Not copied:
- fr-es's add-pack rule keeping a name's note off a common word: its wording is the Spanish
  Wiktionary's, so it is change 48's to transpose (D11 counts the name notes);
- fr-es's refinement rules on the tables (listed once in the direct table, the French-word rule, the name
  filter), which move 130 / 234 / 419 rows and no coverage, and whose word lists are Spanish: change
  48's (Open Question 4).

### D4 — A borrowed gloss gives way to the tables

French's ranks are lemmas, so a form in the section matters only where it is spelled as a ranked lemma.
There the shared rules lend the target verb's gloss, and the result is nearly always wrong: 19 / 35 / 65
top lemmas are glossed only that way — `fond` « fondere », `pensée` « pensare », `habitant` « abitare »,
`zone` « Zonare… », `privé` « privare », `chatte` « gatto; figura araldica… ».

fr-it's own rule: a gloss a lemma only borrows through a form or pointer sense gives way to the direct
table's gloss of the lemma, else the inverted table's, and is kept when no table glosses it. The direct table glosses 18 of the 19 at the 5,000 right
(`fond` « fondo, sfondo, sostanza », `pensée` « pensiero… », `habitant` « abitante »), 26 and 40 at the
10,000 and 20,000. Coverage does not move; dropping borrowed glosses outright would cost 0.0 / 0.1 / 0.1
(81.3 / 70.0 / 57.7).

The rule is the pair's, not the edition's: en-it's borrowing is right (`labor` ← `labour`), so it does
not belong in `reduce_edition_it.py`, and moving it there would move en-it.

The main spec's *A word that is only a form of another takes that word's gloss* says such a form « SHALL
be glossed with the senses of that word »: `fond`, tagged a form of `fondre`, would be « fondere ». This
change MODIFIES that requirement, copied whole, for a pair whose reduction lets a translation table's
gloss of the form win (fr-it), a form the Italian edition reads as a pointer included; every other pair
reads it as before. No open change holds it. Change 35's edition rule still lends the pointer's gloss;
fr-it's reduction then keeps the table's where one exists, as the modified requirement allows.

### D5 — Expressions

| Source | Expressions |
|---|---|
| The section's definitions | 362 |
| The direct table | 14,450 |
| The inverted table | 483 |
| **Total** | **15,295** |

fr-es keyed 12,177 when first committed. Keys follow fr-es's exactly (`reduce_common.reduce_expressions`,
`fallback_expressions`, the apostrophe of D3). The builder keeps only expressions whose words are in the
lexicon.

### D6 — The floor (M6)

Task 0.1 re-runs this measurement on the committed ranks with the real `reduce_edition_it.py` at
implementation, and fixes `FLOORS["fr-it"]` at its figures less two points, the owner settling it
before the first committed measurement; never lowered after it. On this measurement the floor is 79.3 /
68.1 / 55.8. fr-es's floor (81.4 / 68.8 / 54.5) would fail fr-it at the 5,000 (81.3), so fr-it's floor is
its own, as en-it's is. The gap is function words, not dictionary words (*Context*).

Unlike fr-es's (its D8), this floor does not hold fr-it to all three sources: definitions and the direct
table alone pass it. The README's coverage by source shows each one's part (Open Question 1). The reduce
job's floor step runs `--pair fr-it`.

### D7 — The sources registered

- `EDITIONS["it"]` (change 16's) gains `kaikki-it-Francese.jsonl` (`("entries", "fr")`). Change 35's
  Non-Goals leave the French section to this change.
- `kaikki-fr-traductions-it.jsonl` and `kaikki-it-traductions-fr.jsonl` are change 16's (it-fr's two
  tables, the other way round); this change reads them under the same names.
- `DUMPS["fr-it"] = {"it": ("kaikki-it-Francese.jsonl", "kaikki-it-traductions-fr.jsonl"), "fr":
  ("kaikki-fr-traductions-it.jsonl",)}`, fr-es's shape, the native edition first.
- No `PINNED` entry: French's studied side is read as committed. Reading the GSD treebank's order, as
  fr-es's refinement does, would add `PINNED["fr-it"]`: change 48's.

Registering moves no pin: every committed pair's rule files, digests and `pack_version` are unchanged.
Each fr-it update fetches the French dump, 737 MB gzipped, for one 7.5 MB file, as it-fr does; `keep=fr`
reuses the last release's files on a routine update.

### D8 — The pipeline and the checks

- **`lingua-pack-update.yml`**: the pair option; the bless steps run `fr_it_baseline` and the fr-it
  card snapshot. Its `fr` concurrency group already covers `fr-*`.
- **`lingua-extension-check.yml`**: the baselines step and the floor step gain fr-it.
- **`test_reduce_loops.py`**: fr-it reduced after fr-en, and brought along by fr-en's update.
- **`test_reduce_editions.py`**: the Italian edition's digest test names en-it and fr-it.
- **`crates/lingua-wasm/tests/fr_it_baseline.rs`** and `baseline/fr-it.golden`: the French scenario
  (`support/french.rs`, `pages-fr.txt`), modelled on `fr_es_baseline.rs`, with en-it beside it as an
  Italian native's other pair, and a test that the studied side is fr-en's. It needs en-it committed
  (change 36).
- **`cross_native.rs`**: French through Italian.
- **`crates/lingua-pack/tests/committed_tables.rs`**: fr-it in every hard-coded list of committed pairs
  and of French packs, among them `spec_scenario_the_committed_pairs` and
  `spec_scenario_the_acronym_in_both_french_packs` (fr-it glossing `CA` from its own expansions, D1);
  and fr-es's scenarios in its name — the file set (`studied` fr, `native` it); the pin's studied record naming fr-en and the
  six tables; the sources releases `lingua-pack-sources-fr-it-…` for the Italian and French dumps, no
  English one; fr-en's dictionary words; built to its pin and under 5 MiB; listed in no package while the
  list does not name it; the credits.
- **`scripts/lingua-data/testdata/fr-it/`**, run by `pipeline_testdata.rs`.
- **`tables/fr-en/README.md`**, which names the pairs reading `../fr/`, names fr-it.
- **`apps/lingua-extension/test/row-gloss-tables.spec.ts`**: fr-it's rows and expressions through the
  selection card's row cut; none of the prototype's 20,375 rows is empty or ends on an opening mark, and
  none of its 15,295 expressions is empty.
- **The card snapshot change 34 hands over**: `test/word-card-fr-it.spec.ts` and
  `test/baseline/word-card-fr-it.txt`, the full card in Italian over fr-it's golden — change 34's
  renderer names French tenses in Treccani's terms (« passato remoto », « condizionale presente ») —,
  re-blessed with the golden.
- **The programme**, corrected in this pull request: the Italian-glossed pairs' goldens run over their
  studied language's corpus, not an Italian one; fr-it's *Measured figures* row names the inverted table
  and the prototype's figures; rows 37 and 51 say they modify change 36's requirement as this change
  leaves it.

French is guarded in `GUARDED` already; nothing there moves.

### D9 — Coverage, provenance and credits

`tables/fr-it/README.md` publishes:
- the coverage against the floor, by source;
- the table share (70.2 % of the glossed among the 10,000 on this measurement);
- a sample of 100 glosses marked by source, read at implementation;
- the pack's layer size (about 0.53 MB: gloss 174.6 KB, senses 70.1 KB, mwe 184.6 KB at zstd 19,
  against fr-es's 175.1 / 66.0 / 164.2).

The notice and the manifest credit the Italian Wiktionary (its French section, and the French
translations its Italian entries list), the French Wiktionary (the Italian translations its French
entries list), CC BY-SA 4.0 + GFDL, wordfreq's French and Italian lists, and French's studied sources
(the English Wiktionary's French section through fr-en, UD French-GSD), with « levels are estimated ».
`SOURCES.md` and the programme's *Measured figures* record the committed coverage.

### D10 — Change 36's requirement, amended

Change 36's *English is glossed in Italian…* ends « The Italian edition's rules SHALL be read by en-it
and by no other committed pair », and its scenario *A rule of the Italian edition* says no other pair's
digest moves. Once fr-it reads `reduce_edition_it.py`, both are false. So is its *Nothing else moves*,
read literally: a re-reduction after an edition edit moves fr-it's pin with en-it's, and fr-it's golden
names the en-it pack beside it. The scenario is scoped to a commit by en-it's own reducer, with no shared
or edition module edited, fr-it's golden line naming en-it excepted; fr-it's own is scoped to its first
commit the same way. This change MODIFIES that
requirement, copied whole from change 36, to « by en-it and fr-it », and archives after it, as change 27
widened fr-es's to name it-es. es-it (change 51) widens it again. Change 37 (`refine-lingua-en-it-glosses`,
not yet proposed) refines the same requirement: whichever of 37 and this change is implemented second
re-copies the other's block and names it in `archiveAfter`.

### D11 — Known defects, for change 48

Counted at the 5,000 / 10,000 / 20,000:

| Defect | Count | Examples |
|---|---|---|
| Name or place notes on definitions | 28 / 47 / 77 | `y` « città francese » (rank 30), where the direct table has « là, laggiù, vi, ci »; `petit`, `ville` |
| A label before a colon | 46 / 63 / 78 | `je`, `tu` « in funzione di soggetto: io, in italiano può essere sottinteso » |
| Capital openings | 137 / 227 / 320 rows | |
| A word repeated within a row | 254 / 401 / 619 | `que` « che; che » |
| Odd rows at the very top | | `se` « alzarsi » (read backwards), `le` « il, lo, addì; lo, la, l' », `il` described |
| Heraldic senses shown last | 71 / 96 / 115 rows | `un` (rank 7) still opens on « articolo che si usa per le pezze secondarie… » (change 35's D8) |
| Unglossed dictionary words | 380 at the 5,000 (fr-es: 339), function words first | `des` (rank 6), `du` (10), `ça` (26), `dont`, `cet`, `celui`, `lors`, `ci`, `tes`, `lui-même`: the Italian Wiktionary has no French section for `du`, `des`, `ça` or `dont`, and no table lists them. M5 forbids composing a gloss: an owner's question for change 48 (Open Question 3) |
| The determiners' heading in Italian | | `mon` comes out DET from the direct table and `ce` DET by D3's copied rule; the Italian name of the heading is change 34's « determinante » question |
| kaikki's empty senses | 44 / 62 / 79 lemmas | glossed by the tables; reported upstream (Open Question 5) |

fr-es's refinement table rules, transposed, change 130 / 234 / 419 rows and leave coverage unchanged;
they drop `fr` « sottomesso » and `explicite` « explicite ». Its `OTHER_SENSES`, `CORRECTIONS` and
`SPANISH_ALIKE` lists do not transpose.

### D12 — Not shipped

No package lists fr-it before change 50, and no site figure moves (`gloss_coverage --check` reads
shipped pairs).

## Risks / Trade-offs

- **A thin section** → the tables carry the pair (70.2 %); the coverage by source is published.
- **Borrowed glosses** → D4 makes the tables win wherever they gloss; the rest are kept and counted.
- **Copied table rules** → a further copy of fr-es's readers until fr-es re-pins; each copy's tests hold
  its behaviour.
- **A 737 MB download per update** → `keep=fr`, as it-fr.
- **A prototype, not the real edition reader** → task 0.1 re-measures before the floor is fixed.

## Migration Plan

Nothing deploys. The tables are committed by the owner-dispatched update's pull request, and the pack is
not listed. Rollback is a revert.

## Effort

3.5–6.25 ideal days, against the programme's 3.5–6:

| Part | Ideal days |
|---|---|
| Registration and its tests | 0.5–0.75 |
| `reduce-fr-it.py`: fr-es's readers copied, the borrowed-gloss rule, the share, the notice, its tests | 1–1.75 |
| Workflows and `test_reduce_loops` | 0.25–0.5 |
| The dispatch, the committed tables, the README | 0.5–1 |
| The golden, cross-native, committed-tables tests, testdata, the row cut | 0.75–1.25 |
| The card snapshot | 0.25–0.5 |
| The floor, OpenSpec, review | 0.25–0.5 |

## Open Questions

For the owner:
1. **The floor** (task 0.1): the re-measured figures less two points, about 79.3 / 68.1 / 55.8.
   Recommended, accepting that it does not hold fr-it to the inverted table, whose part the README shows.
2. **Borrowed glosses** (D4): a rule of fr-it's own (recommended), fixing 18 / 26 / 40 rows at no
   coverage cost; a shared or edition rule would move en-it.
3. **Function words no source glosses** (`des`, `du`, `ça`, `dont`): the gap accepted here, and a
   person-written list put to the owner in change 48, as en-it's contractions go to change 37.
4. **fr-es's refinement rules on the tables**: change 48's (recommended); they move rows, not coverage.
5. **kaikki's empty senses**: no rule (the tables gloss them); reported upstream to wiktextract as a
   known defect.
