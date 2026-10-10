# The es→fr dictionary tables

The reduced tables Cymbra Lingua's Spanish→French data pack is built from, committed so that every
build makes the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it (add-lingua-spanish-forms-tables). No extension package carries the
pack yet: that is `enable-lingua-spanish`.

They are in two folders (split-lingua-pack-tables-by-language). This one holds what belongs to
es-fr alone: its French glosses, its expressions, the parts of speech of their senses, its notice,
manifest and pin. Spanish's own tables — its forms, ranks, levels, readings, tag pool and dictionary
words — are kept once, in `../es/`, for every pair studying Spanish. es-fr is Spanish's reference
pair (`../es/studied.json`): its reduction writes `../es/`, and this folder's `pin.json` records the
sources of both. That matters for the levels: they are estimated from es-fr's French glosses, so
every pair studying Spanish reads es-fr's.

One other pair reads `../es/`: **es-en** (`../es-en/`, Spanish glossed in English,
add-lingua-pack-es-en), which reduces its own native side from it and computes nothing of it. Its
`pin.json` records es-fr as the reference and the sha256 of each of `../es/`'s six tables its build
read: when es-fr's reduction moves one, es-en is reduced again after es-fr, and until it is the checks
fail naming es-en and the table (`es-en: es/level.tsv`). A change to es-fr's rules that moves no
table of `../es/` leaves es-en's pin and pack as they are.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → French gloss | the French Wiktionary's Spanish entries; else the Spanish Wiktionary's French translations; else the French Wiktionary's translation tables, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses, a noun's with its gender | the same, the gender from `../es/forms.tsv`'s source |
| `mwe.tsv` | expression → French gloss | the same sources, for multi-word headwords |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata: Spanish, the Spanish analyser's version, and `pack_version` (the snapshot, and the rules that reduced it) | — |
| `pin.json` | the raw sources these tables and `../es/` came from, and the pack they build | — |

In `../es/`, Spanish's tables, written by es-fr's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org, the English Wiktionary's Spanish section (CC BY-SA 4.0 + GFDL), with UD Spanish-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0) |
| `grammar.tsv` | form → its readings: dictionary form, Universal Dependencies tag, and whether it may be named as another word | kaikki's tags (CC BY-SA 4.0 + GFDL) |
| `level.tsv` | lemma → estimated CEFR level | derived from `freq.tsv` and `../es-fr/gloss.tsv` (no source of its own) |
| `lexical.tsv` | Spanish's dictionary words: the lemmas es-fr glosses, byte-sorted, one per line | derived from `../es-fr/gloss.tsv` by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | Spanish's pinned tag pool, which every pack studying Spanish lays its pool out from; written by no reducer, kept when the tables are reduced again (`../../SOURCES.md`, *What a pack studies, whatever it glosses*) | this pack's own pool, committed by hand |
| `studied.json` | the pair whose reduction writes `../es/`: es-fr | committed by hand |

## What is in them

- **60,000 lemmas**, the commonest by wordfreq, and **144,952 forms**: each lemma's own form, and
  84,952 inflected forms attested in wordfreq. A form nobody writes is left to the analyser's rules.
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
- **An adjective's or a determiner's apocope reads as its full word** (fix-lingua-spanish-apocopes):
  `buen` → *bueno*, `gran` → *grande*, `primer` → *primero*, `algún` → *alguno*. kaikki's row naming
  the standard word (`buen` lists `bueno`) is no inflection: it made `bueno` a form of `buen`, which
  has no gloss. An adverb or a numeral kaikki calls apocopic stays a word of its own: `muy` is
  « Très », not *mucho*, and `un` the article, not *uno*.
- **A form that is a proper name and another word's keeps the commoner reading**
  (fix-lingua-spanish-card-noise), before any rule above: the form's frequency, met as the name or
  as the word, against the word's lemma. `miró` goes to *mirar* (3.98 against 4.74) and `dolores` to
  *dolor*, while `argentina` and `parís` stay the country and the city. GSD's counts, which meet
  `Miró` only as the surname, gave every such form to the name: 387 forms, 118 of which now read as
  the word.
- **Grammar**: 149,309 readings of 111,983 forms, in 89 Universal Dependencies tags, read from
  kaikki's tags (add-lingua-spanish-grammar-tables). Every verb form says its mood, tense, person
  and number (`hablábamos`: indicative imperfect, first person plural). A participle, an adjective,
  a determiner or a pronoun says its agreement (`escrita`, `rápidas`). A noun says its gender on
  its own form and on its plural: 99.7 % of the nouns have one (`casa`, `casas`: feminine). A form
  of another kept lemma is marked `other`, so the card names it: `vino` is also *venir*'s
  preterite, `fue` *ir*'s. A letter's name keeps its own form but gives none of its plurals
  (add-lingua-spanish-word-card): `es` is *ser*'s alone, not also the plural of the letter E.
- **French glosses** for 22,755 lemmas, and 15,133 expressions (add-lingua-spanish-gloss-tables).
  The French Wiktionary's Spanish entries come first, through the rules every pair shares: up to
  eight senses, grouped by part of speech. Where they say nothing, people's translations fill
  the gap. First, the French words the Spanish Wiktionary lists (`sector` « Secteur »). Then the
  French entries whose translation tables list the word, the commonest first (`decreto`
  « Arrêté »). A gloss is never English and never a machine translation, and a proper noun's
  translation glosses nothing. A letter's sense is no gloss (fix-lingua-spanish-card-noise): the
  card of `a` opens on « À », not on « Première lettre … de l'alphabet espagnol ». The share of the
  commonest lemmas glossed:

  | Lemmas | French Wiktionary | with the translations |
  |---|---|---|
  | top 5,000 | 82.7 % | 87.6 % |
  | top 10,000 | 69.4 % | 77.2 % |
  | top 20,000 | 54.2 % | 63.7 % |
  | all 60,000 | 28.9 % | 37.9 % |

  Expressions: 2,950 from the French Wiktionary's Spanish entries, 12,183 from the translations.
  The builder keeps those whose words the lexicon holds. `LOCUTIONS` in `reduce-es-fr.py`, for the
  verbal locutions no source glosses (`hay que`), is empty: its glosses are written by a person.
