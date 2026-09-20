## Context

The reader has two independent ways to turn a selection into a card, and neither
reaches a phone:

- `document.addEventListener("mouseup", …)` (`content.ts:216`) → `onMouseUp`
  (`content.ts:582`), which tests the selection for a space or a hyphen and calls
  `onCaptureSelection`. Touch devices never fire `mouseup`, and iOS's selection
  handles are native UI that does not reliably dispatch DOM touch events either.
- the keyboard command, which the background forwards as a `captureSelection`
  message (`background.ts:436`). No keyboard, no capture.

On top of that, `content.ts` runs its own long-press gesture — `onTouchStart`
arms a 500 ms timer (`LONG_PRESS_MS`), `fireLongPress` (`content.ts:519`) opens the
single-word popup, calls `window.getSelection()?.removeAllRanges()` and raises
`suppressSelection`, which `preventDefault()`s `selectstart` and `contextmenu` for
a second. On iOS that is the *same physical gesture* as "start a text selection",
so the two race: sometimes Lingua's one-word popup wins, sometimes the native
Copier/Rechercher bar does, often both appear. This is what dogfooding on iPhone
reported.

Constraints that shape the fix: the content script is the only context that sees
the page selection (`browser-extension-architecture`, context table); a web
extension cannot add items to the iOS callout bar; and `content.ts` is a 32 KB
module with no direct unit test, so new logic has to land somewhere testable to
keep the 80 % gate honest.

## Goals / Non-Goals

**Goals:**

- One capture trigger, driven by the selection itself, working identically on
  mouse, touch and the keyboard shortcut.
- A one-word selection opens the popup with the actions that match that word's
  real status; several words open the expression card.
- Delete the touch long-press gesture and everything that exists only to serve it,
  so the reader stops fighting the platform's selection.
- The new routing logic is unit-testable without a browser.

**Non-Goals:**

- Changing the keyboard shortcut, `onClick`, or Alt-click desktop reclassification.
- Any new UI surface, drawer, or side-panel work.
- Suppressing or customising the native iOS callout bar — a web extension cannot,
  and trying is what broke this in the first place.
- Touch gestures of Lingua's own invention (tap-first-word / tap-last-word). The
  user chose the native selection as the single gesture.

## Decisions

### Debounced `selectionchange` is the trigger; `mouseup`/`touchend` only flush it early

`selectionchange` is the one event every path shares — a mouse drag, a keyboard
shift-arrow, a native handle drag, and iOS's press-and-hold word selection all emit
it. It fires continuously during a drag, so the handler debounces (~350 ms of
stability) and captures once the selection has settled.

A pointer lift is a reliable "the selection is finished now" signal where it exists,
so `mouseup` and `touchend` **flush the pending debounce** rather than starting a
second capture path: desktop keeps today's instant feel, and the debounce remains
the guarantee for the handle drags that emit no touch event. One capture function,
two flush conditions.

*Alternatives:* adding a `touchend` listener mirroring `mouseup` — rejected, native
handle drags do not reliably dispatch it, which is the bug. Polling the selection on
a timer — rejected, wasteful on every page the reader touches.

### The reader stops re-selecting, clearing and suppressing

Three behaviours go:

- `fireLongPress`'s `removeAllRanges()` — it deletes the selection the reader is
  making.
- the `selectstart` / `contextmenu` suppressors (`content.ts:198`, `content.ts:205`)
  — they exist only to hide the callout the long-press raised.
- `captureSelection`'s re-selection of the snapped range (`selection.ts:64-70`).
  Re-applying a range fires `selectionchange` again (re-entrancy) and, on iOS,
  tears down and redraws the native callout. The word-boundary snap stays — it is
  what makes "the parity pro|of" capture as "proof" — but it stays *internal to the
  capture*; the page's own selection is left alone. Cost: on desktop the visible
  highlight can now be one or two characters narrower than the captured card. That
  is the trade the spec's "never fights the platform's selection" requirement buys.

### A one-word selection routes through the existing hit-test

