# Design — add-lingua-spanish-grammar-tables

## Context

See proposal.md (Why). `add-lingua-word-grammar` (archived) built the parts this change fills:
- **The vocabulary** (its D1): the 17 UPOS tags and a closed set of features. The set covers the
  Romance categories: the subjunctive imperfect, the conditional, the agreed participle, a noun's
  gender.
- **`grammar.tsv`** (its D4): `form<TAB>lemma<TAB>tag<TAB>other|-`, one line per reading.
- **The builder** (its D3) files each reading under its dictionary form. It also files a reading
  marked `other` as an "also" entry, under the dictionary form the analysis reads the form as.
  Readings whose dictionary form the lexicon does not hold are dropped. A tag outside the
  vocabulary fails the build.
- **`word_grammar`** (its D5) answers the card: the readings of the form as its dictionary form,
  and the other dictionary forms with theirs.

kaikki's extract of the English Wiktionary, Spanish section, gives:
- in a lemma's entry, every inflection with tags (`hablábamos`: first-person, imperfect,
  indicative, plural);
- in a noun's head template (`es-noun`), its gender: `m`, `f`, `mf`, `mfbysense`, `m-p`,
  `f-p`…;
- in a form's own entry, form-of senses with tags. Only these tag the pronominal forms
  (`azotarse`: infinitive, reflexive).

The es-fr tables hold 144,805 forms under 60,000 lemmas (`add-lingua-spanish-forms-tables`).

## Goals / Non-Goals

**Goals:**
- Readings for the forms the pack holds, in the vocabulary as it stands.
- A noun's gender, which French speakers get wrong (*la leche*).
- A homograph names its other dictionary form when the pack keeps it.

**Non-Goals:**
- The card's Spanish labels: tense names in French school terms, gender
  (`add-lingua-spanish-word-card`).
- Sense runs: they cover a gloss, and the glosses are `add-lingua-spanish-gloss-tables`.
- Readings of forms the table does not hold, which a conjugation table would show. No change
  builds one.
- Choosing a reading from context, as in `add-lingua-word-grammar`.
- A noun whose form reads as a verb (`era`, `son`, read as *ser*). One form maps to one lemma,
  and the pack keys a lemma by its own form, so that noun is not in the pack and the card cannot
  name it. Lifting that is a format change, the costed multi-lemma branch of the programme.

## Decisions

### D1 — Which forms get readings

The forms `forms.tsv` holds, and only under the lemmas the pack keeps. A card opens on a form the
analysis resolved through the table: a form outside it never reaches one. Every form of every kept
lemma would make roughly three times the readings, for a conjugation table nothing builds yet.

### D2 — Verb forms

| kaikki | UD |
|---|---|
| `indicative` + `present` / `imperfect` / `preterite` / `future` | `Mood=Ind`, `Tense=Pres` / `Imp` / `Past` / `Fut` |
| `subjunctive` + `present` / `imperfect` (both `-ra` and `imperfect-se`) / `future` | `Mood=Sub`, `Tense=Pres` / `Imp` / `Fut` |
| `conditional`, which kaikki also tags `indicative` | `Mood=Cnd`, no tense, as UD Spanish writes it |
| `imperative` | `Mood=Imp` |
| `first-person`… and `singular` / `plural` | `Person`, `Number`, with `VerbForm=Fin` |
| `infinitive`, `gerund` | `VerbForm=Inf`, `VerbForm=Ger` |
| `participle` + gender + number | `VerbForm=Part`, `Tense=Past`, `Gender`, `Number` |

Three rows of kaikki's tables are no reading of their own:
- **The negative imperative** (`no hables`) is the present subjunctive, which the table lists as
  such. Read as an imperative too, `hables` would show a mood it does not have.
- **The polite imperative** (`hable`, *usted*) carries `second-person-semantically` with
  `third-person`. It is grammatically a third person, as UD writes it, and the same reading as the
  third person.
- **The bare `participle past` row** repeats the masculine singular.

