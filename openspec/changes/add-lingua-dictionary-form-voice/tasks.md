## 1. The listen row

- [x] 1.1 `reading/wordpopup.ts`: one test decides « forme vue » and the second word button (design D1). `listensFor` offers, for a word whose form seen differs, « ▶ <seen> » (key `selection`) then « ▶ <headword> » (key `headword`), each with its accessible name (D2, D3). Otherwise the row is as before.
- [x] 1.2 Specs in `test/wordpopup.spec.ts`:
  - `Es` / `ser` and `ran` / `run`: two labelled buttons, each speaking its text;
  - `Casa` / `casa` and `casa`: one « ▶ Mot »;
  - an expression: « ▶ Sélection »;
  - switching from the form seen to the dictionary form, and stopping it;
  - a pending card completing with its answer keeps reading the form seen.

## 2. Gates

- [x] 2.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [x] 2.2 `openspec validate add-lingua-dictionary-form-voice --strict` passes.
