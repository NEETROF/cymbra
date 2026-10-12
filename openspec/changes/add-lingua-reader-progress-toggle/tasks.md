# Tasks

## 1. The pages left

- [ ] 1.1 `reader/renderer.ts`, `reader/foliate.ts` (D2, D3, D4): `pagesLeftInSection` and `pagesLeftInBook` on every location, from the paginator and the sections' sizes; the arithmetic in a pure function. Verify with a spec of that function: paginated and scrolled flows, the last page, a section with no size, sections outside the reading order.
- [ ] 1.2 `test/fake-renderer.ts`: the fake reports what a test gives it. Verify by the reader specs below.

## 2. The footer

- [ ] 2.1 `state/storage.ts` and `reader/app.ts` (D1): the stored mode, the footer's button cycling the three, followed across tabs; the catalogue's `reader` module in fr, en, es with plural forms. Verify with reader specs on the fake renderer: the cycle, the last page of a chapter, the fallback to the percentage without an estimate, the mode read at start, `yarn test` on `i18n.spec.ts`.

## 3. Integration and dogfood

- [ ] 3.1 Gates in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass.
- [ ] 3.2 Dogfood on the e-ink reader: the three modes, the chapter count against the pages turned to its end, the book estimate after a text size change. Verify by noting the result in the pull request.
