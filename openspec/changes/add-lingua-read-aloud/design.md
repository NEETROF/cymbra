## Context

The card (`src/reading/wordpopup.ts`) is one view in a closed shadow root, shown by
`selection-card.ts` for a page token, a loose word or a phrase. Every content it receives
already carries the two texts this change speaks: `surface`, the selection as it appears on the
page, and `sentence`, the sentence it was found in by position (`sentenceAndSelection`). The
card is shown pending, then completed exactly once; it hides on its close button, Escape, a
click off it, a gesture and a scroll (`content.ts`).

Lingua studies one language today — English, named by `"en"` literals in `sync/` and the
translation host — and promises that no page text leaves the device ("Minimal permission
posture", "No network requests"). The extension runs in five places: Chromium, Firefox desktop
and Android (same zip), Safari macOS and iOS (`dist-safari/`, bundled by `apps/lingua-apple`).

Each of them exposes the Web Speech API's `speechSynthesis` to the content script, backed by the
platform's voices. They differ in what they list: Chrome on macOS lists the Apple voices —
alphabetically, so the novelty voices (`Albert`, `Bad News`, `Bubbles`…) and the Eloquence ones
(`Eddy (English (United States))`…) come before `Samantha` — plus Google voices that synthesise
on Google's servers and say so with `localService: false` (`Google US English`, `Google UK
English Female`, `Google UK English Male`). Chrome often lists nothing until `voiceschanged`
fires. Safari on macOS exposes Apple identifiers in `voiceURI`
(`com.apple.voice.super-compact.en-US.Samantha`), lists no Eloquence voice, and marks **every**
voice `default: true` — 68 out of 68 on the capture; Chrome marks one (the system voice, `Daniel`
on the capture) and uses the name as `voiceURI`. Firefox on macOS marks one too, lists no novelty
and no Eloquence voice — only the legacy `Fred`, `Junior`, `Kathy`, `Ralph` — and wraps the Apple
identifier in its own URN (`urn:moz-tts:osx:com.apple.voice.super-compact.en-US.Samantha`). Measured on 2026-09-24, captures in
`apps/lingua-extension/test/fixtures/voices/`.

## Goals / Non-Goals

**Goals:**

- Hear the selection as seen and its whole sentence from the card, on all five targets.
- Never hand page text to a remote voice, by construction rather than by luck of the default.
- A sensible voice without any setup, and a way out when the automatic choice is wrong.
- Speech that never outlives the card that controls it.
- All decisions in a DOM-free, tested module; `content.ts` stays the thin caller.

**Non-Goals:**

- A slower speaking rate, or any rate control.
- Highlighting the spoken words in the page (`boundary` events).
- Listen buttons on the review cards of the deck (the speaker is reusable for it).
- A voice bundled with the extension (Piper/Kokoro-style WASM): tens of megabytes per voice and
  the same delivery questions as the translation model.
- Any remote voice, even opt-in: that is a change to the privacy promise, not to the card.

## Decisions

### D1. The Web Speech API, called from the content script

The card speaks through `window.speechSynthesis` in the context that owns the card, and
`speak()` is called synchronously inside the button's click handler.

- The press is the user activation. Chrome refuses `speak()` without one, and Safari on iOS only
  produces sound from inside a gesture. A message to the background first would spend the
  gesture on Firefox and Safari, exactly as it does for `sidebarAction.open` (see the
  browser-extension skill), so speech stays where the click is. Nothing is awaited before the
  call: the voice and the preference are resolved before the card is pressed (D4).
- Content scripts run in an isolated world (Chrome) or behind Xray wrappers (Firefox), and Safari
  isolates them too: a page that replaces `speechSynthesis` does not replace the extension's.
- The same API is present in extension pages, so the settings preview (D6) in the side panel
  uses the same module with that page's own `speechSynthesis`.
- Where `speechSynthesis` does not exist, the speaker reports no eligible voice and the card
  shows no listen row. Nothing is feature-sniffed per browser.

_Alternatives considered._ `chrome.tts` — Chromium only (Firefox and Safari implement no `tts`
API) and a manifest permission, for a capability the page API already gives everywhere. An
offscreen document or the background — loses the gesture (above) and adds a hop for nothing. A
remote synthesis service — breaks "no page text leaves the device". A bundled voice — see
Non-Goals.

### D2. Eligible = on-device and in the studied language; the voice is always set explicitly

A voice is eligible when `localService === true` and the primary subtag of its `lang`
(`en-US` → `en`, compared case-insensitively, `_` read as `-` for Android's `en_US`) is the
studied language. Every utterance names its `voice` and that voice's `lang`; the extension never
sets `lang` alone and lets the browser match, because Chrome may resolve a bare `en-US` to a
Google voice that synthesises remotely.

The studied language is passed in, not read inside the module: `content.ts` and the settings
host give it one constant (`"en"` today), so a second pack pair changes one call site.

`localService` is what the browser reports. For Chrome's own Google voices it is `false`, which
is precisely the case this rule exists for. For platform voices — Apple's, Windows', Android's
engine — the platform synthesises, and whether that platform's engine reaches the network is the
device owner's setting, outside a web page's reach. The on-device pass checks what Firefox for
Android reports (task 6.3) — it speaks through Android's engine; Chrome on Android runs no
extension, so it is no target here.

### D3. The automatic choice, and the novelty voices

`pickVoice(voices, lang, preferred)` is a pure function:

1. the preferred voice (by `voiceURI`), if it is still listed and eligible;
2. otherwise the eligible voice marked `default`, only when it is the **only** voice of the whole
   list so marked — Safari marks them all, which says nothing;
3. otherwise the first eligible voice by tier — Apple's enhanced/premium voices, then ordinary
   voices, then the deprioritised ones — and within a tier `en-US`, then `en-GB`, then any other
   region, then the browser's order;
4. none when nothing is eligible.

The deprioritised voices are Apple's novelty voices (`Albert`, `Bad News`, `Bahh`, `Bells`,
`Boing`, `Bubbles`, `Cellos`, `Good News`, `Jester`, `Organ`, `Superstar`, `Trinoids`,
`Whisper`, `Wobble`, `Zarvox`), the Eloquence voices (`Eddy`, `Flo`, `Grandma`, `Grandpa`,
`Reed`, `Rocko`, `Sandy`, `Shelley`) and the legacy ones (`Fred`, `Junior`, `Kathy`, `Ralph`),
matched on the name with any parenthesised suffix removed (Chrome's `Eddy (English (United
States))`), or on the family of the Apple identifier in `voiceURI`
(`com.apple.speech.synthesis.voice.`, `com.apple.eloquence.`) wherever it sits — Firefox wraps it
as `urn:moz-tts:osx:com.apple…` — and on the family, not the name inside it, because Safari's
identifiers do not always repeat the name (`Wobble` is `…voice.Deranged`, `Jester`
`…voice.Hysterical`, `Superstar` `…voice.Princess`). Apple's list has not moved in years, it is only ever a ranking — a
deprioritised voice still speaks when it is the only one — and the voice picker is the way out
when a platform lists something the ranking gets wrong. The tests run the ranking over voice
lists captured on real devices (task 1.1 for macOS, 6.3 for the phones), not over lists imagined
for the test.

On the captures: Chrome macOS picks `Daniel` (its one default voice) among 41 eligible voices, 6
of them ordinary; Safari macOS, where the default says nothing, picks `Samantha` among 25, 6 of
them ordinary; Firefox macOS picks `Daniel` (its one default voice) among 10, 6 of them ordinary.
The same Mac can therefore start on two different voices in two browsers; the
picker settles it per browser.

_Alternative considered._ An allow-list of known good names (`Samantha`, `Daniel`, `Microsoft
David`…) — it fails closed on every platform nobody listed, where a deny-list only fails to
improve the order.

### D4. One speaker per context, one utterance at a time

`createSpeaker(synth, lang, preference)` returns a `Speaker`:

- `available()` — whether an eligible voice is listed now;
- `speaking()` — the key of the current utterance (`"selection"` or `"sentence"`) and its text,
  or null;
- `speak(key, text)` — cancels whatever is speaking and speaks `text` with the chosen voice;
- `stop()`;
- `subscribe(listener)` — called when `available()` or `speaking()` changes.

The voice list is read at creation and again on every `voiceschanged`; the preference is read at
creation and followed through `storage.onChanged` on its key. The click therefore never waits:
when the button is pressed, the voice is already known.

The current utterance is identified by the object itself. `cancel()` makes the old utterance fire
`end` or `error: "interrupted"` asynchronously — on Chrome after the new one has started — so an
event only clears the state when it belongs to the utterance still current. `interrupted` and
`canceled` are the extension's own doing; any other error resets the button without a message
(no raw technical error in the UI) and is logged to the console.

`cancel()` empties the frame's whole queue, the page's own utterances included. That only
happens on the reader's press, on a page they are reading, and is accepted.

### D5. The listen row on the card

`createCard` takes the speaker (`WordPopupOptions.speaker`, optional so every existing test and
host is unchanged). The row sits under the headword lines, before the answer, because the
selection's pronunciation belongs to the headword and the sentence's to the page, not to the
gloss.

- **Texts.** The selection button speaks `content.surface` — the text as seen, so `ran` is heard
  as `ran`, and a phrase as its words. The sentence button speaks `content.sentence`, and is left
  out when that sentence, trimmed, with runs of whitespace collapsed, final punctuation removed
  and case ignored, equals the selection, or is empty.
- **Labels** (French, to be approved on device, task 6.5): `▶ Mot` on a word card, `▶ Sélection`
  on a selection card, `▶ Phrase`; the speaking button reads `■ Arrêter`. Each has a full
  `aria-label` (`Écouter le mot`, `Écouter la sélection`, `Écouter la phrase`, `Arrêter la
  lecture`). Text nodes only.
- **State.** The row is rendered from `speaker.available()` and `speaker.speaking()` on every
  `show` and on every speaker notification, so a list announced late adds the row to the card
  already open, and the end of an utterance restores the label.
- **Pending card.** The row does not depend on the engine: it is offered on the pending card.
- **Lifetime.** `hide` stops the speaker. `show` stops it unless the text being spoken is one
  of the two texts the new content offers — which is exactly the case of a pending card
  completing, and never the case of another word. `content.ts` also stops it when the page's
  visibility becomes `hidden` and when the reader is switched off.
- **The page selection.** The listen buttons call `preventDefault()` on `pointerdown` and
  `mousedown`, so pressing one keeps the reader's selection — and on a touch device the
  platform's callout — as it was. The card's status buttons are untouched: they close the card.

### D6. The voice in Réglages

`mountSettings` gains a "Lecture à voix haute" block, built by the same function in the side
panel, the in-page drawer and the toolbar popup, from a speaker the host passes in
(`SettingsOptions.speaker`):

- a select whose first option is `Automatique (<voice name>)` — the voice D3 would pick — then
  the ordinary eligible voices as `<name> — <region>`, then the deprioritised ones of D3 in a
  group `Autres voix` at the bottom: listed, since a reader may want one, but out of the way —
  on the Chrome macOS capture they are 35 of 41, and in the browser's order they would bury
  `Samantha` under `Bubbles` (group chosen by the founder on 2026-09-24 over hiding them);
- a button `▶ Écouter` that speaks a fixed English sample with the selected voice;
- no block at all when no voice is eligible (the Firefox and Chrome desktops without a local
  English voice, for instance).

The preference is the chosen `voiceURI`, or nothing for the automatic choice, in
`chrome.storage.local` under a new key (`cymbra-lingua-voice`): a preference like the HUD
toggle, small and read before any round-trip, never in the reader's IndexedDB store. It is not
synchronised across devices: voice identifiers are per platform.

**The toolbar popup had its own Réglages.** Dogfooding found the block missing there: the popup
carried a hand copy — markup in `popup.html`, wiring in `popup.ts` sending `setLevel`,
`setCalibration` and `reset` to the tab's content script — and every block added to the
shared view since was absent from it. The popup now mounts `mountSettings` like the side
panel: with an engine port of its own, created the first time Réglages open, persisting the
backup to the store, which the tab restores (`onExternalChange`) exactly as it does after a
change in the side panel. The copy, its styles and the three content-script messages only it
sent are removed, and `test/lint-settings-hosts.spec.ts` fails the build when a host stops
calling `mountSettings` or a page or module holds a Réglages block of its own (recognised by
the builder's block titles).

### D7. Tests

- `test/speech.spec.ts` drives `speech.ts` through a hand-written synthesiser double — jsdom has
  no `speechSynthesis`, and the double has to fire `voiceschanged`, `end` and `error` in the
  orders the browsers do, which a scripted fake does more plainly than a mock. It covers
  eligibility, D3 over the captured lists, the stale-event rule, `interrupted` versus a real
  error, and the preference.
- `test/wordpopup.spec.ts` covers the row: which buttons, labels, toggling, switching, the
  sentence equal to the selection, no voice, a late list, a pending card completing while
  speaking, `hide` stopping, and `pointerdown` prevented.
- The settings block is tested where the other blocks are.
- `speech.ts` is not added to the coverage exclude list. `content.ts` stays excluded and only
  wires `window.speechSynthesis`, the language constant and the storage area.

## Risks / Trade-offs

- [A platform reports a network voice as local] → The rule relies on `localService`. Chrome's
  remote voices report it correctly on desktop; Firefox for Android, which speaks through
  Android's engine, is checked on device (task 6.3). If it lists a voice that reaches the network as local, that voice goes on the
  deprioritised list and the finding is written into the extension README.
- [No eligible voice on a desktop] → Chrome on Linux or ChromeOS can list only Google's remote
  voices: the row is absent there. This is the privacy promise working, and the README says how
  to install a system voice.
- [Firefox for Android without Web Speech] → If GeckoView does not expose `speechSynthesis`, or
  lists no voice, the row is simply absent (D1). Checked on device.
- [Safari on iPhone and the silent switch] → Web Speech may follow the ring/silent switch. The
  extension cannot change the audio session; checked on device and written in the README.
- [`speak` right after `cancel` is dropped by some engines] → Switching is checked on every
  target. If one drops it, the fix is to speak on the next task: transient activation outlives
  the click long enough on Chromium; Safari on iOS is the target to watch.
- [A very long "sentence"] → A block without punctuation reads as one sentence. It is spoken
  whole; the stop button is on the card, and the card's own scroll dismissal stops it too.
- [The ranking ages] → Apple adds voices, other platforms name theirs differently. The picker
  is the way out and the captured lists make a regression visible in review.

## Migration Plan

None. The preference key is new and ignored by older builds; nothing is migrated, synchronised
or stored in the reader's database. Rolling back is a revert and a release.

## Open Questions

None. Three choices this design makes were confirmed by the founder on 2026-09-23:

1. The selection button speaks the text **as seen** (`ran`), not the dictionary form (`run`).
2. The voice picker is **in** this change, not a follow-up.
3. **No rate control** in this change.

What remains open is measured, not decided: what the phones list and what Firefox for Android
reports as remote, and the iPhone's silent switch — all read during the on-device pass (task 6.3),
not captured ahead.
