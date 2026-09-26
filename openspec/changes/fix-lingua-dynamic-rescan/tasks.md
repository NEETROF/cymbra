## 1. Trackers that do not depend on call order

- [x] 1.1 `ReadingObservers.track()` before `start()` records its containers; `start()` observes those still connected; `stop()` forgets them — replace the "tracks nothing before it has been started" test with "observes, once started, what was tracked before"
- [x] 1.2 Same for `ExposureTracker.track()` (lemmas kept, containers observed on `start()`, already-exposed ones skipped) — same test replacement
- [x] 1.3 `ReadingObservers.flush()`: a dirty container not yet watched is observed when queued; tests: a container never tracked is rescanned once it intersects, and stays queued while off screen

## 2. No re-analysis without a change

- [x] 2.1 Pure merge in `blocks.ts`: re-walk dirty roots into the page's block map and report whether a container, a block's text or its text nodes changed; tests (digits-only change → unchanged; same text on a new node → changed; container removed → changed)
- [x] 2.2 `content.ts` `refresh()` uses it and calls `repaint()` only on a change; still tracks the containers

## 3. YouTube's caption line

- [x] 3.1 `.ytp-caption-window-container` in `EXCLUDED_SELECTOR`; the observer's own ignore test uses the same exclusion; tests: `collectBlocks` skips caption text, and a mutation inside the caption area schedules no rescan

## 4. Verification

- [ ] 4.1 Firefox desktop, the probe of design.md re-run on the fixed build: every row highlighted except the caption line; reading exposures recorded without a gesture
- [ ] 4.2 Manual pass Firefox + Chrome on a YouTube watch page with captions on and on one infinite-scroll site: comments and new items highlighted, caption line never, no visible jank while the video plays
- [x] 4.3 Chrome: the same probe page as 4.1
  - Headless Chrome 153, local probe page, level B2 (design, *Implementation notes*): every added row highlighted (the off-screen one once scrolled in), the caption line never; reading exposures recorded without a gesture — none on `main`.

## 5. Gates

- [x] 5.1 `openspec validate fix-lingua-dynamic-rescan --strict`
- [x] 5.2 Extension lint, typecheck, tests green; coverage ≥ 80 %
