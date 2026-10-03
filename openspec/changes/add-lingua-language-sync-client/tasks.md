## 1. The accepted languages

- [x] 1.1 `src/analyzer/pairs.ts`: `acceptedLanguages(port, pairs)` (design D1). Spec:
  - the reader's shipped languages in their order;
  - the default pair's language when none is shipped;
  - English alone for every reader today.

## 2. The sync

- [x] 2.1 `src/sync/sync.ts` (design D2–D5):
  - the card pull names the accepted languages;
  - pulled cards keep their language;
  - pulled statuses and levels in other languages are not applied;
  - pushed cards carry their language, with non-English ones withheld unless `cardLanguage`;
  - the stored set, and both cursors reset when it grows.
- [x] 2.2 `test/sync.spec.ts`, with a device that accepts English alone and one that accepts English and Spanish:
  - the pull's `languages`;
  - a Spanish card filed as Spanish;
  - a Spanish status not applied on an English-only device;
  - pushed cards carry their language;
  - a Spanish card withheld from a server without `cardLanguage`;
  - the reset when Spanish is added, none when updating and none when narrowing;
  - the set saved only after a successful sync.

## 3. Gates

- [x] 3.1 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check`, `yarn build` and `yarn check:variants`.
- [x] 3.2 `openspec validate add-lingua-language-sync-client --strict` passes. In `docs/lingua/spanish-programme.md`, change 11 is marked done.
