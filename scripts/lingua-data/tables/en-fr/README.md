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
| `gloss.tsv` | lemma → French gloss | kaikki.org extract of the French Wiktionary (CC BY-SA 4.0 + GFDL) |
| `mwe.tsv` | expression → French gloss | kaikki.org extract of the French Wiktionary (CC BY-SA 4.0 + GFDL) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | kaikki.org extract of the French Wiktionary (CC BY-SA 4.0 + GFDL) |
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
| `lexical.tsv` | English's dictionary words: the lemmas en-fr glosses, byte-sorted, one per line | derived from `../en-fr/gloss.tsv` by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | English's pinned tag pool, which every pack studying English lays its pool out from; written by no reducer, kept when the tables are reduced again (`../../SOURCES.md`, *What a pack studies, whatever it glosses*) | this pack's own pool, committed by hand |
| `studied.json` | the pair whose reduction writes `../en/`: en-fr | committed by hand |

## Licences

The repository is Apache-2.0; **these files are not**, nor English's in `../en/`. They are derived
from the sources above and carry their licences: the Wiktionary-derived tables (`gloss.tsv`,
`mwe.tsv`, `senses.tsv`, the dictionary words derived from `gloss.tsv`, and the Wiktionary signals in
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
  sources, keeps kaikki's bytes as the release `lingua-pack-sources-en-fr-<snapshot>`, reduces, then
  reduces en-es again from its own pinned sources, pushes the branch `lingua-pack/en-fr/<snapshot>`,
  and writes a report of what changes. Open the pull request from the link in its summary; releases
  keep these tables until it is merged.
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
  `pack.size` in `pin.json` in the same pull request.

A monthly dry run of the update reports how far upstream has drifted, and fails when a source moved
or a table collapsed.
