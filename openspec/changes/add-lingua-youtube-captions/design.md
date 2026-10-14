## Context

Lingua reads pages: a content script walks text, the WASM analyser classifies each word, the CSS
Custom Highlight API paints the result, and `ExposureTracker<BlockReading>`
(`apps/lingua-extension/src/reading/exposure-tracker.ts`) reports a block once it has been on screen
for a short dwell, with its lemmas and its counts for the daily statistics (`BlockReading` in
`src/reading/scan.ts`: `lemmas`, `read`, `unknown`). `ReadingSession.onExposed` / `flushExposure`
(`src/reading/session.ts`) record the lemmas under `ReadingHost.exposureSource()`
(`reading:<hostname>` on a page, `reading:book` in the reader) and add the counts to the day through
`recordReading` (`src/state/dailystats.ts`). The sentence engine translates the reader's sentence
with their selection marked, off every painting thread; « Traduction en ligne » sends the sentence to
Cymbra's online translation service when the reader presses its button
(`src/translate/online.ts`, archived `add-lingua-online-translation`).

How the reader reaches a page differs by variant: Firefox and Safari ship a **static** content
script on every page (`tool/manifests.mjs`, the `content_scripts` entry), always on unless
« Surlignage activé » is off; Chromium is `activeTab`-first and registers the reader dynamically
once every site is granted (`syncReaderRegistration` in `src/background.ts`). So on Firefox and
Safari the reader already runs on YouTube.

**The caption line is already excluded.** `fix-lingua-dynamic-rescan` (open; its code is on `main`)
added `.ytp-caption-window-container` to `EXCLUDED_SELECTOR` (`src/reading/blocks.ts`), and
`ReadingObservers` ignores any mutation inside it (`blockOf` in `src/reading/observer.ts`). The
flicker and over-counted exposures this change's first draft described are gone; what is left is
that captions are not read at all. The rest of a watch page (description, comments) is read as a
page.

### What the spike established (2026-09-23, Chrome 152, youtube.com, signed out)

| Probe | Result |
|---|---|
| `ytInitialPlayerResponse.captions.playerCaptionsTracklistRenderer.captionTracks` | readable from the page's world: written `en`, automatic `en` (`kind: "asr"`), ~50 others |
| `fetch(baseUrl + "&fmt=json3")` | **HTTP 200, empty body** — `baseUrl` carries `exp=xpe`, the proof-of-origin token is required |
| Replaying the player's own request (`…&potc=1&pot=<token>&c=WEB…`) | full track: 315 events, each `tStartMs`/`dDurationMs` |
| Same token, another track of the **same** video (automatic `en`, `fr`) | works |
| Same token, **another** video | empty — the token is bound to the video |
| Without `c=WEB` | empty. Every other `c*`/`xo*` parameter is optional |
| `&tlang=fr` on the replayed request | YouTube's own machine translation, same events and timings; sentences redistributed across cues |
| `PerformanceObserver({type: "resource"})` | sees the player's request (initiator `xmlhttprequest`) with its full query, token included |
| `performance.getEntriesByType("resource")` | **misses it** — the page fills the resource timing buffer long before captions are requested |
| Automatic track | lowercase, no punctuation, per-word `tOffsetMs`, `[Music]`, fillers ("um", "uh"), `acAsrConf` per word |

The token cannot be produced by us: it comes from YouTube's anti-abuse attestation. The only
honest source is the request the player makes for the video the reader is watching. The spike ran
in the page's world of one browser, signed out, three weeks ago; task group 1 repeats what matters
from a real content script in each variant, on the owner's own profiles (an automated browser lands
on YouTube's consent page and is handed a different token, so it cannot answer).

### What changed since the first draft (amended 2026-10-14)

- The caption exclusion shipped with `fix-lingua-dynamic-rescan`: this change's former requirement
  **The page reader never reads the player's captions** and its task 2.1 are dropped.
- `add-lingua-online-translation` shipped (archived 2026-10-13) without touching **No network
  requests**: this change, which archives last among those rewriting it, carries its bullet.
