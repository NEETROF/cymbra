## Why

A large part of the English a learner meets is not in articles but in videos, and on YouTube that
English is written down already: the captions. Today Lingua does nothing useful there. Since
`fix-lingua-dynamic-rescan` the page reader deliberately skips the player's caption line
(`.ytp-caption-window-container` in `EXCLUDED_SELECTOR`, `apps/lingua-extension/src/reading/blocks.ts`):
the player rewrites it every two or three seconds, so reading it as page text made highlights
flicker and counted every rewrite as a new exposure. Skipping it fixed that, and left captions
unread.

A spike (2026-09-23, Chrome 152 on youtube.com) answered the question that decides whether a real
caption mode can be built at all: the full caption track can still be read, but only through the
token the player itself obtains. That answer shapes everything below.

This change was proposed on 2026-09-23 and amended on 2026-10-14 to be ready to build: what has
shipped since (the caption exclusion, online translation, per-language daily statistics) is taken
into account, and the work is split in two so that the part that matters most — reading the
captions as Lingua text — can ship without waiting for the sentence translation line.

## What Changes

- **A YouTube caption mode on desktop** (Chromium, Firefox desktop, Safari on macOS). On a watch
  page, for a reader who studies English and has Lingua switched on, a small script in the page
  reads the list of the video's caption tracks that the page already holds — the list, not the
  captions, with no request of its own and nothing changed in the player — so the « Sous-titres
  Lingua » control appears only on a video that has an English track. Once started, the mode reads the **whole
  English track**, analyses it once, and shows the video's **percentage known before it is
  played**. It starts when the reader asks for it, or automatically under a per-device preference
  that is off by default — never because the extension was granted every site, since it changes
  the player.
- **Lingua's own caption line**, drawn over the player and kept in time with the video, replaces
  the native one while the mode is on: unknown and learning words highlighted exactly as on a page.
  A click on a word or a selected phrase opens the same card as on a page and pauses the video;
  the card shows the engine's translation as a page does when the reader has the engine, and
  « Traduction en ligne » on press, as on a page. A card keeps (on the device) the address of the
  moment it came from.
- **Captions are always read as English**, whatever language the watch page is detected in.
- **Exposure and « Mots lus » count what was shown, not what was fetched.** A caption line counts
  when it was on screen while the video played through it; lines skipped over, `[Music]`-style
  annotations and fillers do not. Caption lines feed the daily « Mots lus » like page text, and are
  also kept apart under their own origin (video) so the statistics can tell them from pages.
- **A degraded fallback**: when the track cannot be read, the mode reads the caption segments the
  player writes into the page, and the popup says the video is in degraded mode.
- **Split out to `add-lingua-youtube-caption-translation`**: the whole-sentence translation line
  under the caption (YouTube's own automatic translation when there is no engine, paced engine
  pre-translation, and the preference that shows or hides it). It archives after this change.
- **Not in scope**: iPhone and iPad (Safari on iPadOS loads YouTube's desktop site, but is left out
  of this first version), Android, Shorts, live streams, embedded players, transcribing audio,
  other video sites, and captions in a language other than English: Spanish is left out by the
  Spanish programme's decision D7 (`docs/lingua/spanish-programme.md`), French by the language
  matrix programme (`docs/lingua/language-matrix-programme.md`), Italian by the Italian programme
  (`docs/lingua/italian-programme.md`). The mode reads English captions for a reader who studies
  English, whatever their native language (French, Spanish or Italian); a reader who does not study
  English is not offered it.

## Capabilities

### New Capabilities
- `lingua-youtube-captions`: reading a YouTube video's English captions as Lingua text — when the
  mode is offered, where the track comes from, the caption line, capture from it, the video
  percentage, what counts as an exposure, the fallback, and restoring the player as the reader
  left it.

### Modified Capabilities
- `lingua-browser-extension`:
  - **No network requests**, as rewritten by `add-lingua-translation-delivery`, gains the online
    translation request that `add-lingua-online-translation` shipped (it never updated this
    requirement) and the caption mode's requests to YouTube.
  - **Daily reading statistics count what was read** counts caption lines played through, and
    keeps their share apart under their own origin.
- `lingua-privacy`: one requirement is **added** — the caption request carries nothing of the
  reader's, and the Lingua annex says so.

Consumed, not redeclared: **The page reader never reads a video player's caption line**, which
`fix-lingua-dynamic-rescan` adds (its code is on `main`); **A card's page address stays on the
device** (a watch address with its time is a page address); **A machine translation is never
stored** (renamed **A machine translation is stored only when the reader keeps it** by the open
`add-lingua-review-translation-fallback`) and, once it lands,
`add-lingua-card-sentence-translation`'s **A card keeps the translation of its sentence** — a card
captured from a caption keeps exactly what a card captured on a page keeps.

## Impact

**Products.** Cymbra Lingua only: the extension's three variants (Chromium, Firefox, Safari via
the Apple host app). No backend change, no new RPC, no `.proto` change. Music, Live, the back
office are untouched; the site's privacy page gains one sentence in the Lingua annex, in French,
English and Spanish, and in Italian on change 41's page where it exists.

**Code.** `apps/lingua-extension`: a YouTube module beside `src/reading/` (track, sentences, timing,
exposure — pure and host-testable), a small script in the page's own world injected by the
background (`scripting.executeScript`, `world: "MAIN"`), the caption line and its card placement
inside the player, a `"video"` surface in the popup (`src/popup/render.ts`), the video share of the
daily statistics (`src/state/dailystats.ts`), new copy in French, English and Spanish, and in Italian
once change 33's `src/i18n/it/` exists (`src/i18n/`), a new build entry (`build.mjs`, `tool/check_variants.mjs`).

**Ordering.** Archives **after** `add-lingua-translation-delivery` (D10): this change's block of
**No network requests** is written on top of that change's text. The open proposal
`add-lingua-usage-counts` writes its own block on top of this one.

**Fragility, stated plainly.** The mechanism depends on YouTube internals that are not a public
API: the caption endpoint, its token, the player's API and its markup. It will break some day; the
design treats breakage as an expected state with a fallback and a signal, not as an exception.

**Stores.** A script of ours in YouTube's own page context on every watch page where the reader
studies English, and the replay of the player's request, must be explained to reviewers
(`apps/lingua-extension/REVIEWERS.md`, AMO in particular, which reads the code) and in the Chrome
Web Store's justifications (`STORE-LISTING.md`). No permission is added.

**Owner's time.** Three short probes on real browser profiles (task group 1, about a day) must come
before any code: automated browsers land on YouTube's consent page and get a different token, so
they cannot answer.

**Size.** About 12–15 ideal days of implementation after the owner's day of probes; the
translation line (`add-lingua-youtube-caption-translation`) is 3–4 more.
