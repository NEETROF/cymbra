## 1. Sections

- [x] 1.1 `reader/app.ts`: `shelfSections(books, languages)`, pure, groups the books (design D2, D4). It returns one section per accepted language holding books, in the given order, then « Autres langues ». Each section keeps the library's order. Specs:
  - books in `es`, `en-GB` and `fr`, for `[es, en]`;
  - `es-MX`, ` SPA ` and `spa` under `es`;
  - a book declaring nothing, or `und`, under « Autres langues »;
  - an accepted language with no book: no section;
  - the library's order kept inside a section.
- [x] 1.2 `ReaderApp.renderShelf` draws the sections (D3, D6):
  - one section: the single `ul.lib-books`, no heading;
  - several: a `section.lib-group` each, with its `h2.lib-group-title` and its `ul.lib-books`.

  `ReaderDeps.languages` gives the reader's languages, and `reader.ts` passes `acceptedLanguages` on the page's port (D1, D5). `copy.ts` gets « Autres langues », and `reader.css` the heading. Specs in `reader-app.spec.ts`:
  - a Spanish and an English book for a reader of `[es, en]`: « Espagnol » then « Anglais », each with its book;
  - two English books: no heading;
  - after a deletion leaves one language, the headings go;
  - the languages are read again each time the shelf is drawn.

## 2. Gates

- [x] 2.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [x] 2.2 `openspec validate group-lingua-library-by-language --strict` passes.
