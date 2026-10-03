# Design — add-lingua-spanish-levels

## Context

See proposal.md (Why). The level machinery is the archived `add-lingua-cefr-levels`.

**In the pack**, the table is the optional `levels` section, from a pair's `level.tsv`
(`lemma<TAB>A1…C2`).

**In the core**:
- `resolve_lemma` presumes a lemma known below the declared level;
- `band_stats` folds each level's lemmas into confirmed, presumed and to learn;
- level-targeted seeding feeds a deck from one level;
- the vocabulary estimate extrapolates from the levels.

**In the extension**, a pack with levels (`hasLevels`) gets:
- the level chips in Réglages and in onboarding;
- the level indicator in the popup;
- the ladder and « Renforcer un niveau » in the statistics.

A pack without levels gets the frequency slider instead.

That change refused computed levels (its D2): a rank band called a CEFR level would be a claim
no list backs. Decision D1 of the Spanish programme supersedes it for Spanish only, on one
condition: the computed levels are labelled as estimated wherever a level is shown. English keeps
its CEFR-J and Octanove levels.

## Goals / Non-Goals

**Goals:**
- A Spanish level table, so a Spanish reader declares a level, gets words presumed known below
  it, and has a ladder.
- A table measured against real CEFR levels before it ships.
- The extension never presents an estimated level as a CEFR assessment.

**Non-Goals:**
- A real Spanish CEFR list. If a licence request succeeds, its table replaces this one and the
  flag goes back to false: a data change, not a format change.
- Levels per sense, as for English.
- The agent plugin (`add-lingua-agent-languages`).

## Decisions

### D1 — English's band sizes, over the commonest Spanish lemmas

The derivation is measured on English, where the truth is known:
1. Sort English's 8,302 CEFR lemmas by their wordfreq rank.
2. Give the first 1,020 the level A1, the next 1,158 A2, and so on, with the true size of each
   level: 1,020, 1,158, 2,015, 2,347, 886, 876.

Against the true levels this gives **39.8 % exact and 82.6 % within one level**. That is the
figure the programme's study recorded (40 % and 83 %).

For Spanish, the same sizes go to the commonest lemmas in rank order. A lemma is **skipped**
when it has no French gloss, or only a proper noun's: a CEFR list would leave out `the`,
`twitter`, `etc`, `madrid` or `méxico`, which wordfreq ranks high.

| Level | Lemmas | Ranks | First words |
|---|---|---|---|
| A1 | 1,020 | 1–1,086 | de, que, el, en, y, a, no, un |
| A2 | 1,158 | 1,087–2,426 | novela, permiso, presentación, puente |
| B1 | 2,015 | 2,427–5,102 | sensible, sobrevivir, tristeza, alquiler |
| B2 | 2,347 | 5,103–8,869 | sarcasmo, sinceridad, surgir, vegetación |
| C1 | 886 | 8,870–10,445 | patata, perseverancia, plenitud |
| C2 | 876 | 10,446–12,069 | gemelo, generalización, hipertensión |

*Rejected — English's rank cutoffs* (A1 up to rank 1,096 … C2 up to 40,612). The English lists
cover only part of each frequency range: B2 has 2,347 words over 10,685 ranks. Every glossed
Spanish lemma inside the same cutoffs would give bands several times larger: C2 alone would hold
thousands of words, and a declared level would presume far more words known than an English
reader's. The band sizes keep both the ladder and the presumption at English's scale. They are
also what the study measured.

### D2 — Monotone, so six levels; the sizes are constants

D1 of the programme falls back to three bands when the derived scale is not monotone. It is
monotone:
- the median rank of each true English level rises from A1 to C2: 929, 2,216, 4,055, 8,165,
  15,295, 26,559;
- the mean true level of each estimated level rises: 1.67, 2.41, 3.12, 3.89, 4.67, 5.03.

So Spanish keeps the six levels.

The six sizes are constants in `reduce-es-fr.py` (`ENGLISH_BANDS`), measured on en-fr's 2026-09-26
level table. They are not read from that table at reduction time: an English dictionary update
would otherwise move the Spanish levels in a pull request that says nothing of Spanish.

### D3 — The pack says its levels are estimated

`PackMeta` gains `levels_estimated: bool`:
- `#[serde(default)]`, so a pack without it reads as false;
- skipped when false, so the en-fr pack's metadata, and so its bytes and pin, do not move;
- unknown fields are already ignored, so an older core reads an es-fr pack as before.

The builder takes it from the manifest's `meta`, as it takes the other fields, and no
`analyzer_version` moves: the analysis is untouched.

The flag reaches the extension along `hasLevels`'s path:
- `Pack::levels_estimated()`;
- the engine's `levelsEstimated(language)` (`lingua-wasm`);
- `LanguagePort.levelsEstimated()` in the extension, through `WasmLanguagePort`,
  `MessagingLanguagePort` and the background's RPC handler;
- the test fakes.

*Rejected — knowing it per language in the extension.* Whether levels are estimated is a fact
about the data: a licensed Spanish list would make it false with no change to the extension.

### D4 — What the reader sees

For a language whose pack's levels are estimated:

| Surface | English, CEFR levels | Spanish, estimated levels |
|---|---|---|
| Réglages, level block title | « Niveau d'anglais » | « Niveau d'espagnol estimé » |
| Réglages, under the chips | the hint | the hint, then the note |
| Popup, level indicator | « Niveau d'anglais » | « Niveau d'espagnol estimé » |
| Statistics, ladder title | « Mon niveau d'anglais » | « Mon niveau d'espagnol estimé » |
| Statistics, ladder column | « enseignés » | « courants » |
| Statistics, ladder note | « « enseignés » : les mots de base introduits jusqu'à ce niveau par les listes d'enseignement. » | « « courants » : les mots les plus fréquents jusqu'à ce niveau. » |
| Statistics, « Renforcer un niveau » | its note | its note, then the note |
| Onboarding, under the chips | — | the note |
| Onboarding, confirmation | « Niveau enregistré : B1. » | « Niveau enregistré : B1 (estimé). » |

The note reads « Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de
droits pour l'espagnol. » It is written in the language labels module, the one place a
language's name is written (`add-lingua-language-choice`), which gains the article form
(« l'espagnol »).

The ladder's own « niveau estimé B1 » keeps its meaning: the reader's position, estimated from
the levels' words.

### D5 — Tests

- **Python**:
  - the bands, given in rank order;
  - a lemma without a gloss skipped, and one with only a proper noun's;
  - the sizes summing to English's 8,302.
- **Rust**:
  - the flag round-trips;
  - a manifest without it builds what it built;
  - the es-fr pack says its levels are estimated, en-fr's does not;
  - the English baseline does not move.
- **Extension**:
  - the labels for both cases;
  - Réglages, the ladder, the statistics, the popup and onboarding with an estimated pack and a
    CEFR one.

## Risks / Trade-offs

- **Estimated levels are 40 % exact** → that is what the label says. Within one level 83 % of the
  time, a declared level presumes roughly the right words, and the reader corrects the rest by
  marking them.
- **Spanish frequencies are not English's** → the sizes, not the ranks, carry over (D1), and the
  study's figure is measured with them.
- **A word left out because it has no gloss** → it also has no card translation, so a level
  would not help a reader learn it.
