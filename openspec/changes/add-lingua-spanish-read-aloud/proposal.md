# add-lingua-spanish-read-aloud — a Spanish voice from Spain by default

## Why

Read-aloud picks a voice automatically for the language a page is read in, and the reader can
choose another in Réglages, one voice per language (`add-lingua-language-choice`). Decision D5 of
the programme (`docs/lingua/spanish-programme.md`) is an **es-ES voice by default, with no accent
setting**.

The code does not do that yet. Within a quality tier, `rankVoices` tries each language's preferred
regions first (English: the United States, then Britain), then the browser's order. Spanish has no
preferred region, so an es-ES voice comes first only when the browser happens to list it first.
Firefox for Android writes regions in three letters (`spa-ESP`), and the table that reads them
knows no Spanish-speaking country, so it could not tell a Spanish voice from a Mexican one even
with a preference.

This is change 25, in G1, the internal Spanish build.

## What Changes

- **Spain first for Spanish.** `PREFERRED_REGIONS` gains `es: ["es"]`. Within a quality tier, a
  voice of Spain comes before the others, which keep the browser's order.
- **Spanish-speaking regions read in three letters.** `THREE_LETTER_REGIONS` gains Spain, Mexico,
  Argentina, Colombia, Chile and Peru (`spa-ESP` is Spain).
- **No accent setting.** The reader who wants another accent chooses the voice in Réglages, as for
  any language.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *A Spanish voice from Spain by default*.

## Impact

- **Products.** Cymbra Lingua's extension: `reading/speech.ts` and its tests. English's choice does
  not move. No package reads Spanish aloud until `enable-lingua-spanish`.
- **Sources and licences**: none.
- **Release.** G1, internal: nothing ships.
