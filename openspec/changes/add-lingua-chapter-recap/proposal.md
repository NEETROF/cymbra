# add-lingua-chapter-recap — the words looked up in a chapter, sorted when it ends

## Why

Reading a book, the reader opens the card of every word they stop on, and most of those words
are never added to the deck: deciding there, in the middle of a sentence, breaks the reading. Once
the card is closed, nothing remembers that the word was looked up — the extension keeps no trace of
it — so the words a reader needed are lost with the chapter. The end of a chapter is the natural
pause to decide, once, what to keep. Decided with the product owner on 2026-10-04 (proposal ⑤ of
the e-ink review study).

## What Changes

- **The words looked up are remembered per book, on the device**: each word or expression whose
  card the reader opened in a book, with the form met, its sentence and its chapter. Never synced,
  never in the backup, deleted with the book.
- **At the end of a chapter, a recap is offered** when at least three words looked up in it are
  still to sort: the list, each row the word as met, its gloss and its sentence, with « + Deck »,
  « Je connais » or leaving it. One tap to dismiss; never at the opening of a book, which belongs
  to the warm-up (a later change).
- **Dismissed, it is not offered again** until at least three new words have been looked up since.
- **A counter « N à trier »** in the reader's bar, and on the Révision page beside the book, opens
  the same list at any time.
- **A word already in the deck, known or ignored** is left out of the list: there is nothing to
  decide about it.
- Laid out for every screen as the review is: on e-ink the recap paints once; on a phone each row's
  actions sit under its word.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-reader` (added by `add-lingua-reader`, not yet archived): added — the words looked up in
  a book are remembered on the device; a recap at the end of a chapter; the counter of words to sort.

This change archives after `add-lingua-reader`, which creates the capability, and after
`refine-lingua-review-session`, whose Révision page shows the counter (`archiveAfter` in
`.openspec.yaml`).

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The extension's book reader (`apps/lingua-extension`, every
  browser), hence the Apple container app (`apps/lingua-apple`) that ships its bundle. It consumes
  the existing word and selection cards, the deck's `addCard` and the status of a word.
- Cymbra ID, Music, Live, the back office and the public site are untouched. Nothing new is
  consumed from `id-*` or `platform-*`: no flag, no RPC, no `.proto`, no backend change. The core,
  its WASM build and the agent plugin are untouched.

**Code.** `src/reader/library.ts` (a store of looked-up words per book, removed with the book),
`src/reader/app.ts` (the chapter change, the bar's counter, the recap), `src/reading/session.ts`
(telling its host that a card was opened), a recap view under `src/reader/`, `src/review/review-page.ts`
(the counter beside a book), the catalogue's `reader` module in French, English and Spanish.

**Data.** A new object store in the library's IndexedDB database, created by an upgrade; nothing
existing moves. Nothing leaves the device.

**Dependencies.** Builds on `refine-lingua-review-session` (#696); implemented once it is merged.
