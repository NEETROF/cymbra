## 1. Voice lists from real devices

- [x] 1.1 Capture `speechSynthesis.getVoices()` after `voiceschanged` — `name`, `lang`, `localService`, `default`, `voiceURI` — on Chrome, Safari and Firefox for macOS, into `apps/lingua-extension/test/fixtures/voices/<target>.json`; the iOS Simulator's list (`safari-ios-simulator.json`) is kept, but it is the host Mac's Safari list voice for voice and does not stand in for an iPhone
- [x] 1.2 Mobile lists are not captured ahead (founder's call, 2026-09-24): Chrome Android, Firefox for Android and a real iPhone are read during the on-device pass (6.3), which records what each lists and whether Chrome Android reports any voice as remote

## 2. The speaker

- [x] 2.1 Add `src/reading/speech.ts`: `isEligible(voice, lang)` (on-device, primary subtag, `_` read as `-`) and `pickVoice(voices, lang, preferred)` with the tiers of D3 and the deprioritised Apple names and identifiers
- [x] 2.2 In the same module, `createSpeaker(synth, lang, preference)` per D4: voice list read at creation and on `voiceschanged`, preference read at creation and followed on its key, `available`, `speaking`, `speak(key, text)` naming the voice and its `lang` synchronously, `stop`, `subscribe`; a `synth` that is absent gives a speaker with no voice
- [x] 2.3 Events: only the current utterance's `end`/`error` clears the state; `interrupted` and `canceled` are silent, any other error resets and logs to the console
- [x] 2.4 `src/state/storage.ts`: `VOICE_KEY = "cymbra-lingua-voice"`, `loadVoice` / `saveVoice` (a `voiceURI`, or nothing for the automatic choice)
- [x] 2.5 `test/speech.spec.ts` with a hand-written synthesiser double: eligibility (remote voice refused even as default, `en_US`, other languages), `pickVoice` over every fixture of 1.1 (never a novelty voice while an ordinary one exists; the default eligible voice first; a removed preference falls back), a late list, the stale `end` after a switch, `interrupted` versus `synthesis-failed`, no `synth`

## 3. The card

- [x] 3.1 `WordPopupOptions.speaker` (optional) passed to `createCard`; the listen row under the headword lines: `▶ Mot` / `▶ Sélection` and `▶ Phrase`, `■ Arrêter` while speaking, full `aria-label`s, text nodes only; the sentence button left out when the normalised sentence equals the selection or is empty
- [x] 3.2 The row rendered from `available()` / `speaking()` on every `show` and every speaker notification, offered on a pending card too
- [x] 3.3 `hide` stops the speaker; `show` stops it unless the text being spoken is one of the new content's two texts
- [x] 3.4 `preventDefault()` on the listen buttons' `pointerdown` and `mousedown`
- [x] 3.5 `src/styles/wordpopup.css`: the row and its buttons from `tokens.css` only, a `.listen[hidden]{display:none}` rule if the row is a flex row; `lint-hex` and `lint-lemma` pass
- [x] 3.6 `test/wordpopup.spec.ts`: one test per scenario of "The card reads the selection and its sentence aloud" and "Closing the card silences it" that the card owns, plus no row without a voice, the row appearing on a late list, and the prevented `pointerdown`

## 4. Réglages

- [x] 4.1 `SettingsOptions.speaker`; the "Lecture à voix haute" block in `mountSettings`: `Automatique (<voice>)`, the ordinary eligible voices as `<name> — <region>`, then the deprioritised ones in an `Autres voix` group at the bottom, `▶ Écouter` speaking a fixed English sample with the selected voice, saving through `saveVoice`; no block without an eligible voice
- [x] 4.2 Styles in `settings.css` from `tokens.css` only
- [x] 4.3 Tests next to the existing settings tests: the block's options (on the Chrome macOS fixture: 6 ordinary voices, then the 35 others under `Autres voix`; no group when there are none, as on a list without novelty voices), the automatic label, saving and clearing the preference, the removed voice showing the automatic choice selected, no block without a voice
- [x] 4.4 The toolbar popup's Réglages are `mountSettings` on an engine port of the popup (created when Réglages first open), not its hand copy: `popup.html`/`popup.ts`/`popup.css` lose the copy, the content script loses `setLevel`/`setCalibration`/`reset`, and `test/lint-settings-hosts.spec.ts` refuses a host without `mountSettings` or a page/module with a Réglages block of its own

## 5. Wiring

- [x] 5.1 `content.ts`: one speaker on `window.speechSynthesis` with the studied-language constant and the preference area, passed to `WordPopup` and to the drawer's settings; stopped when `visibilityState` becomes `hidden` and when the reader is switched off
- [x] 5.2 The side panel passes its own speaker to `mountSettings`
- [x] 5.3 `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (coverage ≥ 80 %, `speech.ts` not excluded), `yarn build`, `yarn check:variants`; grep `dist-chromium/`, `dist-firefox/` and `dist-safari/` for the speaker in the content bundle

## 6. On devices

- [x] 6.1 Chrome macOS with the real pack (`yarn gen:pack:real`): a word, an inflected word (heard as seen), a phrase, the sentence, a selection that is its whole sentence (one button), stop, switch, close while speaking, Escape, scroll, another word while speaking, another tab while speaking, a pending card completing while speaking, the drag selection kept after a press; the automatic voice is not a novelty or Eloquence voice; the Google voices never speak (DevTools network panel quiet, and a remote voice chosen as the system default changes nothing) — passed by the founder on Chrome, Firefox and Safari for macOS, 2026-09-24
- [ ] 6.2 Offline on Chrome macOS: the word and the sentence are spoken
- [ ] 6.3 Chrome Android, Firefox for Android and Safari on iPhone (a local build of `apps/lingua-apple`): run the capture one-liner of 1.1 and save each list as a fixture (Chrome Android: which voices report `localService: false`, and whether the device's TTS engine is set to use network voices — written into the design's first risk); then the row appears (or is absent where no voice exists, and the reason is written down), speech starts from the tap, switching works, the silent switch's effect is recorded
- [x] 6.4 Réglages in the side panel and in the drawer: choosing a voice changes the card's voice on another open tab, `▶ Écouter` plays the sample — passed on the three macOS browsers, 2026-09-24, the toolbar popup included
- [ ] 6.5 The French labels of D5 and the block's copy, approved by the founder as written, fit the card on a phone without being cut

## 7. Close

- [ ] 7.1 `README.md` of the extension: read-aloud, on-device voices only and why, the Chrome desktop without a local English voice, the findings of 6.3
- [x] 7.2 `openspec validate add-lingua-read-aloud --strict`
- [ ] 7.3 After the extension release, dispatch `lingua-apple-release` with `deliver`, so the Safari variant carries the same build
