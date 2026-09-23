## Why

A large part of the English a learner meets is not in articles but in videos, and on YouTube that
English is written down already: the captions. Today Lingua does nothing useful there. Its
observer would see the caption segments the player writes into the page, but they are rewritten
every two or three seconds (word by word for automatic captions), so highlights would flicker,
every rewrite would count as a new exposure, and the sentence engine would translate fragments.

It is proposed now because a spike (2026-09-23, Chrome 152 on youtube.com) answered the question
that decides whether this can be built at all: the full caption track can still be read, but only
through the token the player itself obtains. That answer shapes everything below, and it is the
kind of fact that should be on the record before the platform moves again.

## What Changes

- **The page reader never reads the player's captions** — owned by `fix-lingua-dynamic-rescan`
  (#545), which this change relies on. Measured on Firefox (2026-09-24): today the page reader does
  **not** highlight the native caption line, but only because no content added after the first
  paint is ever re-analysed; fixing that would start reading it, so the exclusion ships with the fix.
- **A YouTube caption mode on desktop** (Chromium, Firefox desktop, Safari macOS). On a watch page
  with English captions, Lingua reads the **whole caption track**, analyses it once, and shows the
  video's **percentage known before it is played**. It starts when the reader asks for it on a
  video, or automatically under a per-device preference that is off by default — never because
  the extension was granted every site, since it changes the player.
- **Lingua's own caption line**, drawn over the player and kept in time with the video, replaces
  the native one while the mode is on: unknown and learning words highlighted exactly as on a page,
  a click on a word or a selected phrase is captured as on a page and pauses the video, and a card
  keeps (on the device) the address of the moment it came from.
- **Exposure counts what was shown, not what was fetched.** Only captions count — there is no audio
  aspect. A caption line counts as read when it was actually on screen during playback; lines
  skipped over, `[Music]`-style annotations and fillers do not count.
- **Translation depends on whether an engine is chosen.** With the sentence engine, each sentence is
  translated ahead of time and a word opened from the line is marked as on a page. Without one,
  and **on YouTube only**, the line can show YouTube's own automatic translation — a whole-line
  translation with no word alignment, labelled as such, hidden until the reader shows it. A remote
  engine, if `add-lingua-remote-translation` ships, plugs in like the device one.
- **A degraded fallback**: when the track cannot be read, the mode reads the caption segments the
  player writes into the page, and the popup says the video is in degraded mode.
- **Not in scope**: mobile (see Impact), Shorts, live streams, embedded players, transcribing audio
  ourselves, other video sites.

## Capabilities

### New Capabilities
- `lingua-youtube-captions`: reading a YouTube video's captions as Lingua text — where the track
  comes from, the caption line, the video percentage, what counts as an exposure, the translation
  shown per setting, the fallback, and restoring the player as the reader left it.

### Modified Capabilities
- `lingua-browser-extension`: **No network requests**, as rewritten by
  `add-lingua-translation-delivery`, gains the caption mode's requests — to YouTube, for the video
  being watched, only while the mode is on.
- `lingua-privacy`: one requirement is **added** — the caption request carries nothing of the
  reader's: it goes to YouTube, for the video being watched, with no page text and no Cymbra
  data, and the Lingua annex says so. Added rather than modified because
  `add-lingua-remote-translation` already modifies **Lingua's privacy disclosures match what it
  collects**; two changes rewriting the same block would overwrite each other at archive.

## Impact

**Products.** Cymbra Lingua only: the extension's three variants (Chromium, Firefox, Safari via
the Apple host app). No backend change, no new RPC, no `.proto` change. Music, Live, the back
office and the site are untouched, except the site's privacy page (one sentence in the Lingua
annex).

**Consumed, not redeclared.** Highlighting, the word popup, the pack's glosses and the analysis
(`lingua-browser-extension`, `lingua-analysis`), exposure counters and promotion
(`lingua-knowledge-model`), phrase capture (`lingua-browser-extension`), the sentence engine and
its marks (`lingua-translation`), and — once it exists — the setting that lets a reader choose it.

**Code.** `apps/lingua-extension`: a YouTube host module beside `src/reading/`, a small script in
the page's own world (the isolated content script cannot read the player's state), and the caption
overlay in the extension's existing shadow-DOM UI host. Per-browser registration in `build.mjs`.

**Translation, as it would actually ship.** The engine reaches readers through
`add-lingua-translation-delivery` (#541) — the per-device "Traduction étendue" setting, on Chromium
and Firefox desktop only. YouTube's own translation is therefore the fallback for every reader who
has not turned it on, and the only sentence translation this mode offers on Safari macOS, which
carries no engine. Should this change ship before the delivery, YouTube would be the first place
Lingua shows a sentence translation at all — a product fact accepted knowingly (decided
2026-09-23), not a side effect.

**Ordering.** Archives **after** `add-lingua-translation-delivery`: it modifies the version of
**No network requests** that change writes, adding the caption mode's requests to YouTube.

**Fragility, stated plainly.** The mechanism depends on YouTube internals that are not a public
API: the caption endpoint, its token and the player's markup. It will break some day; the design
treats breakage as an expected state with a fallback and a signal, not as an exception.

**Stores.** Injecting into youtube.com and reusing the player's token must be explained to
reviewers (`REVIEWERS.md`), AMO in particular, which reads the code.

**Mobile, deliberately excluded.** Chrome Android has no extensions and the YouTube app admits
none; on iPhone Safari the full-screen player is the system's own, where no overlay can be drawn.
Firefox Android (`m.youtube.com`, a different page and client) is a possible later change with its
own spike.
