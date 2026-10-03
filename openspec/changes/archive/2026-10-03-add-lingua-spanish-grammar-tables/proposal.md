# add-lingua-spanish-grammar-tables — the Spanish pack's word grammar

## Why

The word card says what a form is, and which other dictionary forms it may be. The archived
`add-lingua-word-grammar` built that for Romance packs:
- a vocabulary of Universal Dependencies tags;
- readings filed by dictionary form, with "also" entries for homographs;
- `word_grammar` in the core.

A Romance pack fills it with no change to the container, the core or the card's code.

The es-fr pack (`add-lingua-spanish-forms-tables`) carries no grammar table yet, so a Spanish card
would say nothing of what `hablábamos` or `casas` is. kaikki tags every Spanish form: mood, tense,
person and number, or gender and number. Its noun heads also give a noun's gender. The programme
counted ≈435,000 readings, 75 tags and a gender on 99.5 % of nouns, over every form
(`docs/lingua/spanish-programme.md`).

Measured on the pinned 2026-10-03 snapshot, over the forms the pack holds:
- 149,279 readings of 111,945 of its 144,805 forms, in 88 tags;
- a gender on 99.7 % of its nouns;
- a pack 199,380 B larger: 1,507,108 B, 28.7 % of the budget.

This is change 21, in G1, the internal Spanish build. No package ships the es-fr pack yet.

## What Changes

- **`reduce-es-fr.py` writes `grammar.tsv`** in the format `add-lingua-word-grammar` set
  (`form<TAB>lemma<TAB>tag<TAB>other|-`):
  - the readings of the forms `forms.tsv` holds, under the lemmas the pack keeps;
  - read from kaikki's tagged inflection tables, and from a form's own entry for the pairs no
    table lists (the pronominal forms, `azotarse`);
  - a verb form: its mood, tense, person and number, or its infinitive, gerund or agreed
    participle;
  - a noun: its gender, on its own form and on its plural;
  - an adjective, a determiner or a pronoun: its agreement;
  - no reading for a verb with its clitics;
  - a reading of another kept lemma than the form's own is marked `other`, so the card names
    it: `vino` is also *venir*'s preterite.
- **The committed table**: `grammar.tsv`, with `pin.json` re-recorded. `forms.tsv` and `freq.tsv`
  do not move.
- **A test** builds the es-fr pack from the committed tables and asks the card's grammar.
- **README and `SOURCES.md`** name the table.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The Spanish pack's word grammar*.

## Impact

- **Products.** Cymbra Lingua's data pipeline:
  - `scripts/lingua-data/` (the reducer, the tables, the README, `SOURCES.md`);
  - `crates/lingua-pack` (the test).

  No core change: the container, the vocabulary and `word_grammar` are used as
  `add-lingua-word-grammar` left them. The card's Spanish labels (tense names in French school
  terms, gender) are `add-lingua-spanish-word-card`. No package carries the es-fr pack until
  `enable-lingua-spanish`. The extension, the server, the protos, ID, Music, Live, the back office
  and the site are not affected.
- **Size.** The es-fr tables become 12.6 MB of text, 1.9 MB compressed. That passes the ≈10 MB the
  programme set for its open question: commit them, attach a snapshot, or keep attested forms only.
  The design commits them, attested forms only, as the programme preferred (D6).
- **Sources and licences**: kaikki / English Wiktionary, CC BY-SA 4.0 + GFDL, already credited.
- **Release.** G1, internal: nothing ships.
