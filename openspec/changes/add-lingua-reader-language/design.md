# Design — add-lingua-reader-language

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `src/reading/session.ts` | With several accepted languages, `repaint()` asks `detectLanguage(blocks, languages, languageHint(host.doc))`. `languageHint` reads the root element's `lang`, as its primary subtag. |
| `src/reader/epub-meta.ts`, `library.ts` | The first `dc:language` of the package is kept on the book's record (`BookRecord.language`, BCP 47 or null). Nothing reads it. |
| `src/reader/app.ts` | Each section the renderer shows is attached to the session as a `ReadingHost` (document, window, source, exposure source). The reader page already styles each section's document (`book-style.ts`). |
| `lingua-core` `detect_document_language` | The hint breaks a tie and decides when no block votes; a hint that is not a candidate is ignored. |

## Goals / Non-Goals

**Goals:**
- A section that cannot tell its language by its text is read in the book's.
- The declarations XHTML sections actually use are read.
- English unchanged: one accepted language, no detection.

**Non-Goals:**
- Routing per block.
- One language for the whole book, with no vote per section. A book may hold a preface or an
  appendix in another language, which the vote reads in theirs.
- Correcting a wrong `dc:language`. Converted books often say `en` or `und`. The declaration only
  decides the sections with too little text to vote, and the vote overrides it elsewhere.
- Showing a book's language in the library.

## Decisions

### D1 — A document's declared language

`languageHint(doc)` reads `lang`, then `xml:lang`, on the root element and then on `body`. EPUB
content documents are XHTML. They declare `xml:lang`, often beside `lang` and sometimes alone.
`getAttribute("xml:lang")` matches that qualified name in an XML document and in an HTML one. A
declaration on `body` covers all the text too.

The tag is reduced by `primaryLanguage(tag)`:
- the primary subtag, in lowercase;
- `eng` and `spa`, the ISO 639-2 codes some packages use, as `en` and `es`;
- any other code as it is. The engine ignores a hint that is not a candidate (routing D2).

### D2 — The reader page declares the book's language on a section that declares none

When a section's document declares no language (D1), the reader page sets the root element's `lang`
to the open book's `BookRecord.language`, the first `dc:language` of its package, before it attaches
the section to the session. The session then reads it as any declaration, through `languageHint`, and
its hint does not change shape.

This is what a package's `dc:language` means: the language of the publication's content. Setting it
on the section also gives the browser what it needs for automatic hyphenation, spelling and screen
readers. Web pages are unchanged: nothing declares on their behalf.

*Rejected — a fallback on the reading host (`ReadingHost.language`).* It leaves the section's document
untouched, but adds a second declaration path the session must combine, and leaves the browser
unaware of the text's language.

*Rejected — the book's language overriding the vote.* A Spanish book's English preface would be read
in Spanish.

*Rejected — the language most sections were read in so far, as the hint.* It is better evidence than
a declaration. But the opening sections come before any vote, and a wrong declaration misleads only
the sections too short to vote.

## Risks / Trade-offs

- **A wrong `dc:language`** → only the sections with too little text to vote follow it, and they hold
  few words.
- **A package listing several `dc:language`** → the first one is the book's, as the library already
  keeps it.
