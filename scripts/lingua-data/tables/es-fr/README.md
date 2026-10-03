# The es→fr dictionary tables

The reduced tables Cymbra Lingua's Spanish→French data pack is built from, committed so that every
build makes the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it (add-lingua-spanish-forms-tables). No extension package carries the
pack yet: that is `enable-lingua-spanish`.

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org extract of the English Wiktionary, Spanish section (CC BY-SA 4.0 + GFDL), with UD Spanish-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0) |
| `grammar.tsv` | form → its readings: dictionary form, Universal Dependencies tag, and whether it may be named as another word | kaikki's tags (CC BY-SA 4.0 + GFDL) |
| `gloss.tsv` | lemma → French gloss | the French Wiktionary's Spanish entries; else the Spanish Wiktionary's French translations; else the French Wiktionary's translation tables, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same |
| `mwe.tsv` | expression → French gloss | the same sources, for multi-word headwords |
| `level.tsv` | lemma → estimated CEFR level | derived from `freq.tsv` and `gloss.tsv` (no source of its own) |
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
- **Grammar**: 149,279 readings of 111,945 forms, in 88 Universal Dependencies tags, read from
  kaikki's tags (add-lingua-spanish-grammar-tables). Every verb form says its mood, tense, person
  and number (`hablábamos`: indicative imperfect, first person plural). A participle, an adjective,
  a determiner or a pronoun says its agreement (`escrita`, `rápidas`). A noun says its gender on
  its own form and on its plural: 99.7 % of the nouns have one (`casa`, `casas`: feminine). A form
  of another kept lemma is marked `other`, so the card names it: `vino` is also *venir*'s
  preterite, `fue` *ir*'s.
- **French glosses** for 22,826 lemmas, and 15,133 expressions (add-lingua-spanish-gloss-tables).
  The French Wiktionary's Spanish entries come first, through the rules every pair shares: up to
  eight senses, grouped by part of speech. Where they say nothing, people's translations fill
  the gap. First, the French words the Spanish Wiktionary lists (`sector` « Secteur »). Then the
  French entries whose translation tables list the word, the commonest first (`decreto`
  « Arrêté »). A gloss is never English and never a machine translation, and a proper noun's
  translation glosses nothing. The share of the commonest lemmas glossed:

  | Lemmas | French Wiktionary | with the translations |
  |---|---|---|
  | top 5,000 | 82.9 % | 87.7 % |
  | top 10,000 | 69.6 % | 77.3 % |
  | top 20,000 | 54.4 % | 63.8 % |
  | all 60,000 | 29.0 % | 38.0 % |

  Expressions: 2,952 from the French Wiktionary's Spanish entries, 12,181 from the translations.
  The builder keeps those whose words the lexicon holds. `LOCUTIONS` in `reduce-es-fr.py`, for the
  verbal locutions no source glosses (`hay que`), is empty: its glosses are written by a person.
- **Estimated levels** for 8,302 lemmas (add-lingua-spanish-levels). No Spanish CEFR list can be
  shipped, so the levels are derived from frequency, and the manifest says `levels_estimated`;
  the extension labels them « estimé ». The commonest lemmas, in rank order, take the sizes of
  English's CEFR levels: 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2. A lemma with no
  French gloss, or only a proper noun's, is skipped (`the`, `twitter`, `madrid`). A1 runs to
  rank 1,086 and C2 to 12,069.

  Measured on English's own CEFR lemmas, this rule agrees with the lists for 39.8 % of them, and
  within one level for 82.6 %. The scale is monotone: each estimated level's mean true level
  rises from 1.67 at A1 to 5.03 at C2.

## Measured

`scripts/lingua-data/measure/es-pud.sh` builds the pack from these tables and runs the real analyser
over UD Spanish-PUD, which the reduction never reads. Punctuation, numbers, symbols, foreign words
and proper nouns are left out. On the 2026-10-03 tables:

| | Measured | Gate |
|---|---|---|
| words resolved in the lexicon | 99.38 % of 19,276 | 98.5 % |
| content words taking PUD's lemma | 95.92 % of 9,439 | 93.5 % |
| auxiliaries taking PUD's lemma | 97.95 % of 634 | 97 % |

The pack is 2,186,617 B, with the grammar, the glosses and the levels.

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences:
- `forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `grammar.tsv`, `gloss.tsv`, `senses.tsv`, `mwe.tsv`: CC BY-SA 4.0 and the GFDL (kaikki);
- `level.tsv`: derived from `freq.tsv` (CC BY-SA 4.0) and `gloss.tsv`;
- `freq.tsv`: CC BY-SA 4.0.

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=es-fr` and `mode=update`.
  It works as for en-fr:
  1. reads today's sources;
  2. keeps kaikki's bytes as the release `lingua-pack-sources-es-fr-<snapshot>`: the extract, and
     the three files derived from the French and Spanish Wiktionaries' dumps (`pack_sources.py
     DUMPS`);
  3. reduces;
  4. pushes the branch `lingua-pack/es-fr/<snapshot>`.
- **After editing the reduction rules** — `reduce-es-fr.py`, the override and locution lists included, or
  `reduce_common.py`, which every pair shares (`pin.json` lists both under `reducer.files`): the
  check lane fails until the tables are reduced again from the pinned sources. Run
  `scripts/lingua-data/build.sh --reduce es-fr <out>` (Python 3.12, `requirements-reduce.txt`), or
  `lingua-pack-update` with `mode=reduce`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks en-fr.
