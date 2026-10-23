# The en→fr dictionary tables

The reduced tables Cymbra Lingua's English→French data pack is built from, committed so that every
release builds the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it (pin-lingua-pack-sources).

They are in two folders (split-lingua-pack-tables-by-language). This one holds what belongs to
en-fr alone: its French glosses, its expressions, the parts of speech of their senses, its notice,
manifest and pin. English's own tables — its forms, ranks, levels, readings, tag pool and dictionary
words — are kept once, in `../en/`, for every pair studying English. en-fr is English's reference
pair (`../en/studied.json`): its reduction writes `../en/`, and this folder's `pin.json` records the
sources of both.

One other pair reads `../en/`: **en-es** (`../en-es/`, English glossed in Spanish,
add-lingua-pack-en-es), which reduces its own native side from it and computes nothing of it. Its
`pin.json` records en-fr as the reference and the sha256 of each of `../en/`'s six tables its build
read: when en-fr's reduction moves one, en-es is reduced again after en-fr, and until it is the checks
fail naming en-es and the table (`en-es: en/level.tsv`). A change to en-fr's rules that moves no
table of `../en/` leaves en-es's pin and pack as they are.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → French gloss | kaikki.org, the French Wiktionary's English entries (CC BY-SA 4.0 + GFDL) |
| `mwe.tsv` | expression → French gloss | kaikki.org, the French Wiktionary's English entries (CC BY-SA 4.0 + GFDL) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | kaikki.org, the French Wiktionary's English entries (CC BY-SA 4.0 + GFDL) |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata; `pack_version` is the snapshot | — |
| `pin.json` | the raw sources these tables and `../en/` came from, and the pack they build | — |

