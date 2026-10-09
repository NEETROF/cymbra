# add-lingua-french-grammar-tables — French's word grammar: each form's readings, and the tag pool pinned

## Why

Change 45 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en, fr-es). Change 43 (`add-lingua-french-forms-tables`) reduces
French's forms and ranks into `tables/fr/` from the English Wiktionary's French section, and commits
`tables/fr/tags.tsv` empty « until change 45 ». No French form carries a reading yet, so a card on
`parle` could not say that it is the present of *parler*, nor a card on `maisons` that *maison* is
feminine.

The word card reads a form's readings as Universal Dependencies tags (`add-lingua-word-grammar`,
archived): a Romance pack fills them with no change to the container, the core or the card's code,
as Spanish's did (`add-lingua-spanish-grammar-tables`, archived). The source tags every French
form: 7,395 verbs carry a conjugation table (the study's « ≈ 7,380 fully tagged verbs »), 7,058 of
them every simple finite cell, and a noun's head gives its gender. What French's dictionary does
differently is what this design measured: the passé simple is tagged `historic past`, a verb's table
lists one past participle and hangs the agreed ones under the participle's own entry, a pronominal
verb's forms are written with their pronoun (`nous évanouissons`), an invariable noun lists no
plural.

Measured on the French section derived on 2026-10-08 and change 43's prototype tables (124,040
forms, 60,000 lemmas):
- 125,887 readings of 88,620 forms, in 78 tags, 7,970 of them naming another kept word;
- on UD French-PUD, which the reduction never reads, 99.94 % of the finite verbs and 99.09 % of the
  nouns whose lemma the tables give carry a reading, and 99.60 % and 99.84 % of those agree with the
  treebank's features;
- a pack 155,620 B larger: 1,394,724 B.

Two decisions of the owner bind these readings (2026-10-09):
- **M8, one form, one lemma.** A form's readings are filed under its one lemma; a reading of another
  word the pack keeps is only named on the card, as English's `leaves` names *leaf* and Spanish's
  `vino` *venir*, and counts nothing. A noun whose own form reads as a verb (`porte`) left the
  tables with change 43, and no reading of it is kept: 543 nouns, measured.
- **M21, « moods merged on five-reading forms ».** Change 40 (its D1) placed the merge on the word
  card, change 51. The card can only merge what the readings say, so this change gives it what it
  needs: `parle`'s five readings — present indicative and present subjunctive of the first and third
  persons singular, imperative of the second — stored unmerged, the two presents with a tense and the
  imperative without one, all under the form's own lemma. 2,142 forms have exactly these five.

The architecture pins each studied language's tag pool so that a new native's sense tags cannot
reshuffle the paradigms: this change writes French's pin.

## What Changes

- **`reduce-fr-en.py` writes `grammar.tsv`** (`form<TAB>lemma<TAB>tag<TAB>other|-`), in the same
  pass as the forms, after they are chosen; no shared module is edited:
  - the readings of the forms `tables/fr/forms.tsv` holds, under the lemmas `freq.tsv` ranks;
  - a verb form's mood, tense, person and number — the passé simple as `Tense=Past`, the
    conditional and the imperative with no tense, as Spanish writes them —, its infinitive, its
    present participle (`VerbForm=Part|Tense=Pres`, as UD French writes it), its past participle
    with its gender and number; a pronominal verb's forms without their pronoun; a compound tense no
    reading;
  - a noun's gender on its own form and its plural, and both numbers on a noun spelled alike in both
    (`temps`); an adjective's, determiner's (articles included), pronoun's or numeral's agreement;
  - a form of a form read along one part of speech with its own agreement (`dirigée`, the feminine
    of the participle `dirigé`: *diriger*'s feminine past participle), as change 43 maps the form;
  - no reading from what the dictionary marks doubtful, regional or of a register, from a
    capitalised headword (`CE`), from an entry that is only an alternative form or a neologism
    (`estre`, archaic spelling of *être*), from a letter's name toward its plural, or from a feminine
    noun's masculine (`déesse` → `dieu`);
  - `other` on a reading of another ranked lemma (`fils`: *fil*), only toward a word the dictionary
    holds as an entry that is not only regional.
- **The committed tables**: `tables/fr/grammar.tsv`, and **`tables/fr/tags.tsv`, French's pinned
  tag pool** — the 78 tags its readings carry, in byte order, written once by a person; fr-en's
  manifest and pin re-recorded. `forms.tsv`, `freq.tsv`, `lexical.tsv` and `studied.json` do not
  move.
- **A measurement** (`measure/fr_readings.py`, run by change 43's `fr-ud.sh`): per part of speech,
  the share of UD French-PUD's and GSD test's words that carry a reading and the share whose
  treebank features are among them — reported, never gating.
- **Tests**: `crates/lingua-pack/tests/fr_en_grammar.rs` asks `word_grammar` what the card would
  show; `committed_tables.rs` holds the pin to the readings' tags.
- **README and `SOURCES.md`** name the table and the pin.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *French's word grammar*, *French's pinned tag pool* and *French
  readings are measured on held-out treebanks*. No requirement is modified: this change reads change
  43's forms and folder, change 38's source and change 39's French baseline as they define them, and
  archives after the three (`archiveAfter`).

## Impact

- **Products.** Cymbra Lingua's data pipeline only:
  - `scripts/lingua-data/` — *new*: `tables/fr/grammar.tsv`, `tables/fr/tags.tsv`,
    `measure/fr_readings.py`; *changed*: `reduce-fr-en.py` and its tests, `tables/fr-en/`
    (`manifest.json`, `pin.json`, `README.md`), `measure/fr-ud.sh`, `SOURCES.md`; *consumed*:
    `reduce_common.py`, unchanged.
  - `crates/lingua-pack` — *new*: `tests/fr_en_grammar.rs`; *changed*: `tests/committed_tables.rs`;
    *consumed*: the builder, the vocabulary and `word_grammar`, unchanged.

  ID, Music, Live, the back office, the site, the backend, lingua-core, lingua-wasm, the extension,
  the Apple host app and the agent plugin are untouched. The card's French — the names of French
  tenses in English and Spanish, the merge of moods, the present participle — is change 51's.
- **Sources and licences**: none new — kaikki / English Wiktionary (CC BY-SA 4.0 + GFDL), credited by
  change 43; UD French-PUD (CC BY-SA 3.0) and GSD's test section (CC BY-SA 4.0), measured against,
  never shipped or read by the reduction.
- **Release.** Silent: no package lists fr-en, no reader holds a French pack before change 52.
- **What does not move.** en-fr, es-fr, es-en and en-es — tables, pins, packs, goldens — byte for
  byte; the French golden too, over its fixture until change 48, whose switch will show these
  readings (design D12).
- **Size.** `grammar.tsv` is 8.0 MB of text, 0.76 MB compressed; `tables/fr/` holds 11.1 MB
  (1.6 MB compressed), committed as Spanish's were. The fr-en pack grows by 155,620 B to
  1,394,724 B.
- **Effort, against 3.5–6 ideal days**: 3.5–5.5 (design, *Effort*).
