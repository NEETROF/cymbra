# add-lingua-card-frequency — the card says how common its word really is

## Why

Dogfooding the Spanish reading (2026-10-04), the card of `Es` said « Peu fréquent — au-delà de ton
niveau ». `Es` is *ser*, one of the commonest Spanish words.

The line speaks of the reader's calibration, not of the word. A word not presumed known reads
« Peu fréquent ». A reader who declared a level, « Débutant » included, has a calibration of 0. So
every word they do not know reads « au-delà de ton niveau », the commonest ones too. The card's
base requirement, *Word popup on click*, asks for « the frequency rank in plain language », which
the line no longer gives.

The owner chose the real frequency: the word's rank in the pack, in plain bands.

## What Changes

- **The frequency line gives the word's rank**, the dictionary form's in the pack's frequency list,
  in five bands:

  | Rank | Line |
  |---|---|
  | 1–100 | « Très courant — parmi les 100 mots les plus fréquents. » |
  | 101–1 000 | « Courant — parmi les 1 000 mots les plus fréquents. » |
  | 1 001–5 000 | « Assez courant — parmi les 5 000 mots les plus fréquents. » |
  | 5 001–20 000 | « Peu fréquent — au-delà des 5 000 mots les plus fréquents. » |
  | beyond, or unranked | « Rare — au-delà des 20 000 mots les plus fréquents. » |

- **It no longer depends on the reader** (their level or calibration).
- **Unchanged lines**: a word in the deck keeps « Dans ton deck — en cours d'apprentissage. », and an
  expression keeps its own line.
- **The rank arrives with the grammar.**
  - The engine exposes a dictionary form's rank (`frequencyRank`).
  - The extension's port returns it with the word's grammar, so the card waits for no extra answer.
  - When the answer does not come in time, the card shows no frequency line rather than a guess.
- **Every language**: English cards change too.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *The word card says how common its word is*. *Word popup on
  click*, which asks for « the frequency rank in plain language », is not modified: the new
  requirement says how.

## Impact

- **Products.** Cymbra Lingua:
  - `crates/lingua-wasm`: a `frequencyRank` binding over the existing `Pack::rank`;
  - `apps/lingua-extension`: the port's `wordGrammar` returns the rank, and the selection card
    writes the line.

  No pack or table change: the ranks are the packs' own (wordfreq). No server or proto change.
  ID, Music, Live, the back office and the site are not affected.
- **The English baseline (S0) does not move.** The core's `word_grammar` JSON is unchanged: the rank
  is a separate binding, which the extension's port merges.
- **Release.** With the next release of the extension. Every card's frequency line changes, English
  included.
