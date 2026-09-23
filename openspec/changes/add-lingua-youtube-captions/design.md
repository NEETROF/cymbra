## Context

Lingua reads pages: a content script walks text, the WASM analyser classifies each word, the CSS
Custom Highlight API paints the result, and `ExposureTracker` records a block as read once it has
been on screen for a short dwell. The sentence engine (`add-lingua-translation-engine`, #532)
translates the reader's sentence with their selection marked, off every painting thread.

How the reader reaches a page differs by variant (`build.mjs`): Firefox and Safari ship a
**static** content script on every page, always on unless "Surlignage activé" is off; Chromium is
`activeTab`-first and registers the reader dynamically once every site is granted. So on Firefox
and Safari the reader is **already running on YouTube today**.

None of this fits a YouTube caption. The player writes `.ytp-caption-segment` spans into
`.ytp-caption-window-container` and replaces them every few seconds, word by word for automatic
captions. The `MutationObserver` in `src/reading/observer.ts` would see them, but the highlight
would chase text that is already gone, each rewrite would count as a new exposure, and a caption
cue is a fragment, not a sentence.

### What the spike established (2026-09-23, Chrome 152, youtube.com, signed out)

| Probe | Result |
|---|---|
| `ytInitialPlayerResponse.captions.playerCaptionsTracklistRenderer.captionTracks` | readable from the page's world: written `en`, automatic `en` (`kind: "asr"`), ~50 others |
| `fetch(baseUrl + "&fmt=json3")` | **HTTP 200, empty body** — `baseUrl` carries `exp=xpe`, the proof-of-origin token is required |
| Replaying the player's own request (`…&potc=1&pot=<token>&c=WEB…`) | full track: 315 events, each `tStartMs`/`dDurationMs` |
| Same token, another track of the **same** video (automatic `en`, `fr`) | works |
| Same token, **another** video | empty — the token is bound to the video |
| Without `c=WEB` | empty. Every other `c*`/`xo*` parameter is optional |
| `&tlang=fr` on the replayed request | YouTube's own machine translation, same events and timings; sentences redistributed across cues (the first cue read "politiques," — the tail of the previous sentence) |
| `PerformanceObserver({type: "resource"})` | sees the player's request (initiator `xmlhttprequest`) with its full query, token included |
| `performance.getEntriesByType("resource")` | **misses it** — the page fills the resource timing buffer long before captions are requested |
| Automatic track | lowercase, no punctuation, per-word `tOffsetMs`, `[Music]`, fillers ("um", "uh"), `acAsrConf` per word |

The token cannot be produced by us: it comes from YouTube's anti-abuse attestation. The only
honest source is the request the player makes for the video the reader is watching.

## Goals / Non-Goals

**Goals:**
- A caption mode on desktop YouTube (Chromium, Firefox desktop, Safari macOS) that gives captions
  the full reading experience: highlight, popup, cards, percentage, exposure, translation.
- Read the whole track once, so the percentage and the translations are ready before they are
  needed.
- Treat a YouTube change as an expected state: fall back, say so, record it.

**Non-Goals:**
- Mobile. Chrome Android has no extensions; the YouTube app admits none; iPhone Safari plays
  full-screen in the system player where no overlay exists. Firefox Android (`m.youtube.com`,
  `c=MWEB`, a different page) is a later change with its own spike.
- Audio. Nothing is transcribed and nothing heard is counted.
- Other video sites (Netflix, Vimeo, embedded players on third-party pages).
- Tracks in other studied languages than English — the mode follows the studied language, and
  today that is English.

## Decisions

### D0 — The page reader already excludes the caption area

`fix-lingua-dynamic-rescan` (#545) excludes `.ytp-caption-window-container` from page reading,
mode on or off, because the rescan it repairs would otherwise read each new caption window. The
caption mode builds on that exclusion: it owns the captions, and the page reader never competes
for the same text. The rest of the watch page (description, comments) is still read as a page.

### D1 — The track comes from the player's own request, seen by a `PerformanceObserver`

```
 page world                                 extension (isolated world)
 ──────────                                 ──────────────────────────
 player ── XHR /api/timedtext?…pot=… ─┐
                                      └──▶ PerformanceObserver(resource)
                                             │  keep URL if v == current video
                                             ▼
                                       rebuild: lang/kind of the chosen track,
                                       fmt=json3 (+ tlang when needed)
                                             │
                                             ▼
                                       fetch (same origin, page cookies)
```

The observer only reads what the page's performance timeline already exposes; it neither wraps
`fetch` nor patches `XMLHttpRequest`. That is less code, nothing a reviewer has to trust in the
page's world, and nothing that interferes with the player.

*Alternatives.* Patching `XMLHttpRequest.prototype.open` in the main world works and is what most
caption extensions do, but it runs our code inside every request the page makes. Reading
`getEntriesByType` fails (buffer full, see spike). Scraping the transcript panel
(`/youtubei/v1/get_transcript`) needs its own parameters, gives no per-word timing, and is equally
internal.

*To confirm first (task 1.1).* The spike ran in the page's world. Content scripts share the
document's performance timeline in Chromium and Gecko; this must be proven in each variant,
Safari included, before anything is built on it.

### D2 — A minimal script in the page's world reads the player's state

The isolated content script cannot read `ytInitialPlayerResponse` or the player's
`getPlayerResponse()`. A small script registered with `world: "MAIN"` reads only: the video id,
the caption track list (language, `kind`, name), and the caption toggle state, and posts them to
the content script with `window.postMessage` under a random per-load channel id. It holds no
Lingua state and receives no command other than "turn captions on/off". Parsing the watch page's
HTML is rejected: it is stale after every in-page navigation (`yt-navigate-finish`).

Availability of `world: "MAIN"` differs by variant (Chromium `scripting.registerContentScripts`;
Firefox ≥ 128; Safari to verify). Where it is missing, the fallback is a `<script src>` pointing at
a web-accessible resource — same code, same channel.

### D3 — Our own caption line, in the existing UI host

The line's markup is built with DOM construction APIs, never a dynamically built `innerHTML`
string, as the stats view now is (`lingua-browser-extension`) — AMO reads this code.

The line is rendered in the extension's closed shadow-DOM host, positioned over the player's
caption area and following the player's size (default, theatre, full-screen: the host is moved
inside the player element when it goes full-screen, since only descendants of the full-screen
element are painted). The native line is hidden with a stylesheet scoped to
`.ytp-caption-window-container`, not removed. Timing uses `video.currentTime` on `timeupdate`
plus `requestAnimationFrame` while playing, and a binary search over cue start times; a line is
repainted only when the cue index changes. The player's `ad-showing` state hides the line and
suspends timing and exposure: during an advertisement `video.currentTime` is the advertisement's.

Selection inside the closed shadow root is invisible to `document.getSelection()`, which is what
`selection.ts` reads on a page. The line therefore owns its capture: a click on a highlighted word
opens the popup, and a selection inside the line (read from the shadow root — `getComposedRanges`
where available, the root's own selection otherwise) is classified by the same rules as
`SelectionWatcher` (length cap, settle time) and fed to the same capture path, with the caption
sentence as source. Both pause the video first; the popup remembers whether it did.

Because the line is our own DOM inside our own host, highlighting stays within "In-place
highlighting without DOM mutation": we mutate nothing of the page.

### D4 — Cues become sentences once, at load

The track is turned into a list of sentences, each knowing the cues (and, for automatic tracks,
the words) it spans. Written tracks split on sentence punctuation. Automatic tracks have none: the
first version splits on gaps between words longer than a threshold (from `tOffsetMs`), capped at a
maximum length, and never inside a cue. Sentences feed three things: the analysis (percentage),
the engine (translation ahead), and the popup's source sentence.

*Alternative rejected for now:* a punctuation-restoration model. It would be one more download for
a quality gain we have not measured; the gap heuristic is the baseline to measure against.

### D5 — Exposure is a line played through

A small `CaptionExposureTracker`, the video counterpart of `ExposureTracker`, records a cue when
playback has covered most of its span — 80 % of `dDurationMs`, accumulated in **video time** from
`timeupdate` deltas while `!video.paused` and not during an advertisement — once per video
session. The page tracker's 1 500 ms wall-clock dwell does not transfer: many cues are shorter
than that, and at rate 2 a 2 s cue is on screen for 1 s. Measuring in video time makes the rule
independent of playback rate. Seeks do not record the cues they jump over. Annotations (`[…]`) and a filler list are
dropped before analysis, so they count neither in the percentage nor in exposures. The recorded
lemmas go through the same batch path to the engine as page exposures, with the same distinct-day
promotion rule — a video watched once in a day is one day of exposure, as a page is.

### D6 — Translation: the engine when chosen, YouTube's otherwise

| Engine | Caption line translation | Word popup |
|---|---|---|
| none (setting off, model not ready, or Safari) | YouTube's automatic translation of the line (`tlang=<gloss language>`), fetched once per video when the track is translatable, labelled "Traduction automatique YouTube", shown whole-line | pack gloss, as today |
| on the device | engine translation of the sentence, computed ahead in reading order | engine sentence + mark, as on a page |
| remote (if #535 ships) | same as device, remote host | same as device |

"Engine chosen" is what `add-lingua-translation-delivery` makes `createTranslatorPort()` answer:
a port when the device's `translationHost` is `"local"` and its model is ready, `null` otherwise —
always `null` on Safari, which carries no engine. This change reads that seam and adds no setting of
its own for the engine. A model still downloading counts as no engine: the line falls back to
YouTube's translation until the model is ready.

YouTube's translation is kept out of the engine path on purpose: it has no alignment, its cue
boundaries are not its sentence boundaries, and its quality is not ours. It is therefore never
used to mark a word, never offered outside YouTube, and never mixed with an engine answer. It adds
no third party — YouTube already serves the video — and it sends no reader text, so it respects
"a reader who chose nothing: no sentence leaves the device". The translation line is hidden by
default (the point is to read the English) and its visibility is a per-device preference.

Pre-translation with the device engine is paced: the next few sentences ahead of the playhead,
never the whole track at once, so the engine stays available for a click (`A translation blocks
nothing else`).

### D7 — Degraded mode reads the player's segments

If no player request is seen within a timeout after captions are on, or the replay returns an
empty body, the mode reads `.ytp-caption-segment` text through the existing observer, restricted
to the caption container, and shows it in our line as it changes. The percentage becomes "lines
seen so far" and is labelled so. The failure kind (`no-request`, `empty-track`, `parse`,
`no-main-world`) is recorded in the local diagnostics log, holding no caption text.

### D8 — The player's captions setting is ours to restore

At start the mode notes whether the reader had captions on. If it had to turn them on (D1), it
turns them off at the end; otherwise it only removes the hiding stylesheet. The mode ends on
toggle-off, on navigation to a non-watch page, and on moving to a video it does not start on.

YouTube remembers the captions toggle across videos. Turning captions on through the player's own
control would teach it that preference, so the main-world script uses the player API's per-video
track selection rather than the sticky toggle where the API allows it (to verify, task 1.2).

### D8b — The card's address is the moment

A card captured from a caption keeps as its local address the watch URL with `t=<line start in
seconds>`, so its source opens the video at the line. The address stays on the device under
**A card's page address stays on the device**; nothing new is synced.

### D9 — Activation per variant, never by the grant alone

The reader is already present on YouTube wherever it runs (static on Firefox and Safari; on
Chromium through `activeTab` or the every-site registration). The caption mode is a second step
on top of it: offered by a control near the player and in the popup, started on a click, or on
every watch page when the per-device "start automatically on YouTube" preference is on (off by
default). No new permission is added: on Chromium, starting on demand rides `activeTab`, and the
automatic start needs the existing every-site grant plus the preference. The main-world script
(D2) is injected only when the mode starts, not on every YouTube load.

## Risks / Trade-offs

- **[YouTube changes the endpoint, the token or the markup]** → degraded mode (D7), a recorded
  failure kind, and a spike-derived test fixture per format so a change shows as a failing parser
  test rather than a silent empty line.
- **[The token becomes bound to a request we cannot replay]** → the mode falls back to degraded;
  the design does not depend on anything stronger than "the player's request, replayed once".
- **[Store review sees token reuse as abuse]** → we replay only the request the player made, for
  the video the reader watches, once; documented in `REVIEWERS.md`. No token is stored or shared.
- **[Automatic tracks segment badly]** → the gap heuristic is measured on a small fixed set of
  videos; a bad sentence costs a worse translation, not a wrong status.
- **[Full-screen and theatre layouts]** → the host follows the player element; covered by manual
  passes on each variant.
- **[The percentage misleads on a video with few captions]** → the popup shows words analysed, as
  for a page.
- **[`tlang` quality on automatic tracks]** → labelled as YouTube's, hidden by default.
- **[YouTube becomes the first place Lingua shows a sentence translation]** → with no engine in any
  shipped build, that is what this change does. Accepted knowingly; the label says whose it is.
- **[Captions left on after a tab closes mid-mode]** → restoration cannot run on close; D8 avoids
  the sticky toggle where possible, and at worst the reader finds captions on next time.

### D10 — Sequenced after the translation delivery

`add-lingua-translation-delivery` rewrites **No network requests** to list every request the
extension may make (signed in; extended translation). The caption mode adds requests to YouTube, so
this change modifies that rewritten requirement and archives after it. Its delta carries the
delivery's full text plus one bullet; archiving it first would drop the delivery's bullets.

## Migration Plan

Additive, behind the existing activation flow; no stored state changes shape. Rollback is a
release without the YouTube registration: nothing persisted by this mode needs cleaning beyond the
per-device "show translation line" preference, which is ignored when absent.

## Open Questions

- Does the isolated world's `PerformanceObserver` see the player's request in all three variants
  (task 1.1)? If not in one, that variant patches `XMLHttpRequest` in the main-world script.
- Does Safari's content script `world: "MAIN"` work in the Apple host app's extension?
- Gap threshold and maximum sentence length for automatic tracks — to be set from measurement.
- Should a video count toward the daily reading stats as a separate source, so the reader can see
  "read on YouTube" apart from pages? (The exposure counter already keeps "the source of the last
  encounter".)
- The 80 % threshold of D5 is a starting value to check against real viewing.
- The stale `lingua-browser-extension` requirement **No network requests** ("In v1 … no network
  request at all") already contradicts sync; it should be rewritten by a change of its own rather
  than patched here.
