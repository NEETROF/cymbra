# Design — add-lingua-spanish-detection-guard

## Context

`analysis/language.rs`:
- `block_is_studied(text, studied)` keeps a block of 12 bytes or more when
  `whichlang::detect_language` returns the studied language's class;
- `detect_document_language` weighs each such block by its length for the language whichlang
  finds.

whichlang 0.1.1 knows Spanish and Portuguese but neither Catalan nor Galician. It sends their blocks
to Spanish or Portuguese.

## Goals / Non-Goals

**Goals:**
- Catalan and Galician blocks no longer analysed or counted as Spanish, measured.
- Spanish prose kept: no false refusal on real Spanish text.
- English unchanged, byte for byte.

**Non-Goals:**
- Recognising Catalan or Galician as languages of their own: no reader studies them.
- Routing per block. A page mixing Spanish and Catalan keeps its Spanish blocks; the Catalan ones
  are excluded, like any other language.
- A guard on English: no neighbour is confused with it at a comparable rate.

## Decisions

### D1 — Function words, counted, compared

For a block whichlang reads as Spanish, count three kinds of words, lowercased:
- **Catalan markers**: words Catalan uses and Spanish does not. Words beginning with an elision,
  `l'`, `d'`, `s'` or `n'` (`l'home`, `d'aquesta`), count as Catalan markers too.
- **Galician markers**: words Galician uses and Spanish does not.
- **Spanish markers**: words Spanish uses and neither neighbour does.

The block is refused as Spanish when Catalan markers outnumber Spanish ones, or Galician markers do.
A tie, or no marker, keeps it Spanish: a short line cannot be judged, and Spanish recall comes
first. A word shared with either neighbour (`el`, `de`, `que`, `con`, `pero`, `como`) is in no
table. The tables are sorted and binary-searched, like the closed classes.

*Rejected — a second detector with Catalan and Galician classes.* It would be a new dependency and a
new model in the WASM bundle, for a question a few dozen function words answer.

*Rejected — a ratio threshold.* "More than Spanish" has no parameter to tune or justify. The
measurement shows it refuses no Spanish prose.

### D2 — One detection function

`detect(trimmed)` is whichlang's answer with the guard applied: a guarded Spanish block comes back as
Portuguese, a language neither the gate nor the vote studies. `block_is_studied` and
`detect_document_language` both call it, so the gate and the vote never disagree.

English's code path is unchanged: the guard turns only Spanish answers into Portuguese, so whichlang's
English answers are untouched.

### D3 — Spanish `1.1.0`

The guard changes which Spanish blocks are analysed, which the analyser version exists to signal.
`add-lingua-spanish-analysis` set Spanish to `1.0.0`, and this change makes it `1.1.0`.

### D4 — Measured, and how

The measurement uses Wikipedia extracts fetched on 2026-10-03 and kept out of the repository:
- 10 Catalan articles, 1,214 paragraphs (Barcelona, Lisboa, Literatura, Mar, Catalunya, Cuina,
  Muntanya, Novel·la, Riu, València);
- 4 Galician articles, 318 paragraphs (Barcelona, Lisboa, Literatura, Mar);
- 3 Spanish articles, 442 paragraphs (Barcelona, Lisboa, Literatura).

The API's rate limit stopped the fetch there. The Galician and Spanish samples are small, and
`add-lingua-spanish-forms-tables`' measurement harness is where a larger corpus belongs.

It counts paragraphs of 12 bytes or more:

| | read as Spanish, before | after the guard |
|---|---|---|
| Catalan | 50.8 % (65.0 % of the text) | 8.5 % (1.4 % of the text) |
| Galician | 27.0 % (21.6 %) | 6.6 % (0.7 %) |
| Spanish | 95.0 % (99.2 %) | 94.1 % (98.9 %) |

The 4 Spanish paragraphs refused were two Catalan and one French bibliography entry, and a list of
Catalan articles. The fixtures committed with the change are sentences written for it, not the
extracts.

## Risks / Trade-offs

- **The remaining leak**: a short Catalan or Galician line with no marker (a caption, a list item)
  still reads as Spanish. It is 1.4 % and 0.7 % of the text measured, and the spec says so.
- **A Spanish text quoting Catalan at length** has its quoted blocks excluded. That is right: they are
  not Spanish.
