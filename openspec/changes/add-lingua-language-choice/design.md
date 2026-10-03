# Design — add-lingua-language-choice

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `src/reading/settings-view.ts` | One « Niveau d'anglais » block, built once, for the reading language; one voice block whose select saves a single voice (`saveVoice(area, uri)`). |
| `src/state/level-choice.ts` | `needsLevelChoice(port, language)`: true when the language has levels, no declared level, and no level decision **in any language** exists. |
| `src/state/storage.ts`, `src/reading/speech.ts` | `VOICE_KEY` holds one `voiceURI`; `SpeechSettings.voice` is it, and the speaker picks it whatever its language. |
| `src/popup/popup.html`, `popup.ts`, `src/stats/ladder.ts`, `src/onboarding/onboarding.html` | French copy with « anglais » written in. |
| `src/analyzer/pairs.ts` | `SHIPPED_PAIRS`, `acceptedLanguages`, `readingLanguage`. |
| The engine | `studiedLanguages` / `setStudiedLanguages` (profile), and level and calibration per language. |

## Goals / Non-Goals

**Goals:**
- A reader can choose the languages they study, in the settings and at onboarding, once the package
  ships more than one.
- Every setting that depends on a language has one value per language.
- No surface names a language by hand.

**Non-Goals:**
- Choosing the primary language apart from the order: the first language ticked is the primary one.
- Statistics and review by language (`add-lingua-language-stats-review`).
- A book's language (`add-lingua-reader-language`).

## Decisions

### D1 — The labels module, enforced by lint

`language-labels.ts` holds, per studied language: its name (« Anglais »), its « de » form
(« d'anglais », for « Niveau d'anglais »), and its feminine adjective (« anglaise », for « aucune voix
anglaise »). Helpers build the few sentences the surfaces need. `test/lint-language-labels.spec.ts`
reads every `.ts` and `.html` under `src/` except the module and refuses the word « anglais »
(any form). Static page copy that named English becomes language-neutral, or is filled from the
module at load.

### D2 — « Langues étudiées »: shipped languages, ticked from the profile

`mountStudiedLanguages(container, port, persist)` lists the studied side of each shipped pair, once,
in the pairs' order. A box is ticked when the profile holds the language.
- Ticking appends the language.
- Unticking removes it; the box of the only ticked language is disabled.
- Each change calls `setStudiedLanguages` and persists the backup: every context restores it and
  follows (`followSurfaceLook`, `readLanguages`, the sync's accepted set).
- A profile language the package does not ship is not listed, and is kept.
- With a single shipped language, the block is hidden.

### D3 — Level blocks per accepted language

The level block becomes `levelBlock(language)`: its title from D1, its chips, hint and calibration
for that language. `mountSettings` keeps a container of blocks for `acceptedLanguages(port)`. It
rebuilds the container when that list changes on refresh, and refreshes each block otherwise.
Resetting statuses resets the calibration of every accepted language; a full reset returns the
profile to English and calibrates it.

`needsLevelChoice` keeps its signature and filters `exportDeclaredLevels()` by language.

### D4 — The voice per language, the old value as English

`VOICE_KEY` holds a map from language to `voiceURI`. Reading accepts the former single string as
`{ en: <uri> }`, so a voice chosen before is kept for English, and the first save writes the map.
`SpeechSettings.voice` becomes `voices`, and the speaker picks `voices[lang]` for its language.

The settings' select saves for `speaker.lang`, the language the host's speaker reads. In a reading
page's drawer, that is the page's language (change 12). In the side panel and the popup, it is the
reader's first language. When the reader studies several languages, the block's title names the
language it edits.

*Rejected — one voice select per language in the block.* A select lists the voices the browser has
for one language, and the speaker the host hands in reads one language at a time. Editing the
language being read is what the reader can hear in the preview.

### D5 — Onboarding: languages, then a level each

When the package ships several languages, onboarding first shows the boxes of D2, saved like in
Réglages. It then shows a level chip row per chosen language, titled from D1. With one language,
onboarding is unchanged apart from neutral copy.

## Risks / Trade-offs

- **The first save rewrites `VOICE_KEY` as a map** → an older build downgraded to would read an
  object as no voice and fall back to the automatic choice. Nothing breaks.
- **More level blocks make Réglages longer** → one per accepted language; most readers have one.
- **The lint rejects a legitimate « anglais »** (a comment) → comments say "English"; French copy
  goes through the module.
