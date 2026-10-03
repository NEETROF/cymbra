# add-lingua-reader-language — a book is read in its own language

## Why

Since `add-lingua-language-routing`, the reading session chooses each document's language among the
reader's. Each long enough block votes, weighted by its length, and the document's declared language
breaks a tie or decides when nothing votes. In the EPUB reader, each section is a document of its
own, and the hint misses what books declare:
- **A book declares its language in its package** (`dc:language`), which the library already reads
  and keeps (`BookRecord.language`). A section rarely declares anything else, so a section with too
  little text to vote (a title page, a dedication, a chapter's opening page) is read in the reader's
  first language, whatever the book's.
- **XHTML sections declare their language with `xml:lang`**, often alone. `languageHint` reads `lang`
  only.

This is change 15 of `docs/lingua/spanish-programme.md`, in R3, a silent English release. Every
reader studies English alone, so no detection is asked and nothing changes for them.

## What Changes

- **A section's declared language** is read from:
  - its root element's `lang`, then its `xml:lang`;
  - then the same two on its `body`.

  It is reduced to the primary subtag. `eng` and `spa`, the three-letter codes some packages use for
  the shipped languages, read as `en` and `es`.
- **The book's declared language is the hint of a section that declares none.** The reader page
  hands the session the open book's `dc:language`. The vote still decides as for a page:
  - a section in English in a Spanish book is read in English;
  - a section with too little text to vote is read in the book's language, when it is one of the
    reader's.
- **A reader of one language**: no detection is asked, as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-reader`: ADDED — *A book is read in its own language*. The capability is created by
  `add-lingua-reader`, still open on its release task, so this change archives after it.

## Impact

- **Products.** Cymbra Lingua's browser extension only:
  - the reading session's language hint (`src/reading/session.ts`);
  - the reader page's section host (`src/reader/app.ts`).

  It consumes the routing of `add-lingua-language-routing` and the book metadata the library
  already keeps. No engine, pack, server or proto change. ID, Music, Live, the back office and the
  site are not affected.
- **Release.** R3, silent: one language, no detection.
