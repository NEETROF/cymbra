# add-lingua-apple-review — the Cymbra Lingua app reviews the deck

## Why

On an iPhone, an iPad or a Mac, the review lives inside Safari: the reader opens a page, then the
extension's drawer, to review a deck they built while reading. The Cymbra Lingua app is already
installed — it carries the extension — yet it only explains how to turn the extension on. A review
is a few minutes on the sofa or the train, and the place to start one is an app, not a web page.

The review is changing fast (`refine-lingua-review-session`, the review translation, the chapter
recap). One implementation keeps every device on the same review: the app shows the extension's own
Révision page, decided with the product owner on 2026-10-12 over a Flutter rewrite.

## What Changes

- **The app opens on the review**: the same Révision page the drawer shows — language filter, due
  count, « Réviser », sessions, answers, read-aloud, end summary — in the app's web view,
  laid out for an upright phone, a tablet and a Mac window.
- **One code base**: a fourth build of the extension's code, for the app, with the analysis engine
  running in the page and no browser-extension API. Every change to the review reaches the app.
- **The deck comes from the extension, on the device**: the extension saves its state into the
  container it shares with the app; the app reviews it and hands back what changed; the extension
  applies those changes the way it applies a sync, so neither side overwrites the other. Nothing
  leaves the device for this.
- **Without the extension's data yet**, the app says how to turn the extension on, as today.
- Unchanged: the extension's own review, sync with an account, and the App Store listing's scope.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-apple-app`: added — the app reviews the deck the extension keeps; a review made in the app
  reaches the extension.

This change archives after `refine-lingua-review-session`, whose Révision page it shows.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The Apple container app (`apps/lingua-apple`, iOS, iPadOS,
  macOS) and the extension (`apps/lingua-extension`): a new `app` build target and the state relay
  through the shared container. Consumes the existing Révision page, engine and catalogue.
- Cymbra ID, Music, Live, the back office and the public site are untouched. No backend, no `.proto`,
  no flag. The core and its WASM build are untouched.

**Code.** `apps/lingua-extension/build.mjs` (the `app` target), an app entry page and its storage
adapter, `src/state/` (the relay of the state and of the app's changes);
`apps/lingua-apple/Shared (App)/ViewController.swift` (the web view, a scheme handler serving the
bundle, a script bridge), `Shared (Extension)/SafariWebExtensionHandler.swift` (state in, changes
out), the macOS targets' App Group entitlement.

**Store.** A real review in the app strengthens the listing (App Store guideline 4.2); the listing
text is the product owner's, with the release.
