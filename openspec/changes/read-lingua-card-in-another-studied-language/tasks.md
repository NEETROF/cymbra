# Tasks

Implemented on `main` once #810 (change 34), #814 (change 35) and #880 (change 52) have merged, never
stacked on them (design D9). The owner answered the design's three questions on 2026-10-11 (task 0.1).

## 0. Before the implementation

- [x] 0.1 [manual] Answered by the owner on 2026-10-11: (1) the highlight goes on the page where the word was marked only (D6); (2) one word only (D8); (3) no re-read on a page written in the reader's native language (D1).

## 1. The cards (apps/lingua-extension)

- [x] 1.1 `src/reading/selection-card.ts`: an exported predicate `unknownToDocument(cls, gloss, grammar)` — class `Unknown`, no gloss, a grammar answer with no reading, no other dictionary form and a `null` rank (an absent answer is not empty) (D1).
- [x] 1.2 `SelectionCards` gains, in its options, `otherLanguages: () => StudiedLanguage[]` (the reader's accepted languages but the document's, in the profile's order, already filtered by the guard table of task 2.3, and empty when the document is not analysable or declares the reader's native language, design D1) and `portsFor(language)` (the `phraseGloss`, `gloss` and `wordGrammar` of that language's view); `readElsewhere(written)` asks each in order and answers the first one-token, non-name, glossed reading with its grammar and rank, or null (D2).
- [x] 1.3 `openForToken` (an unknown word without a gloss, through `completeWithGrammar`) and `openLooseWord`: when the document's answer is empty (1.1) and `otherLanguages()` is not empty, the card waits for `readElsewhere` under one more `GRAMMAR_WAIT_MS`; within it the re-read card is shown (and the document's never is); past it the document's card is shown and the re-read replaces it on arrival if the card view still shows it (the generation check of `show`); the loose path stays pending under `ANSWER_TIMEOUT_MS` as today. Known, learning and ignored words, names, words the document's pack knows and expressions never call it (D1, D3, D8).
- [x] 1.4 The re-read card: headword, form seen, gloss, grammar (`language` = the card's language), the band from its rank, `readIn` set, no `translating` (D4). Every other card's content unchanged byte for byte (`readIn` absent).

## 2. The session

- [x] 2.1 `src/reading/session.ts`: the cards get `otherLanguages` and `portsFor` (`port.for(language)`); `onGesture` writes the status, card (`cardGloss` asked of the card's language), retirement and learnt-word count through `port.for(g.readIn ?? this.language)` (D5).
- [x] 2.2 The document's re-read words: a map, form as written lowercased → language and dictionary form, filled when a re-read card is shown, emptied when the host changes or detaches; `repaint` asks each remembered form's status in its language (`phraseGloss` of the form) and paints those tokens by it — none for known or ignored, learning, unknown — leaving `resolved`'s counts, `stats` and the exposures as the analysis gave them (D6).
- [x] 2.3 The guard table: Spanish ← `ca`, `gl`, `oc`; French ← `ca`, `oc`, `ro`, each entry naming the guard it mirrors (changes 42, 42b, `lingua-analysis` *Catalan and Galician are not read as Spanish*); `otherLanguages` leaves out a language whose entry holds the document's declared language (`languageHint`); `primaryLanguage`'s `THREE_LETTER` gains `cat`, `glg`, `oci`, `ron`, `rum` (D7).

## 3. The card view and the copy

- [x] 3.1 `src/reading/wordpopup.ts`: `WordPopupContent.readIn` and `Gesture.readIn`; the language line before the frequency line when `readIn` is set; the word's and dictionary form's listen buttons with the speaker of `readIn`, the sentence's with the document's (D4).
- [x] 3.2 `src/reading/session.ts` / `src/reading/speech.ts`: a speaker per studied language asked, created on first use with that language's stored voice preference (`A voice per studied language`), stopped with the session.
- [x] 3.3 `src/i18n/{fr,en,es}/selection.ts`: `readAs` — « Mot ${masculine} » (« Mot espagnol »), "${masculine} word" ("Spanish word"), « Palabra ${feminine} » (« Palabra española »), the name from `languages.ts`; the catalogue's alignment test and the labels lint pass (no language name outside the labels module).

## 4. Tests

- [x] 4.1 `test/selection-card.spec.ts`: each path (click, one-word selection over a token, loose word) re-reads an empty answer and shows the other language's card with its line, band and `readIn`; a known, learning or ignored word, a name, a ranked word without a gloss, a word no other language glosses, an expression and a single accepted language are not re-read and their cards are unchanged; the first glossing language wins in the profile's order; a non-glossing answer is skipped; a late re-read replaces the card only while it is shown; no translation is asked for a re-read card.
- [x] 4.2 Session tests: a gesture on a re-read card writes Spanish and nothing French (status, card with es gloss, learnt-word count); the re-read word is unpainted after « Je connais » and painted learning after « + Deck », the page's percentage and exposures unchanged; the memory is emptied on a new document; a page declaring `ca`, `gl`, `oc`, `glg` gets no Spanish re-read, one declaring `ro` no French one, one declaring `fr` a Spanish one when French is studied; a French speaker's French page (not analysable, or declaring `fr`) gets no re-read — « demande » and « hablaba » keep today's card.
- [x] 4.3 Card view tests: the language line in the three interface languages; the word's button speaks with the card language's voice, the sentence's with the document's; a card without `readIn` renders byte for byte as before.
- [x] 4.4 Mocks follow the extension's conventions; coverage ≥ 80 %.

## 5. Gates

- [x] 5.1 Nothing else moves: `git diff --stat origin/main -- crates scripts/lingua-data apps/lingua-extension/test/baseline` is empty; `cargo test -p lingua-wasm` passes without re-blessing; `word-grammar.spec.ts` and the card snapshots pass as committed.
- [x] 5.2 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test`, `yarn build` (the bundles' growth against `main` per target in the pull request), `yarn check:variants`.
- [x] 5.3 The design's measurement re-run through the implementation on the same pages (the click path through `SelectionCards` with the real packs), its counts in the pull request beside the design's.
- [x] 5.4 `openspec validate read-lingua-card-in-another-studied-language --strict` passes; `python3 scripts/openspec_archive_order.py read-lingua-card-in-another-studied-language` exits 10 naming only the open changes of `archiveAfter` (0 once they are archived); row 52b of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 6. Owner

- [x] 6.1 [manual] The owner's wording review of the language line's drafts (« Mot espagnol », "Spanish word", « Palabra española ») (M9). Approved as drafted (the owner, 2026-10-11).
- [ ] 6.2 [manual] Dogfood: the owner's case (es-en and fr-en, the French *Grammaire de la langue espagnole*: « hablaba », « Je connais », « + Deck », a second device), a French speaker on the English *Spanish verbs* and on a French page (today's card), a page declaring `ca`; Chromium, Firefox and Safari.
- [ ] 6.3 [manual] The owner releases the extension with the change (M18).
