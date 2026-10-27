## Context

`add-lingua-read-aloud` split speech into a DOM-free `speech.ts` (eligible voices, the automatic
choice, a speaker that owns one utterance at a time) behind a `SpeechEngine` seam, with the
browser's `speechSynthesis` wrapped by `browserSpeechEngine`. The reader's preferences reach the
speaker through a `VoicePreference` (`load` / `watch`) that returns a `SpeechSettings`
(`voices`, `androidVoices`, `remoteVoices`), backed by `storedVoicePreference` over the storage
area. Réglages' "Lecture à voix haute" block is built once in `settings-view.ts` and reused by the
side panel, the in-page drawer and the toolbar popup. That design listed *no rate control* as a
non-goal, confirmed on 2026-09-23. This change reverses that, for a stated need.

## Goals / Non-Goals

**Goals:**

- A speaking-rate choice in the existing block, honoured by the card's listen buttons and by the
  voice preview.
- Reuse the preference channel, so the speaker follows a change from another surface with no new
  wiring in `content.ts`.
- Keep the new logic testable in jsdom, inside the 80 % gate.

**Non-Goals:**

- A faster-than-normal rate, word highlighting, pitch, a per-language rate, a continuous slider, a
  control on the card.

## Decisions

**D1 — A short list of presets, not a slider.** The block is a place to choose a voice, not an
instrument. Four presets — 0.25× (ultra slow), 0.5× (very slow), 0.75× (slow), 1× (default) — answer the actual request
(slower for a learner), in three steps because a beginner is more likely to find one slow step
insufficient than to want to go faster. No faster step: nobody asked, and speed does not help to
parse a sentence. A slider invites
fiddling over a sentence that lasts three seconds, needs a live preview to make sense of, and is
harder to make accessible on the popup. The list is a constant in `speech.ts`, so adding a step
later is one line and one label.

**D2 — One rate per device, not per language.** The voice is per language because a `voiceURI`
names one language's voice. A pace belongs to the reader: someone who slows French down will slow
Italian down. A single key `SPEECH_RATE_KEY` holds it. A value that is not one of the presets
(hand-edited storage, a future step removed) reads as normal rather than being passed on as is:
an arbitrary number would reach the synthesiser untested.

**D3 — The rate is part of `SpeechSettings`.** It travels with `voices`, `androidVoices` and
`remoteVoices` through `VoicePreference.load` and `watch`; `storedVoicePreference` adds the key to
its change filter. The speaker keeps the rate with the rest of its settings and hands it to the
engine on each `speak`. No second subscription, no new message between the content script and
Réglages.

**D4 — The engine takes the rate; the speaker never touches the utterance.** `SpeechEngine.speak`
gains a `rate` argument and `browserSpeechEngine` sets `utterance.rate`. Everything about
eligibility stays where it was: the rate is applied to the voice already chosen, so nothing here
can select a different voice or reach a remote one.

**D5 — Next reading, not live.** `SpeechSynthesisUtterance.rate` is fixed when the utterance
starts on the engines Lingua targets; re-timing a reading under way would mean cancelling and
re-speaking from an offset the API does not give. The reader presses again.

**D6 — The preview speaks at the chosen rate.** The "hear this voice" button already speaks a
sample sentence through the speaker, so it picks the rate up for free; the point is that the reader
tunes the speed against the very sentence they will hear, not against a different path.

## Risks / Trade-offs

- **"0.25× is slow" is not the same on every voice.** Chrome's remote and local voices, macOS and
  iOS voices and Android's engine scale differently, and some clamp at a floor. The presets are a
  starting point, confirmed on the devices during the on-device pass; if one platform makes the
  lowest step unusable the list changes, not the mechanism.
- **Firefox for Android depends on Android's engine honouring `rate`.** If it ignores it, the
  control would silently do nothing there. The on-device pass reads this; if confirmed, the
  control is hidden on that target rather than shown dead.
- **A fifth control in a block that is already long.** Placed directly under the voice select, with
  the same builder, it adds a row and no new surface.

## Migration Plan

None. The key is new and ignored by older builds; an absent key means normal, so nothing is
migrated and nothing is synchronised. Rolling back is removing the control: the key is left behind
and harmless.

## Open Questions

1. The exact preset values (0.25 / 0.5 / 0.75 / 1) are a proposal, to be confirmed on the devices.
2. Whether Firefox for Android honours `rate` through Android's engine — read in the on-device
   pass (task 4.1), not decided ahead.
