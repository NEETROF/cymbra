## 1. Speech module

- [ ] 1.1 In `src/reading/speech.ts`, add the preset list (`SPEECH_RATES`), the default (1) and a
  helper reading any stored value as a preset or normal; extend `SpeechSettings` and
  `DEFAULT_SPEECH_SETTINGS` with the rate.
- [ ] 1.2 Give `SpeechEngine.speak` a rate; have the speaker pass the current settings' rate on each
  reading, and `browserSpeechEngine` set `utterance.rate`.
- [ ] 1.3 Tests in `speech.spec.ts`: default rate, a preset reaching the engine, an unknown stored
  value read as normal, a change picked up by the next reading and not the current one, and the
  eligible voice unchanged whatever the rate.

## 2. Storage

- [ ] 2.1 In `src/state/storage.ts`, add `SPEECH_RATE_KEY` with `loadSpeechRate` / `saveSpeechRate`,
  fold it into `storedVoicePreference` (load and the change filter).
- [ ] 2.2 Tests: absent key reads as normal, round trip, a change notifies `watch`.

## 3. Réglages

- [ ] 3.1 In `src/reading/settings-view.ts`, add the speed choice under the voice choice in the
  read-aloud block (same builder for the side panel, the drawer and the popup), hidden whenever
  the voice choice is hidden; the voice preview speaks at the chosen rate.
- [ ] 3.2 Labels in `src/i18n/{en,fr,es}/settings.ts` ("Vitesse de lecture", the preset names);
  the three locales stay aligned.
- [ ] 3.3 Tests in `settings-view.spec.ts`: shown with the voice block, hidden without it, default
  selected, a change saved, and the preview using the rate.

## 4. Dogfooding

- [ ] 4.1 On-device pass: macOS Chrome, Windows Chrome, Firefox desktop, Firefox for Android,
  Safari macOS and iPhone — are the presets audibly distinct and is the slowest usable? Does
  Firefox for Android honour `rate`? Adjust the list, or hide the control on a target that ignores
  it.

## 5. Release

- [ ] 5.1 `openspec validate add-lingua-read-aloud-speed --strict`; lint, typecheck and the
  extension's tests with coverage ≥ 80 %.
- [ ] 5.2 Ordinary extension release, then `lingua-apple-release` with `deliver` for the Safari
  variant. Archive the change after the release, once `add-lingua-read-aloud` is archived.
