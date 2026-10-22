# Tasks

## 1. The glossary and the pass

- [ ] 1.1 [agent] `docs/lingua/italian-glossary.md`: each concept's French term, Italian term, surfaces and reason where it differs; built from the surfaces as they stand (D1).
- [ ] 1.2 [agent] The reviewer agent's pass over every surface of the design's *Context* table, against the French source, the glossary and I6 (D2); each finding applied in this pull request or answered with a source; a finding that changes meaning put to the owner.
- [ ] 1.3 `docs/lingua/italian-wording-review.md`: the surfaces read, each finding, its line and resolution, the count of strings read (D2).

## 2. The check

- [ ] 2.1 `apps/lingua-extension/test/italian-glossary.spec.ts` (D3): the extension's catalogue, `_locales/it` and the Safari app's `copy.js` against their French; the glossary's exceptions.

## 3. Gates

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`; in `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; `swift test` in `apps/lingua-apple/LinguaSignIn` if `SignInCopy` changed; every count of the listing files rechecked against its cap if a listing changed.
- [ ] 3.2 `openspec validate review-lingua-italian-wording --strict`; the archive order; `docs/lingua/italian-programme.md` row 44's status.
- [ ] 3.3 [owner] The report's summary read, and any meaning finding decided, before change 45's release (D4).
