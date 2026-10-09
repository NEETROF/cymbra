# add-lingua-french-forms-tables — French's forms and frequencies, kept once in `tables/fr/`

## Why

Change 43 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en, fr-es). Change 39 made French a studied language served by the
baseline analysis (analyser `0.1.0`): the pack's form→lemma lookup, else the lowercased form. No
pack studying French exists outside the hand-written fixture of `testdata/fr-en/`, so today every
French word but the fixture's 286 forms reads as itself. The forms table decides how each French
token is lemmatised, which words are counted, known and reviewed, and which reading of a homograph a
reader meets. Every later change of the stage reads it: the analysis (41) is written and measured
against it, the grammar (45) reads the readings of the forms it holds, the levels (46) its ranks,
the fr-en and fr-es packs (48, 49) are built on it.

Decision M24 keeps a studied language's side once, in `tables/<studied>/`, written only by the
reduction of its reference pair — « for a language studied later the first pair built for it ».
For French that pair is fr-en: its forms and its glosses come from one source, the English
Wiktionary's French section, which change 38 derives from the English edition's dump
(`kaikki-French.jsonl` in its catalogue, « French's studied side and fr-en's glosses (changes 43,
45, 48) »).

The programme measured that source: 403,269 entries, about 7,380 fully tagged verbs, and a
prototype reducer past Spanish's gates on UD French-PUD (99.04 % resolved, 95.93 % content lemmas,
99.90 % auxiliaries). This change prototyped the reduction again on the real data — the French
section derived on 2026-10-08 from the English dump regenerated on 2026-10-03, UD French-GSD and
French-PUD at their current commits, wordfreq 3.1.1 — and found that Spanish's rules, transposed
as they are, fail the resolution gate (97.61 % on PUD): the English Wiktionary lists a French
verb's agreed participles (`dirigée`, `composées`) under the participle, not under the verb; it
writes the elided pieces (`l'`, `qu'`, `s'`) with no pointer or with several; wordfreq cannot rank a
hyphenated word and splits elisions into bare letters. With the rules of this design, the tables
pass on PUD with the real analyser (99.12 % resolved, 96.38 % content lemmas, 99.90 %
auxiliaries) and hold on GSD's held-out test section (98.91 / 95.65 / 99.72).

Two decisions of the owner bind these tables (2026-10-09):
- **M8, one form, one lemma**, homographs included: `porte` maps to one lemma by a stated rule. The
  rule is Spanish's, and its cost is measured (design D5): GSD counts `porte` 39 times as *porter*
  and 23 times as the noun, so the noun leaves the pack, as `cuenta` left Spanish's.
- **M21, French tokenisation**: `au`/`aux` split, `du`/`des` whole, one span per elision piece.
  Change 40's proposed pre-pass hands the lookup the word a piece stands for (`l'` → `le`, `s'` →
  `si` or `se`, `t'` → `toi` or `te`), splits `au` into `à` + `le` and an inversion into its words,
  and keeps whole a hyphenated run the pack lists. The tables serve it (design D4): every word that
  pre-pass writes is a form (all 32, measured), the nouns ending in a pronoun (`rendez-vous`,
  `qu'en-dira-t-on`) are listed whole so the inversion rule never splits them, `au` and `aux` are no
  word, `du` and `des` are words of their own, and no plain word beginning with a piece is a form.

## What Changes

- **A reducer for French's reference pair** (`scripts/lingua-data/reduce-fr-en.py`), with French's
  studied rules in it, as es-fr's holds Spanish's; no shared module is edited:
  - **forms** from the French section's tagged inflections and form-of links, a doubtful or
    alternative inflection left out, and a form of a form reaching the word it is a form of along
    one part of speech (`dirigée` → `dirigé` → *diriger*);
  - **M21, as change 40 writes it**: every word its pre-pass writes a form; the fourteen elided
    pieces forms too, by a reviewed closed table (`l'` → *le*, `qu'` → *que*, `s'` → *se*, …), for
    the treebanks that write them as words; no plain word that begins with one (`c'est`,
    `d'abord`, `l'on`: the pre-pass splits them), while a hyphenated run may be (`c'est-à-dire`);
    the dictionary's nouns ending in a pronoun kept whole; `au` and `aux` neither forms nor ranks;
    `du` and `des` lemmas of their own; a verb form joined to its clitic pronouns by hyphens
    (`souviens-toi`) left out;
  - **a spelling variant** — an ASCII spelling of a ligature (`coeur`), a post-1990 spelling
    (`connait`) — read as the word it spells;
  - **one lemma per form**: a reviewed override list (empty), then UD French-GSD's counts, then the
    form's own entry, then the lemma's frequency, then alphabetical order; a form that is a name
    and a commoner word keeps the commoner reading;
  - **the 60,000 commonest lemmas** by wordfreq `fr`, wordfreq's elision stems (`l`, `d`, `qu`, …)
    no words, a hyphenated word ranked when GSD attests it, at the lower of wordfreq's estimate and
    GSD's own frequency; no word whose every form reads as another (`tenue`, read as *tenir*);
    their forms attested in wordfreq.
