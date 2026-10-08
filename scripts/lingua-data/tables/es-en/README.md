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
`2026.10.08+ee357fe.08034dc`: the snapshot, the rules' digest, and the digest of the studied tables
`pin.json` records), reduced again from the same sources with the English edition's layout rules
(below):

- **English glosses for 31,885 lemmas** of Spanish's 60,000, 31,756 from the English Wiktionary's
  Spanish entries and 129 from the Spanish Wiktionary's translations; **15,515 expressions**, 14,803
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

- **Dictionary words are es-fr's**: 9,928 lemmas es-en glosses are no dictionary word of Spanish,
  and 798 dictionary words have no English gloss, so the pack carries a lexical section, and the
  vocabulary estimate counts the same 22,755 words for both pairs.
- **The pack is 2,567,804 B**, 49.0 % of the 5 MiB budget.

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

## Meanings, not the page's layout

The English Wiktionary writes a nested sense with its parents' glosses first, a shortened form as a
pointer that carries its meaning after its target, and its own typography; read as written, es-en
glossed « venir » by two sense-group labels, « su » « apocopic form of suyo », « lo » by its article
alone and « como » by the city of Como. A pre-pass of `reduce_edition_en.py`, `read_as_meanings`,
reads the entries before the shared rules and the etymology merging do (refine-lingua-es-en-glosses):

- **A nested sense under a label or a pointer** is read by its own gloss: a parent ending on a colon
  (« places in Peru: »), naming senses (« Figurative senses. ») or pointing (« diminutive of casa »,
  « apocopic form of suyo », « alternative form of Quebec: »). A parent that is a meaning (« to make »
  over « to create ») keeps glossing its senses.
- **A shortened or respelled form** — a sense worded « apocopic », « apheretic », « syncopic » or
  « prepositional form of », « pronunciation » or « eye dialect spelling of », never known by a tag
  alone — reads as the meaning its pointer carries (« mi » « my », « muy » « very », « cincuenta y
  un » « fifty-one »), else as its target's senses in the same part of speech, from a target of
  three letters or more (« su » by « suyo », « toy » by « estoy » by « estar »), in its place; else it
  stays a pointer (« er », whose target « el » has two letters). **A pronoun's case form** reads as
  the meaning it carries after a colon or a semicolon (« lo » « him, you (formal), it, that »).
- **A function word does not open on a place**: the proper-noun lines of a headword with an initial
  capital, not all capitals, go after every other line when the lower-case word is a preposition, a
  conjunction, a pronoun, a determiner or an article (« como »).
- **One English typography**: a gloss stops at a line break; a source's numbered sense goes (« [sense
  1] », « (difference from sense 4 …) »); the edition's descriptions open in lower case (a closed list
  of openers, « The » before a capital kept); one ellipsis « … », spaced between two words; straight
  double quotes paired “ ”.

Against the tables before it: **268 rows change, 111 of the top 10,000** — 250 / 104 glossed by their
own senses, 18 / 7 borrowed from a pointer's target —, the first sense of 102 / 27; **9 lemmas gain a
gloss** (« cosita », « cajita », « chiquillo », « ramita », « québec », « mui », « vien », « kiero »,
« pid ») and none loses one; **332 expressions change and 25 gain a gloss** (« cincuenta y un »,
« vigésimo primer », « cuando quier », « po favó »…); the runs of 46 rows move. Rule by rule: the
nested senses 22 / 11 rows, the shortened forms and case forms 42 / 18 (4 lemmas and the 25
expressions gained), « como » 1, the typography 219 / 87 rows and 330 expressions. The coverage is
unchanged to the decimal; the tables are the same reduced from the English edition's dump of
2026-10-03 (`../../SOURCES.md`, *Extract and dump are measured against each other*).

Left as they are, for the owner or a shared fix: the labels the packs do not carry (obsolete,
regional, register: 3,965 rows), the part of speech a row opens on (« hasta » « even »), a proper
noun before a common word (« chile » « Chile (…) »; a case-aware card would choose by the token's
capital), « q », « k » and « t » borrowing « que »'s and « tiempo »'s senses through an
abbreviation, IPA and upstream wording (« indiference »), and « etc »'s period, which every pair loses
(`reduce_common.clean_gloss`).

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
