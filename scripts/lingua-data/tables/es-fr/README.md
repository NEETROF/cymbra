# The es→fr dictionary tables

The reduced tables Cymbra Lingua's Spanish→French data pack is built from, committed so that every
build makes the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it (add-lingua-spanish-forms-tables). No extension package carries the
pack yet: that is `enable-lingua-spanish`.

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org extract of the English Wiktionary, Spanish section (CC BY-SA 4.0 + GFDL), with UD Spanish-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0) |
| `gloss.tsv` | lemma → French gloss | empty until `add-lingua-spanish-gloss-tables` |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata: Spanish, the Spanish analyser's version, and `pack_version` (the snapshot, and the rules that reduced it) | — |
| `pin.json` | the raw sources these tables came from, and the pack they build | — |

## What is in them

- **60,000 lemmas**, the commonest by wordfreq, and **144,805 forms**: each lemma's own form, and
  84,805 inflected forms attested in wordfreq. A form nobody writes is left to the analyser's rules.
- **No verb with its clitics** (`dámelo`, `hacerlo`): the analyser's enclitic rule reads them. A
  string that is also a plain form keeps it: `principales`, which is also *principar* + `les`, maps
  to *principal*.
- **One lemma per form.** Where a form has several, the reducer decides in this order:
  1. `OVERRIDES` in `reduce-es-fr.py`, each row with its reason. It is empty: a lemma's own form
     reads as itself, so an override takes the other lemma out of the pack (`vino` stays the noun,
     and the card names *venir*);
  2. GSD's counts (`fue` → *ser*);
  3. the form's own entry (`casa` → *casa*);
  4. the commoner lemma (`luces` → *luz*).

## Measured

`scripts/lingua-data/measure/es-pud.sh` builds the pack from these tables and runs the real analyser
over UD Spanish-PUD, which the reduction never reads. Punctuation, numbers, symbols, foreign words
and proper nouns are left out. On the 2026-10-03 tables:

| | Measured | Gate |
|---|---|---|
| words resolved in the lexicon | 99.38 % of 19,276 | 98.5 % |
| content words taking PUD's lemma | 95.92 % of 9,439 | 93.5 % |
| auxiliaries taking PUD's lemma | 97.95 % of 634 | 97 % |

The pack is 1,307,728 B.

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences:
- `forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `freq.tsv`: CC BY-SA 4.0.

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=es-fr` and `mode=update`.
  It works as for en-fr:
  1. reads today's sources;
  2. keeps kaikki's bytes as the release `lingua-pack-sources-es-fr-<snapshot>`;
  3. reduces;
  4. pushes the branch `lingua-pack/es-fr/<snapshot>`.
- **After editing the reduction rules** — `reduce-es-fr.py`, the override list included, or
  `reduce_common.py`, which every pair shares (`pin.json` lists both under `reducer.files`): the
  check lane fails until the tables are reduced again from the pinned sources. Run
  `scripts/lingua-data/build.sh --reduce es-fr <out>` (Python 3.12, `requirements-reduce.txt`), or
  `lingua-pack-update` with `mode=reduce`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks en-fr.