- **A noun's gender in its sense runs** (add-lingua-spanish-word-card): `casa	NOUN|Gender=Fem:1`,
  so the card's heading reads « nom féminin ». It is the gender the English Wiktionary's `es-noun`
  head gives, the one the readings carry; a noun of both genders (`estudiante`) keeps `NOUN`. 12,050
  runs carry one.
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
and proper nouns are left out. On the 2026-10-04 tables:

| | Measured | Gate |
|---|---|---|
| words resolved in the lexicon | 99.38 % of 19,276 | 98.5 % |
| content words taking PUD's lemma | 95.79 % of 9,439 | 93.5 % |
| auxiliaries taking PUD's lemma | 97.95 % of 634 | 97 % |

PUD itself is not consistent on apocopes: it keeps `gran` as its own lemma but takes `primer` to
*primero*, which is why reading apocopes as their full words moved the content words from 95.92 %
to 95.79 %.

The pack is 2,224,439 B, with the grammar, the glosses, the levels and the expressions' names
(add-lingua-spanish-expression-keys).

## Licences

The repository is Apache-2.0; **these files are not**, nor Spanish's in `../es/`. They are derived
from the sources above and carry their licences:
- `../es/forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `../es/grammar.tsv`, `gloss.tsv`, `senses.tsv`, `mwe.tsv`, and `../es/lexical.tsv` (derived from
  `gloss.tsv`): CC BY-SA 4.0 and the GFDL (kaikki);
- `../es/level.tsv`: derived from `../es/freq.tsv` (CC BY-SA 4.0) and `gloss.tsv`;
- `../es/freq.tsv`: CC BY-SA 4.0.

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand — except `../es/tags.tsv` and `../es/studied.json`, which no reducer writes.
`tags.tsv` is Spanish's pinned tag pool: every pack studying Spanish stores its readings against it,
so editing it changes how each of them stores them. Its pull request says so, and every pair
studying Spanish reads that one file. `../es/` is written by es-fr's reduction alone: a change there
— new sources, new rules, or a lemma es-fr glosses, which moves the dictionary words and may move a
level — reaches every pair studying Spanish, es-en today, whose packs are recorded again in the same pull
request (`lingua-pack-update` reduces them along with es-fr, after it).

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=es-fr` and `mode=update`.
  It works as for en-fr:
  1. reads today's sources;
  2. keeps kaikki's bytes as the release `lingua-pack-sources-es-fr-<snapshot>`: the four files
     derived from the English, French and Spanish Wiktionaries' dumps (`pack_sources.py DUMPS`),
     each dump fetched once, read in one pass, named in the release notes by its address,
     regeneration date and sha256, and not kept — about 3.6 GiB in all (`../../SOURCES.md`, *The
     editions' dumps*);
  3. reduces, then reduces es-en again from its own pinned sources;
  4. pushes the branch `lingua-pack/es-fr/<snapshot>`.

  The pin committed today names kaikki's per-language extract of the English Wiktionary's Spanish
  section (2026-09-28), from before the dumps: it stays readable as it is recorded, and the next
  update moves es-fr to the dump. Both readings of one regeneration give the same tables but for
  four readings the dump adds to `../es/grammar.tsv` — the feminine plurals of *beta*, *delta*,
  *kappa* and *zeta* (`../../SOURCES.md`, *Extract and dump are measured against each other*): the
  dump leaves the page's categories on the entry, where es-fr's letter-name rule reads a sense's.
  They break *A letter's name gives no reading of its plural* (`betas` read as the plural of the
  letter *beta*), so a change of its own must fix them before es-fr's next update is merged — by a
  rule on the sense's gloss, not by reading the entry's categories, which the dump puts on every
  entry of the page and which would drop the committed `Masc|Plur` readings of its other nouns.
  That update's report then shows the upstream drift, and no longer those four. es-en, brought
  along, keeps its own pin until its own update.
- **After editing the reduction rules** — `reduce-es-fr.py`, the override and locution lists
  included; `reduce_common.py`, which every pair shares; or `reduce_edition_fr.py`, the French
  Wiktionary's rules, which every pair glossed in French loads (`pin.json` lists the three under
  `reducer.files`, the modules the reducer loads, `../../SOURCES.md`, *The Wiktionary editions'
  rules*): the check lane fails, for es-fr and every other pair studying Spanish, until the tables
  are reduced again from the pinned sources. Run `scripts/lingua-data/build.sh --reduce es-fr <out>`
  (Python 3.12, `requirements-reduce.txt`), or `lingua-pack-update` with `mode=reduce`. The English and Spanish Wiktionaries' rules
  (`reduce_edition_en.py`, `reduce_edition_es.py`) are not es-fr's — its forms are read from the
  English Wiktionary by its own rules, its glosses from the French one — so editing them asks
  nothing of these tables.
- **A change of rules meant to move no table**: dispatch `lingua-pack-update` with `mode=reduce`,
  `pair=all` and `expect=identical`, as for en-fr. The `reduce` job of `lingua-extension-check`
  reduces every pair again on each pull request that touches `scripts/lingua-data`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks en-fr.
