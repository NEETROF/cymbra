## Context

`add-lingua-translation-delivery` (#546) put « Traduction étendue » in readers' hands on Chromium
and Firefox desktop. Firefox for Android shares the Firefox package, so it carries the engine, the
download and the setting; its D8 hides the setting at run time (`offered` is false when
`runtime.getPlatformInfo().os === "android"`), on the 2026-09-23 measurement.

The 2026-09-26 measurement (SM-P610, 4 GB, Firefox release; `main`'s build, the check lifted,
timestamped probes sent to a Mac over `adb reverse`, `dumpsys meminfo` every 15 s; nothing of it
committed):

| What | Measured |
|---|---|
| Download, 25.8 MB | 3.8 s on Wi-Fi; 53 s at 500 KB/s (throttled local host); ready at the first attempt both times |
| Event page during a download | held by the settings page's own heartbeat — on Android the page stays alive when the reader goes back to the article |
| Event page while reading | 0 restarts: the reading page's 5 s heartbeat holds it |
| Cold start (engine spawn → first answer) | 4.1–4.7 s: first selection of a visit, and after every return from another app (~1 min away) |
| Why the return is cold | Android freezes the tab → heartbeat stops → Firefox tears the event page down (restart logged on return) → engine gone |
| Warm translation | 0.4–1 s; 2.3 s for a 637-character sentence |
| Memory | extension process 195 → 375 MB; Android reclaimed ~150 MB from other Firefox processes; nothing killed |
| Handles | the same 84-character sentence translated 5 times in 6 s (0.4 s each) |

Two things the card already absorbs: it shows the pack's answer at once and the translation
replaces it when it lands (`TRANSLATION_WAIT_MS` = 15 s, above the measured cold start), so a cold
start is a late upgrade, not a wait. What remains worth removing is the cold start the reader sees
most often, and work nobody reads.

## Goals / Non-Goals

**Goals:**
- Offer the setting on Firefox for Android with the desktop behaviour.
- Take the cold start out of the reader's way where a signal comes before the request: the
  selection gesture, and the return to a page that was translating.
- Stop translating again what the page already has.

**Non-Goals:**
- Making the cold start itself faster (wasm compile, model read). Firefox does not keep compiled
  WebAssembly modules in IndexedDB; that is a separate investigation.
- Keeping the engine alive while the tab is frozen: Android decides that, and nothing an extension
  sends from a frozen tab arrives.
- Remote translation (`add-lingua-remote-translation`), Safari (no engine), Chrome for Android (runs
  no extension).
- Loading the engine when a page opens: a page read without selecting anything must cost nothing.

## Decisions

### D1 — Offered wherever the engine is packaged

`offered` becomes true in every variant that carries the engine (Chromium, Firefox desktop and
Android); the platform check goes. The `offered` field stays in `ModelStatus`: it is what an
unreachable background answers (`NOT_OFFERED`), and what a variant without an engine means.

*Alternative:* an allow-list of measured devices (RAM, model). Rejected: an extension cannot read
the device's memory on Firefox (`navigator.deviceMemory` is Chromium-only), and the cost is stated
in the setting before it is ticked.

### D2 — A selection that begins loads the engine

The reader's gesture is the earliest honest signal that a translation is coming. `SelectionWatcher`
gains an `onBegin` callback, called on the first usable `selectionchange` of a gesture — before the
settle timer, before the handles move. `ReadingSession` answers it with a `warm` message when, and
only when, the page's translator source has a translator (setting on, model ready — the
condition of `create-port.ts`). Once per gesture; a collapsed selection re-arms it.

The background answers `warm` only when `model.ready()`, and calls `EngineAccess.warm()`: on Firefox
`EngineChannel.warm()` starts the worker and loads the model without translating; on Chromium
`OffscreenEngine.warm()` makes sure the document exists and forwards a `warm` op to it. Warming
arms the idle release exactly as a translation does, so an engine warmed for a selection that ends
up needing no translation (a single word the pack glosses) is released after the idle period.

On a tablet the long-press, the handles and the settle take one to several seconds, which is the
share of the 4.1–4.7 s the reader stops seeing. On desktop the cold start is ~0.2 s and warming is
simply harmless.

*Alternatives:* warm on `touchstart`/`mousedown` — every tap and click would load 180 MB, most of
them never selecting anything. Warm on page load — rejected in Non-Goals.

### D3 — A page that was translating restores the engine when it comes back

The keep-warm wrapper (`keepWarm` in `keepalive.ts`) sees every translation its page asks, so it
can remember when the last one was asked. It also listens to `visibilitychange`: when the page becomes visible and its last
translation was asked less than the idle period ago, it sends `warm`. The engine then loads while
the reader finds their place, and the first selection after a return is warm or nearly.

The window is the idle period on purpose: it restores the engine only where it would still be
loaded had Android not frozen the tab. The idle period moves from `translate/host/channel.ts` to a
module a surface may import (`translate/port.ts`), since `translate/host/` is out of a surface's
reach (`lint-translator-placement`).

*Alternative:* warm on every return to any page with the setting on. Rejected: a reader who opens
a tab and reads nothing would load the engine each time.

### D4 — The page keeps the answers it received

The port a surface gets is wrapped in a small answer memory, per page: a successful answer is kept
under its request (sentence and selection span), the 32 most recent, and the same request is
answered from it without a message. Only translations are kept; an `unavailable` answer is asked
again next time. The memory is the page's: it goes with the page, is never stored, and never
crosses tabs.

It covers what the handles do when they come back to a span already answered, and a card reopened
on the same selection. It does not merge different spans of the same sentence: the engine marks
the selection inside its answer, so a different span is a different answer.

*Alternative:* drop a queued request superseded by a newer one. Not needed: the measured repeats
were sequential, each answered before the next was asked.

### D5 — What does not change

The heartbeat (5 s, from the page's first translation), the ten-minute idle release, the download
and its states, the model, its host and its manifest, the settings copy (it already states 25,8 Mo
and about 200 Mo of memory), the manifest and its permissions. On Android the settings page stays
alive when the reader goes back to the article, and its own heartbeat carries the download
(measured); if Android closes it, the download stops as `interrupted` and the setting offers to
resume, as on desktop.

### D6 — Copy

`STORE-LISTING.md` (AMO description fr + en: the listing is shared by desktop and Android) names
Firefox for Android among the places extended translation is offered. The Lingua privacy annex
names no platform and is unchanged. `TRANSLATION.md`, `README.md` and `REVIEWERS.md` drop "hidden on
Firefox for Android".

## Risks / Trade-offs

- [180 MB on a 4 GB tablet] → Measured without a kill; the setting states the memory before it is
  ticked; the idle release and Android's own tab freezing give it back.
- [Warming for a selection that needs no translation] → Released by the idle period like any
  engine; the reader chose the setting knowing its cost.
- [A device with less memory than the SM-P610] → Not measured. If Android kills the extension
  process, the next selection is cold (the card still upgrades); a device where it fails is what
  reopens `add-lingua-remote-translation` (its gate 0.1).
- [Mobile data] → 25.8 MB once, stated before ticking. Firefox exposes no connection type to an
  extension, so no Wi-Fi-only option.
- [Warm on return costs a load the reader may not use] → Bounded to pages that translated within
  the idle period.

## Migration Plan

No data to migrate: the setting's stored value and the model database are the desktop ones. A
Firefox for Android reader who updates sees the setting, off. Rollback: restore the platform check;
a model already downloaded on Android stays until the setting is turned off.

## Open Questions

- The measured gain of D2 and D3 on the SM-P610 is recorded by this change's device pass (task 5),
  not assumed.
