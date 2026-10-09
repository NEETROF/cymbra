# add-lingua-french-detection-guard — Catalan, Occitan and Romanian are not French

## Why

Change 42 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 3 (French studied: fr-en, fr-es). Once French is a studied language, whichlang — the
detector that gates every block and votes on each document's language — decides what a reader of
French is shown as French. It has sixteen classes and none for Catalan, Occitan, Romanian,
Franco-Provençal, Picard or Walloon; it reads many of their blocks as French. Measured on 2026-10-09
(the design's Measurement names the sources), blocks of 12 bytes or more read as French:

| | Blocks | Read as French | Share of the text |
|---|---|---|---|
| Catalan (UD AnCora, Tatoeba, Wikipedia) | 16,553 | 27.2 % | 30.6 % |
| Occitan (UD TTB, Tatoeba, Wikipedia) | 27,630 | 43.3 % | 55.4 % — 79.1 % of Occitan Wikipedia's text |
| Romanian (UD RRT, Tatoeba) | 5,631 | 23.5 % | 25.6 % |
| Italian (UD ISDT and PUD, Tatoeba, Wikipedia) | 13,946 | 0.2 % | 0.1 % — whichlang has an Italian class |

For a reader of French, an Occitan Wikipedia article is today four-fifths « French »: highlighted
as broken French, every word unknown, the percentage meaningless, words that do not exist in French
proposed for the deck. The corpus change 39 committed shows it already: of its `mixte` page's
Occitan block, `fr-en.golden` analyses all eighteen tokens as French. This is the French
counterpart of what `add-lingua-spanish-detection-guard` did for Catalan and Galician read as
Spanish (2026-10-03).

## What Changes

- **A guard on French detection.** A block whichlang reads as French is not French when its
  Catalan, Occitan or Romanian function words outnumber its French ones; a tie, or no marker at
  all, stays French — the Spanish guard's rule. Two sorted tables, measured (design D3): 154
  function words those neighbours write and French does not (`amb`, `els`, `és`, `molt`, `lo`,
  `los`, `e`, `dins`, `èra`, `în`, `să`, `și`, `pentru`…), and 71 French function words they do
  not use (`le`, `et`, `est`, `une`, `du`, `je`, `il`, `dans`, `avec`, `pour`, `était`…).
- **Read as the measurement showed it must be** (D4): a neighbour's word counts only where it is
  written in lowercase (`El Niño`, `Los Angeles` are names); a word joins across a hyphen, a
  middle dot or a full stop between letters (`e-mail`, `étudiant·e·s`); a run holding a digit is
  not a word (`2e`); a decomposed word is composed before it is looked up.
- **Run for French alone** (D5). Detection takes the languages it is asked about and applies the
  guard of the language whichlang found among them: Spanish's (unchanged) or French's. A reader of
  English never pays for either; the en-fr reader's French pages cost what they cost today. The
  gate and the vote keep calling one function, so they never disagree.
- **Measured with the guard**, on the same 148,267 blocks:
  - Catalan: 7.1 % of the blocks read as French (2.7 % of the text); Occitan: 24.9 % (16.3 %;
    5.6 % of Occitan Wikipedia's text); Romanian: 7.8 % (5.2 %);
  - French: 26 of the 41,312 French blocks read as French are refused (0.063 %; 0.081 % of the
    text). None of them in UD's seven French treebanks, 19,827 Tatoeba sentences, or four
    Québec, Belgian and Swiss books; all in Wikipedia articles about Catalonia, Occitania, Italy
    and the regional languages, and 14 of the 26 are Catalan, Occitan or Arpitan text quoted
    there.

  The remaining leak is short lines with no function word of either table, and the spec says so.
- **What it costs** (D5): natively 12.5–15.4 µs per KB of text read as French, about 5–6 % of a
  page analysis; in WebAssembly about as much as whichlang itself (14–18 µs per KB). The Spanish
  guard costs 20–23 µs per KB measured the same way.
- **French's analyser version is bumped** (D6) — `1.1.0` in the programme's order, after change
  41's `1.0.0`, as Spanish's guard followed Spanish's analysis. English and Spanish keep theirs.
- **`fr-en.golden` moves** (D7) on the version (the pack line and the seventeen `analyse` probes)
  and on the `mixte` page alone: its Occitan block leaves both analyses (46 → 29 counted tokens;
  the reader's known share 37 → 41 %). en-fr, es-fr, es-en and en-es do not move — run with the
  prototype, without re-blessing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Catalan, Occitan and Romanian are not read as French*. No existing
  requirement is rewritten: *Per-block language detection*, *A document's language, chosen among
  the reader's* and *Catalan and Galician are not read as Spanish* hold as written; change 39's
  two requirements and change 40's rewording of them are left to those changes. It archives after
  `add-lingua-french-baseline`, whose requirement makes French a studied language.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: French's guard and its tables in `analysis/language.rs`, the
    detection function taking the languages asked about, French's analyser version; *consumed*:
    whichlang, `unicode-normalization` (already a dependency), Spanish's guard unchanged.
  - `crates/lingua-wasm` — *consumed*: the engine and its `detectLanguage` binding, unchanged; the
    French baseline re-blessed, its scenario asserting the `mixte` page's blocks.
  - `crates/lingua-pack`, `scripts/lingua-data/testdata/fr-en/` — the fixture's manifest and the
    tests that name French's version follow it.
  - `apps/lingua-extension` — `test/packs.spec.ts` reads French's new version; nothing a reader
    sees.

  ID, Music, Live, the back office, the site, the backend, the Apple host app and the agent are
  untouched. No pack, table, pin, wire field or proto changes.
- **Release.** Silent. No listed pair studies French; nothing a reader sees changes until
  `enable-lingua-french` (change 52).
- **Order.** After change 39. Independent of changes 40 and 41 in code and spec: it touches
  detection, not the tokeniser or the cascade, and adds a requirement without a version number.
  The one coupling is the version line: `1.1.0` after 41 (recommended), `0.3.0` if it merges after
  40 and before 41 (41 keeps `1.0.0`); not before 40, whose spec names `0.2.0`. The golden figures
  above are measured on `main` (`0.1.0`) and are re-measured on top of whichever of 40 and 41 has
  merged (D7).
- **Not here.** A guard for Franco-Provençal, Picard or Walloon (D2: they share French's function
  words; open question 3 offers eight Arpitan words); Spanish's own leak on Occitan (19.2 % of Occitan text read as Spanish, open question 2);
  recognising any of these languages as languages of their own.
- **Effort, against 2–4 ideal days.** The detection function and French's guard with its tables:
  0.75–1.25. Unit tests: 0.5–1. The version bump, its literals, the fixture, the re-bless and the
  golden review: 0.5–1. Spec, programme: 0.25–0.5.
