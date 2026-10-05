# Design — add-lingua-card-frequency

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `selection-card.ts` `rarityText(cls, calibration)` | « Dans ton deck… » for a learning word. Otherwise « Peu fréquent — au-delà de tes N mots les plus courants. », or « … de ton niveau. » when the calibration is 0, which a declared level pins. |
| `selection-card.ts` | Every word card asks `wordGrammar(written, lemma)` when it opens. A card holding its gloss waits for it at most `GRAMMAR_WAIT_MS` and is drawn once (*A card holding its gloss waits only briefly for its grammar*). A card without its gloss is drawn pending until the answer. |
| `lingua-core` `Pack::rank(lemma)` | The pack's frequency rank of a dictionary form: 1 for the commonest, none for a lemma the pack does not rank. It is not exposed to the extension. |
| `crates/lingua-wasm/tests/english_baseline.rs` | Pins the `wordGrammar` JSON. Only a dictionary update or an analyser change may move it. |

## Goals / Non-Goals

**Goals:**
- The line says how common the word is, truthfully, for every reader.
- No extra wait: the card is drawn when it was drawn before.

**Non-Goals:**
- Saying why a word is highlighted. The colour says that, and Réglages show the level.
- A frequency line for an expression. The packs do not rank expressions.
- The form's own frequency (`es` against *ser*). The packs rank lemmas.

## Decisions

### D1 — Five bands

| Rank | Line |
|---|---|
| ≤ 100 | « Très courant — parmi les 100 mots les plus fréquents. » |
| ≤ 1 000 | « Courant — parmi les 1 000 mots les plus fréquents. » |
| ≤ 5 000 | « Assez courant — parmi les 5 000 mots les plus fréquents. » |
| ≤ 20 000 | « Peu fréquent — au-delà des 5 000 mots les plus fréquents. » |
| beyond, or unranked | « Rare — au-delà des 20 000 mots les plus fréquents. » |

Round numbers a reader can picture, each a different stage for a learner:
- the words of nearly every sentence (100);
- a beginner's vocabulary (1 000), close to the A1 size of the levels (about 1,000 lemmas);
- an intermediate's (5 000);
- the long tail.

The numbers are formatted as French writes them (`toLocaleString("fr-FR")`). A word the pack does
not rank is « Rare »: es-fr ranks 60,000 lemmas and en-fr about 40,000, so an unranked word lies
beyond the 20,000 commonest in both.

### D2 — The line speaks of the word, not of the reader

The line no longer reads the calibration. The highlight already says that the reader does not know
the word. A reader at « Débutant » learns from the line that `es` is among the 100 commonest words,
which is what they need to decide to learn it.

*Rejected — keeping « au-delà de tes N mots les plus courants » for a calibrated reader.* It speaks
of the reader, and it told every reader who declared a level that the commonest words were rare.

### D3 — The rank comes with the grammar

`lingua-wasm` gets a binding: `frequencyRank(lemma, language?)`. It returns the rank from the pack
of that language, or `undefined` when the pack does not rank the lemma. The extension's
`LanguagePort.wordGrammar(written, lemma)` asks the engine for both and returns the grammar with
`rank`: a number, or `null` when the lemma is unranked. On Firefox and Safari the background answers
the port, so the card still gets one message, and the bound that already covers the grammar covers
the rank.

*Rejected — the rank inside the core's `WordGrammar` JSON.* The English baseline pins that JSON, and
only a dictionary update or an analyser change may move it.

*Rejected — the rank on every analysed token.* Every page's analysis would carry it for words the
reader never opens.

*Rejected — a second message from the card.* It doubles the round trips on the hosts where they
cost, and needs a second bound.

### D4 — No answer, no line

| The card's answer | The line |
|---|---|
| a rank | its band (D1) |
| `null`: the pack does not rank the lemma | « Rare » |
| none yet (a pending card), or none in time (the bound passed) | no line: the element is hidden |

A word in the deck keeps « Dans ton deck — en cours d'apprentissage. », whatever its rank. An
expression keeps its own line.

## Risks / Trade-offs

- **A wordfreq rank is a corpus's.** Wikipedia, subtitles and the web weigh some words oddly. It
  is still the best frequency the packs carry, and it orders the levels too.
- **A cold engine** → the card completes without its grammar, and now without a frequency line. It
  used to show the calibration's line, which was the wrong one for most readers.