In `../en/`, English's tables, written by en-fr's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | ESDB's inflections (permissive), with the French Wiktionary's form links |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0) |
| `level.tsv` | lemma → CEFR level | CEFR-J Wordlist v1.5 (commercial use with citation) + Octanove Vocabulary Profile C1/C2 v1.0 (CC BY-SA 4.0) |
| `grammar.tsv` | form → its readings: dictionary form, Universal Dependencies tag, and whether it may be named as another word | ESDB's slots (permissive) and kaikki's form links (CC BY-SA 4.0 + GFDL) |
| `lexical.tsv` | English's dictionary words: the lemmas en-fr glosses less the 3,046 it glosses by a proper noun's senses alone (21,753; *Names are no dictionary words*), byte-sorted, one per line | derived from `gloss.tsv` and `senses.tsv` by `reduce-en-fr.py` (the French Wiktionary's `dictionary_words`, `reduce_edition_fr.py`), filed by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | English's pinned tag pool, which every pack studying English lays its pool out from; written by no reducer, kept when the tables are reduced again (`../../SOURCES.md`, *What a pack studies, whatever it glosses*) | this pack's own pool, committed by hand |
| `studied.json` | the pair whose reduction writes `../en/`: en-fr | committed by hand |

## Names are no dictionary words

English's dictionary words leave out the lemmas en-fr glosses by a proper noun's senses alone
(leave-lingua-names-out-of-english-and-spanish-words, the owner's decision of 2026-10-10, as French's
since refine-lingua-fr-en-glosses): every run of their `senses.tsv` row is `PROPN`, the French
Wiktionary's « Nom propre ». Of en-fr's 24,799 glossed lemmas, 3,046 are names alone — 388, 993 and
1,868 of the 5,000, 10,000 and 20,000 commonest: `york` (rank 348), `london` (442), `america`,
`june`, `david`, `england`, `uk` — so `../en/lexical.tsv` lists 21,753. They keep their gloss: a
reader who selects `London` reads « Londres ». A word with a common sense beside a name's stays one
(`bill`, `mark`, `will`, `hope`).

The rule is the French Wiktionary's module's (`reduce_edition_fr.py` `dictionary_words`), which en-fr
and es-fr alone load, so it is in their rule digests and in no other pair's; fr-en keeps its own copy
(`reduce-fr-en.py`). The levels are CEFR-J's and Octanove's and do not read it: eleven names alone
carry one and keep it — the eight months the French Wiktionary files as proper nouns (`january`,
`february`, `april`, `june`, `july`, `september`, `november`, `december`, A1), `olympics` (A1),
`badminton` (A2) and `ph` (B2) — and the vocabulary sizes still count them: the estimate's universe
is 22,337 words (25,372 with the names). English's names rule (`crates/lingua-core` `engine.rs`)
reads a levelled lemma as a word and sets the others aside in a page that never writes them in
lowercase. Some common words the French Wiktionary files only as names leave too (`easter`,
`islam`, `hebrew`, `esperanto`, `renaissance`, the mis-filed `unseen`): the owner accepted them
(the change's Q3).

The pack therefore carries a lexical table: 1,840,736 B (1,835,638 B before), 35.1 % of the 5 MiB
budget.

## An expression's gloss keeps its senses whole (keep-lingua-expression-glosses-whole)

The owner decided on 2026-10-11 that an expression's gloss is no longer cut: its senses were cut at 42
characters, mid-word, and the joined gloss at 80. Each sense the shared rules pick — up to three, one
per entry in turn, a repeat left out — is now whole; only a sense past 300 characters, a word's sense
bound (`reduce_common.EXPRESSION_SENSE_LEN`), is cut after a whole word and ends with « … ». The joined
gloss is not cut: the card pages it as it pages a word's (160 characters a page), and the selection
card's row still shows its first sense within 80 characters. The rule lives in `reduce_common.py`, so
every pair re-pinned, reduced again from its pinned release assets: only `mwe.tsv`, `manifest.json` and
`pin.json` moved; `gloss.tsv`, `senses.tsv`, `NOTICE`, the studied tables and the coverage are byte for
byte.

| | Cut (42 / 80) | Whole |
|---|---|---|
| Expressions | 17,416 | 17,416: none added or left out, 2,878 changed |
| Longest gloss | 80 | 537 (« situation awareness ») |
| Glosses past 80 / 160 / 300 characters | — | 1,352 / 312 / 30 |
| Longest sense | — | 300 (the bound) |
| Senses the 300 bound cuts | — | 19 rows (« machete order » 458 characters, « lumped element » 412, « penis fencing » 411, « comic strip » 410, « machine learning » 357…) |
| `mwe.tsv` | 744,353 B | 883,153 B |
| The pack | 1,840,736 B | 1,876,816 B, 35.8 % of the 5 MiB budget |
| Expression cards that page | 0 | 175, three pages at most |
| Rows whose first sense changes | — | 2,522 |

Every changed row holds the old row's senses, each a prefix of its new one, but for 25 — the cut had fallen inside a sense holding its own `;`.

**Known data defects** the whole text shows, left as the source writes them (each for en-fr's own refinement):

- « look forward to »: « Attendre (avec plaisir, avec intérêt, avec impatience) ; se réjouir (d’avance) de. Note d’usage : Suivi par un nom ou un gérondif en -ing » — a note to the edition's readers inside the sense, which the cut hid.
- A « ; » inside a sense, French typography's (« Attendre (…) ; se réjouir (d'avance) de »): 201 expression glosses, 138 before. The card's pages and the row read `;` as a sense separator, so such a sense shows as several. Left as it is (design D4): a word's sense has it written « , » because its sense runs count it; an expression has no sense runs.

## Licences

The repository is Apache-2.0; **these files are not**, nor English's in `../en/`. They are derived
from the sources above and carry their licences: the Wiktionary-derived tables (`gloss.tsv`,
`mwe.tsv`, `senses.tsv`, the dictionary words derived from `gloss.tsv` and `senses.tsv`, and the Wiktionary signals in
`../en/forms.tsv` and `../en/grammar.tsv`) under CC BY-SA 4.0 and the GFDL, `../en/freq.tsv` under
CC BY-SA 4.0, `../en/level.tsv` under CEFR-J's terms (commercial use allowed with citation) and
CC BY-SA 4.0, ESDB's relations under its permissive licence. `NOTICE` gives the full attribution. See
`../../SOURCES.md`.

## Changing them

Never by hand — except `../en/tags.tsv` and `../en/studied.json`, which no reducer writes.
`tags.tsv` is English's pinned tag pool: every pack studying English stores its readings against it,
so editing it changes how each of them stores them. Its pull request says so, and every pair
studying English reads that one file. `../en/` is written by en-fr's reduction alone: a change there
— new sources, new rules, or a lemma en-fr glosses, which moves the dictionary words — reaches every
pair studying English, en-es today, whose packs are recorded again in the same pull request
(`lingua-pack-update` reduces them along with en-fr, after it).

- **Take in upstream changes**: dispatch `lingua-pack-update` with `mode=update`. It reads today's
  sources — kaikki as the French Wiktionary's dump (737 MB gzipped), from which it derives the
  English entries in one pass and keeps no dump (`../../SOURCES.md`, *The editions' dumps*) —,
  keeps the derived file as the release `lingua-pack-sources-en-fr-<snapshot>`, whose notes name
  the dump by its address, regeneration date and sha256, reduces, then reduces en-es again from
  its own pinned sources, pushes the branch `lingua-pack/en-fr/<snapshot>`, and writes a report of
  what changes. Open the pull request from the link in its summary; releases keep these tables
  until it is merged. The pin committed today names kaikki's per-language extract of the French
  Wiktionary's English section (2026-09-24), from before the dumps: it stays readable as it is
  recorded, and the next update moves en-fr to the dump — the same entries, read the same way
  (`../../SOURCES.md`, *Extract and dump are measured against each other*: en-fr's and `../en/`'s
  tables byte for byte the same both ways), so its report shows the upstream drift alone.
- **After editing the reduction rules** — `reduce-en-fr.py`; `reduce_common.py`, which every pair
  shares; or `reduce_edition_fr.py`, the French Wiktionary's rules, which every pair glossed in French
  loads (`pin.json` lists the three under `reducer.files`, the modules the reducer loads,
  `../../SOURCES.md`, *The Wiktionary editions' rules*): the check lane fails until the tables are
  reduced again from the pinned sources — `scripts/lingua-data/build.sh --reduce en-fr <out>`
  (Python 3.12, `requirements-reduce.txt`), or `lingua-pack-update` with `mode=reduce`. The diff is
  then the rules' effect alone, in this folder and in `../en/`; the checks of every other pair
  studying English fail too until en-fr is reduced again. The English and Spanish Wiktionaries' rules (`reduce_edition_en.py`,
  `reduce_edition_es.py`) are not en-fr's: editing them asks nothing of these tables.
- **A change of rules meant to move no table** — dispatch `lingua-pack-update` with `mode=reduce`,
  `pair=all` and `expect=identical`: it fails, naming the pair and the file, on any byte beyond
  `pack_version` and the pins. The `reduce` job of `lingua-extension-check` reduces every pair again
  on each pull request that touches `scripts/lingua-data`, and fails when the result is not the
  committed tables.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request, and the parts the package carries
  (`pack_sources.py record-parts --pin tables/en-fr/pin.json --base <base> --layer <layer>`, the two
  files `build.sh --parts en-fr <base> <layer>` writes).

A monthly dry run of the update reports how far upstream has drifted, and fails when a source moved
or a table collapsed.
