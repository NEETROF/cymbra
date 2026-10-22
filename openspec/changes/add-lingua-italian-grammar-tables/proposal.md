# add-lingua-italian-grammar-tables — Italian's word grammar and its pinned tag pool

## Why

Change 13 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Change 9
reduces Italian's forms and ranks into `tables/it/` from the English Wiktionary's Italian section
and pins one placeholder tag (`NOUN`) in `tables/it/tags.tsv`, so the split container has a pool.
No Italian form carries a reading yet: a card on `parlavo` cannot say it is *parlare*'s imperfect,
first person singular, nor a card on `case` that *casa* is feminine.

The word card reads a form's readings as Universal Dependencies tags (`add-lingua-word-grammar`,
archived); a Romance pack fills them with no change to the container, the core or the card's code,
as Spanish's did (`add-lingua-spanish-grammar-tables`, archived) and French's does (matrix change
45, `add-lingua-french-grammar-tables`). This change is French's for Italian.

Measured on the Italian section (`kaikki-Italian.jsonl`, 770,859,636 B, the extract change 9
reduces), with the stress marks dropped as change 9's amendment reads table forms:
- **13,382 verbs carry a conjugation table** (twenty tagged rows or more) — 441,628 form–lemma
  pairs over every cell, in 49 tags: 378,193 pairs with one reading, 40,673 with two, 14,257 with
  three, 8,385 with four (`parli`: present indicative second person singular and present
  subjunctive first, second and third), 20,475 reading both the present indicative and the present
  subjunctive. The cut to change 9's 60,000 lemmas and attested forms keeps a fraction of them, as
  French's 35 %.
- The tables tag the **passato remoto** `historic past`, the **imperfetto** `imperfect`, the
  conditional, the imperative, the **gerund** (13,609 rows) and the present participle (13,609).
  Two families of rows give no reading of their own: the **negative imperative** (87,141 rows with
  the formal ones below; the second person singular is the infinitive, « non parlare », the plurals
  repeat the affirmative), and the **formal imperative** (`Lei`/`Loro`: the present subjunctive's
  third persons, already read as such). A defective verb's missing cells are written `-`: no form.
- **Nouns**: `it-noun` heads give the gender — masculine 36,150, feminine 30,150, by sense 3,162,
  plural-only 321, both 113 —, and 8,915 heads give no plural (invariable: `città`, `re`, `crisi`),
  read in both numbers as Spanish's `crisis` and French's `temps`.
- **Adjectives**: four forms (`rosso`, `rossa`, `rossi`, `rosse`; 18,132 feminine plurals) or two
  (`grande`, `grandi`; 7,648 plural rows with no gender).

## What Changes

- **`reduce-it-fr.py` writes `grammar.tsv`** (`form<TAB>lemma<TAB>tag<TAB>other|-`), in the same
  pass as the forms, after they are chosen; no shared module is edited (design D1):
  - the readings of the forms `tables/it/forms.tsv` holds, under the lemmas `freq.tsv` ranks (D2);
  - a verb form's mood, tense, person and number — the passato remoto as `Tense=Past`, the
    imperfetto as `Tense=Imp`, the conditional and the imperative with no tense, as Spanish and
    French write them —, its infinitive, its gerund (`VerbForm=Ger`), its present participle, its
    past participle with its gender and number; a reflexive verb's rows without their pronoun,
    under the lemma change 9 files them under; no reading from a negative or a formal imperative
    row, nor from a compound tense (D3);
  - a noun's gender on its own form and its plural, both numbers on an invariable noun; an
    adjective's, a determiner's (the articles among them), a pronoun's or a numeral's gender and
    number (D4);
  - a form of a form along one part of speech with its own agreement (`parlata`, the participle's
    feminine, when change 9 files it under *parlare*), as change 9 maps the form; a reading's part
    of speech one the dictionary holds its lemma as (D5);
  - no reading from what the dictionary marks doubtful, regional or of a register, from a
    capitalised headword, from an entry that is only an alternative form, from a feminine noun's
    masculine (`attrice` → `attore`), or toward a word an override row of change 9 sets aside (D5);
  - `other` on a reading of another ranked lemma, only toward an entry of the dictionary that is not
    only regional (D6).
- **The readings are stored unmerged** (D7): `parli` carries its four readings, `parla` its two
  (present indicative third singular, imperative second singular), so that the word card (change
  18) can merge the moods as French's change 51 does — the programme's M21 read for Italian.
- **The committed tables**: `tables/it/grammar.tsv`, and **`tables/it/tags.tsv`, Italian's pinned
  tag pool**, replacing change 9's placeholder `NOUN` with the readings' tags in byte order —
  nothing holds the placeholder's index, no Italian pack having shipped (D8); it-fr's manifest and
  pin re-recorded, its base and layer with them.
- **A measurement** (`measure/it_readings.py`, run by change 9's `it-ud.sh`): per part of speech,
  the share of UD Italian-PUD's and MarkIT's test section's words that carry a reading and the
  share whose treebank part of speech and features are among them — reported, never gating (D9).
- **Tests**: `crates/lingua-pack/tests/it_fr_grammar.rs` asks `word_grammar` what the card would
  show over the it-fr pack built from the committed tables; `committed_tables.rs` holds the pin to
  the readings' tags.
- **No golden moves** (D10): the Italian baseline runs over its fixture until change 16, and the
  fixture keeps no grammar; English, Spanish, French and their packs do not move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *Italian's word grammar*, *Italian's pinned tag pool* and *Italian
  readings are measured on held-out treebanks*. None MODIFIED: change 9's scenario that `tags.tsv`
  holds the one canonical tag reads « until change 13's pool », as its requirement says. This
  change archives after changes 5 and 9, after the split (3, 4) whose base carries the pool, and
  after French's grammar change, whose requirements it transposes.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `scripts/lingua-data/reduce-it-fr.py` (the readings), `test_reduce_it_fr.py`;
  `measure/it_readings.py` (new), `measure/it-ud.sh` (runs it).
- `scripts/lingua-data/tables/it/{grammar.tsv (new), tags.tsv}`, `tables/it-fr/{manifest.json,
  pin.json, README.md}`; `SOURCES.md`.
- `crates/lingua-pack/tests/{it_fr_grammar.rs (new), committed_tables.rs}`.
- Untouched: lingua-core, lingua-wasm, the builder, the extension, every golden, the six pairs'
  tables and pins, `tables/it/{forms.tsv, freq.tsv, lexical.tsv, studied.json}`.
- **Effort, against 3–5 ideal days**: 3.5–5 (design, *Effort*).
