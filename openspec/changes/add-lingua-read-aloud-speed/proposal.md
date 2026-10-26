# add-lingua-read-aloud-speed — let the reader slow the voice down, or speed it up

## Why

`add-lingua-read-aloud` put a listen row on the card and a voice choice in Réglages, and left the
speaking rate out on purpose: the first cut had to prove that nothing leaves the device. A
learner has now asked for it. Hearing `thorough` or a whole sentence at the voice's native pace is
often too fast to parse in a language one is still learning, and the cure everyone reaches for is
the same — slower. The platform already supports it (`SpeechSynthesisUtterance.rate`, honoured by
the on-device voices on every target), so this is a setting and one argument, not a new capability
of the browser.

**Products.** Cymbra Lingua only: `apps/lingua-extension` (all three variants; the Safari variant
ships inside `apps/lingua-apple`, whose native code does not change). Cymbra ID, Music, Live, the
back office, the site and the backend are untouched: no `.proto`, no migration, no flag, no Rust.

## What Changes

- **A "Vitesse de lecture" choice in the "Lecture à voix haute" block of Réglages**, next to the
  voice choice: a short list of presets — ultra slow, very slow, slow, normal — with *normal* the default. It
  applies to every listen button on the card and to the "hear this voice" preview in Réglages, so
  the reader judges the speed on the same sentence they will hear on a page.
- **One speed for all languages.** Unlike the voice (which is per studied language, because voice
  identifiers are), a pace is the reader's, not the language's: it is kept once per device.
- **Takes effect on the next reading.** A reading already under way is not re-timed; the change
  is heard the next time a listen button, or the preview, is pressed.
- **Nothing else moves.** The on-device-only rule, the two opt-in fallbacks, the silence-follows-
  the-card rule and the voice choice keep their text and stay true: the rate is a number handed to
  the voice that was already chosen, and it changes neither which voice speaks nor where the text
  goes.

Not in this change: a faster-than-normal rate (nobody asked for it, and it does not help parse a
sentence), highlighting the words as they are spoken, a pitch control, a per-language speed, a
continuous slider, and a speed control on the card itself. Each is its own change if
wanted. No manifest, permission, store listing, card model or sync contract changes.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`: one new requirement, that the reader chooses the speaking rate of
  read-aloud in Réglages and that every reading honours it. ADDED only. "The reader chooses the
  voice" and "One Réglages on every surface" (from `add-lingua-read-aloud`, still in flight) keep
  their text; the speed block is built by the same Réglages builder, so it reaches the side panel,
  the in-page drawer and the toolbar popup together.

## Impact

- **Extension**: `src/reading/speech.ts` — `SpeechSettings` gains the rate, the engine's `speak`
  takes it, and `browserSpeechEngine` sets `utterance.rate`; `src/state/storage.ts` — a
  `SPEECH_RATE_KEY` with load/save, folded into the existing voice preference so the speaker
  re-reads it on change; `src/reading/settings-view.ts` — the speed control in the existing
  read-aloud block, and the preview reading at the chosen rate; `src/i18n/{en,fr,es}/settings.ts`
  — the labels; `src/styles/settings.css` only if the control needs a rule `tokens.css` does not
  already give.
- **Background and content wiring**: untouched. The speaker already receives its settings through
  the voice preference; the rate rides the same channel.
- **Dogfooding**: whether *slower* is slow enough differs by voice and by platform (iOS and macOS
  voices scale differently from Chrome's, and Firefox for Android depends on Android's engine),
  and none of them exists in jsdom — the presets are confirmed on the devices, as the voice lists
  were.
- **Release**: an ordinary extension release on the two stores, then a dispatch of
  `lingua-apple-release` with `deliver` so the Safari variant carries the same build. Nothing for
  a store reviewer to re-read: no new origin, no new permission.
