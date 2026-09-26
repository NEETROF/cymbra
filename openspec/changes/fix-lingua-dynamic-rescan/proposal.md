## Why

Two things the extension promises have never happened, on any page, since the first reading
build (#357, 2026-09-11):

- **Content added after the first paint is never analysed.** A paragraph appended to an article, a
  sentence added to a highlighted paragraph, comments a site loads on scroll — none of it is
  highlighted. `lingua-browser-extension` requires the opposite ("Dynamically inserted content").
- **A block the reader sees is never recorded as read**, for a reader who makes no gesture. The
  exposure counters that feed "Exposure-confirmed known" receive nothing from plain reading.

Measured on 2026-09-24 on Firefox 156 with the current build, the real pack and level A1 declared,
through a temporary instrumentation (removed since). The container tracker was called once
(`track=1`), the exposure tracker was called once without an observer (`exptrack-noio=1`), and not
one IntersectionObserver callback fired. Every later mutation ended `pending`, and no reading
exposure was recorded even 8 s later.

The cause is an ordering fault. `activate()` runs the first scan, whose `track()` calls hand the
painted blocks to the two IntersectionObservers, **before** `start()` creates those observers — and
`track()` returns silently when there is no observer yet. The unit tests call `start()` first, and
even assert that tracking before start does nothing, so they could not see it. A second fault
compounds it: a changed container that was never tracked is queued "until it scrolls into view",
but nothing ever observes it, so it never leaves the queue.

It is proposed now because fixing it changes what the reader does on every dynamic site — and on
YouTube it would start reading the player's caption line as page text, which
`add-lingua-youtube-captions` identified. The exclusion has to ship with the fix, not after it.

## What Changes

- **Tracking before start is kept, not dropped.** A container handed to either tracker before it
  is started is observed as soon as it starts. The call order in `content.ts` stops mattering.
- **A changed container that was never tracked is observed**, so it is rescanned when it is on
  screen, instead of waiting forever.
- **A rescan that changes no block's text does not re-analyse the page.** Once rescans run, YouTube's
  player clock ("0:15", every second) and similar churn would otherwise trigger a whole-document
  analysis every second.
- **The page reader never reads a video player's caption line.** On YouTube, the player's caption
  area is excluded from scanning, highlighting and exposure, whatever else the reader does. This
  requirement moves here from `add-lingua-youtube-captions`, which keeps the caption mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: three requirements **added** — content added after the first paint
  is analysed; a block the reader sees counts as read from the first paint; the page reader never
  reads a video player's caption line. Added rather than modified because
  `add-lingua-apple` already modifies **In-place highlighting without DOM mutation**, whose
  "Dynamically inserted content" scenario these make precise.

## Impact

**Products.** Cymbra Lingua's extension only — all three variants share the reading code. No
backend, no `.proto`, no stored state.

**Code.** `apps/lingua-extension/src/reading/observer.ts`, `exposure-tracker.ts`, `blocks.ts`, and
the rescan in `content.ts`; their tests.

**Behaviour readers will notice.** Dynamic sites start being highlighted as they change, and plain
reading starts counting toward exposure-confirmed words. The first is the promise; the second
means promotions may begin for readers who never had one — both as specified, neither seen before.

**Measured on Firefox only.** The code is shared by every variant; Chromium and Safari were not run
and are expected to behave the same.

**Relation to `add-lingua-youtube-captions`.** That change's **The page reader never reads the
player's captions** is removed there and owned here; its task 2 points here.
