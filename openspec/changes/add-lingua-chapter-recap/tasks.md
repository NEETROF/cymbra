# Tasks

## 1. Remembering the lookups

- [x] 1.1 `reader/library.ts` (D2): a `lookups` store (upgrade), `recordLookup`, `lookupsOf(hash)`, removed with the book. Verify with library specs on a fake IndexedDB: one record per word and chapter, removal with the book, an upgrade keeping books and positions.
- [x] 1.2 `reading/session.ts` (D1): an `onLookup` option called when a word or expression card is shown; `reader/app.ts` records it with the chapter. Verify with session and reader specs: a tap records once per chapter, a web page records nothing.

## 2. The recap

- [x] 2.1 `reader/app.ts` (D3, D4): the end of a chapter from `onRelocate`, the words still to sort, the threshold of three, « Plus tard » and its count. Verify with reader specs on the fake renderer: forward change offers, backward and opening do not, two words do not, dismissed then two more do not, three more do.
- [x] 2.2 The recap sheet (D5): rows with the form, the gloss and the sentence; « + Deck », « Je connais », « Laisser »; the catalogue's `reader` module in fr, en, es. Verify with view specs: a card added with the row's sentence and source, a status set, text nodes only, `yarn test` on `i18n.spec.ts`.

## 3. The counter

- [x] 3.1 The bar's « N à trier » and the sheet for the whole book (D6). Verify with reader specs.
- [x] 3.2 The Révision page's count beside a book, from the drawer's session and the side panel's `getStats`. Verify with review page and messaging specs.

## 4. Integration and dogfood

- [x] 4.1 Gates in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass.
- [ ] 4.2 Dogfood on the e-ink reader: a chapter with several lookups, the recap at its end in one paint, adding and marking known. Verify by noting the result in the pull request.
