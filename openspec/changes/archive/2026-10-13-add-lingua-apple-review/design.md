# add-lingua-apple-review — design

## Context

See proposal.md for the why. What exists:
- **The app** (`apps/lingua-apple`) shows `Main.html` in a `WKWebView` (`Shared (App)/ViewController.swift`):
  how to turn the extension on, and the sign-in hand-off (`LinguaSignIn`). The iOS app and extension
  share the App Group `group.com.cymbra.lingua`; the macOS targets share `VMFJ6KRW77.com.cymbra.lingua`
  (`IdTokenHandoff.appGroup`), already in both macOS entitlements.
- **Native messaging**: the extension's `SafariWebExtensionHandler` answers `browser.runtime.sendNativeMessage`
  from the background (the sign-in hand-off reads the App Group).
- **The extension's state** is one backup string, saved by the background owner (IndexedDB), and the
  engine applies remote changes as ops under last-write-wins (`applyCardOps`, `applyStatusOps`).
- **The Révision page** (`mountReview`) is plain DOM over a `LinguaPort` and an `AsyncStorageArea`;
  the engine port can run the WASM engine in the page (`WasmAnalyzerPort`, as Chromium's content script).
- The mobile reader study (2026-10-01) recommended thin native shells around the extension's web code
  over a native rewrite.

## Goals / Non-Goals

**Goals:** the app reviews on every Apple device with the extension's review; one implementation.

**Non-Goals:** reading books or pages in the app; an Android app; sync with an account from the app
(the extension already syncs, and the app's changes reach it); notifications; widgets.

## Decisions

### D1 — A fourth build target, `app`
`build.mjs` gains `app`: an entry page mounting the Révision and Statistiques pages with
`WasmAnalyzerPort` in the page (packs read from the bundle), a storage adapter over the app's bridge,
and no `chrome.*`. The catalogue, the review view and its CSS are the same files. `check_variants`
gains the target; the placement lint already keeps the translation engine out of painting threads.

### D2 — The app serves the bundle under a custom scheme
A `WKURLSchemeHandler` serves the `app` build and the packs from the app bundle (`lingua-app://`),
so the WASM engine and module scripts load under one origin, not `file://`. The packs are read from
the Safari extension the app embeds, not copied a second time (about 13 MB).

### D3 — The state crosses the App Group as a snapshot one way and ops the other
- Extension → app: after each save, the background sends the backup — with the preferences the
  review reads and the daily counts Statistiques draws — to its native handler, which writes it to
  the App Group (debounced), in a file rather than the group's defaults. The app's page loads it into its engine at launch and when
  the app returns to the foreground.
- App → extension: the app's page exports the card and status ops its review produced
  (`exportCardOps`, `exportStatusOps` since its load) to the App Group. The extension's background asks
  its handler for them at startup and on a visible page, applies them with `applyCardOps` /
  `applyStatusOps`, saves, and clears them. The day's reviews and words learned travel beside them,
  per opening of the app, and are added once (see Risks). Last-write-wins on both sides: no snapshot ever overwrites
  newer data. Nothing leaves the device; the extension's own sync then carries the result to an account.

*Alternative.* The app syncing with the server itself: needs the reader signed in, a Dart or Swift
sync client, and does nothing offline.

### D4 — The platform's voices and the interface language
Read-aloud uses WebKit's `speechSynthesis` in the web view, under the read-aloud rules. The
interface language is the one the App Group already holds for sign-in (`InterfaceLanguageStore`).

## Risks / Trade-offs
- [The extension has not run since the last capture] → the app shows the state it last received, and
  says when it was saved.
- [An op lost if the app is killed before writing] → ops are written after every answer.
- [A review's daily counts counted twice] → the app sends, with its ops, the whole of what each
  opening counted (reviews, words learned), under that opening's id; the extension keeps what it
  already added per opening and adds only the difference, so a payload written or read again
  adds nothing.

## Migration Plan
App and extension ship together; an older extension writes no state, and the app then shows how to
turn the extension on.
