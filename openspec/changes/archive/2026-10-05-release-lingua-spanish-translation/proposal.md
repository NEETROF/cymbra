# release-lingua-spanish-translation — measuring the Spanish marks

## Why

Since `add-lingua-spanish-translation-pivot` (change 26), Spanish is translated through English on
every platform, and its sentences come back without marking the selected word. The owner decided it
so: the marks go through two chained alignments, and the study's indicative sample scored about
12–13 correct out of 20 against about 17 for English.

The programme fixed the rule before any measurement (decision D2).
- **≥ 90 % correct marks and ≤ 25 % withheld:** the mark is shown.
- **75–90 %:** the sentence is translated, without bold.
- **Under 75 % correct, or over 30 % of marks withheld:** Spanish translation is withdrawn from
  releases.

The study measured English at 83 correct of 87.

This change measures both languages on a committed corpus, with the real engine, exactly as the
extension marks a selection. It then applies the rule. It is change 27, R5 in
`docs/lingua/spanish-programme.md`. Translation itself is no longer staged per platform since the
owner's decision of 2026-10-05. The engine is the same WebAssembly build everywhere, so one
measurement serves every platform.

## What Changes

- **A committed corpus of selections.**
  - The sentences: 100 sentences of the Parallel Universal Dependencies treebank (PUD), every tenth
    of its 1,000, in English and in Spanish. They are the same sentences in both languages.
  - One word per sentence and language, chosen by a fixed rule from the treebank's annotation. The
    rule cycles noun, verb, noun, adjective, so 50 nouns, 25 verbs and 25 adjectives.
  - The repository commits the rule and the list of selections (sentence id, token, word). PUD
    itself is fetched at pinned commits and checked by sha256, never committed, as `es-pud.sh`
    already does.
- **A harness that marks as the extension does.** `tool/measure_marks.ts` runs:
  - the pinned engine;
  - the catalogue's models, from `models.cymbra.app` or a local assembly of it;
  - each selection as `relay.ts` handles it: the sentence with the selection tagged, the selection
    alone, then `reconcileMarks`, the extension's own modules.

  Every pivot is the real one.
- **Each mark judged, and the judgments committed.** A mark is one of:
  - *correct* when it covers the French rendering of the selected word in that sentence;
  - *wrong* when it covers something else;
  - *withheld* when there is none.

  The rates follow D2's wording, fixed here before the run:
  - correct is the share of shown marks;
  - withheld is the share of all selections.
- **The rule, applied.** Spanish joins `MARKED_LANGUAGES` only on D2's first tier. On the middle
  tier nothing changes. On the lowest, the decision goes back to the owner, whose decision of
  2026-10-05 turned translation on regardless of the marks.
- **An experiment beside it, for the owner's question.** The harness also reports a mark located
  from the pack's own French gloss of the selected word, found in the translated sentence. No
  reader-facing change follows from it in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: a requirement is added. A language's selection is marked only once its
  marks, measured on the committed corpus, reach D2's first tier.

## Impact

- **Products:** Cymbra Lingua's translation marks for Spanish, depending on the measurement.
  Nothing is consumed from ID or the platform.
- **Code:**
  - `apps/lingua-extension/tool/measure_marks.ts`, `tool/marks/` (the selection rule, the corpus,
    the judged results);
  - `src/translate/markup.ts`, if Spanish passes.
- **Data:** PUD is read at pinned commits (CC BY-SA), never committed. The corpus file holds
  sentence ids, token indices and the selected words.
- **CI:** none. The measurement runs by hand, like `es-pud.sh`. Its results are committed with the
  judgments.
