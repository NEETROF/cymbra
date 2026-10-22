# add-lingua-italian-edition-rules — the Italian Wiktionary's rules in the gloss reducer

## Why

Change 35 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. en-it, fr-it and es-it are glossed from the Italian Wiktionary's English, French and Spanish
sections (changes 36, 47, 51). The shared reducer cleans a gloss by the rules of the edition that wrote
it (`generalise-lingua-gloss-reducer`): which senses only point at another word, which notes the edition
writes for its own readers, which senses name a letter, how a gloss of translation-table words is cased.
French, English and Spanish have their modules. Italian has none.

Measured on kaikki.org's Italian Wiktionary dump of 2026-10-21, derived by the repository's own
`derive`:

| Section | Senses | Headwords |
|---|---|---|
| English (en-it) | 41,754 | 25,700 |
| French (fr-it) | 35,026 | 26,108 |
| Spanish (es-it) | 8,660 | 6,861 |

- **The edition writes in lower case.** 93.5–95.3 % of its senses open in lower case. Its labels
  (obsolete, vulgar, heraldry) are in kaikki's tags, not in the text.
- **Its pointers are worded in Italian** (« plurale di », « variante statunitense di », « abbreviazione
  di »). In the English section, 868 untagged senses only point, 51 of them in the 5,000 commonest
  English lemmas.
- **It writes notes to its own readers.** « vedi italiano » means the Italian word is the same.
  « definizione mancante; se vuoi, aggiungila tu » is a placeholder.
- **Its heraldry crowds out meanings.** The French section glosses 95 of the 5,000 commonest French
  lemmas by heraldry alone (`le`, `vivre` « biscia… »).
- **Most of the French section is French verb forms.** 69.9 % of its senses are conjugated forms, 75.5 %
  inflections of any kind. The programme's expectation for fr-it, « as fr-es was, or better », holds only once both
  translation tables are read.

## What Changes

- **`scripts/lingua-data/reduce_edition_it.py`** (design D1): `IT = common.Edition(code="it", …)`
  with the edition's pointer wordings, notes and letters, and `capitalised=False`, plus a pre-pass
  `read_as_meanings`, in the Spanish and English editions' shape. Imported by en-it, fr-it and es-it
  alone; no committed pair moves.
- **Pointers** (D2, D3):
  - the Italian wordings, ending on the word they name, are pointers;
  - a pointer lends the word it names, not kaikki's last link;
  - an untagged sense with text after its target stays a meaning;
  - « contrazione di » never makes a pointer, and « abbreviazione di » only when its word lends a
    gloss: both say what the word is, as en-es reads « Contracción de » and « Abreviatura de ».
- **Tagged senses and nested glosses** (D4, D5): a sense kaikki tags `form-of` that is not worded as an
  inflection is a meaning when the word has no other; a nested gloss is read by its own text.
- **Notes and letters** (D6, D7):
  - the placeholder, « (inv) » and template residue go;
  - « vedi italiano » after a meaning goes;
  - « vedi italiano » alone reads as the headword itself (`panda` « panda »);
  - a letter's name goes.
- **Order and heraldry** (D8): obsolete, archaic and heraldic senses come after the others in their
  entry (`close` opens on « vicino »); a word glossed by heraldry alone takes no gloss from its
  definitions, so the translation tables gloss it (94 of 95 French, 31 of 32 Spanish, 8 of 13 English
  such lemmas at the 5,000).
- **Typography and case** (D9): gender marks out (« merda f » → « merda »), spaces, « … », straight
  quotes as « »; a table gloss keeps its words' case.
- **Measured effect** (D11): en-it 85.0 / 72.4 / 53.3 from the English section's definitions alone
  (84.9 / 72.4 / 53.3 before), 115 / 166 / 213 rows changed at the 5,000 / 10,000 / 20,000, a sample of
  50 rows 43 right, 5 partly, 2 wrong in the source itself.
- **Change 27's it-es** reads the Italian Wiktionary's Spanish section backwards. It drops the edition's
  notes itself, so it never imports this module, whose later edits would re-pin it (D12). Its design
  is amended here.
- **The programme's figures** for en-it, fr-it and es-it are corrected from the measurement (D12).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *The Italian Wiktionary's senses are read as meanings*. MODIFIED *A gloss is
  cleaned by the rules of the Wiktionary edition that wrote it* (held by no open change): the editions
  named gain Italian; a pair loads the rules of the one edition whose senses are its definitions, and
  no other's; a scenario holds that editing the Italian rules moves only the pairs glossed from them.

## Impact

- **Products.** Lingua's data only: `scripts/lingua-data/reduce_edition_it.py` (new),
  `test_reduce_editions.py`, `SOURCES.md` (its table of edition modules);
  `openspec/changes/add-lingua-pack-it-es/{design.md, specs, tasks.md}` (the notes it-es drops); `docs/lingua/italian-programme.md` (the measured figures). No table, pin, pack, golden,
  extension, site or backend change.
- **Prerequisites**: none beyond what main holds. The module is read once a pair glossed in Italian
  is reduced (changes 36, 47, 51); changes 36 and 47 register the English and French sections, and
  change 27a already registers the Spanish one.
- **Order**: before changes 36, 47 and 51. Changes 37, 48 and 52 refine through it.
- **Effort, against 2–4 ideal days**: 2–3 (design, *Effort*).
