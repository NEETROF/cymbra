## Context

foliate-js's paginator turns a page by scrolling its multi-column container. With the
`animated` attribute set on the paginator element, that scroll is eased over 300 ms
(`vendor/foliate-js/paginator.js`, `#scrollTo`), and a swipe's snap animates too. Without it,
the scroll is one assignment. `src/reader/foliate.ts` left the attribute off, for the e-ink
screen (`add-lingua-reader` D4).

## Goals / Non-Goals

**Goals:**

- A page that slides when turned, for readers on an ordinary screen who ask for it.
- No change at all for the e-ink device unless its reader chooses it.

**Non-Goals:**

- A page curl. foliate-js has none; drawing one means snapshotting the section's iframe and
  deforming it, with the highlights, the selection and the night colours in step — a separate
  change, if ever.
- Sliding from one section to the next. foliate-js loads a new document there instead of
  scrolling; the turn stays a jump, as it is today.

## Decisions

### D1. Part of the display preference, not a key of its own

The turn joins `ReaderDisplay` (text size, page colour) instead of a new storage key beside
the flow. The display already travels end to end — the shared "Aa" panel writes it, the reader
page watches it and hands it to `BookRenderer.setDisplay` — so the choice needs no new seam,
no new watcher and no new `ReaderDeps` member. `readerDisplayOf` reads anything but `slide` as
`instant`, so stored displays from before this change keep the jump.

### D2. Off by default, and off under reduced motion

`instant` is the default because the reader's first target is an e-ink tablet. The adapter
also drops the attribute when `prefers-reduced-motion: reduce` matches, whatever the choice:
the system setting is the reader's stronger statement.

### D3. Applied with the display, in the adapter

`FoliateRenderer.applyDisplay` toggles `animated` on the paginator next to its styles, so the
choice applies when a book opens and live when it changes in either place. The reader page and
its tests see nothing of it: the adapter is the only file that knows foliate's attributes.
