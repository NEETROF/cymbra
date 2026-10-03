## 1. A document's declared language

- [ ] 1.1 `session.ts`: `languageHint` reads `lang`, then `xml:lang`, on the root element and then on `body` (design D1). `primaryLanguage(tag)` keeps the primary subtag and reads `eng` and `spa` as `en` and `es`. Specs:
  - an XHTML section declaring `xml:lang` only;
  - a declaration on `body`;
  - `es-419`, ` ES_mx `, `spa` and `eng`.

## 2. The book's language

- [ ] 2.1 `ReadingHost.language`; the session's hint falls back to it (design D2). Specs, with en-fr and es-fr shipped:
  - a section declaring nothing, in a book declaring `es`, is asked with the hint `es`;
  - a section's own declaration wins over the book's;
  - with one accepted language, nothing is detected.
- [ ] 2.2 `reader/app.ts`: the section host declares the open book's language. Spec in `reader-app.spec.ts`.

## 3. Gates

- [ ] 3.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [ ] 3.2 `openspec validate add-lingua-reader-language --strict` passes. In `docs/lingua/spanish-programme.md`, change 15 is marked done.
