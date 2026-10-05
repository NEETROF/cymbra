## 1. The list

- [ ] 1.1 `packs.json` lists `en-fr` then `es-fr`, and `check_variants`'s `SHIPPED_PAIRS` follows (design D1).
- [ ] 1.2 `scripts/lingua-data/testdata/es-fr/`, the tiny fixture `yarn gen:pack` builds (design D2).
- [ ] 1.3 The manifest summary names both languages within 112 characters, in the wording the owner settles (design D3).
- [ ] 1.4 « Langues étudiées » says « Plusieurs langues à la fois : gratuit pour l'instant. », in Réglages and at onboarding (design D6).

## 2. Checks

- [ ] 2.1 In `apps/lingua-extension`:
  - `yarn gen:pack` and `yarn gen:pack:real` build both packs;
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check` and `yarn build` pass, the variant check included;
  - the English baseline does not move.
- [ ] 2.2 `openspec validate enable-lingua-spanish --strict` passes.

## 3. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 3.1 Chrome (macOS): the pass of design D4.
- [ ] 3.2 Firefox (macOS): the pass of design D4.
- [ ] 3.3 Firefox for Android: the pass of design D4.
- [ ] 3.4 Safari (macOS): the pass of design D4.
- [ ] 3.5 Safari (iOS): the pass of design D4.

## 4. Release (owner)

- [ ] 4.1 The owner's go-ahead before merge.
- [ ] 4.2 The first Spanish-capable packages go to the beta channels; each store submission is the owner's.
- [ ] 4.3 In `docs/lingua/spanish-programme.md`, change 28 is marked done.