The voseo forms (`hablás`) are second person singular. Every verb is `VERB`: `AUX` is a role in a
sentence, which a card cannot see.

### D3 — Nouns, adjectives, determiners, pronouns

- **A noun's gender** comes from `es-noun`'s argument, or else from its senses' tags. The noun's
  own form reads `NOUN|Gender=…|Number=Sing` (`Plur` for a plural-only noun, `f-p`: `gafas`), and
  its plural `NOUN|Gender=…|Number=Plur`. A noun of both genders (`mf`, `mfbysense`: `estudiante`)
  has one reading per gender. A feminine kaikki lists (`humana`, of `humano`) reads with its own
  gender.
- **An adjective's own form** reads `ADJ|Gender=Masc|Number=Sing` when it has a feminine of its own
  (`rápido`), and `ADJ|Number=Sing` when one form serves both (`grande`). Its forms read by their
  tags: naming both genders (`grandes`) means no gender. A superlative reads `Degree=Sup`.
- **A determiner's, a pronoun's or a numeral's forms** read their gender and number (`esta`:
  `DET|Gender=Fem|Number=Sing`). There is no `PronType`: kaikki does not state it per form.

### D4 — Where a reading comes from

- A lemma's table first.
- A form's own entry, through its form-of senses, only for the pairs no table lists. Such a sense
  knows no gender: `casas` reads "plural" from its own entry and "feminine plural" from `casa`'s
  table, so the table wins. This adds the pronominal forms (`azotarse`, `criándose`), which
  combine a verb with `se` but which kaikki also lists as plain forms.
- **Never a reading**:
  - a combined form's sense, which names the pronoun (the enclitic rule's, as in the forms
    table);
  - the table's bookkeeping and the headword repeated (`canonical`);
  - rows kaikki could not parse or marks as misspellings (`error-unrecognized-form`,
    `misspelling`, `pronunciation-spelling`).

### D5 — Every relation is believable

`add-lingua-word-grammar` marks `other` only on relations its reducer believes, because ESDB and
the French Wiktionary's form links mix in noise (`uses` is no form of `us`). kaikki's tables are
structured paradigms, so every relation stands. Each reading of another kept lemma than the one its
form maps to is marked `other`:
- `fue` names *ir*;
- `vino` (the noun) names *venir*;
- `casas` names *casar*;
- `fuera` names *ser* and *ir*;
- `bajo` names *bajar*.

The marks number 8,776.

### D6 — The table's size, in git

`grammar.tsv` is 9.2 MB of text: 149,279 lines such as
`hablábamos<TAB>hablar<TAB>VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin<TAB>-`. Compressed it is
0.9 MB, and the pack grows by 199 KB, since a paradigm stores suffix edits that zstd folds.

The es-fr tables then hold 12.6 MB of text, past the ≈10 MB that the programme's open question
named (commit them, attach a snapshot, or keep attested forms only). Attested forms are already the
rule (D1). They are committed:
- git stores them compressed, about 1.9 MB;
- an update's diff stays readable line by line;
- a release still reads nothing but the tables.

*Rejected — a compact table* (one line per form, tags pooled). It changes the builder's input,
which en-fr shares, to save what git already saves.

### D7 — The test

`crates/lingua-pack/tests/es_fr_grammar.rs` builds the es-fr pack from the committed tables and
asks `word_grammar` what the card would show:
- `casa` and `casas`: their gender, `casas` naming *casar*;
- `hablábamos`, `escrita`, `rápidas`: what each form is;
- `fue` naming *ir*, and `vino` naming *venir*.

## Risks / Trade-offs

- **kaikki's tags drift** → the snapshot pins them. An unknown combination yields no reading
  rather than a wrong one (`verb_features` returns nothing), and a tag outside the vocabulary
  fails the build.
- **Labels missing until change 24** → no package ships the es-fr pack before
  `enable-lingua-spanish`, which comes after the word card.
- **A large text table** → D6. If git size ever matters, the compact table is the way out; the
  pack does not change.
