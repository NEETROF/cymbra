# The es→en dictionary tables

The reduced tables Cymbra Lingua's Spanish→English data pack is built from: Spanish glossed in
English, for English speakers studying Spanish — the first pair of a studied language's second
native language (add-lingua-pack-es-en, change 21 of `docs/lingua/language-matrix-programme.md`).
They are committed so that every build makes the same pack with no download, and so that a change
to the dictionary is a pull request whose diff shows it. No extension package carries the pack:
`apps/lingua-extension/packs.json` does not list es-en until the interface speaks English and the
pair is enabled (change 34).

This folder holds what belongs to es-en alone: its English glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. Spanish's own tables — its forms, ranks,
levels, readings, tag pool and dictionary words — are kept once, in `../es/`, and written by es-fr's
reduction alone (`../es/studied.json` names es-fr, Spanish's reference pair). es-en's reduction reads
them as committed and computes nothing of them: its lemmas and their ranks are `../es/forms.tsv` and
`../es/freq.tsv`, its readings, levels and dictionary words es-fr's.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → English gloss | the English Wiktionary's Spanish entries; else the English translations the Spanish Wiktionary's Spanish entries list (both CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same; a noun's gender is not here, the builder reads it from `../es/grammar.tsv`'s readings |
| `mwe.tsv` | expression → English gloss | the same sources, for multi-word headwords |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources, and the French Wiktionary, whose senses (es-fr's glosses) decide Spanish's dictionary words and which lemmas take a level | — |
| `manifest.json` | the pack's metadata: Spanish glossed in English, the Spanish analyser's version, `levels_estimated`, and `pack_version` (es-en's snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | es-en's raw sources, the pack these tables build with `../es/`, the rules that reduced them, and the sha256 of each of `../es/`'s six tables they were built on (`studied`) | — |

## What is in them

On the 2026-10-08 tables (pinned snapshot `2026.10.08`, `pack_version`
`2026.10.08+9e2a442.08034dc`: the snapshot, the rules' digest, and the digest of the studied tables
`pin.json` records):

- **English glosses for 31,876 lemmas** of Spanish's 60,000, 31,747 from the English Wiktionary's
  Spanish entries and 129 from the Spanish Wiktionary's translations; **15,490 expressions**, 14,778
  and 712. Up to eight senses grouped by part of speech, as every pair's, in the lower case the
  English Wiktionary writes a foreign word's senses in. A letter glosses no word: a sense that only
  names one (« the letter r », « the letter E in the Spanish spelling alphabet »), an entry written
  under a single capital letter (`A` « bishop »), and a letter the Spanish Wiktionary translates as
  itself are left out.
- The share of the commonest lemmas glossed, which the `reduce` job holds to es-fr's published
  figures (`gloss_coverage.py --pair es-en`, its floor in `FLOORS`):

  | Lemmas | English Wiktionary | with the translations | es-fr (the floor) |
  |---|---|---|---|
  | top 5,000 | 92.9 % | 93.0 % | 87.6 % |
  | top 10,000 | 86.3 % | 86.5 % | 77.2 % |
  | top 20,000 | 76.3 % | 76.5 % | 63.7 % |
  | all 60,000 | 52.9 % | 53.1 % | 37.9 % |

- **Dictionary words are es-fr's**: 9,923 lemmas es-en glosses are no dictionary word of Spanish,
  and 802 dictionary words have no English gloss, so the pack carries a lexical section, and the
  vocabulary estimate counts the same 22,755 words for both pairs.
- **The pack is 2,567,750 B**, 49.0 % of the 5 MiB budget.

## Its sources

- **The English Wiktionary's Spanish section** — the file es-fr reads for Spanish's forms, derived
  at an update from the English Wiktionary's dump (`../../SOURCES.md`, *The editions' dumps*) —
  read here for its senses, cleaned by the English Wiktionary's rules (`reduce_edition_en.py`).
  es-en pins its own: derived when es-en is updated and published under its own release
  (`lingua-pack-sources-es-en-<snapshot>`). The pin committed today names kaikki's per-language
  extract of that section, from before the dumps (`lingua-pack-sources-es-en-2026.10.08`, kaikki's
  regeneration of 2026-10-03): it stays readable as it is recorded, and es-en's next update moves
  it to the dump. When es-fr's update
  brings es-en along, es-en is reduced from its own pin and nothing of it is published. A run that
  reduces several pairs keeps the release assets it fetched in `work/cache/<sha256>`, so an asset
  two pins name is fetched once.
- **The Spanish Wiktionary's English translations** (`kaikki-es-traductions-en.jsonl`): the English
  words its Spanish entries list, derived from kaikki's dump of the whole edition — the dump es-fr
  derives its French translations from — and kept as an asset of es-en's own release. They gloss
  what the English Wiktionary leaves out, at most three words per part of speech.
- No inverted table (the English Wiktionary's English entries are en-es's source, change 22), no
  pivot through a third language, no machine translation.

## The two settings of the English edition

Two settings of `reduce_edition_en.py` change what es-en's glosses say, and no other pair's:

- `LONG_PARENTHESIS` (M20): a parenthesis of this many characters or more goes from a gloss
  (« grave (a hole made in the Earth to bury a corpse) » → « grave » at 40). **0**: every one is
  kept.
- `MERGE_SAME_POS_ETYMOLOGIES`: kaikki writes one entry per etymology, and the round-robin across a
  word's entries takes one sense of each in turn; merged, a word's entries of one part of speech
  read as one, the first etymology's senses first. **Off**: read as written.

Reduced both ways over the top 10,000 lemmas, the bound at 40 changes 1,057 glosses and the merging
247. Both are committed at their defaults; the owner picks them on two samples of 100 of those
glosses, and a value chosen re-pins es-en alone.

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences: `gloss.tsv`, `senses.tsv` and `mwe.tsv`, CC BY-SA 4.0 and the GFDL (kaikki).
Spanish's tables in `../es/` carry theirs (`../es-fr/README.md`). `NOTICE` gives the full
attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=es-en` and `mode=update`.
  It reads today's English and Spanish Wiktionaries' dumps — not the French one —, derives the
  Spanish section and the English translations from them in one pass each, keeps the two
  derived files as the release `lingua-pack-sources-es-en-<snapshot>`, whose notes name the dumps
  by their address, regeneration date and sha256, reduces, and pushes the branch
  `lingua-pack/es-en/<snapshot>`. No dump is kept.
- **When Spanish's tables move**: es-fr's update or re-reduction writes `../es/`, and brings es-en
  along on the same branch, reduced again from its own pinned sources; es-en's `pack_version` moves
  with the studied tables. A pull request that moves a table of `../es/` without recording es-en
  again fails, naming es-en and the table (`pack_sources.py check-reducer`, `pack_report.py`).
- **After editing the reduction rules** — `reduce-es-en.py`, `reduce_common.py` (every pair's), or
  `reduce_edition_en.py` (the English Wiktionary's, which no pair glossed in French loads), and
  es-fr's rules too, whose reduction writes `../es/`: the check lane fails until the tables are
  reduced again from the pinned sources. Run `scripts/lingua-data/build.sh --reduce es-en <out>`
  (Python 3.12, `requirements-reduce.txt`), after es-fr's when both are due, or `lingua-pack-update`
  with `mode=reduce`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks en-fr and es-fr, in the same job:
each dump is read once for every pair.
