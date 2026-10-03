## 1. The database

- [ ] 1.1 `model-db.ts`: records per model, the legacy record read as its model's, `prune(keep)` (design D1). Specs:
  - a legacy record keeps the en-fr model complete;
  - two models complete apart;
  - pruning one keeps the files the other names;
  - erasing still deletes everything.

## 2. The state and the controller

- [ ] 2.1 `model-manifest.ts`: `modelsFor(catalogue, languages)` (design D2). Specs: a union without duplicates, in order; a language without a route needs nothing.
- [ ] 2.2 `setting.ts`: `ready { models }` and `missing { models, total }`, parsed defensively, a legacy `ready` read with no model (design D3). Specs.
- [ ] 2.3 `model-controller.ts`: the needed models from `languages()`; the download of what is missing; reconciling into `ready`, `missing` or `removed`; pruning; `ready(language)`. Specs, with a catalogue of two routes sharing en-fr:
  - every needed model downloaded;
  - a language added makes `missing`, and `resume` downloads it;
  - a language removed prunes its model and keeps en-fr;
  - `ready(language)` per route.
- [ ] 2.4 `translation-setting.ts`: `missing` says what the missing model costs and offers « Télécharger ». Spec.

## 3. The download

- [ ] 3.1 `model-download.ts`: `downloadModels(models, deps)`, one progress over every model, each marked complete once its files are in (design D4). The worker, `DownloadHost`, the offscreen bridge and the background pass the list. Specs.

## 4. Translation in the document's language

- [ ] 4.1 `TranslationRequest.language` and the warm message's language. These pass through the wire, the messaging port (whose specs check the forwarding), the answer memory (whose key includes the language) and the keep-warm port.
- [ ] 4.2 The selection card asks in the session's language (design D5). Spec: the request names the document's language.
- [ ] 4.3 The relay, the channel, the offscreen bridge and the engine worker translate in a language: a route of one model is loaded per id, and a longer route or no route answers « unavailable ». The background checks `ready(language)`. Specs for the channel, the relay and the background's answer.

## 5. Gates

- [ ] 5.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.

  End to end on Chromium: turning the setting on downloads en-fr, an English sentence is translated, and turning it off deletes it.
- [ ] 5.2 `openspec validate generalise-lingua-translation-model-state --strict` passes. In `docs/lingua/spanish-programme.md`, change 17 is marked done.
