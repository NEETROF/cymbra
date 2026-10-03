# Design — add-lingua-spanish-word-card

## Context

See proposal.md (Why). The card's grammar comes from `word_grammar` (`add-lingua-word-grammar`):
- the readings of the form as the card's dictionary form;
- the other dictionary forms it may be;
- the gloss grouped by part of speech, each group under a tag.

`reading/grammar-labels.ts` turns those tags into French:
- `readingName` names one reading, through a table of verb forms keyed by studied language, with
  English's only;
- `grammarLines` builds the card's lines;
- `senseHeading` names a group with its gender, when the tag carries one;
- `of` elides « de » before `[aeiou]`.

The card's content (`WordPopupContent`) carries no language: every card names English forms.

The es-fr pack's readings (`add-lingua-spanish-grammar-tables`) are UD tags such as
`VERB|Mood=Sub|Number=Sing|Person=3|Tense=Imp|VerbForm=Fin`. Spanish shares one form between
persons far more than English does:
- `hablaba`: 1st and 3rd person singular of the imperfect;
- `hable`: 1st and 3rd person of the present subjunctive, and the *usted* imperative.

Its sense runs come from the French Wiktionary and carry a part of speech only. A noun's gender is
in its readings (`casa`: `NOUN|Gender=Fem|Number=Sing`), not in its runs.

## Goals / Non-Goals

**Goals:**
- A Spanish form named in the French school terms a French reader learned Spanish with.
- A noun's gender on the card, and an adjective's or participle's agreement.
- French that reads right: articles, elision, one statement for a form of several persons.
- English's lines unchanged.

**Non-Goals:**
- Telling the *usted* imperative apart. It is a third person, as UD writes it.
- A conjugation table. The pack holds the readings of the forms it knows, not every paradigm.
- Choosing a reading from context.

## Decisions

### D1 — Spanish verb forms, as French schools name them

| Tag | Name |
|---|---|
| `Mood=Ind\|Tense=Pres` | présent de l'indicatif |
| `Mood=Ind\|Tense=Imp` | imparfait de l'indicatif |
| `Mood=Ind\|Tense=Past` | passé simple |
| `Mood=Ind\|Tense=Fut` | futur |
| `Mood=Cnd` | conditionnel |
| `Mood=Sub\|Tense=Pres` | présent du subjonctif |
| `Mood=Sub\|Tense=Imp` | imparfait du subjonctif |
| `Mood=Sub\|Tense=Fut` | futur du subjonctif |
| `Mood=Imp` | impératif |
| `VerbForm=Inf` | infinitif |
| `VerbForm=Ger` | gérondif |
| `VerbForm=Part` | participe passé, with its agreement unless masculine singular (« participe passé féminin ») |

The present and the imperfect name their mood, because the subjunctive has both too.

A finite form names its person and number: « 3e personne du singulier du passé simple ». The table
is keyed by studied language beside English's, as `add-lingua-word-grammar` D7 planned. English
keeps its names (prétérit, présent, forme en -ing).

### D2 — One statement for a form of several persons

The readings of one tense and one number merge their persons:
- `hablaba` reads « 1re et 3e personnes du singulier de l'imparfait de l'indicatif »;
- `hable` reads « 1re et 3e personnes du singulier du présent du subjonctif et 3e personne du
  singulier de l'impératif ».

Readings of several tenses are joined as they are today (« a, b et c »). English has no form of
several persons in its tables, so its lines do not move.

### D3 — Articles and elision

- **Articles.** A name carries its article: le, la or l’. « l’infinitif » and « l’imparfait de
  l’indicatif » take no space after the apostrophe.
- **« de » before a name**: « du » before a consonant, « de l’ » before a vowel:
  « du passé simple », « de l’imparfait », « de l’impératif ».
- **« de » before a studied word**: « d’ » before any vowel, accented ones included: « d’él »,
  « d’ir ». Not before « h », as French writes « la conjugaison de hablar ».

### D4 — Agreement, and nothing for the dictionary form

**Agreement.** A noun's, adjective's, determiner's or pronoun's reading names its gender and
number. Without a gender, it names its number alone, as English's plural does.

| Form | Reading | Line |
|---|---|---|
| `rápidas` | `ADJ\|Gender=Fem\|Number=Plur` | féminin pluriel de *rápido* |
| `rápida` | `ADJ\|Gender=Fem\|Number=Sing` | féminin singulier de *rápido* |
| `grandes` | `ADJ\|Number=Plur` | pluriel de *grande* |
| `esta` | `DET\|Gender=Fem\|Number=Sing` | féminin singulier de *este* |
| `casas` | `NOUN\|Gender=Fem\|Number=Plur` | féminin pluriel de *casa* |
| `leaves` (English) | `NOUN\|Number=Plur` | pluriel de *leaf*, as today |

**Nothing for the dictionary form.** A card opened on its own dictionary form shows a reading
only as another possibility (« peut aussi être… »). A reading that merely says what the dictionary
form is gives no line:
- the infinitive of `hablar`;
- the singular of a noun (`casa`);
- the masculine singular of an adjective (`rápido`), or its singular when it has one form for both
  genders.

`vino`, the noun, still names *venir*'s preterite as another dictionary form.

### D5 — The card knows its language

`WordPopupContent` gains `language`, the studied language of the document the word was met in. The
reading session sets it, as it knows the language it reads in (`add-lingua-language-routing`).
`renderGrammar` passes it to `grammarLines`. A card with no language reads as English, as every
card does today.

### D6 — A noun's gender in its sense runs

`senseHeading` already names « nom féminin » when a run's tag carries a gender
(`add-lingua-word-grammar`, its scenario of `leche`). The es-fr reducer gives a noun run the gender
the noun's readings carry:
- `casa	NOUN|Gender=Fem:1`;
- for a noun of both genders (`estudiante`), the plain `NOUN`.

The genders come from the English Wiktionary's `es-noun` heads, already read for the readings. The
runs' parts of speech come from the French Wiktionary, as before. `reduce_common.py` does not
change, so en-fr's rule set does not move. The tables are reduced again from the pinned sources,
and only `senses.tsv` changes.

## Risks / Trade-offs

- **Long lines for syncretic forms** → D2 merges persons. The card wraps, and the actions sit below
  the lines (`add-lingua-word-grammar` D6).
- **The plural repeats the heading's gender** (« nom féminin » and « féminin pluriel de *casa* »):
  it also covers a noun with a feminine of its own, `humanas` of `humano`, which a heading cannot
  say.
- **« de » before h is a choice** (D3). A Spanish h is silent, but « d’hablar » reads oddly to a
  French eye.