- Remote translation (`add-lingua-remote-translation`) was withdrawn; « Traduction en ligne » on
  press replaced it.
- The owner decided (2026-10-14): two changes (D12); the mode is offered only on a video with an
  English track, known at page load (D2, D9); caption lines count in « Mots lus » under their own
  origin (D5); iPadOS is left out of v1 with iPhone (D13).
- The engine seam is now `createTranslatorPort(signal)` in `src/translate/create-port.ts`, which
  returns a `TranslatorSource` — `(language) => TranslatorPort | null`; "no engine" is
  `source("en") === null`.

## Goals / Non-Goals

**Goals:**
- A caption mode on desktop YouTube (Chromium, Firefox desktop, Safari on macOS) that gives
  English captions the reading experience of a page: highlight, word and selection cards, deck
  capture, percentage, exposure, « Mots lus ».
- Offer it only where it works: decided at page load from the list of tracks the page holds.
- Read the whole track once, so the percentage is known before the video is played.
- Treat a YouTube change as an expected state: fall back, say so, record it.

**Non-Goals:**
- The sentence translation line under the caption — YouTube's automatic translation, paced engine
  pre-translation, its visibility preference: `add-lingua-youtube-caption-translation` (D12).
- iPhone and iPad (D13), Android. Chrome Android has no extensions; the YouTube app admits none;
  iPhone Safari plays full-screen in the system player where no overlay exists; Safari on iPadOS
  loads the desktop site but is left out of this first version by the owner's decision
  (2026-10-14). Firefox Android (`m.youtube.com`, `c=MWEB`, a different page) is a later change with
  its own probes.
- Audio. Nothing is transcribed and nothing heard is counted.
- Other video sites (Netflix, Vimeo, embedded players on third-party pages), Shorts, live streams.
- Tracks in a studied language other than English. The Spanish programme keeps Spanish out of this
  change (`docs/lingua/spanish-programme.md`, decision D7); the language matrix programme keeps
  French out the same way (`docs/lingua/language-matrix-programme.md`, Scope). The mode reads English
  tracks for a reader who studies English, whatever other languages the reader studies and
  whatever their native language (French or Spanish). An English speaker, who studies Spanish or
  French and not English, is not offered it.
- Sending the video share of « Mots lus » to the server (open question 1).

## Decisions

### D0 — The page reader keeps excluding the caption area; the mode does not go through it

The exclusion owned by `fix-lingua-dynamic-rescan` stays as it is, mode on or off. The caption mode
is a reader of its own: it gets its text from the track (D1) or, degraded, from its own observer of
the caption area (D7) — never from the page reader, whose observer ignores that area by design.
Two readers never compete for the same text.

*Alternative rejected:* lifting the exclusion while the mode is on. The page reader would then
re-analyse a text the mode already owns, every two seconds.

### D1 — The track comes from the player's own request, seen by a `PerformanceObserver`

```
 page world                                 extension (isolated world)
 ──────────                                 ──────────────────────────
 player ── XHR /api/timedtext?…pot=… ─┐
                                      └──▶ PerformanceObserver(resource)
                                             │  keep URL if v == current video
                                             ▼
                                       rebuild: lang/kind of the chosen track,
                                       fmt=json3
                                             │
                                             ▼
                                       replay as the page (see below)
```

The observer only reads what the page's performance timeline already exposes; it neither wraps
`fetch` nor patches `XMLHttpRequest`. That is less code, nothing a reviewer has to trust in the
page's world, and nothing that interferes with the player.

**Where the replay runs, per variant.** The request must leave as the page's own: same origin, the
page's cookies. A Chromium content script's `fetch` does. A **Firefox** content script's `fetch`
runs with the extension's principal — another origin, possibly without the page's cookies — while
`content.fetch` makes the request as the page; Safari is unmeasured. The probes (task group 1)
record, per variant, which of `fetch` and (Firefox) `content.fetch` returns the track. Where neither
does, that variant replays from the page-world script (D2), which then accepts one more command —
"replay this URL" — and only for an `/api/timedtext` URL naming the current video.

