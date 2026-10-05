# Design

## Context

The extension marks a selection through the engine's alignment (`src/translate/markup.ts`). The
sentence goes out with the selection tagged `<b>`; the engine moves the tag onto the target span it
aligns with. `relay.ts` also translates the selection alone. `reconcileMarks` (`reconcile.ts`) then
moves, splits or trims the tag's marks with that second translation.

Spanish goes through English (`translateViaPivoting`), so its tag crosses two alignments. Since
change 26, `MARKED_LANGUAGES` is `["en"]` and Spanish is sent untagged.

Decision D2 fixed the thresholds before measuring. This design fixes the method before measuring,
too: the corpus, the harness, how a mark is judged and how the rates are computed.

## Goals / Non-Goals

**Goals:**
- One measurement of the marks, English and Spanish, on the same sentences, with the extension's own
  marking and the real engine.
- Judgments anyone can check and a measurement anyone can run again.
- D2 applied to Spanish.

**Non-Goals:**
- Measuring translation quality itself. The marks are what D2 gates.
- Per-platform runs. The engine is one WebAssembly build, and platforms differ in speed and memory,
  not in output.
- Shipping the gloss-located mark. It is an experiment here.

## Decisions

### D1 — The corpus

The source is the Parallel Universal Dependencies treebank, in English and Spanish: the same 1,000
sentences (news and Wikipedia), translated and annotated.

| Treebank | Commit | sha256 of the test file |
|---|---|---|
| UD_English-PUD | `f16eba4ae7f3d161870ed320676c5088b8fa476c` | `c80584f2…f0aa` |
| UD_Spanish-PUD | `818a82b8628c9cbec78750c7e83ccba34b9ce22b` | `48a7b5c7…98d3`, the file `es-pud.sh` pins |

Sentences are every tenth one, in file order: the 1st, the 11th… the 991st, so 100 per language,
and the same ids in both languages.

The selection rule picks, for item k (0 to 99), a word of part of speech NOUN, VERB, NOUN or ADJ by
k modulo 4. Within the sentence, the selection is the first token of that part of speech that:
- is not the sentence's first token;
- is made of letters only, three or more;
- is not inside a multiword token (`del`, an enclitic verb).

Without one, the first NOUN, VERB or ADJ that meets those conditions is taken instead. Without that
either, the next sentence is used. The token's character offsets come from aligning the tokens'
forms with the sentence's `# text`.

The rule is code. Its output is committed as `tool/marks/corpus.json`: the sentence id, the
language, the token index, the word and its offsets. PUD's text is fetched at measurement time and
checked by sha256 (CC BY-SA, never committed), as `es-pud.sh` does.

### D2 — The harness

`tool/measure_marks.ts` runs with Node's type stripping, so it imports the extension's own
`markup.ts` and `reconcile.ts`. It uses:
- the engine the package ships, `engine/` from `yarn fetch:engine`;
- the catalogue's models, read from a directory (a local `assemble_model_site.mjs`) or from
  `models.cymbra.app`. Each file is checked against the catalogue's sha256 before use.

For each selection it does what `relay.ts` does:
1. the sentence with the selection tagged;
2. the selection alone;
3. `readMarked`, then `reconcileMarks`.

It translates through the language's route: one model for English, the pivot for Spanish.

The output is one JSON line per selection, committed as `tool/marks/results-<lang>.jsonl`:
- the French sentence;
- the marks after reconciliation;
- the fragment's own translation;
- the experiment's mark (D5).

### D3 — How a mark is judged (fixed before the run)

- **Correct:**
  - the marks cover the French rendering of the selected word in that sentence;
  - they may include the article, preposition or auxiliary the rendering carries (« la maison », « a
    abandonné »);
  - they may be split when the rendering is discontinuous.
- **Wrong:**
  - the marks cover another word, or only a function word;
  - or they cover the rendering together with an unrelated content word.
- **Withheld:** no mark.
- **A word the translation omitted:** any mark is wrong, and none is withheld.

Each item is judged by reading it. The judgments are committed as `tool/marks/judged-<lang>.tsv`,
one row per selection, with a short reason for every wrong one. Anyone can audit them.

### D4 — The rates and the rule

- **Correct:** correct ÷ (correct + wrong), the share of shown marks.
- **Withheld:** withheld ÷ all selections.

This is how D2 reads (« ≥ 90 % correct marks and ≤ 25 % withheld »), and how the study counted
English (83 of 87 shown marks).

| Spanish's result | What happens |
|---|---|
| ≥ 90 % correct and ≤ 25 % withheld | `es` joins `MARKED_LANGUAGES`, with a test |
| 75–90 %, or otherwise short of the first tier and above the last | Nothing changes: translated, unmarked |
| < 75 % correct or > 30 % withheld | Reported to the owner, whose decision of 2026-10-05 turned translation on regardless of the marks |

English is measured beside Spanish as the reference, not gated.

### D5 — The experiment: a mark located from the gloss

This answers the owner's question: why not find the French word from the dictionary instead of the
alignment?
1. The harness takes the selected word's lemma from the pair's `forms.tsv`, then that lemma's French
   gloss from `gloss.tsv`.
2. It looks for the gloss's words in the French sentence, ignoring case and accents, with the stem
   rule `reconcile.ts` uses.
3. It marks the first match, or nothing.

That mark is judged with the same criteria and reported beside the engine's. Whether to ship it is a
later change.

## Risks / Trade-offs

- **One judge.** Every judgment is committed with its reason, so a second reader can redo any of
  them.
- **The domain.** PUD is news and Wikipedia; readers also read novels. The rule mixes nouns, verbs
  and adjectives as reading does, and the domain is the same for both languages.
- **Sample size.** At 100 selections, a 90 % rate carries about ±6 % of sampling error. A result
  near a boundary is reported as such, and the corpus can grow with the same rule.
