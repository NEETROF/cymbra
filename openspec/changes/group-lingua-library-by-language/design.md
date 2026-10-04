# Design — group-lingua-library-by-language

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `src/reader/library.ts` | `list()` returns every book, the last read first, then the last imported. Each record keeps the first `dc:language` of its package (`language`, BCP 47 or null). |
| `src/reader/app.ts` | `renderShelf()` draws one `ul.lib-books` of cards: cover, title, authors, delete. It runs when the library opens, after an import and after a deletion. |
| `src/reading/session.ts` | `primaryLanguage(tag)` reduces a declaration to its primary subtag, `eng` and `spa` as `en` and `es` (`add-lingua-reader-language` D1). |
| `src/analyzer/pairs.ts` | `acceptedLanguages(port)`: the reader's studied languages that a shipped pair studies, in their order, or the default pair's alone. The reading session reads in these. |
| `src/analyzer/language-labels.ts` | The one place a language's name is written (« Espagnol »), lint-enforced. |

## Goals / Non-Goals

**Goals:**
- A reader of several languages finds a language's books together, their first language first.
- A library in one language looks as it does today.

**Non-Goals:**
- Choosing or correcting a book's language. A converted book often declares `en` or `und`; it goes
  where it declares.
- A section per language the reader does not study (« Français », « Allemand »): the interface
  names only the studied languages.
- Sorting or filtering beyond this: a search, a collapsible section, another order.

## Decisions

### D1 — The languages that head a section

The sections follow `acceptedLanguages(port)`: the languages the reader studies that the extension
reads, in the reader's order. Those are the languages the reading session reads a book in, and
`languageName` names each.

*Rejected — every studied language.* A build that does not read Spanish would head an « Espagnol »
section, though it reads none of those books in Spanish.

*Rejected — a section per language the books declare.* The reader asked for their languages, and a
name like « Allemand » would be the only place the interface names a language it does not read.

### D2 — A book's language

A book's language is `primaryLanguage(book.language)`, the reduction the reading session applies to
a section's declaration. A book whose language is not one of D1's, or which declares none, goes in
« Autres langues ».

### D3 — No heading for one section

When every book falls in one section, whichever it is, the library draws the single grid of today
with no heading. A heading separates groups, and one section separates nothing. A reader of English
alone whose books all declare English sees no change.

### D4 — Order

The sections come in the reader's order, « Autres langues » last. A section holds its books in the
library's order, the last read first, then the last imported. A section with no book is not drawn.

### D5 — When the languages are read

The library reads the languages each time it draws the shelf: when it opens, after an import and
after a deletion. Coming back from a book draws it again. A change of languages made in another tab
while the library is on screen shows the next time it is drawn.

*Rejected — watching the reader's profile.* The library page shows no Réglages of its own, so the
languages change elsewhere, and the reader comes back to the library through a redraw.

### D6 — Markup

Several sections: one `section.lib-group` each, holding an `h2.lib-group-title` and its own
`ul.lib-books`. Screen readers list the headings under the page's `h1` « Bibliothèque ». One
section: the `ul.lib-books` alone, as today. The heading's style comes from the token sheet, like
the rest of the page.

## Risks / Trade-offs

- **A wrong `dc:language`** → the book is in the wrong section, and the reader cannot move it. It
  is still in the library, and opens as before.
- **A reader of English alone with a converted book declaring `und`** → their library gains two
  headings. That is the rule the owner chose: the headings show only when there is more than one
  group.
