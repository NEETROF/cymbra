## 1. The selection watcher (testable core)

- [x] 1.1 In `apps/lingua-extension/src/reading/selection.ts`, add a `SelectionWatcher` with injectable seams: a timer pair (`setTimeout`/`clearTimeout`), a selection reader (defaults to `window.getSelection`), and an `onCapture(kind, capture)` callback where `kind` is `"word" | "phrase"`.
- [x] 1.2 Implement the debounce: every `notify()` restarts a ~350 ms timer; `release()` fires it immediately (the pointer-lift path); a collapsed or empty selection cancels the pending capture without emitting anything. Added beyond the plan: `hold()` stretches the settle time to 1200 ms while a pointer is down, so a slow mouse drag does not pop the card up mid-gesture — it stretches rather than blocks, because a native selection-handle drag may never send the lift.
- [x] 1.3 Classify the settled selection — over `MAX_SELECTION_LENGTH` emits nothing, a text holding a space or a hyphen is `"phrase"`, anything else is `"word"` — reusing the existing `/[-\s]/` rule so the mouse behaviour is unchanged.
- [x] 1.4 Ignore selections anchored inside a `[data-cymbra-lingua-skip]` host, so the popup's and drawer's own shadow content never triggers a capture.
- [x] 1.5 Drop the re-selection in `captureSelection` (`sel.removeAllRanges()` / `sel.addRange(range)`): keep the word-boundary snap internal to the returned `Capture` and leave the page selection untouched. Expose the snapped `Range` on the `Capture` so callers can hit-test it.
- [x] 1.6 Unit-test the watcher in `test/selection.spec.ts` with fake timers: debounce coalescing, `flush()`, collapse-cancels, the three classifications, the skip-host guard, and that `captureSelection` no longer mutates the document selection.

## 2. Wire it into the content script

- [x] 2.1 In `content.ts`, construct the watcher in `start()` and replace the `mouseup` listener with `document.addEventListener("selectionchange", …)` calling `notify()`.
- [x] 2.2 Keep `mouseup` and add `touchend` purely as `release()` calls — no capture logic of their own; `mousedown`/`touchstart` call `hold()` and `touchcancel` calls `release()` so the held state cannot wedge.
- [x] 2.3 Route `kind === "word"`: resolve the snapped range through `hitAt(node, offset, true)` and render with `showPopup(hit)` (status-aware); fall back to the existing capture card on a miss.
- [x] 2.4 Route `kind === "phrase"` to the existing expression card path (`this.popup.show({ …, expression: true })`), unchanged.
- [x] 2.5 Make the `captureSelection` message handler (the keyboard shortcut, `background.ts:436`) go through the same routing so the shortcut and the pointer produce identical panels.
- [x] 2.6 Delete `onTouchStart`, `onTouchMove`, `onTouchEnd`, `onTouchCancel`, `cancelLongPress`, `fireLongPress`, the `longPress` / `longPressFired` / `suppressClickUntil` / `suppressSelection` fields, the `LONG_PRESS_MS` / `LONG_PRESS_MOVE_TOL` constants, and the four touch listeners in `start()`.
- [x] 2.7 Delete the `selectstart` and `contextmenu` suppressors (`content.ts:198`, `content.ts:205`) — they existed only for the long-press.
- [x] 2.8 Drop `onClick`'s `suppressClickUntil` early-return; keep its live-selection guard so a click landing during a drag does not open the wrong single-word popup ahead of the debounce.
- [x] 2.9 Delete `openWordAt`, whose only caller was `fireLongPress`, and grep the extension for remaining references to the removed fields and comments mentioning the long-press (`hitAt`'s doc comment, `showPopup`, `onClick`'s "Alt-click / long-press" notes) and correct them.

## 3. Popup placement next to the native callout

- [x] 3.1 In `positionCard` (`reading/wordpopup.ts`), reserve a callout gutter when the card flips above the selection on a touch device, so it does not land under the platform's Copier/Rechercher bar.
- [x] 3.2 Cover the flip-with-gutter case in `test/wordpopup.spec.ts`.

## 4. Gates

- [x] 4.1 `cd apps/lingua-extension && yarn typecheck && yarn lint && yarn test && yarn format:check`.
- [x] 4.2 `yarn build` (all variants) and grep `dist-chromium/`, `dist-firefox/` and `dist-safari/` to confirm no long-press code survived the bundle.
- [x] 4.3 Confirm coverage stays ≥ 80 % for the package; the deleted touch handlers should raise it, not lower it.
- [x] 4.4 `openspec validate fix-lingua-touch-selection --strict`.

## 5. On-device passes

- [x] 5.1 iPhone / Safari (via `apps/lingua-apple`): select three words → the expression card opens and "+ Deck" creates the phrase card with its source sentence. Done 2026-09-21, iPhone 15 Pro Max / iOS 27.2, local development build (real en-fr pack, production backend).
- [x] 5.2 iPhone: press-and-hold a single word → the native selection appears *and* the popup opens with the actions matching that word's status (including a word already marked Known or Ignored). Done 2026-09-21 — the card carries "forme vue" and the rarity line, so the word is resolved through the hit-test rather than falling back to the generic capture card.
- [x] 5.3 iPhone: drag a native selection handle from one word to three → the panel follows and settles on the phrase, with no extra gesture. Done 2026-09-21. This is the pass with no pointer-event backing at all — iOS selection handles are native UI — so it is the debounce alone that carries it.
- [x] 5.4 iPhone: confirm the popup and the native Copier/Rechercher bar do not overlap, near the top and near the bottom of the viewport. Done 2026-09-21 — and it corrected the premise: iOS places its callout BELOW the selection when there is room, the same preference the card has, so they overlap in the common case and not (thanks to the gutter) in the flipped one. Widening the gutter to both branches was built and tried on device, then rejected by the user: it pushes the card too far from the words. Overlap accepted — the bar is one tap from gone and only really shows under the taller multi-word card. See design.md.
- [x] 5.5 Firefox Android: the same three passes (phrase, single word, handle drag). Done 2026-09-21 on a Galaxy Tab S6 Lite (`org.mozilla.firefox`, web-ext temporary add-on, real en-fr pack). A single word opens the status-aware popup — the card carries "forme vue" and the rarity line, which only `showPopup(hit)` renders — and stops offering "Je connais" once the word is marked known. A handle drag from one word to three lands on the expression card with no further gesture. The platform callout and the card coexist without overlapping, both when the card sits below the selection and when it flips above.
- [ ] 5.6 Desktop Chromium and Firefox: a mouse drag still opens the expression card with no perceptible delay, a plain click on a painted word and Alt-click on a non-painted one are unchanged, and the keyboard shortcut still works.
- [ ] 5.7 Confirm the debounce value on device and adjust the constant if 350 ms reads as laggy or as too eager (design's open question).
