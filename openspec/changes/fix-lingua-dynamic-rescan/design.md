## Context

The reading content script paints a page in `activate()` (`src/content.ts`):

```
activate()
  await refresh([document.body])   ← first scan
     └─ observers.track(blocks)    ← ReadingObservers: io is null → return
     └─ repaint()
          └─ exposure.track(...)   ← ExposureTracker:  io is null → return
  hud.mount()
  observers.start()                ← creates the MutationObserver + IntersectionObserver
  exposure.start()                 ← creates the dwell IntersectionObserver
```

Both `track()` methods begin with `if (!this.io) return;`. So after the first paint:

- `ReadingObservers.visible` stays empty. Every mutation marks its block container dirty, and
  `flush()` queues any container that is not in `visible` in `pending` "until it scrolls into
  view". Nothing ever observes it, though, so no container ever leaves the queue.
- `ExposureTracker` observes nothing, so no block is ever recorded as read — until some later
  `repaint()` (a status change, a level change) calls `track()` again, now with an observer. A
  reader who makes no gesture records nothing.

Measured on Firefox 156 (2026-09-24), current `dist-firefox` build, real pack, level A1 declared,
through a temporary counter instrumentation. After the first paint the counters read `track=1`
and `exptrack-noio=1`, with zero IntersectionObserver callbacks. Each probe added a mutation and
the matching flush, and every one ended `pending`:

| Probe | Highlighted |
|---|---|
| Paragraph present at load | yes |
| `<p>` appended to an article already painted | no |
| New wrapper with a `<p>`, in a container that was empty at load | no |
| Sentence appended to a paragraph already highlighted | old words only |
| YouTube comments loaded on scroll (3 388 characters) | no |
| YouTube caption windows, replayed with the player's structure and cadence | no |

The unit tests start the observers before tracking, and both files assert "tracks nothing before
it has been started", so the fault was invisible to them. `content.ts` is excluded from coverage
as an entry point.

There is also a second, independent fault. The comment in `flush()` says *"not yet observed — treat
as visible"*, but the code queues the container instead, and a container that was never tracked
is never observed. This is what keeps a section loaded into an empty container — comments,
infinite-scroll feeds — out of reach, even once the ordering is fixed.

## Goals / Non-Goals

**Goals:**
- Content added after the first paint is analysed, as `lingua-browser-extension` already promises.
- Plain reading is recorded from the first paint.
- Neither depends on the order of calls in `content.ts`.
- Making rescans live does not make busy pages re-analyse themselves continuously.
- YouTube's caption line is never read as page text.

**Non-Goals:**
- The YouTube caption mode (`add-lingua-youtube-captions`).
- Changing the rescan's visibility priority, debounce or dwell.
- Promotion thresholds: `lingua-knowledge-model` is unchanged; this change only lets reading reach it.

## Decisions

### D1 — Tracking before start is remembered

`ReadingObservers.track()` and `ExposureTracker.track()` called before `start()` record what they
were given; `start()` observes it (the containers still connected). `stop()` forgets it. This fixes
the fault where it lives: the call order in `content.ts` stops mattering, and the fix is covered by
unit tests, which `content.ts` is not.

The two tests that assert "tracks nothing before it has been started" encoded the fault; they
become "observes, once started, what was tracked before".

*Alternative rejected:* only reordering `activate()` so `start()` runs first. It fixes today's
caller and leaves the trap for the next one, and the proof would sit in a file tests do not reach.
It would also start the MutationObserver before the first paint, turning the page's own loading
churn into rescans of a page not yet painted.

### D2 — A dirty container that was never tracked is observed

`ReadingObservers` keeps a `WeakSet` of the containers its IntersectionObserver watches. In
`flush()`, a dirty container that is not visible is queued as today, and, if it is not watched
yet, it is observed at that moment. Its first intersection callback then drains it if it is on
screen. Off-screen additions still wait until they scroll in, as the observer's design intends.

*Alternative rejected:* treating a never-observed container as visible, as the stale comment
says. That would rescan far-below additions immediately (an infinite-scroll feed filling below
the fold) and undo the visible-first priority.

### D3 — A rescan that changes no block does not re-analyse

`refresh()` re-walks the dirty roots and then always calls `repaint()`, which analyses **every**
block of the page again. That is harmless while rescans never run; once they do, a page with a
live clock, a counter or a progress label would re-analyse itself every second. YouTube's player
updates `0:15` every second.

The merge of re-walked blocks into the page's block map moves into a pure function in `blocks.ts`.
It reports whether anything changed: a container added or removed, a block's text, or the text
nodes it spans, since a node replaced with the same text still needs its highlight moved.
`refresh()` repaints only when it did. A digits-only clock produces no block at all, so nothing
changes and nothing is analysed.

### D4 — YouTube's caption area is excluded by the player's own class

`.ytp-caption-window-container` joins `EXCLUDED_SELECTOR` in `blocks.ts`. It holds every caption
window, and the player replaces the window at each line, observed as 1 removed and 4 to 7 added
nodes per cue, 1 to 2 s apart. The observer's own ignore test, today `#host` and
`[data-cymbra-lingua-skip]`, uses the same exclusion. A mutation there then does not even
schedule a rescan. The `ytp-` prefix belongs to YouTube's player, so no host check is needed, and
the rule also covers the player on `m.youtube.com`.

### D5 — `content.ts` changes only where D3 needs it

`activate()` keeps its order: with D1, it is correct as written. `refresh()` calls the D3 merge
and skips `repaint()` when nothing changed, still handing the (unchanged) containers to
`track()`.

## Risks / Trade-offs

- **[Dynamic sites now rescan]** → The mechanism that was designed for this, visible-first
  priority and a 250 ms debounce, now actually runs, and D3 keeps churn without new text from
  costing an analysis. Manual passes on YouTube and on one infinite-scroll site check that nothing
  janks (task 4.2).
- **[Promotions begin for readers who never had one]** → This is specified behaviour that never
  ran. Promotions keep the `exposure` provenance and stay reversible in bulk; the distinct-day
  threshold (4) is unchanged.
- **[Measured on Firefox only]** → The code is shared. Manual pass on Chrome (task 4.3).
- **[The class name changes]** → The captions would then be read as page text again (flicker and
  over-counting), not break anything else. `add-lingua-youtube-captions` adds its own breakage
  signal for the caption mode.

## Migration Plan

None: no stored state changes. Rollback is a release without the change.

## Open Questions

None blocking.
