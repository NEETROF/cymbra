# add-lingua-chapter-recap — design

## Context

See proposal.md for the why. What exists:

- **The book reader** (`src/reader/app.ts`) hosts a reading session on each section of the book.
  The renderer reports every move with a `BookLocation` (`cfi`, `fraction`, `section`: the
  table-of-contents label the page sits in); `onRelocate` keeps the section title in the reader's
  bar and saves the position. A card met in a book already carries `bookSource(title, section)`.
- **The library** (`src/reader/library.ts`) keys a book by the SHA-256 of its file and keeps, in
  its own IndexedDB database, the books, their files and their positions — on the device only.
  Deleting a book removes its records.
- **The cards**: the reading session (`src/reading/session.ts`) opens a word card or an expression
  card on a tap or a selection, and creates a card on « + Deck » through `addCard`, with the
  sentence and the source.
- **Nothing records a lookup today**: a card closed without a gesture leaves no trace.
- **Review**, after `refine-lingua-review-session`: the Révision page beside a book is the drawer
  (Firefox, Safari) or Chrome's side panel, which already asks the reading session for the book's
  language (`getStats`, `reviewLanguage`).

## Goals / Non-Goals

**Goals:**
- Turn the words a reader looked up into a short, optional decision at a natural pause.
- Keep the reading uninterrupted: nothing appears mid-chapter, nothing at the opening of a book.

**Non-Goals:**
- The warm-up at the opening of a book (proposal ② of the study), a later change.
- Lookups outside the book reader (web pages): the reader's sentence and chapter make the recap;
  a page has neither.
- Synchronising or backing up the list; a « book report » across books.

## Decisions

### D1 — A lookup is a card opened in the book, kept once per word and chapter

The reading session tells its host when it shows a word or an expression card in a book (a new
`onLookup` option, called by the book reader only). The reader stores the dictionary form, the
form met, the sentence, the chapter label, the language and the time, keyed by book, word and
chapter: opening the same word twice in a chapter is one lookup. Opening a card is enough — the
reader stopping on a word is the signal; adding it is the decision the recap prepares.

*Alternative.* Recording only words without a gloss: misses the words a reader knows of but cannot
place, which are the ones worth sorting.

### D2 — Stored beside the positions, on the device

A `lookups` object store in the library's database (a version upgrade, nothing existing moves),
keyed by `[hash, chapter, lemma]` with an index on `hash`. It is removed with the book, as its file
and position are, never synchronised and never in the engine's backup: it is about the reader's
books, which never leave the device.

### D3 — The end of a chapter is a change of section label, forward

`onRelocate` compares the new location's section with the last one: when the label changes and
`fraction` grew, the chapter just left ended. A move backwards, a jump through the table of
contents to an earlier chapter, or the first location after opening never offers a recap. A book
without a table of contents has no chapter labels, so no recap — the counter still works.

### D4 — What is offered, and when not again

At the end of a chapter, the words still to sort from that chapter are read: a word is still to
sort when the deck holds no card for it and its status is neither known nor ignored, asked of the
engine at that moment. Three or more → the recap. « Plus tard » remembers how many lookups the book
had; the recap comes back at a later chapter end only once three more were recorded since.

### D5 — The recap: a sheet over the page, one row per word

A sheet over the reader page, in the surfaces' look: the chapter's title, then one row per word —
the form met, the pack's gloss (asked when the sheet opens), the sentence with the word marked —
and three actions: « + Deck » (the same `addCard` as the word card, with the row's sentence and the
book's source for that chapter), « Je connais » (the known status), « Laisser » (nothing, the row
leaves). On a phone the actions sit under the word; on e-ink the sheet paints once and rows are
removed without animation. Its copy is the catalogue's `reader` module, in French, English and
Spanish.

### D6 — The counter

The reader's bar shows « N à trier » beside the section title when N > 0; it opens the same sheet
for the whole book, grouped by chapter. The Révision page shows the count when its host says it is
beside a book: the drawer asks its session, the side panel reads it from the `getStats` answer it
already receives for the book's language.

## Risks / Trade-offs

- [A reader who looks up many words gets a long list] → only the chapter just left is offered; the
  rest waits behind the counter.
- [Chapters labelled the same across sections] → the label is the chapter; two sections under one
  label are one chapter, which is what the reader sees in the bar.
- [The recap interrupts the reading] → offered only at a chapter change, dismissed in one tap, and
  not again until three new words.

## Migration Plan

Extension only. The library database gains a store on upgrade; a rollback leaves it unused.
