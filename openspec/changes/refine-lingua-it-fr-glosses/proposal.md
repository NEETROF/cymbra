# refine-lingua-it-fr-glosses — it-fr's glosses read as Italian meanings, and Italian's forms as its words, before French speakers read Italian

## Why

Change 17 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Change 16
(`add-lingua-pack-it-fr`, #1054) committed it-fr's glosses on 2026-10-24 — 32,691 rows, coverage
**89.2 / 81.8 / 73.3 %** against the floor 87.2 / 79.8 / 71.3 — and listed what still reads wrong
(its design's *Known data defects*) for this change, « measured on the committed tables »; the owner
approved change 16 with them (« `che`, `della`, `dirotto` « Écanguer », 458 unglossed lemmas… — are
change 17's »). Changes 9, 11, 12 and 13 left this change the forms and readings they found wrong
too. Change 20 (`enable-lingua-italian`) lists it-fr only once this change has merged (its D2).

Measured on the committed tables, and on the pinned sources where a defect is the source's (design,
*Measured*; criteria written before the figures):
- **The wrong sense first.** The shared round-robin opens a row on the section's first entry:
  `fra` (rank 193) « Frère; Entre », `spesso` (188) « Gros, épais; …souvent », `gente` (128)
  « Gentil, fin; Gens », `fino`, `pure`, and `loro` (41) on the courtesy pronoun `Loro`. 1,580 rows
  hold two parts of speech or more, 191 of the 1,000 commonest.
- **Names and examples in common words.** `bibbia`, `rinascimento`, `corano` are read as proper nouns
  alone, so they lose their level; `io` (30) ends on Io, Inachus's daughter, and holds three example
  sentences with their translations; `qui` ends on « Riri »; 31 rows keep a commune worded otherwise
  (`vittoria` « …, commune et ville… », `ro`).
- **Pointers.** 56 ranked lemmas stay unglossed behind a pointer the shared rules cannot follow
  (`lettere` « Pluriel de lettera », `paro` « Variante de paio », `pochissimo` « Superlatif de poco :
  « très peu » »); `gemelli` is « Gémeaux » alone; 1,103 verb forms read their verb's infinitive
  (`assicurato` « Assurer »), which the word card already explains.
- **Articles and slips.** `della` (12) « De », `del` (9) « Du. Note d'usage : il s'utilise devant… »,
  `che` (3) « Quoi; Quel » with no « Que », `volere` holding its example in three senses, `dirotto`
  « Écanguer ».
- **Forms and readings.** `sto` (136) and `vado` (451) are lemmas of their own, through an ellipsis
  and an obsolete noun; `viti` reads as *vita*, `signorini` as *signorina*, `avi` and `ava` as
  *avere*, `usa` as *uso*, `signor` unglossed; `dati` (« i dati ») as *dare* — PUD's ×10 —, `lega`
  as *legare*; `fate` as a subjunctive of *fare*.
- **456 levelled lemmas** lost their level for want of a gloss: 14 are reachable here, 442 are the
  sources' gaps (`entro`, `supporto`, `verifica`, `web`, `ok`) or point at a word with no gloss.

## What Changes

Every rule is it-fr's own, in `reduce-it-fr.py`, which no other pair loads, with fr-en's treebank
rule imported as fr-es imports it: no shared, French-edition or treebank module is edited, so only
it-fr re-pins (design D1).

- **The order of a row** (D2): a function word's row opens on the part of speech UD Italian-MarkIT
  reads it as at least ten times and twice as often as the section's first — fr-en's rule, at two —:
  10 rows, all of the 1,000 (`fra` « Entre, durant; Frère », `spesso`, `proprio`, `attraverso`,
  `molto`, `solo`). A reviewed table, keyed by case and part of speech, orders what MarkIT is too thin
  for or reads wrongly: `gente`, `fino`, `pure`, `loro`, and `come` held at « Comme ».
- **Names, notes and examples** (D3, D4, D8, D9, D10): a name's sense a common entry repeats is the
  common entry's (`bibbia`, `rinascimento`, `corano` take their level back); a pronoun's or an
  adverb's row holds no name (`io`, `qui`, `qua`, `oltre`, `anzi`); a sense naming its place a commune
  is a commune's note, provincial capitals, chefs-lieux and the owner's two cities (`piacenza`,
  `cesena`) spared (14 rows); an article's usage note
  goes (6 rows); an Italian example joined to its French by a dash goes (`io`'s three).
- **Pointers** (D5, D6, D8): an untagged « Pluriel de X », an alternative form not marked obsolete and
  a superlative's quoted meaning are read (56 lemmas gained, 7 of the 5,000; 27 rows read the section
  instead of a table); a noun's pointer lends beside a name or an archaic sense (`gemelli`, `spezie`,
  `arme`); the `di` family's links and « souvent traduit par X » read as glosses (`della` « De la »,
  `degli` « Des »). A verb form's borrowed gloss is left as it is, the owner's choice (D7).
- **Corrections** (D10): a reviewed `CORRECTIONS` list, fr-es's, for `che` (« Qui, que; Quoi; Que;
  Quel »), `volere`'s example, `dirotto` (« À verse (dans a dirotto) ») and `dati` (« Données »); three split words that are
  no Italian (`c'hwerty`) offered no more.