*Alternatives.* Patching `XMLHttpRequest.prototype.open` in the main world works and is what most
caption extensions do, but it runs our code inside every request the page makes; it stays the
fallback for a variant whose isolated world does not see the request. Reading `getEntriesByType`
fails (buffer full, see spike). Scraping the transcript panel (`/youtubei/v1/get_transcript`)
needs its own parameters, gives no per-word timing, and is equally internal.

### D2 — A minimal page-world script reads the list of tracks on every watch page

The isolated content script cannot read the player's state. A small script in the page's world
(`world: "MAIN"`) runs on every watch page where the reader runs (D9), and reads only: the video
id, the caption track list (language, `kind`, whether YouTube translates it), whether the video is
a Short, a live stream or a premiere in progress, and the player's captions state. It reads them
from the player's in-memory response (`movie_player.getPlayerResponse()`), reads them again on
`yt-navigate-finish`, and posts them to the content script with `window.postMessage` under a random
per-load channel id. It makes **no request**, reads no caption text, holds no Lingua state, and
accepts no command but "captions on / off / select this track" (D8) — plus "replay this URL" on a
variant that needs it (D1).

It is injected by the background with `chrome.scripting.executeScript({ world: "MAIN", files })`
when the content script reports a watch page: page code may not call `chrome.scripting`
(`test/lint-page-context.spec.ts`; the new directory joins its `IN_THE_PAGE` list). Where
`world: "MAIN"` is missing (Safari is unmeasured; Firefox has it from 128, the manifest's minimum),
the fallback is a `<script src>` pointing at a web-accessible resource — same code, same channel —
at the cost of making the extension's presence visible to the page.

*Alternatives rejected.* Parsing the watch page's HTML (`ytInitialPlayerResponse` in an inline
script): stale after every in-page navigation. Asking YouTube (`/youtubei/v1/player`): a request of
our own, which the owner ruled out. The player's CC button state: it says whether captions exist,
not in which language. Injecting only once the mode starts (the first draft's D9): the mode could
then not know, before being started, whether the video has English captions, and would offer a
button that answers "no English captions" — the owner ruled that out (2026-10-14).

### D3 — Our own caption line, inside the player

The line is an element of ours placed inside the player element (`#movie_player`), positioned over
the player's caption area and following its size in the default, theatre and full-screen layouts
(only descendants of the full-screen element are painted). Its words are **light-DOM text under our
own host element**, which carries `data-cymbra-lingua-skip` so the page reader ignores it, and are
slotted into a closed shadow root that gives the line its look. The native line is hidden with a
stylesheet scoped to `.ytp-caption-window-container`, not removed. Pointer events on the line stop
there, so a click on a word does not play or pause the video through the player.

Because the text is in the document, the existing machinery works on it unchanged: the highlight
registry paints it, `document.getSelection()` sees a selection in it, and `SelectionWatcher`
(`src/reading/selection.ts`, which reads through an injectable `read`) classifies it by its usual
rules (length cap, settle time). Highlighting stays within "In-place highlighting without DOM
mutation" in spirit: we add our own element and change nothing of the page's.

*Alternative rejected:* the text inside the closed shadow root, read with
`Selection.getComposedRanges()`. Firefox only has it from 142, while the manifest accepts 128; the
root's own selection is not standard. Light DOM costs one skip attribute.

**The cards follow the player into full screen.** The word card, the HUD and the drawer are closed
shadow hosts on `documentElement` (`src/reading/wordpopup.ts`, `hud.ts`, `drawer.ts`), invisible
while the player is full screen. While the mode is on and the player is full screen, the word card's
host is moved into the player element, and back when it leaves full screen. The HUD and the drawer
stay where they are.

**Timing** uses `video.currentTime` on `timeupdate` plus `requestAnimationFrame` while playing, and
a binary search over cue start times; a line is repainted only when the cue index changes. The
player's `ad-showing` state hides the line and suspends timing and exposure: during an
advertisement `video.currentTime` is the advertisement's. The line's markup is built with DOM
construction APIs, never a dynamically built `innerHTML` string — AMO reads this code.

### D4 — Cues become sentences once, at load

The track is turned into a list of sentences, each knowing the cues (and, for automatic tracks,
the words) it spans. Written tracks split on sentence punctuation. Automatic tracks have none: the
first version splits on gaps between words longer than a threshold (from `tOffsetMs`), capped at a
maximum length, and never inside a cue. Sentences feed the analysis (percentage) and the card's
source sentence.

*Alternative rejected for now:* a punctuation-restoration model. It would be one more download for
a quality gain we have not measured; the gap heuristic is the baseline to measure against.

### D5 — A line played through is an exposure, and words read

A small `CaptionExposureTracker`, the video counterpart of `ExposureTracker`, records a cue when
playback has covered most of its span — 80 % of `dDurationMs`, accumulated in **video time** from
`timeupdate` deltas while `!video.paused` and not during an advertisement — once per video
session. The page tracker's 1 500 ms wall-clock dwell does not transfer: many cues are shorter
than that, and at rate 2 a 2 s cue is on screen for 1 s. Measuring in video time makes the rule
independent of playback rate. Seeks do not record the cues they jump over. Annotations (`[…]`) and
a filler list are dropped before analysis, so they count neither in the percentage nor in exposures.

The tracker emits the same payload as a page block, a `BlockReading` (`lemmas`, `read`, `unknown`),
computed in English (D11). It goes through the same flush as page reading, so the distinct-day
promotion rule is untouched — a video watched once in a day is one day of exposure, as a page is.
Its exposure source is `video:youtube`, so the per-word counter's "source of the last encounter"
says a video.

**« Mots lus ».** The same event adds the line's `read` and `unknown` to the day, in English. The
day's record (`DailyStat` in `src/state/dailystats.ts`) gains an optional `video` share
(`{ exposures, unknownSeen }`), part of the totals, absent meaning zero; `recordReading` takes the
origin. `pushStats` (`src/sync/sync.ts`) maps the fields it pushes one by one, so the share stays on
the device and the pushed totals include it: no `.proto` or backend change.

*Alternatives rejected.* Counting words read from the whole track at load: it would count what was
fetched, not what was seen. A separate "video" counter outside « Mots lus »: the owner wants video
reading to count as reading (2026-10-14).

### D6 — Translation on the card, as on a page

The caption line's cards are the page's: `SelectionCard` (`src/reading/selection-card.ts`) asks the
engine for a single word only when the pack has no gloss (`wordEngine`), for several words always
(`openExpression`), and offers « Traduction en ligne » on a card for several words (`onlineRequest`).
The caption mode feeds that path with the caption sentence, `language: "en"` and
`translator: () => source("en")`, and warms the engine when a selection begins, as a page does. It
pre-translates nothing: the card asks when it opens, as on a page. A card added to the deck keeps
exactly what a card captured on a page keeps — today no translation (**A machine translation is
never stored**), and the sentence translation once `add-lingua-card-sentence-translation` lands.

The whole-line translation of the first draft's D6 — YouTube's automatic translation when there is
no engine, paced engine pre-translation, the preference that shows it — moves to
`add-lingua-youtube-caption-translation` (D12). The first draft's "remote (if #535 ships)" row is
gone: remote translation was withdrawn, and « Traduction en ligne » on press replaced it.

### D7 — Degraded mode reads the player's segments with its own observer

If no player request is seen within a timeout after captions are on, or the replay returns an
empty body, the mode reads the `.ytp-caption-segment` text the player writes, with a
`MutationObserver` of its own restricted to `.ytp-caption-window-container` — the page's
`ReadingObservers` ignores that area by design (D0) — and shows it in our line as it changes. The
line on screen is the source sentence; the percentage becomes "lines seen so far" and is labelled
so. The failure kind (`no-request`, `empty-track`, `parse`, `no-main-world`) and its time are
appended to a bounded local list (`cymbra-lingua-caption-failures` in `src/state/storage.ts`, like
`REFRESH_FAILURES_KEY`), holding no caption text and no video id; no surface reads it, the
background's devtools do.

### D8 — The player's captions setting is ours to restore

At start the mode notes whether the reader had captions on. If it had to turn them on (D1), it
turns them off at the end; otherwise it only removes the hiding stylesheet. The mode ends on
toggle-off, on navigation to a non-watch page, and on moving to a video it does not start on.

YouTube remembers the captions toggle across videos. Turning captions on through the player's own
control would teach it that preference, so the page-world script selects the track for this video
through the player's API (`setOption("captions", "track", …)`) rather than the sticky toggle, where
the probes show it does not stick.

### D8b — The card's address is the moment

A card captured from a caption keeps as its local address the watch URL with `t=<line start in
seconds>`, so its source opens the video at the line. That is a page address, kept on the device
under **A card's page address stays on the device** as it stands; nothing new is synced and the
requirement is not modified, so this change does not wait for `add-lingua-reader`, which modifies
it.

### D9 — Offered at page load where the reader runs; started only on request

Where the reader runs on a watch page — always on Firefox and Safari (static script), on Chromium
with the every-site grant or after a click on the toolbar icon (`activeTab`) — the content script
asks the background to inject the page-world script (D2) once per document, and decides the offer
from what it posts: the reader studies English, the browser is a desktop one (D13), the page is
`/watch`, the video is not a Short, a live stream or a premiere in progress, and it has an English
track. Only then does the « Sous-titres Lingua » control appear near the player, and the popup offer
the mode. The decision is taken again on every in-page navigation.

The mode starts on a click on that control or in the popup, or on every offered watch page when the
per-device « Démarrer automatiquement sur YouTube » preference is on (off by default). No new
permission is added: on Chromium, a reader without the every-site grant gets the offer after
clicking the toolbar icon, and the automatic start needs the existing grant plus the preference.

*Cost.* One small script per watch page where the reader already runs, reading an object already in
memory; no request. That is the price of offering the mode only where it works.

### D10 — Sequenced after the translation delivery

`add-lingua-translation-delivery` rewrites **No network requests** to list every request the
extension may make (signed in; extended translation). `add-lingua-online-translation` then shipped a
request carrying the selected sentence without updating it, and this change adds the caption
requests. This change's block carries the delivery's text, the online translation bullet with its
exception to "page text", and the caption bullet; it archives after the delivery (`archiveAfter`).
Its bullets are written so a later block adds a bullet and a clause without rewording these:
`add-lingua-youtube-caption-translation` extends the caption bullet, and the open
`add-lingua-usage-counts` adds its own bullet on top.

**Daily reading statistics count what was read** is modified by no other open change (checked
2026-10-14). **The page reader never reads a video player's caption line** and **A card's page
address stays on the device** are consumed, not modified.

### D11 — Captions are read as English

The caption mode runs its own analysis fixed to English — the English port, `source("en")`, English
cards, exposures and counts in English — whatever `ReadingSession.language` holds: the page session
takes the document's detected language (`session.ts`), which `cardLookup` uses for the page's cards.
A watch page in French with English captions keeps both: the page in French, the captions in
English.

### D12 — Two changes

The owner split the work (2026-10-14). This change keeps reading the captions as Lingua text; the
whole-sentence translation line goes to `add-lingua-youtube-caption-translation`, which archives
after it. The reading part is useful alone and carries the risk (the token replay); the
translation line is where the product questions are (Lingua showing YouTube's own translation,
pacing the engine, a preference). Splitting lets the first ship and be measured without the second.

### D13 — Desktop browsers only; iPadOS left out of v1

The background answers the content script with `runtime.getPlatformInfo().os`; `ios` and `android`
are never offered the mode. Safari on iPadOS loads YouTube's desktop site and would otherwise
qualify; the owner left it out of this first version with iPhone (2026-10-14). The probes record
what Safari on an iPad answers; should it not say `ios`, the fallback is a Macintosh platform with
touch points (`navigator.maxTouchPoints > 1`). Desktop Safari on macOS stays in.

## Risks / Trade-offs

- **[YouTube changes the endpoint, the token, the player API or the markup]** → degraded mode (D7), a
  recorded failure kind, and a synthetic fixture per format so a change shows as a failing parser
  test rather than a silent empty line.
- **[The token becomes bound to a request we cannot replay]** → the mode falls back to degraded;
  the design does not depend on anything stronger than "the player's request, replayed once".
- **[The token replay rests on a three-week-old, single-browser spike]** → task group 1 re-measures
  in each variant before any code.
- **[Firefox's replay goes out as the extension]** → `content.fetch`, or the page-world replay (D1).
- **[Store review sees token reuse as abuse — AMO first]** → we replay only the request the player
  made, for the video the reader watches, once; documented in `REVIEWERS.md`. No token is stored or
  shared. If refused, open question 3.
- **[Automatic tracks segment badly]** → the gap heuristic is measured; a bad sentence costs a worse
  card sentence, not a wrong status.
- **[Full-screen and theatre layouts]** → the line and the word card live in the player (D3);
  covered by manual passes on each variant.
- **[A selection in the line on an old Firefox]** → light-DOM text (D3) needs no
  `getComposedRanges`.
- **[YouTube remembers captions the mode turned on]** → per-video track selection (D8); at worst the
  reader finds captions on next time.
- **[Inflated exposures or « Mots lus »]** → only lines played through, once per session, never
  during an advertisement (D5); the video share is kept apart, so it can be checked.
- **[The percentage misleads on a video with few captions]** → the popup shows words analysed, as
  for a page.
- **[One more script on every watch page]** → it reads an object in memory, sends nothing, and runs
  only where the reader already runs (D9).

## Migration Plan

Additive, behind the existing activation flow. The day's statistics record gains an optional
`video` share; a record without it reads as zero, and a build without this change keeps the field
untouched when it adds to the day (`addDailyCounts` spreads the record). Rollback is a release
without the YouTube module: nothing to clean beyond the per-device « Démarrer automatiquement sur
YouTube » preference and the local failure list, both ignored when unknown.

## Open questions for the owner

1. **Should the video share of « Mots lus » reach the server (and the back office)?** It needs an
   optional `DailyStat` field in `backend/lingua/proto/stats.proto`, a column and a back-office line.
   *Recommended:* not in this change — it would add the only backend and `.proto` work to a change
   that has none; propose it once the mode has readers whose share is worth reading.
2. **Offer the mode on a video whose only English track is automatic?** *Recommended:* yes, labelled
   « Sous-titres automatiques » — most English videos have only that track, and the label tells the
   reader why some sentences read oddly.
3. **If AMO refuses the token replay, ship Firefox with the degraded mode only?** *Recommended:* yes
   — degraded mode exists for exactly this, the other variants are unaffected, and the explanation
   in `REVIEWERS.md` can be revisited with AMO afterwards.
4. **Show the « Sous-titres Lingua » control on every offered video, or only in the popup until the
   reader has used the mode once?** *Recommended:* on every offered video — the decision of
   2026-10-14 was that the button appears where it works; it is one quiet control, with no badge,
   counter or animation.

## Unknowns the probes settle (not decisions)

- Whether the isolated world's `PerformanceObserver` sees the player's request in each variant; if
  not in one, that variant observes from the page-world script (D1).
- Where the replay must run per variant — `fetch`, Firefox's `content.fetch`, or the page world (D1).
- Whether Safari's `scripting.executeScript` accepts `world: "MAIN"` in the Apple host app's
  extension (D2).
- Whether selecting a track through the player's API leaves YouTube's sticky captions preference
  alone (D8).
- What `runtime.getPlatformInfo().os` says in Safari on macOS and on an iPad (D13).
- The json3 structure of a written and an automatic track, recorded as keys and types only, for
  synthetic fixtures (public repository: no real video's caption text is ever committed).
- Gap threshold and maximum sentence length for automatic tracks, and the 80 % threshold of D5 —
  starting values (700 ms, 25 words, 80 %) adjusted from the owner's manual passes (task 9.2).

## Appendix — probe snippets (task group 1)

All snippets are pasted into **the extension's background console**, in this order, with one
YouTube watch page open in a tab:

- Chrome: `chrome://extensions` › Developer mode › Cymbra Lingua › "service worker".
- Firefox: `about:debugging#/runtime/this-firefox` › Cymbra Lingua › Inspect › Console.
- Safari: Develop menu › Web Extension Background Content › Cymbra Lingua (Develop is enabled in
  Safari's settings › Advanced).

The extension needs access to youtube.com: on Chrome, turn « Toujours surligner » on for the probes
(P1 finds the tab by its address, which a click on the icon alone does not reveal); on Firefox and
Safari, the access Lingua already has to every site. Snippets running in the `ISOLATED` world run exactly where
the content script runs. Nothing they return is a token or caption text; paste only what they return
into the task notes.

**P1 — the YouTube tab and a runner**

```js
const [tab] = await chrome.tabs.query({ url: "https://www.youtube.com/watch*" });
const run = async (func, world = "ISOLATED") =>
  (await chrome.scripting.executeScript({ target: { tabId: tab.id }, world, func }))[0]?.result;
```

**P2 — the player's request, seen and replayed from the content script's world** (captions off
before; run it, then press `c` in the player within 30 s)

```js
await run(() => new Promise((resolve) => {
  const shape = (v) => Array.isArray(v) ? [v.length ? shape(v[0]) : "empty"]
    : v && typeof v === "object" ? Object.fromEntries(Object.keys(v).map((k) => [k, shape(v[k])])) : typeof v;
  const probe = async (get, url) => {
    try {
      const r = await get(url); const t = await r.text();
      const j = t ? JSON.parse(t) : null;
      return { status: r.status, length: t.length, events: j?.events?.length ?? 0,
               shape: j ? shape(j.events.find((e) => e.segs) ?? {}) : null };
    } catch (err) { return { error: String(err) }; }
  };
  const po = new PerformanceObserver(async (list) => {
    const e = list.getEntries().find((x) => x.name.includes("/api/timedtext"));
    if (!e) return;
    po.disconnect();
    const u = new URL(e.name); u.searchParams.set("fmt", "json3");
    const bare = new URL(u); bare.searchParams.delete("pot");
    resolve({
      seen: true, initiator: e.initiatorType, hasPot: u.searchParams.has("pot"), c: u.searchParams.get("c"),
      fetch: await probe((x) => fetch(x), u.href),
      contentFetch: typeof content !== "undefined" && content.fetch ? await probe((x) => content.fetch(x), u.href) : "n/a",
      withoutPot: await probe((x) => fetch(x), bare.href),
    });
  });
  po.observe({ type: "resource" });
  setTimeout(() => resolve({ seen: false }), 30000);
}));
```

**P3 — the list of tracks, from the page's world** (run it; then click a suggested video and run it
again)

```js
await run(() => {
  const r = document.getElementById("movie_player")?.getPlayerResponse?.();
  const list = r?.captions?.playerCaptionsTracklistRenderer?.captionTracks ?? [];
  return { video: r?.videoDetails?.videoId ?? null, live: r?.videoDetails?.isLive ?? null,
           tracks: list.map((t) => ({ lang: t.languageCode, kind: t.kind ?? "", translatable: !!t.isTranslatable })) };
}, "MAIN");
```

**P4 — captions on for this video only, then off** (captions off before)

```js
await run(() => {
  const p = document.getElementById("movie_player");
  p.loadModule?.("captions");
  p.setOption?.("captions", "track", { languageCode: "en" });
  return p.getOption?.("captions", "track")?.languageCode ?? null;
}, "MAIN");
// later, to turn them off again:
await run(() => { document.getElementById("movie_player")?.unloadModule?.("captions"); return "off"; }, "MAIN");
```

**P5 — the platform**

```js
await chrome.runtime.getPlatformInfo();
```

**P6 — an advertisement** (during an advertisement, then after it)

```js
await run(() => ({ ad: document.getElementById("movie_player")?.classList.contains("ad-showing") ?? null,
                   time: document.querySelector("video")?.currentTime ?? null }));
```
