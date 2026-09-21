## 1. Core — the phrase gloss

- [ ] 1.1 Add `analysis/function_words.rs`: the English closed classes as dictionary forms and `is_function_word(lemma, studied)`, with one test per class the spec names (article or other determiner — a possessive and a demonstrative included —, pronoun, preposition or particle, conjunction, auxiliary or modal, negation) and one for content words that rank high (`time`, `say`)
- [ ] 1.2 Make `pipeline::resolve_lemmas` `pub(crate)` and move the token classification of `analyse_page` (plain token, listed compound, unlisted compound, out-of-lexicon proper noun) into one private function; the existing `engine.rs` tests pass unchanged
- [ ] 1.3 Add `PhraseGloss` / `PhraseToken` / `PhrasePart` and `engine::gloss_phrase` + `gloss_phrase_json`: no language gate, no token gate, a gloss for every class, each part classified on its own, `parts` omitted when empty
- [ ] 1.4 Host tests in `engine.rs`, one per scenario of the `lingua-analysis` delta except parity, on a lexicon and pack built inline with exactly the forms the scenario names (`give`/`up`, `city`, `in`/`spite`/`of`, `error`/`prone`, `x-ray`, no form of `Jenkins`)
- [ ] 1.5 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace` (the agent links the core), and `cargo llvm-cov` still ≥ 80% with the shared ignore regex

## 2. WASM binding and parity

- [ ] 2.1 Bind `phraseGloss(text) -> String` on `LinguaEngine` in `crates/lingua-wasm`, next to `gloss`
- [ ] 2.2 `tests/parity.rs`: give the golden helper a file and its own update variable, and add `fixtures/phrase_golden.json` — the phrase gloss of selections made of the fixture pack's own words (a known word, an inflected form, an unlisted compound, a sentence-cased name) — written with `LINGUA_UPDATE_PHRASE_GOLDEN` and asserted on the host and in the WASM run
- [ ] 2.3 `git diff --exit-code crates/lingua-wasm/tests/fixtures/golden.json` is clean and the page-analysis parity test passes without `LINGUA_UPDATE_GOLDEN`: the proof that the page analysis did not move

## 3. Extension — the port seam

- [ ] 3.1 Add the `PhraseGloss` types to `src/analyzer/types.ts` and `phraseGloss(text)` to `AnalyzerPort`; declare it on `WasmEngine`, implement it in `WasmAnalyzerPort` (JSON parse) and `MessagingLinguaPort` (RPC — `handleRpc` dispatches by name)
- [ ] 3.2 Add it to every `AnalyzerPort` double: `makeFakePort` in `test/helpers.ts` and the `fakePort()` literal in `test/scan.spec.ts`; `yarn typecheck` passes
- [ ] 3.3 `yarn gen:wasm`, then confirm `yarn build` fails with the guard's message when `src/wasm/pkg` predates the binding

## 4. Extension — the decisions

- [ ] 4.1 `classifySelection`: whitespace makes a phrase, a hyphen no longer does; update its doc comment, `CaptureKind`'s, and the test that pins `repo-wide` as a phrase
- [ ] 4.2 Add `src/reading/selection-card.ts` with injected ports and no DOM, and move `statusOfClass` and `rarityText` into it from `content.ts` (`test/rarity-text.spec.ts` follows): the routing of D4 (whitespace → expression card; page token → word card plus its follow-up; otherwise the first token of the phrase gloss, a proper noun or no token giving today's raw-text card), the row rule of D5 (compound parts as candidates, alone or inside a phrase, none when the compound itself is known or ignored), and the choice of a card's gloss (none for an expression)
- [ ] 4.3 In the same module, the life of a request: settled exactly once by the answer, the rejection or a 3-second timer on an injected clock; completed only if the card view's generation is unchanged; the fallback offering actions only when their key does not depend on the answer; an answer after the fallback dropped
- [ ] 4.4 In the same module, the click rule of D4: a card opened by a capture since the last pointer-down makes the following click open and hide nothing, while a click on a painted untreated word inside a link is still cancelled
- [ ] 4.5 `test/selection-card.spec.ts`: one test per scenario of the `lingua-browser-extension` delta, plus the card-gloss rule (an expression never gets one, whatever the pack holds) and the click rule for a plain click, the click that ends a gesture, and that click landing on an untreated word inside a link

## 5. Extension — the card

- [ ] 5.1 `CardView`: a generation bumped by every `show` and every `hide` — its own close button and the hide that follows an action included — returned by `show` and readable; `WordPopup` exposes it
- [ ] 5.2 `WordPopupContent`: a pending state (headword, kind line, waiting line, close button, no action), labelled rows, and the no-translation line for an expression; `Gesture` gains `expression`, set from the content
- [ ] 5.3 Styles from `tokens.css` only; no user string holds "lemme" (`lint-hex`, `lint-lemma` pass)
- [ ] 5.4 `test/wordpopup.spec.ts`: a pending card offers no action and can be closed; the generation changes on `show`, on `hide`, on the close button and after an action; rows render under the label; the no-translation line renders with no row; the gesture of an expression card carries `expression: true`

## 6. Extension — wiring

- [ ] 6.1 `content.ts`: `onCapture` and `showPopup` delegate to `selection-card.ts` and only pass the contents it returns to `WordPopup.show`; `onGesture` takes the card's gloss from it; `onClick` applies its click rule; pointer-down reports the start of a gesture
- [ ] 6.2 Remove `packGlossKey` and its tests (added by pull request #518, which merges first)
- [ ] 6.3 `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `src/reading/selection-card.ts` is not in the coverage exclude list of `vitest.config.ts`

## 7. On real pages

- [ ] 7.1 With the real pack (`yarn gen:pack:real`) on Chrome macOS, with a mouse: a free combination, a phrasal verb whose words are known, an inflected word in an unanalysed block (the card is keyed by the dictionary form and the word turns "learning" on analysed pages), `don't`, a name, a listed and an unlisted compound on an analysed page, a compound inside a phrase, a known word's gloss, a double-click and a drag whose card survives the click, a double-click on an unknown word inside a link (card open, link not followed), and a click on a highlighted link word while an earlier selection is still live
- [ ] 7.2 Firefox for Android, and Safari on iPhone from a local build of `apps/lingua-apple`: the pending card appears at once while the event page wakes, the answer and the actions arrive together, dragging a selection handle never shows a stale answer, closing a pending card keeps it closed, and tapping a highlighted word while an answer is pending leaves that word's card alone
- [ ] 7.3 Read the two French lines of D5 on a phone with the founder and settle their wording

## 8. Close

- [ ] 8.1 `README.md` of the extension: what a selection opens, the pending card, and the new port method
- [ ] 8.2 `openspec validate add-lingua-phrase-gloss --strict`
- [ ] 8.3 After the extension release, dispatch `lingua-apple-release` with `deliver`, so the Safari variant carries the same build
