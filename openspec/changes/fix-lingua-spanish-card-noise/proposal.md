# fix-lingua-spanish-card-noise — no surname and no letter on a word's card

## Why

The owner's dogfood (2026-10-04) showed two cards that read the dictionary wrong:
- **`miró`** was a word of its own, ranked « assez courant » for the painter. Its card said « peut
  aussi être la 3e personne du singulier du passé simple de *mirar* », where *mirar*'s preterite is
  what the reader meets. kaikki has an entry for the surname `Miró`, and UD Spanish-GSD meets the form
  6 times, all as that surname, never as the verb. GSD's counts gave the name the form.
- **`a`**'s card opened on « Première lettre et première voyelle de l'alphabet espagnol », before
  « À, au… ». `de` read « De; Nom de la lettre d ». The French Wiktionary's Spanish entries carry a
  letter section, which the gloss reduction read first.

The first is not alone. On the current tables, 387 forms that are also another word's map to a
proper name:
- `dolores` (*dolor*'s plural) to the given name;
- `cruces`, `nieves`, `jueces` and `rojas`;
- `tomé` and `seguí`, two verb forms.

A finding of the Spanish programme's dogfood, not a numbered change of
`docs/lingua/spanish-programme.md`.

## What Changes

- **A form that is a proper name and another word's keeps the commoner reading.** The form's
  frequency, met as the name or as the word, is weighed against the word's lemma, before GSD's counts
  are read. The verdicts:
  - `miró` goes to *mirar* (3.98 against 4.74);
  - `dolores` goes to *dolor*;
  - `argentina` stays the country (5.38 against *argentino*'s 4.83), and `parís` stays the city.

  118 forms now read as the word.
- **A letter's sense is no French gloss.** An entry of the French Wiktionary's `character` part of
  speech, and a sense naming a letter (« Nom de la lettre d. », « Lettre s. », « … lettre de
  l'alphabet … »), are left out. So `a` opens on « À », and `de` reads « De ».
- **The es-fr tables are reduced again** from the pinned snapshot. The gates still pass.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A Spanish form reads as the common word before a proper name*, and *A
  letter is no French gloss of a Spanish word*.

## Impact

- **Data.** `scripts/lingua-data/reduce-es-fr.py`, and every es-fr table re-reduced from the pinned
  snapshot; `pin.json` records the new rules and pack. `reduce_common.py` does not change, so en-fr
  does not move: the letters are taken out of the Spanish entries before the shared gloss reduction
  reads them.
- **Measured.**
  - PUD: words resolved 99.38 %, content words 95.79 % and auxiliaries 97.95 %, all unchanged.
  - Glosses: 22,755 lemmas glossed, against 22,823 before. The difference is lemmas glossed only by a
    letter's sense, and names that gave way to a word. The share of the 5,000 commonest lemmas
    glossed is 87.6 %, against 87.7 %.
- **Products.** Cymbra Lingua only, once the es-fr pack ships (`enable-lingua-spanish`). No
  extension, engine, server or proto change.
