# Design — add-lingua-spanish-read-aloud

## Context

`rankVoices` (`reading/speech.ts`) orders the eligible voices of a language:
1. **Tier**: a voice the reader downloaded for its quality (Apple's enhanced or premium, Chrome's
   suffix), then an ordinary voice, then Apple's novelty voices.
2. **Region**: the language's preferred regions in order (`PREFERRED_REGIONS`, English's `us`,
   `gb`).
3. **The browser's order.**

The automatic choice is the first voice. A region is read from the voice's language tag
(`es-ES`), or from Firefox for Android's three-letter one (`spa-ESP`) through
`THREE_LETTER_REGIONS`, which lists English-speaking countries only.

## Goals / Non-Goals

**Goals:**
- An es-ES voice by default, as decision D5 asks, whatever order the browser lists voices in.
- The same on Firefox for Android.

**Non-Goals:**
- An accent setting. D5 rules it out; the reader chooses another voice in Réglages.
- A preference among the Latin-American accents: after Spain, the browser's order stands.

## Decisions

### D1 — Spain first, within a tier

`PREFERRED_REGIONS.es = ["es"]`. The region ranks after the tier, as English's does, so a Mexican
voice the reader downloaded in enhanced quality still comes before a compact voice of Spain. A
voice downloaded for its quality is a choice the reader made, and the tier already honours it for
English.

*Rejected — region before tier.* It would make a device's novelty or compact voice of Spain win
over the voice its reader went out of their way to install.

### D2 — The Spanish-speaking regions in three letters

`THREE_LETTER_REGIONS` gains `esp` → `es`, `mex` → `mx`, `arg` → `ar`, `col` → `co`, `chl` → `cl` and
`per` → `pe`. Spain is the one D1 needs. The others let a Latin-American voice read as itself rather
than as an unknown region, which keeps the browser's order among them.

## Risks / Trade-offs

- **A device with no voice of Spain** → the first Spanish voice the browser lists, as today.
- **A reader who prefers Latin-American Spanish** → they choose the voice once in Réglages, and the
  choice is kept for Spanish.
