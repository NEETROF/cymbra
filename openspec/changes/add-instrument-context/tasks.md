## 1. The context itself

- [x] 1.1 Persisted instrument context (keyboard | drums) beside the existing play preferences, defaulting to keyboard, seeded into the player/app state
- [x] 1.2 One predicate `drumsVisible`, read from the drum-visibility flag, used by **every** consumer — the modal trigger, the switcher's presence, the hub's drum option. Never duplicated, or two surfaces will disagree
- [x] 1.3 A durable "choice already offered" marker that survives relaunch **and a sign-out/sign-in cycle** — the naive implementation re-prompts on every sign-in
- [x] 1.4 Notifier tests: default is keyboard, the value round-trips a relaunch, and the offered-marker survives a sign-out/sign-in

## 2. Stickiness — the property that must not erode

- [x] 2.1 Nothing in the player path writes the context. Assert it: open and play a keyboard score with the context on drums, and check the context afterwards
- [x] 2.2 Same assertion for a score opened from a deep link or from the library — an incoming link must not reconfigure the app
- [x] 2.3 The hub's instrument filter is **seeded** from the context and then independent, retained for the session; adjusting it does not write back. Test both directions

## 3. The one-time choice

- [x] 3.1 Modal shown on the **home** — the next time the user is on, or arrives at, the home while `drumsVisible` is true — after sign-in while beta-scoped, at first launch once global. One rule, no phase-specific branch; a flip landing mid-play defers to the next arrival at the home, never a dialog over the player
- [x] 3.2 Wording is a **choice**, not an unlock ("Par quoi veux-tu commencer ?"), states that it can be changed later, and neither option blocks — see `mockups/context.html`
- [x] 3.3 Never shown when drums are not visible; shown at most once
- [x] 3.4 Localise fr/en
- [x] 3.5 Widget tests: shown on first visibility on the home, deferred (not shown) when the flip lands mid-play then shown on returning home, not shown again, never shown when drums are invisible

## 4. The switcher

- [x] 4.1 Segmented control in the home header, present **only** when drums are visible; today's home is unchanged for everyone else, with no new control and no question
- [x] 4.2 Switching re-seeds the discovery surfaces (home sections, hub filter, surfaced courses) — including a hub filter the user had adjusted this session: the explicit context act outranks the working state, where navigation alone never re-seeds. The sound picker family is deliberately absent — it follows the score, and its filtering belongs to `add-drum-audio-channel`
- [x] 4.3 Read through the notifier and react to state per the Riverpod layering rules — the screen never calls a service, and no provider imperatively invalidates a sibling
- [x] 4.4 Decide and implement the phone-class placement; the mockup's position is comfortable on a tablet and competes with the header on a phone
- [x] 4.5 Widget test for presence/absence by visibility, and for the re-seeding

## 5. Degradation — both failure modes are silent

- [x] 5.1 Context names a not-currently-visible instrument (campaign closed, or a snapshot not yet resolved) → the home renders as keyboard **silently**, no error. Presentational only: the stored context and the offered marker are never written by a visibility change
- [x] 5.2 Context in force with nothing to show → an explicit invitation naming the cause and offering to switch, never a bare empty screen; the courses surface under drums (no drum course exists yet) reuses the same invitation; localised fr/en
- [x] 5.3 Tests for both — plus the two round trips a persisted fallback would lose: a cold start with an unresolved snapshot (home renders keyboard, reapplies drums once resolved, stored value untouched) and a sign-out/sign-in cycle (context still drums, no re-offer)

## 6. Bundled drum scores

- [x] 6.1 Author ~4 short drum scores across the beginner / intermediate / advanced folders. **Author, do not source** — a basic groove is an idiom, not a copyrightable work, and this is what the existing bundled scores already are
- [x] 6.2 Do **not** promote the public-domain drum scores found in the catalog: crawler-classified, `pending`, never human-reviewed. A classifier's output is not a licence confirmation, and bundling is a higher bar than cataloguing
- [x] 6.3 Between them, exercise unpitched notes, the part-list instrument table, the percussion clef, two voices on one staff, and open/closed hi-hat
- [x] 6.4 Declare them in `pubspec.yaml` and record each in `assets/scores/CREDITS.md` with its status, following the existing table
- [x] 6.5 List the bundled drum scores **only where drums are visible**: while `drumsVisible` is false they appear in no listing — not the welcome try surface, not the bundled sections — though the bytes ship in every binary; widget test
- [x] 6.6 Confirm a bundled drum score opens end to end without a backend: the cascade lanes and the pad strip render, nothing crashes. The full core loop — gauge and end-of-session summary — is a **general-availability prerequisite** owned by `add-drum-audio-channel`, `add-drum-input-mapping` and `add-drum-scoring`, and is verified there

## 7. Gates

- [x] 7.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 7.2 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 7.3 `openspec validate add-instrument-context --strict`

## 8. Manual verification

- [x] 8.1 As a campaign member signing in for the first time: the modal appears once, either choice proceeds, and the switcher is then visible on the home — VALIDÉ 2026-08-25 en **production** (campagne `midi-drums`, flag `drums.enabled` scopé `beta:midi-drums`) : le modal se présente une fois au premier sign-in du membre, le choix passe, et le sélecteur d'instrument est ensuite présent sur l'accueil
- [x] 8.2 Sign out and back in: the modal does **not** reappear — VALIDÉ 2026-08-25 en **production** : le choix est bien retenu au-delà de la session, le modal ne se represente pas
- [x] 8.3 As a non-member: no modal, no switcher, no instrument question anywhere — the home is byte-for-byte today's, and no bundled drum score is listed anywhere — VALIDÉ 2026-08-25 en **production**, depuis un compte non-membre (donc pas depuis une session staff, qui matche tous les scopes bêta et aurait rendu le test vacuous) : aucun modal, aucun sélecteur, aucune partition batterie listée
- [x] 8.4 With the context on drums, open a keyboard score from the library: it plays normally and the context is unchanged on return — VALIDÉ 2026-08-25 en **production** : le contexte batterie ne contamine pas l'ouverture d'une partition clavier, et survit intact au retour
- [x] 8.5 Set the hub's filter to the other instrument, navigate away and back: the filter is retained and the home still reflects the context — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 8.6 Close the campaign without widening the scope: the home renders as keyboard silently, still works, and the stored context is untouched — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [ ] 8.7 Without an account, with drums visible, pick drums and open a bundled drum score: the cascade lanes and the pad strip render and nothing crashes — playing it through to the gauge and summary is the general-availability gate, verified once audio, input and scoring have landed
- [x] 8.8 With the context on drums, kill and relaunch: the home may render keyboard while the flags resolve, then shows drums again — the context was not rewritten and the modal did not reappear — VALIDÉ 2026-08-25 en production (backend 0.21.1)
