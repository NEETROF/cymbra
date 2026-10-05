# Tasks

## 1. The choice

- [ ] 1.1 `reading/studied-languages-view.ts`: one option per shipped language as a single choice, the studied language checked (design D1). Choosing writes the profile as that language alone. A profile holding several languages shows its first. Specs:
  - one shipped language hides the block;
  - choosing Spanish writes Spanish alone;
  - a profile holding English then Spanish shows English checked.
- [ ] 1.2 The confirmation and the wait (design D2, D3). A change in the settings is confirmed in place, then records `cymbra-lingua-language-changed-at` in the area the builder is given. Within 30 days of it, the other options are disabled and the date is shown. Onboarding's choice records nothing. Specs:
  - « Annuler » leaves the profile and the options unchanged;
  - « Changer » writes the profile and the mark;
  - 10 days after a change, the other options are disabled with the date;
  - 31 days after, a change is possible;
  - a choice at onboarding writes no mark.
- [ ] 1.3 `reading/settings-view.ts` passes the area; the block's title is « Langue étudiée ». `onboarding/onboarding.ts` mounts it without counting the choice, under « Quelle langue apprends-tu ? » (design D5).

## 2. The review

- [ ] 2.1 `review/review-page.ts`: with no language filtered, the due count and the session cover the accepted languages, not every language the backup holds (design D4). Specs:
  - a reader who studies Spanish, with English cards due, is offered the Spanish cards only;
  - a reader of English alone is offered the same cards, in the same order, as before.

## 3. Checks

- [ ] 3.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check` and `yarn build` pass, the variant check included;
  - the English baseline does not move.
- [ ] 3.2 `openspec validate limit-lingua-to-one-language --strict` passes.

## 4. Dogfood (owner)

- [ ] 4.1 On a dogfood build with both pairs, check on one browser at least:
  - the choice at onboarding;
  - a confirmed change in the settings, and the date it gives;
  - the options disabled during the wait;
  - after a change, the review offers the new language's cards only.
