# add-lingua-spanish-word-card — a Spanish card names its forms as French schools do

## Why

The es-fr pack now carries readings for 111,945 forms, and a noun's gender
(`add-lingua-spanish-grammar-tables`). The card cannot say any of it yet:
- its labels (`reading/grammar-labels.ts`) name English's verb forms only, as
  `add-lingua-word-grammar` planned (« the verb-form names are keyed by studied language, and only
  the English table ships now »);
- it does not know which language its word is in;
- it names no adjective's or participle's agreement;
- it elides « de » only before an unaccented vowel.

A French reader learning Spanish knows the tenses by their French school names: passé simple,
imparfait du subjonctif, impératif. The gender of a noun is what French speakers get wrong
(*la leche*).

This is change 24, in G1, the internal Spanish build. No package ships the es-fr pack yet.

## What Changes

- **Spanish verb forms in French school terms**:
  - présent and imparfait de l'indicatif, passé simple, futur, conditionnel;
  - présent, imparfait and futur du subjonctif, impératif;
  - infinitif, gérondif, participe passé with its agreement.

  A form of several persons names them once: `hablaba` is « 1re et 3e personnes du singulier de
  l'imparfait de l'indicatif de *hablar* ».
- **Agreement**: an adjective's, determiner's or pronoun's form names its gender and number
  (`rápidas`: « féminin pluriel de *rápido* »).
- **Nothing to say for the dictionary form itself**: no line for an infinitive's own card, a
  noun's singular or an adjective's masculine singular.
- **French elision**: « de l'imparfait », « l'infinitif », « d'él » — every vowel, accented ones
  included.
- **The card knows its language**: the word card's content carries the document's studied
  language, and the labels name that language's forms.
- **A noun's gender in the sense headings**: the es-fr reducer writes the gender kaikki gives a
  noun into its sense runs (`NOUN|Gender=Fem`), so the heading reads « nom féminin ». A noun of
  both genders keeps « nom ».
- **No letter's name in the readings of its plural** (found dogfooding, 2026-10-04): the card of
  `Es` (*ser*) said « peut aussi être le féminin pluriel d'e », the plural of the letter E. A
  letter's name keeps its own form, but none of its plurals is a reading.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *A Spanish card names its forms as French schools do*.
- `lingua-data-packs`: ADDED — *The Spanish pack's sense runs carry a noun's gender*, and *A
  letter's name gives no reading of its plural*.

## Impact

- **Products.** Cymbra Lingua:
  - `apps/lingua-extension`: `reading/grammar-labels.ts`, the word card's content and its
    rendering;
  - `scripts/lingua-data/reduce-es-fr.py` and the es-fr `senses.tsv` and `grammar.tsv`, re-reduced
    from the pinned sources.

  English's lines read as before; the English baseline and the en-fr pack do not move. No package
  carries the es-fr pack until `enable-lingua-spanish`.
- **Sources and licences**: none new.
- **Release.** G1, internal: nothing ships.
