# add-lingua-pack-lexical-layer — what a pack studies does not depend on how it glosses

## Why

A pack glossed in another native language must analyse its studied language exactly as the first
pack does ([language matrix programme](../../../docs/lingua/language-matrix-programme.md), change 5).
Today the native side of a pack still decides three studied-side facts.

- **Dictionary words.** The vocabulary estimate's universe and the English ladder's own figures
  count the ranked lemmas the pack *glosses*. Measured on the real tables with 70 % of the glosses
  kept:
  - English: the universe falls from 25,372 to 20,251 words, and B1's typical vocabulary from
    3,359 to 3,000.
  - Spanish: the universe falls from 22,755 to 18,371 words.
- **Spanish names.** A capitalised mid-sentence word is set aside as a name when the pack has no
  gloss for it.
  - A thinner gloss set flips 10–19 of 1,930 tokens and moves the percentage on 4–7 of 10 pages
    (Madrid and Guadalquivir become names).
  - A richer one flips others the other way: an English-glossed pack knows `augusto`.
- **Stored readings.** The tag pool mixes the readings' tags, which belong to the studied
  language, with the senses' tags, which belong to the native side. A sense tag one native uses and
  another does not reshuffles how every reading is stored.

The Spanish noun's gender, shown on the card, also rides on the native side's sense runs.

This change is the second of stage 1 and ships as a silent release: the en-fr and es-fr packs keep
their bytes, and both invariance baselines do not move.

## What Changes

- **A pack's dictionary words are its studied language's.**
  - They are the lemmas its studied language's reference pack glosses: en-fr for English, es-fr for
    Spanish.
  - A pack whose glosses differ names its dictionary words in an optional `lexical` table, built
    from a `lexical.tsv`.
  - A pack without the table reads its glossed lemmas as its dictionary words, which is exactly
    today's rule, so en-fr and es-fr carry no table and keep their bytes.
  - The vocabulary estimate, the ladder's own figures and the Spanish names rule read dictionary
    words, not glosses.
  - A core that predates the table ignores it and falls back to the glosses.
- **A builder that keeps the lexicon the studied language's.** When it writes a lexical table, it
  refuses a dictionary word or a glossed lemma the lexicon does not hold, so no native language's
  glosses can add a lemma.
- **A pinned tag pool.**
  - Packs built from committed tables order their tag pool as: the studied language's pinned pool
    (today's en-fr and es-fr pools, committed as `tags.tsv`), then any other reading tag, then the
    tags only senses carry.
  - Readings are then stored alike whatever the senses use, and en-fr and es-fr keep their pools.
  - Test packs built without a pin keep today's single sorted pool.
- **A noun's gender from its readings.**
  - The builder gives a noun's sense runs the gender its dictionary form is read with, when that is
    a single one. It refuses a run its readings contradict.
  - Measured on es-fr: all 13,435 noun runs are reproduced, so its bytes do not move.
- **A cross-native invariance test.** For English and for Spanish, a pack built from the first
  pack's studied tables, glossed in another native language with other glosses, answers every
  probe of the language's baseline alike once glosses and senses are removed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED.
  - *A pack's dictionary words do not depend on its glosses*;
  - *A form's readings do not depend on the pack's native language*;
  - *A noun's gender comes from its readings*.
- `lingua-analysis`:
  - MODIFIED — *A Spanish document's names are set aside*. Unheld, and its « the pack does not
    gloss » becomes « is not one of the pack's dictionary words ».
  - ADDED — *An analysis does not depend on the native language*.
- `lingua-knowledge-model`: ADDED — *A vocabulary size counts dictionary words*. No requirement
  specified the vocabulary estimate's universe until now.

No requirement this change modifies is held by an open change. *Versioned pack container* and
*Size budget* stay as they are:
- en-fr and es-fr never carry the table.
- The table is a bitset of about 5 KB for English and 7 KB for Spanish.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core`: the pack reader, its dictionary-word predicate, the names rule;
  - `crates/lingua-pack`: the lexical table, the pinned pool and the gender derivation;
  - `scripts/lingua-data`: the committed `tags.tsv` of en-fr and es-fr, kept by the build and
    update scripts;
  - the tests.

  No extension, agent, server or `.proto` change. ID, Music, Live, the back office and the site
  are untouched.
- **No pack byte moves.** No reducer changes, so no rule digest, `pack_version` or pin moves.
- **No analyser version moves.** The fallback is today's rule, so every analysis of the shipped
  packs is unchanged.
- **Release.** Silent, stage 1. The first pack carrying a lexical table is es-en (change 21).
- **Coverage.** The logic lives in `lingua-core` and `lingua-pack`, both host-tested.