- **Italian's forms and readings** (D11, D12): a verb's finite form whose own entry holds no meaning is
  read as its verb (32 lemmas: `sto` → *stare*, `vado` → *andare*, `sapete`), never a participle, a
  contraction or another word's inflection (`dee`, `cogli` keep theirs); no form of a form through a
  plural (`viti`), from a feminine noun's masculine row (`signorini` → *signorino*) or from the Roman
  dialect's rows another word claims (`avi`, `ava` → *avo*); `signor` and `od` read as *signore* and
  *o*; `usa` → *usare*; `fate` reads no subjunctive. The owner's rows (2026-10-25): `lega` → *lega*,
  `prova` → *prova*, and `dati` → *dati*, the plural noun as its own lemma glossed « Données » (a row
  cannot read it as *dato* while `dato` stays *dare*); `dato` stays *dare*, « mi ha dato » being its
  very common participle.
- **Measured and left** (D13): the sources' gaps; `Rossi`, `Bianchi`, `Conti`, `Bari` (French's
  `Lot`); `Esposito` after « Dott. », « la Nato », « tra USA e NATO » and « dai retta », Italian's
  analysis's, recorded in the programme's backlog; the `di` retry's partitives and its 30 literal
  phrases, which stay offered; species names in common rows.
- **One re-reduction** (D14): the owner's dispatch (`lingua-pack-update`, mode `reduce`) at it-fr's
  pin, every moved line of `forms.tsv`, `freq.tsv` and `grammar.tsv` listed by its rule; coverage
  about **89.3 / 82.0 / 73.4 %** before the forms rules move the ranks; PUD 99.26 / 91.35 / 99.90
  expected with the four rows `signor`, `lega`, `prova`, `dati`; the six other pairs byte for byte; `it-fr.golden`
  and change 18's card re-blessed once, every moved probe read (D15).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *it-fr's glosses read an Italian word's meanings in the order Italian uses them*, *it-fr
    reads a pointer as its word's meaning where the section names one*, *it-fr's glosses hold no
    name, usage note or example a common word does not need, and no split word that is no Italian*,
    *it-fr corrects, by name, the slips its sources write* and *Italian's readings set aside, by name,
    a reading the section's tables give a form wrongly*.
  - MODIFIED, each copied whole from the open change that holds it, the clauses this change edits
    named in the pull request: *Italian is glossed in French from the French Wiktionary's Italian
    section and the translation tables* and *it-fr is committed at its studied tables' snapshot, held
    to a coverage floor, and the Italian baseline runs on it* (`add-lingua-pack-it-fr`: a proper noun's
    sense no longer stays beside every common word, a split word that is no Italian is no expression,
    the forms, ranks and readings may move with their lines listed); *Italian's forms and frequencies*
    (`add-lingua-italian-forms-tables`: the own-entry rule, the chain through a plural, three
    exclusions, the apocope rows); *Italian's acronyms are a reviewed table, glossed through their
    entry's expansions or synonyms* (`keep-lingua-italian-acronyms-and-names`: `usa` reads as
    *usare*). Change 13's *Italian's word grammar* and change 14's levels are read as written.
  - This change archives after those five changes (`archiveAfter`); change 20, which MODIFIES
    *Italian's forms and frequencies* too, archives after it and re-copies its block (tasks 5.5).
- `lingua-analysis`: none. The analysis reads the new forms and dictionary words as written.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-it-fr.py` (the pre-pass, its post-passes, the forms
    rules, `OVERRIDES`, `APOCOPES`, `FIRST_RUN`, `CORRECTIONS`, the readings' list),
    `test_reduce_it_fr.py`, `test_reduce_editions.py`, `tables/it/` (`forms.tsv`, `freq.tsv`,
    `grammar.tsv`, `lexical.tsv`, `level.tsv`), `tables/it-fr/` (`gloss.tsv`, `senses.tsv`, `mwe.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_fr.py`, `reduce_french_treebank.py`, `reduce_italian_treebank.py`,
    `gloss_coverage.py`, `measure/it-ud.sh`, unchanged.
  - `crates/lingua-wasm/tests` — *changed*: `baseline/it-fr.golden` re-blessed; `italian_baseline.rs`
    only where a probe it asserts moves. `crates/lingua-pack/tests/` (`committed_tables.rs`,
    `it_fr_grammar.rs`) against the new pin.
  - `apps/lingua-extension/test/baseline/word-card-it-fr.txt` (change 18's) re-blessed once.
  - ID, Music, Live, the back office, the site, the backend, lingua-core and the engine, the
    extension's source, the Apple host app and the agent plugin are untouched.
- **What does not move.** en-fr, es-fr, es-en, en-es, fr-en and fr-es — tables, pins, packs, goldens
  and snapshots — byte for byte; Italian's analyser version.
- **Release.** Silent: no package lists it-fr before change 20.
- **Order.** After changes 9, 11, 13, 14 and 16 (on `main`, their requirements still open); before
  change 20; either side of change 18.
- **The owner's**: five questions and one consequence of the fifth, all answered on 2026-10-25
  (tasks 0.1–0.6); the golden's re-bless approved on its listed probes (task 4.1).
- **Effort**: 3–4.5 ideal days (design, *Effort*), against the programme's 2–3.5.
