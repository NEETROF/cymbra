## Why

Selecting several words on a phone does nothing. The whole-selection card — the
path that turns "a bounded phrase" into an expression card — is wired to a single
`mouseup` listener (`apps/lingua-extension/src/content.ts:216`), an event touch
devices never fire; the only other entry point is a keyboard shortcut, unreachable
on a phone. Worse, Lingua's own 500 ms long-press *is* iOS's start-a-selection
gesture: `fireLongPress` opens the single-word popup, calls `removeAllRanges()` on
the selection the reader just started and `preventDefault()`s `selectstart` for a
second. The two gestures race, so the reader gets a one-word popup, the native
Copier/Rechercher bar, or both — never the phrase card. Reported from dogfooding on
iPhone; the same hole exists on Firefox Android and Chrome Android.

## What Changes

- **The native selection becomes the single capture trigger, on every pointer.** A
  debounced `selectionchange` replaces `mouseup`: when the selection settles and is
  non-collapsed, one word opens the status-aware word popup and several words open
  the expression card. Desktop mouse behaviour is preserved by the same code path,
  so there is one trigger instead of two.
- **The touch long-press reclassify path is removed.** `onTouchStart` /
  `onTouchMove` / `onTouchEnd` / `onTouchCancel` / `fireLongPress` and their
  `suppressSelection` / `longPressFired` / `suppressClickUntil` machinery go away
  with it, together with the `selectstart` and `contextmenu` suppressors that exist
  only to serve them. Reclassifying a Known/Ignored word on touch survives as the
  same physical gesture — a press-and-hold already makes iOS select the word under
  the finger, which now opens the popup with its real status.
- **A single-word capture resolves its status.** `onCaptureSelection` currently
  shows the popup with no `status`, so an already-Known word is still offered "Je
  connais". It routes through the existing hit-test so the offered actions match the
  word's state.
- Alt-click on desktop keeps reclassifying a non-painted word; a plain tap on a
  painted word keeps opening its popup. Neither is touched.
- The native iOS callout bar (Copier / Rechercher) is accepted, not fought: keeping
  the selection means keeping its menu. The popup is anchored so the two do not sit
  on top of each other.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: "Selection capture on a keyboard shortcut" becomes
  selection capture on *any* pointer — the requirement stops being keyboard-only and
  gains the touch behaviour and the one-word/many-words split.

## Impact

**Product: Cymbra Lingua only.** No backend, no proto, no other app.

- `apps/lingua-extension/src/content.ts` — the listener set (`mouseup` and the four
  touch handlers out, debounced `selectionchange` in), `onClick`'s live-selection
  guard, `onCaptureSelection`.
- `apps/lingua-extension/src/reading/selection.ts` — unchanged contract; the
  word-snapping and `Capture` shape are reused as-is.
- Tests: `test/selection.spec.ts` extended, plus a new spec for the debounce and the
  one-word/many-words routing. The removed touch handlers take their dead paths out
  of the coverage surface.
- Consumed, not redeclared: the word popup (`reading/wordpopup.ts`) already accepts
  `status` and `expression` — no new UI surface is introduced, so the
  `browser-extension-architecture` injected-surface rules apply unchanged.
- On-device validation required on iPhone (Safari, via `apps/lingua-apple`) and
  Firefox Android; desktop Chromium/Firefox regression-checked for the mouse path.
