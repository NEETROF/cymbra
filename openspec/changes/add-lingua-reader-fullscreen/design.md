## Context

The reader page is `reader.html`, an extension page opened in a tab (`add-lingua-reader` D1).
`ReaderApp` (`src/reader/app.ts`) builds its DOM into `#reader-root`: a reading toolbar
(`.reading-bar`), a stage with the book and its panels, a footer. The reading module's own
surfaces — the word popup (`reading/wordpopup.ts`), the drawer (`reading/drawer.ts`) — attach
their shadow hosts to `document.documentElement` of the reader page. The toolbar's Réviser /
Stats / Réglages call the session's `HudActions`: the in-page drawer on Firefox and Safari
(`__REVIEW_IN_PAGE__`), the browser's side panel on Chromium (a message to the background).

Browser support of the Fullscreen API on an arbitrary element, as the reader will meet it:

| Host                                          | Element fullscreen                                                        |
| --------------------------------------------- | ------------------------------------------------------------------------- |
| Chrome / Edge / Firefox desktop, Safari macOS | yes                                                                       |
| Safari iPadOS 26, extension page              | **no** — measured: no Fullscreen API at all on the reader page (task 3.4) |
| Firefox for Android                           | yes (to verify on the device, in an extension page)                       |
| Safari iOS (iPhone)                           | no — video elements only; `fullscreenEnabled` is false                    |

## Goals / Non-Goals

**Goals:**

- One toolbar control that hides the browser's bars and brings them back, on every host that
  allows it, with everything the reader shows still on screen.
- The control's state follows the browser's, whichever way fullscreen was left.

**Non-Goals:**

- iPhone and iPad. Safari exposes no Fullscreen API to the extension's page there (measured on
  iPadOS 26); an installed web app would hide the bars, but that is the PWA D1 rejected.
- Entering fullscreen automatically, or restoring it when a book opens: browsers refuse
  fullscreen outside a user gesture.
- A keyboard shortcut. The browser's own (F11, ⌃⌘F) already hide its bars on desktop; the
  control is for the touch devices where there is no such key.
- Fullscreen in the library view as a feature of its own: it stays in fullscreen if the reader
  goes back to it, but the control lives in the reading toolbar.

## Decisions

### D1. The whole page goes fullscreen, not the book

`document.documentElement.requestFullscreen()`. Only the fullscreen element and its
descendants are painted in fullscreen; the popup and the drawer are children of
`documentElement`, not of the book or of `#reader-root`, so making the book (or the root)
fullscreen would hide them. The page's root is the one element that contains everything.

Alternative: fullscreen on `.reading-view`, moving the shadow hosts under it — touches the
reading module, shared with the content script, for no gain.

### D2. A seam, `FullscreenHost`, injected into `ReaderApp`

```ts
interface FullscreenHost {
  readonly available: boolean; // fullscreenEnabled (or webkit)
  active(): boolean; // fullscreenElement != null (or webkit)
  enter(): Promise<void>;
  exit(): Promise<void>;
  onChange(listener: () => void): void; // fullscreenchange (or webkitfullscreenchange)
}
```

`src/reader/fullscreen.ts` implements it over a `Document`, with the `webkit` fallbacks for
iPadOS before 16.4; `ReaderApp` receives it through `ReaderDeps` (default: none — no control),
as it already receives the library and the renderer. jsdom has no Fullscreen API, so the app's
tests pass a fake; the document adapter is tested against a fake document. A refusal (the
promise rejects: no gesture, a policy) leaves the control as it was: the state is read from
`onChange`, never assumed from the call.

### D3. The control follows `fullscreenchange`, not its own clicks

The label and `aria-pressed` are set from `active()` on every change event. Escape, Safari's
close control, the Android back gesture and a refused request all end in the same place: the
control says what pressing it does now.

### D4. On Chromium, leave fullscreen before opening the side panel

The side panel is browser chrome; a tab in fullscreen shows none. `ReaderDeps` gains
`reviewOutsidePage` (`!__REVIEW_IN_PAGE__`, set by `reader.ts`): when true and the page is in
fullscreen, Réviser / Stats / Réglages call `exit()` then the action, **synchronously in the
click** — the background opens the panel with the click's user gesture (`sidePanel.open`
requires one), so the action is not delayed until the exit settles. On Firefox and Safari the
drawer is in the page and fullscreen is kept.

Alternative: switch to the in-page drawer while in fullscreen on Chromium. The drawer is not
built into the Chromium bundle (`__REVIEW_IN_PAGE__` is a compile-time constant), and a second
review host on one browser is two things to keep working.

### D5. Safe area through `viewport-fit=cover` and `env(safe-area-inset-*)`

`reader.html`'s viewport gains `viewport-fit=cover`; `.reading-bar` and `.reading-foot` pad
with `env(safe-area-inset-*)` (zero where there is no notch or indicator, so desktop and
windowed layouts are unchanged). Only the page's own bars move; the book's paging is untouched.

### D6. The words

`COPY.fullscreen` "Plein écran" and `COPY.leaveFullscreen` "Quitter le plein écran", shown as
a symbol (⛶) with the words as its label and title, like the "Aa" control. French only, as
every sentence of the extension.

## Risks / Trade-offs

- [Chrome shows the side panel badly after a fast exit] → verified by hand in a headed Chrome
  (task 3.2); if it fails, the action waits for `fullscreenchange` inside the same task, which
  still carries the gesture on Chrome's user-activation v2 (5 s window).
- [Firefox for Android refuses fullscreen to an extension page] → `available` is false there
  and the control is not shown; checked on the device (task 3.3).
- [iPadOS swallows the first tap after entering fullscreen, or a selection gesture near the
  top edge pulls down Safari's close control] → checked on the iPad (task 3.4); the book's tap
  zones are the outer thirds, not the top edge.
- [Keyboard input is restricted in Safari fullscreen] → the reader types nothing; arrow keys
  are not affected.

## Migration Plan

None: no stored state, no permission. Shipped with the next extension release — the same
release as `add-lingua-reader` (its task 7.4); this change archives after it.
