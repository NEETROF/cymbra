# group-lingua-library-by-language — the library groups its books by language

## Why

Dogfooding the Spanish reading (2026-10-04), with English and Spanish books in one library, showed
that the library lists every book in one grid, the last read first. A reader of two languages looks
for their Spanish book among the English ones. The library already keeps each book's language
(`BookRecord.language`, the first `dc:language` of its package), and `add-lingua-reader-language`
reads a short section in it. Nothing shows it.

The owner chose the layout on 2026-10-04:
- one section per language they study, in their order;
- the other books last.

This is not a numbered change of `docs/lingua/spanish-programme.md`: it comes from the dogfood of
its reading.

## What Changes

- **Sections.** The library shows its books in sections:
  - one per language the reader studies and the extension reads, in the reader's order, headed with
    the language's name (« Espagnol », « Anglais »);
  - then « Autres langues », for the books in any other language or declaring none.
- **A book's language** is the first language its package declares, reduced to its primary subtag
  as the reading session reduces a section's (`es-MX`, `spa` → `es`).
- **Order.** A section holds its books in the library's order: the last read first. An empty
  section is not shown.
- **One section, no heading.** When every book falls in one section, the library shows no heading,
  as today.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-reader`: ADDED — *The library groups its books by language*. The capability is created
  by `add-lingua-reader`, still open on its release task, so this change archives after it, and
  *A library of the reader's own books* is not modified.

## Impact

- **Products.** Cymbra Lingua's browser extension only:
  - the reader page's library (`src/reader/app.ts`, `reader.css`, `copy.ts`);
  - the reader page's wiring (`src/reader/reader.ts`), which gives the library the reader's
    languages.

  No engine, pack, server or proto change. ID, Music, Live, the back office and the site are not
  affected.
- **Release.** With the next release of the extension. A reader of English alone whose books all
  declare English sees nothing new. One with a book in another language, or declaring none, sees two
  sections: « Anglais » and « Autres langues ».
