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
models: « le ou les modèles de vos paires de langues — un seul (environ 25 Mo) pour une langue lue
directement, deux (environ 52 Mo) pour l'espagnol lu en français » — the sizes from the catalogue;
the voice: « une voix de la langue lue, installée sur votre appareil ».

### D2 — The account's language

A row in the account part of Annex B: « la langue de l'interface de Lingua, envoyée à Cymbra ID pour
choisir la langue de ses e-mails » (M12, change 17).

### D3 — The erasure path

English and Spanish name it by function ("the Lingua erasure in the account page") and quote the
interface's label in that language once change 17 ships it; the French keeps « Compte → Tes
données → Effacer mes données Lingua ».

## Risks / Trade-offs

- **A disclosure that lags the code** → the annex says what changes 11, 12 and 17 do; a later
  change that sends a new field edits it in the same pull request, as change 12 did.

## Migration Plan

Published with the owner's next site deploy.