- **Committed tables.** `tables/fr/`: `forms.tsv` (124,040 rows, 2.26 MB in the prototype),
  `freq.tsv` (60,000 rows, 0.84 MB), `studied.json` naming fr-en, and the two files every studied
  folder holds, empty until the changes that fill them: `tags.tsv` (45) and `lexical.tsv` (48).
  `tables/fr-en/`: an empty `gloss.tsv` (the glosses are change 48), `NOTICE`, `manifest.json`
  (French's analyser version, read from lingua-core), `pin.json` and `README.md`.
- **Pinned sources.** `kaikki-French.jsonl`, derived from the English edition's dump and published
  under fr-en's release (510,058,226 B raw, 26,410,463 B as its zstd asset); UD French-GSD's
  training and development sections at a commit (`PINNED`), read for their counts only;
  wordfreq 3.1.1.
- **A measurement** (`scripts/lingua-data/measure/fr-ud.sh`): the real analyser over UD
  French-PUD, held to Spanish's gates (98.5 / 93.5 / 97), and over GSD's test section, reported;
  both fetched at a pinned commit, checked by sha256, never committed, never read by the reduction.
- **The pipeline knows fr-en**: `build.sh` keeps 60,000 lemmas for a French pair;
  `lingua-pack-update` offers fr-en and runs one French dispatch at a time; the monthly check and
  the reduce job take it in through `pairs`; the extension check builds its pack from the committed
  tables against its pin. `packs.json` is unchanged: no package carries fr-en.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *French's forms and frequencies* and *French forms are measured on
  held-out treebanks*. No requirement is modified, and none of `lingua-analysis`: this change reads
  change 38's catalogue, change 39's French and change 40's pre-pass as they define them, and
  archives after the three (`archiveAfter`).

## Impact

- **Products.** Cymbra Lingua's data pipeline only:
  - `scripts/lingua-data/` — *new*: `reduce-fr-en.py` and its tests, `tables/fr/`,
    `tables/fr-en/`, `measure/fr-ud.sh`, the French sections of `SOURCES.md`; *changed*:
    `pack_sources.py` (`DUMPS["fr-en"]`, `PINNED["fr-en"]`), `build.sh` (the lemma cut);
    *consumed*: `reduce_common.py` (unchanged, so no other pair's rule digest moves), change 38's
    catalogue and derivation.
  - `crates/lingua-pack` — *new*: the committed-tables test's French scenarios; *consumed*: the
    builder and `lingua-pack-measure`, unchanged but for the harness's doc line naming both
    languages' treebanks.
  - `.github/workflows/lingua-pack-update.yml` — the pair option and the concurrency group;
    `lingua-extension-check.yml` builds and reduces fr-en through the loops it already has.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the extension,
  the Apple host app and the agent plugin are untouched.
- **Sources and licences**: kaikki / English Wiktionary (CC BY-SA 4.0 + GFDL), wordfreq
  (CC BY-SA 4.0), UD French-GSD (CC BY-SA 4.0, counts only) — credited in fr-en's `NOTICE` and
  manifest; UD French-PUD (CC BY-SA 3.0), measured against and never shipped.
- **Release.** Silent. No package lists fr-en; no reader holds a French pack before change 52.
- **What does not move.** en-fr, es-fr, es-en and en-es — tables, pins, packs and goldens — byte for
  byte, and the French golden too: it runs over its fixture pack until change 48 (design D11).
- **Size.** 3.1 MB of tables committed. The pack they build (forms and ranks, nothing else) is
  1,239,104 B, against 1,308,123 B for Spanish's same two tables; each French pack carries it. The
  reviewers' source archive copies `scripts/lingua-data` whole and grows by the tables.
- **Not here.** The readings and the tag pool (45), the levels (46), the glosses, expressions and
  dictionary words (48), fr-es (49), the French rules of the analyser — elision, contractions,
  inversion, the cascade, NFC, names, function words (40, 41) —, expression keys such as
  `d'abord` (44).
- **Effort, against 8.5–13.5 ideal days**: 7–10.5 (design, *Effort*). The design names where the
  change splits if it runs over ≈ 10.
