## 1. The context itself

- [ ] 1.1 Persisted instrument context (keyboard | drums) beside the existing play preferences, defaulting to keyboard, seeded into the player/app state
- [ ] 1.2 One predicate `drumsVisible`, read from the drum-visibility flag, used by **every** consumer — the modal trigger, the switcher's presence, the hub's drum option. Never duplicated, or two surfaces will disagree
- [ ] 1.3 A durable "choice already offered" marker that survives relaunch **and a sign-out/sign-in cycle** — the naive implementation re-prompts on every sign-in
- [ ] 1.4 Notifier tests: default is keyboard, the value round-trips a relaunch, and the offered-marker survives a sign-out/sign-in

## 2. Stickiness — the property that must not erode

- [ ] 2.1 Nothing in the player path writes the context. Assert it: open and play a keyboard score with the context on drums, and check the context afterwards
- [ ] 2.2 Same assertion for a score opened from a deep link or from the library — an incoming link must not reconfigure the app
- [ ] 2.3 The hub's instrument filter is **seeded** from the context and then independent, retained for the session; adjusting it does not write back. Test both directions

## 3. The one-time choice

- [ ] 3.1 Modal shown when `drumsVisible` first becomes true for this installation — after sign-in while beta-scoped, at first launch once global. One rule, no phase-specific branch
- [ ] 3.2 Wording is a **choice**, not an unlock ("Par quoi veux-tu commencer ?"), states that it can be changed later, and neither option blocks — see `mockups/context.html`
- [ ] 3.3 Never shown when drums are not visible; shown at most once
- [ ] 3.4 Localise fr/en
- [ ] 3.5 Widget tests: shown on first visibility, not shown again, never shown when drums are invisible

## 4. The switcher

- [ ] 4.1 Segmented control in the home header, present **only** when drums are visible; today's home is unchanged for everyone else, with no new control and no question
- [ ] 4.2 Switching re-seeds the discovery surfaces (home sections, hub filter, sound picker family, surfaced courses)
- [ ] 4.3 Read through the notifier and react to state per the Riverpod layering rules — the screen never calls a service, and no provider imperatively invalidates a sibling
- [ ] 4.4 Decide and implement the phone-class placement; the mockup's position is comfortable on a tablet and competes with the header on a phone
- [ ] 4.5 Widget test for presence/absence by visibility, and for the re-seeding

## 5. Degradation — both failure modes are silent

- [ ] 5.1 Context names a no-longer-visible instrument (campaign closed without the scope being widened) → fall back to keyboard **silently**, no error, working home
- [ ] 5.2 Context in force with nothing to show → an explicit invitation naming the cause and offering to switch, never a bare empty screen; localised fr/en
- [ ] 5.3 Tests for both

## 6. Bundled drum scores

- [ ] 6.1 Author ~4 short drum scores across the beginner / intermediate / advanced folders. **Author, do not source** — a basic groove is an idiom, not a copyrightable work, and this is what the existing bundled scores already are
- [ ] 6.2 Do **not** promote the public-domain drum scores found in the catalog: crawler-classified, `pending`, never human-reviewed. A classifier's output is not a licence confirmation, and bundling is a higher bar than cataloguing
- [ ] 6.3 Between them, exercise unpitched notes, the part-list instrument table, the percussion clef, two voices on one staff, and open/closed hi-hat
- [ ] 6.4 Declare them in `pubspec.yaml` and record each in `assets/scores/CREDITS.md` with its status, following the existing table
- [ ] 6.5 Confirm the no-account core loop works end to end on a bundled drum score: gauge and end-of-session summary, no backend

## 7. Gates

- [ ] 7.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 7.2 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [ ] 7.3 `openspec validate add-instrument-context --strict`

## 8. Manual verification

- [ ] 8.1 As a campaign member signing in for the first time: the modal appears once, either choice proceeds, and the switcher is then visible on the home
- [ ] 8.2 Sign out and back in: the modal does **not** reappear
- [ ] 8.3 As a non-member: no modal, no switcher, no instrument question anywhere — the home is byte-for-byte today's
- [ ] 8.4 With the context on drums, open a keyboard score from the library: it plays normally and the context is unchanged on return
- [ ] 8.5 Set the hub's filter to the other instrument, navigate away and back: the filter is retained and the home still reflects the context
- [ ] 8.6 Close the campaign without widening the scope: the context falls back to keyboard silently and the home still works
- [ ] 8.7 Without an account, pick drums and play a bundled drum score through to the summary
