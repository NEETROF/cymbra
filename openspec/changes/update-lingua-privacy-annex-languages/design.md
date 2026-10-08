# Design — update-lingua-privacy-annex-languages

## Context

See proposal.md (Why). Annex B today (French line numbers; the English page has the same rows):
« les mots anglais » (155); « un modèle de traduction (25,8 Mo…) » (162–171); « voix anglaise »
(173–178); the sync table (180–187): statuses « …et niveau d'anglais déclaré », the deck « …sa
traduction, la langue de cette traduction et son état de révision — sans l'adresse de la page »,
statistics « …avec votre langue maternelle (celle des traductions) », the installation id; the
erasure path in French interface labels (193–196); `updated: 29/09/2026`. The server stores a
`language` on statuses, levels, cards and days. The catalogue's routes: en-fr one model (25,8 Mo),
es-fr two (52,0 Mo), es-en one, en-es one (change 25).

## Goals / Non-Goals

**Goals:**
- The annex true for every shipped pair and every pair the matrix adds, in each published language.

**Non-Goals:**
- The rest of the policy; the requirement held by `add-lingua-remote-translation`.

## Decisions

### D1 — Name the languages generically, the facts exactly

« les langues que vous étudiez », and each synced row says « pour chaque langue étudiée »; the
models: « le ou les modèles de vos paires de langues — un modèle (environ 26 Mo) quand la traduction
est directe, deux (environ 52 Mo) quand elle passe par l'anglais » — the catalogue's sizes, reviewed
by each change that adds a model; a pair added later: « les modèles d'une paire ajoutée ensuite ne sont
téléchargés que lorsque vous le demandez » (its models are marked missing and wait for the reader,
`translate/host/model-controller.ts`);
the voice: « une voix de la langue lue, installée sur votre appareil ».

### D2 — The account's language

Its own paragraph in the account part of Annex B, not a row of the sync table: the extension sends a
language on sign-up, on a code resend, on a password reset (signed out) and when a password is added
(`account/flow.ts`), Cymbra ID stores it on the account, and nothing syncs it. Worded so it is true
today (the browser's whole tag), under either answer to change 17's task 3.3, and after change 20:
« l'extension indique à Cymbra une langue — celle de l'interface de Lingua ou celle de votre
navigateur — que votre compte garde comme sa langue : Cymbra vous écrit dans cette langue
(vérification, réinitialisation de mot de passe) et vos autres apps Cymbra peuvent l'adopter » (M12).
The delta spec's requirement first named the interface language alone; it now names the language
the extension sends — the interface language or the browser's — kept as the account's language (its
name and the scenario's unchanged). The language settings themselves stay on the device and are not
stored as such, while the native language reaches Cymbra as each card's gloss language and with each
day's statistics (changes 11, 12). The site that publishes this annex is deployed after
`add-lingua-native-language-server` is deployed and checked from outside (its 5.2) and before the
store release that carries `add-lingua-native-language-sync-client` (its 6.1).

### D3 — The erasure path

English and Spanish name it by function only ("on the account page, the option that erases your
Lingua data") — their interface labels are drafts until change 33; the French keeps « Compte → Tes
données → Effacer mes données Lingua ».

## Risks / Trade-offs

- **A disclosure that lags the code** → the annex says what changes 11 and 12 do, and words the
  account's language so it holds before and after changes 17 and 20; a later change that sends a new
  field edits it in the same pull request, as change 12 did.

## Migration Plan

Published with the first site deploy after `add-lingua-native-language-server`'s 5.2 and before the
store release that carries `add-lingua-native-language-sync-client` (task 3.2).
