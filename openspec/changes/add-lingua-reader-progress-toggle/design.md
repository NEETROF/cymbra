# add-lingua-reader-progress-toggle — design

## Context

See proposal.md for the why. What exists:

- **The footer** (`src/reader/app.ts`): ‹ and › around `reading-progress`, which `onRelocate`
  fills with the book's fraction as a percentage.
- **The renderer seam** (`src/reader/renderer.ts`): `BookLocation` carries `cfi`, `fraction` and
  `section` (the table-of-contents label). foliate-js sits behind it (`src/reader/foliate.ts`);
  the page's tests run on a fake.
- **foliate-js lays out one section at a time.** Its paginator knows the page shown and the pages
  of the section on screen (`page`, `pages`; in the paginated flow the count holds a blank page at
  each end); its section progress knows each section's size in bytes. Nothing knows how many
  screens the rest of the book would take without laying it out.
- **The reader's display preferences** (`READER_FLOW_KEY`, `READER_DISPLAY_KEY`) live in
  `chrome.storage.local`, read at start and followed when another page changes them.

## Goals / Non-Goals

**Goals:**
- Say what is left in the unit the reader is deciding about, in one tap, on every screen.
- Never present an estimate as exact.

**Non-Goals:**
- Laying out the whole book to count its pages (seconds on an e-ink tablet, again at every text
  size change).
- Page numbers of a printed edition (`page-list`), or reading time.

## Decisions

### D1 — One figure, three modes, a tap goes to the next

The footer's figure is a button. A tap goes percentage → chapter → book → percentage. The mode is
stored under its own key in `chrome.storage.local` (`percent` by default; an unknown value reads as
`percent`) and applies to every book and every reader tab.

*Alternative.* A setting in « Aa »: two taps away, and the reader's question comes up while reading,
at the footer.

### D2 — The pages left in the chapter: counted on the laid-out section

The renderer reports, with every move, the pages after the one shown in the section on screen: in
the paginated flow the section's text pages (the paginator's count without its two blank pages)
less the page shown; in the scrolled flow the screens below the one shown. Zero says « Dernière
page du chapitre ». A section is what foliate lays out; a chapter of the table of contents that
spans several sections counts the pages of its section on screen — the same unit a page turn moves
through.

### D3 — The pages left in the book: an estimate, marked as one

pages left in the book = pages left in the section + (bytes of the sections after it) ÷ (bytes of
the section on screen ÷ its text pages). Sections outside the linear reading order count for
nothing, as foliate's progress already does. Shown with « ≈ », rounded to the unit, and recomputed
at every move — so it follows a change of text size at the next page. A section too small to
measure (no text page, or no size) gives no estimate: the figure falls back to the percentage for
that move.

### D4 — The seam carries numbers, the page writes them

`BookLocation` gains `pagesLeftInSection` and `pagesLeftInBook` (both `number | null`). The adapter
computes them; the page only picks one by the mode and writes it with the catalogue's plural forms,
in the interface language. The fake renderer reports what a test gives it.

## Risks / Trade-offs

- [The book estimate jumps between chapters with very different layouts (a chapter of verse, then
  prose)] → it is marked « ≈ », recomputed at every move, and the exact chapter count is one tap
  away.
- [foliate changes how it counts its blank pages] → the adapter is the only file that knows; its
  arithmetic is checked on the device in the dogfood task.

## Migration Plan

Extension only. A new preference key, absent until the reader taps; a rollback ignores it.
