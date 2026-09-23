# add-lingua-read-aloud — hear the selection and its sentence, on the device

## Why

The Lingua card tells a reader what a word or a phrase means, but never how it sounds. A
learner who meets `thorough` or `colonel` on a page has no way to hear it without leaving the
page for a dictionary site, and a sentence read aloud is how most learners check that they
parsed it right. Every target Lingua ships on — Chromium, Firefox desktop and Android, Safari
macOS and iOS — already carries a speech synthesiser the page can drive, with voices installed
on the device, so this needs no download, no model and no new permission.

The one trap is privacy. Chrome's desktop voice list mixes the operating system's voices with
voices that synthesise on Google's servers, and nothing on the page tells them apart unless the
extension checks. Lingua promises that no page text leaves the device; reading a sentence aloud
with the wrong voice would break that promise silently. This change keeps it by construction.

**Products.** Cymbra Lingua only: `apps/lingua-extension` (all three variants; the Safari
variant ships inside `apps/lingua-apple`, whose native code does not change). Everything here
is new to Lingua and nothing is consumed from the platform. Cymbra ID, Music, Live, the back
office, the site and the backend are untouched: no `.proto`, no migration, no flag, no Rust.

## What Changes

- **Two listen buttons on the card.** The word card and the selection card gain a listen row:
  one button reads the selection as it appears on the page — the word, or the words selected —
  and one reads the whole sentence it was taken from. The sentence button is not offered when
  the selection already is the whole sentence. A button that is speaking becomes its own stop
  button; pressing the other one switches to it.
- **On-device voices only.** The card speaks only with a voice the browser reports as running
  on the device, in the studied language. A voice that synthesises remotely is never used, even
  when it is the browser's default and the only one available. With no eligible voice, the card
  shows no listen row: nothing is offered that would not work or would leak the page.
- **Silence follows the card.** Closing the card — its close button, Escape, a click off it, a
  gesture, the scroll that dismisses it — stops the speech, and so does leaving the page or its
  tab. A pending card completing with its answer does not interrupt a reading already started.
- **A voice choice in Réglages.** A "Lecture à voix haute" block lists the eligible voices, with
  an automatic choice by default and a way to hear each one; it is absent when there is none.
  The automatic choice never lands on a novelty voice (macOS ships en-US voices such as
  "Bubbles" and "Zarvox" that sort before the real ones).

Not in this change: a slower speaking rate, highlighting the words in the page as they are
spoken, listen buttons on the review cards of the deck, a voice bundled with the extension, and
any remote voice, even opt-in — each is its own change if wanted. No manifest, permission, store
listing, privacy text, card model or sync contract changes.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`: new requirements for reading the selection and its sentence
  aloud from the card, for read-aloud never leaving the device, and for the voice choice in
  Réglages. Every delta is ADDED. "Word popup on click", "Minimal permission posture" and "No
  network requests" keep their text and stay true: the popup still shows everything it did, no
  permission is added, and no voice that would issue a request is ever used.

## Impact

- **Extension**: a new `src/reading/speech.ts` — the eligible voices, the automatic choice and
  a speaker that owns one utterance at a time — with the browser's synthesiser behind an
  injected seam, so it is tested in jsdom (which has none) and stays inside the 80 % gate.
  `src/reading/wordpopup.ts` gains the listen row, `src/reading/settings-view.ts` the voice
  block (one builder for the side panel and the in-page drawer), `src/state/storage.ts` the
  voice preference, `src/styles/wordpopup.css` and `settings.css` their styles from
  `tokens.css`, and `src/content.ts` the wiring as a thin caller.
- **Background**: untouched. Speech runs where the card runs, in the content script, on every
  variant — the engine living in the event page on Firefox and Safari does not matter here.
- **The EPUB reader** (`add-lingua-reader`, docs merged, implementation to come) hangs the same
  card, so it gets the listen row without further work. So does the caption line of
  `add-lingua-youtube-captions` (docs merged), whose popup pauses the video before it opens: the
  voice never talks over the soundtrack, and closing the card silences it before the video
  resumes.
- **Dogfooding**: the voice lists differ by device — macOS Chrome (novelty voices, Google remote
  voices), Windows, Chrome on Android, Firefox for Android (GeckoView), Safari on iPhone (the
  ring/silent switch) — and none of them exists in jsdom, so the on-device pass is part of the
  change, not an afterthought.
- **Release**: an ordinary extension release on the two stores, then a dispatch of
  `lingua-apple-release` with `deliver`, so the Safari variant carries the same build. Nothing
  for a store reviewer to re-read: no new origin, no new permission.