`onCaptureSelection` shows the popup with no `status`, so `wordpopup.ts:116` offers
"Je connais" even for a word already known. For a single-word capture the handler
resolves the snapped range through `hitAt(node, offset, /* allowReclassify */ true)`
— the same call Alt-click uses — and, on a hit, renders through `showPopup(hit)`,
which is status-aware. On a miss (a word in a block the analyser never saw) it falls
back to today's capture card. This is what preserves reclassification on touch after
the long-press is deleted: press-and-hold selects the word, the popup opens with
"Ignorer" / "Remettre à apprendre" as appropriate.

### The routing decision lives in `reading/selection.ts`, not `content.ts`

`content.ts` has no unit test. The new logic — debounce, "has the selection
settled", one-word vs phrase vs too-long classification, and the
ignore-our-own-shadow-host guard — goes into `reading/selection.ts` as a small
watcher with injectable timers and an `onCapture(kind, capture)` callback.
`content.ts` keeps only the wiring: construct it, and route `kind` to
`showPopup(hit)` or the capture card. That keeps the change inside the file
`selection.spec.ts` already covers.

### Self-suppression guard

`selectionchange` fires for selections inside the reader's own closed shadow hosts
too (the popup, the drawer). The watcher ignores any selection whose anchor node is
inside a `[data-cymbra-lingua-skip]` host — the same marker `blocks.ts` /
`observer.ts` already honour — and ignores a *collapse* to empty rather than
treating it as "hide the popup", because pressing a popup button collapses the page
selection (the mechanism the architecture skill's `mousedown`/`preventDefault` note
describes).

### The callout bar is accommodated, not fought

`positionCard` (`wordpopup.ts:194`) prefers just below the selection and flips above
only when there is no room. iOS draws its callout *above* the selection, so in the
common case the two do not overlap. When the card does flip above, it reserves a
callout gutter (~44 px) on touch so it lands over the callout's position rather than
under it. Verified on device, not in a test.

## Risks / Trade-offs

- **[The 350 ms debounce feels laggy on desktop]** → `mouseup` flushes it, so the
  mouse path stays immediate; the delay is only paid where nothing signals the end
  of a drag.
- **[`selectionchange` is poorly supported in jsdom]** → the watcher takes its
  timer and its selection reader as injected seams, so the unit tests drive it
  directly; only the `document.addEventListener` line stays untested, in
  `content.ts`, where nothing is tested today anyway.
- **[Deleting the long-press regresses a desktop habit]** → it does not: the
  long-press handlers are touch-only (`onTouchStart` et al.); Alt-click is the
  desktop reclassify path and is untouched.
- **[Dropping the snap re-selection makes the highlight disagree with the card]** →
  cosmetic, desktop-only, and the alternative re-introduces the re-entrancy plus the
  iOS callout flicker. Revisit only if dogfooding complains.
- **[iOS may keep firing `selectionchange` while the magnifier is up]** → the
  debounce restarts on each one, so the capture lands when the reader stops moving;
  worst case the popup appears one debounce after the finger settles.
- **[A page with its own `selectionchange` handler]** → the listener is passive and
  never calls `preventDefault`, so the page keeps its behaviour. Removing the
  `selectstart`/`contextmenu` suppressors strictly *reduces* the reader's
  interference with the host page.

## Migration Plan

No data, no storage, no protocol: the change is behavioural and ships with the
extension build. Rollback is reverting the commit. Because the same source produces
`dist-chromium`, `dist-firefox` and `dist-safari`, the change reaches all three at
once, and the on-device passes (iPhone via `apps/lingua-apple`, Firefox Android)
gate the release rather than a flag.

## Open Questions

- Is 350 ms the right debounce on a real iPhone, or does the magnifier's event
  stream want more? Settle on device, the constant is one line.
- Does dragging an iOS selection handle dispatch `touchend` to the page in the
  Safari version we target? If it does, the flush makes the capture instant there
  too; if not, the debounce carries it. Either way the behaviour is correct — this
  only decides how snappy it feels.
