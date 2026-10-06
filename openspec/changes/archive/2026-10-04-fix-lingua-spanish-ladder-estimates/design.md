# Design — fix-lingua-spanish-ladder-estimates

## Context

See proposal.md (Why). `LinguaEngine::level_ladder(language)` returns one row per level: the
reader's counts on that level's words, and `typicalVocabulary`, computed once per language by
`level_vocabulary(level, dictionary_words, pack)`. The extension's `ladderView` shows the latter as
« estimés », with a legend. A pack says whether its levels are estimated (`levels_estimated`).

## Goals / Non-Goals

**Goals:**
- The Spanish ladder's « estimés » mean what English's do, and say where they come from.
- English unchanged, byte for byte.

**Non-Goals:**
- Spreading the Spanish levels over the frequency tail, so that `level_vocabulary` could extend
  them. That would change which Spanish words have which level on the cards, and a reader's presumed
  words when they declare a level.
- A table of vocabulary sizes from the research literature. Its sizes count word families, not this
  dictionary's lemmas.

## Decisions

### D1 — Borrow English's figures, in the engine

`level_ladder` computes the typical vocabularies from the English pack when the asked pack's levels
are estimated and an English pack with CEFR-list levels is loaded. It uses the same cache as English's
own ladder. The engine always holds the default pair's pack, English's.

Each row then also carries `typicalFrom: "en"`. The key is absent when nothing was borrowed, so a
pack with CEFR lists answers exactly as before. A Spanish pack in an engine without English keeps its
own figures, as today.

*Rejected — in the extension, a second `levelLadder("en")` request.* The engine already holds both
packs. Every surface that shows the ladder would need the second request, and only the engine knows
whether an English pack with lists is there.

*Rejected — hiding the column for estimated levels.* The owner asked for the Spanish ladder to read
like English's.

### D2 — The legend names the source

With `typicalFrom`, the legend's « estimés » sentence ends « repris de l'anglais, dont l'espagnol
reprend les tailles de niveaux. » instead of « extrapolé des mots des niveaux inférieurs sur tout le
dictionnaire. ». The languages are named through `language-labels.ts`, as the lint requires.

## Risks / Trade-offs

- **English's figures stand in for Spanish's.** A Spanish reader of a level is assumed to know as many
  words as an English one. That is the assumption the Spanish levels already make, by taking
  English's level sizes, and the ladder says it.
