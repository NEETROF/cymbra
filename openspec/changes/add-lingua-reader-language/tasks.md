## 1. A document's declared language

- [x] 1.1 `session.ts`: `languageHint` reads `lang`, then `xml:lang`, on the root element and then on `body` (design D1). `primaryLanguage(tag)` keeps the primary subtag and reads `eng` and `spa` as `en` and `es`. Specs:
  - an XHTML section declaring `xml:lang` only;
  - a declaration on `body`;
  - `es-419`, ` ES_mx `, `spa` and `eng`.

## 2. The book's language

- [x] 2.1 `reader/app.ts`: before attaching a section, declare the open book's language on its root element when the section declares none (design D2). Specs in `reader-app.spec.ts`:
  - a section declaring nothing takes the book's `es`;
  - a section's own declaration (`lang` or `xml:lang`) is kept;
  - a book declaring nothing leaves the section as it is.
- [x] 2.2 Specs with the reading session, with en-fr and es-fr shipped:
  - a section declaring nothing, in a book declaring Spanish, is asked with the hint `es`; the core gives the hint the sections with no text to vote (`add-lingua-language-routing` D1);
  - a section's own declaration wins over the book's;
  - with one accepted language, nothing is detected.

## 3. Gates

- [x] 3.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [x] 3.2 `openspec validate add-lingua-reader-language --strict` passes. In `docs/lingua/spanish-programme.md`, change 15 is marked done.
