# Design

## Context

- `apps/music/store/` already holds harness-generated captures: 8 surfaces (`library`,
  `synthesia`, `staff`, `courses`, `measures`, `drums`, `drums_staff`, `drums_stage`) ×
  4 locales (en/fr/it/es) × 5 targets, produced by `tool/capture_store_screenshots.sh`
  driving `integration_test/capture_test.dart` (fixtures, faked MIDI keyboard, Wait Mode
  played to a non-zero score). The site ships fr/en/es only.
- `apps/site` is static Astro on Cloudflare Pages. `Base.astro` emits title, description,
  `hreflang` and `noindex`, but no Open Graph or Twitter tags. There is no `src/assets/`
  yet; the site has no image pipeline in use.
- `site-check.yml` gates its jobs with a `dorny/paths-filter` `site` filter that already
  watches files outside `apps/site` the site depends on (Lingua's `packs.json`,
  `model-manifest.json`).
- The site deploy builds `apps/site` alone; it must not need Flutter or a simulator.

## Goals / Non-Goals

**Goals:**
- The page shows the app with media produced by the harness only, kept in step by a check.
- Media cost nothing on first render and stay within a fixed weight budget.
- A shared link renders with a picture, in the page's language.

**Non-Goals:**
- Background music, a voice-over, or a YouTube-specific cut (the same file can be uploaded
  as is; editing for YouTube is later).
- Share previews for the home, Lingua or legal pages (the layout input makes it a one-line
  change per page; Lingua's comes with `add-site-lingua-showcase`).
- Rewriting the page copy or its feature cards, analytics on the site, any store-listing
  change.

## Decisions

**D1 — Copy the captures into the site, and check the copies byte for byte.** A sync
script (`yarn sync:music-media`) copies the chosen captures from `apps/music/store/` into
`apps/site/src/assets/music/<locale>/`; a vitest test hashes each copy against its source
and fails naming the drifted file. *Alternative:* import straight from `../music/store` in
Astro — rejected: it couples the site build to a sibling app's tree layout and makes
`site-deploy` depend on files it does not otherwise read. The copy keeps the site
self-contained; the check keeps it honest. The `site` paths filter gains
`apps/music/store/**` so a capture refresh runs the check.

**D2 — Gallery = the iPad 13" set, five surfaces.** iPad landscape (2752×2064, 4:3) is
the closest to how the app is used on a piano's music stand and reads well on a laptop and
a phone. Five surfaces carry the page's claims: `staff`, `synthesia`, `measures`, `courses`,
`drums_stage` (library is a list, and the two other drums shots repeat the third). Astro's
`<Picture>` (sharp, already a dependency) emits AVIF + WebP at 3 widths with a PNG
fallback; images after the first are `loading="lazy"`. *Alternative:* the macOS 1440×900
set — smaller, but 16:10 desktop framing undersells a mobile-first app.

**D3 — The video is drawn by the app, frame by frame, on a virtual clock.** A
`showcase_test.dart` scenario reuses the capture fixtures (no backend, faked keyboard) and
the iPad 13" layout, opens a bundled piece in Wait Mode, then stops the app's tickers and
moves time itself: for each of the 30 frames per second it sets a `VirtualClock`
(overriding `clockProvider`), advances the player by one frame, lets the faked keyboard
play the keys the playhead waits on, and renders the frame from the app's own layer tree
(the same `RepaintBoundary.toImage` the screenshots use) to a PNG. Halfway through it
switches from the staff to the falling notes, so one video shows both readings.
`tool/record_showcase_video.sh <locale>` runs it with `flutter drive`, renders the sound
(D6), and has `ffmpeg` encode H.264 + AAC (`+faststart`, CRF raised until the file fits
the budget) and extract the poster. It writes to `build/` and moves the result into
`apps/music/store/video/<locale>/` only on success, so a failed run leaves the previous
video. *Alternatives:* `simctl io recordVideo` on the simulator — rejected before implementation: iPadOS 26 ignores the app's landscape lock, so the simulator shows it in portrait,
its recording has no sound, and its timing would have to be lined up with the sound log
after the fact; a frame drawn at an exact instant has none of these problems. Screen
recording by hand is rejected by the harness's own rule. Local-only, like the screenshot
command: CI never records.

**D4 — One video per locale, MP4 H.264 + AAC only.** The app's text is visible in the player,
so a French video on the Spanish page would contradict the gallery. H.264 MP4 plays in
every target browser; adding WebM would double the weight to maintain for no reach gain.
`<video muted loop playsinline preload="none" poster=…>`; a few lines of script start it
when it enters the viewport and `prefers-reduced-motion` is not set, otherwise it shows
`controls`. Browsers refuse to autoplay with sound, so the file carries its audio but the
element starts `muted`, with a visible "🔊 Listen" button (localised) that unmutes it and
restarts from the beginning, so the visitor hears the piece from its first note. Budget:
4 MB and 30 s per file, and an audio track present, checked by reading the MP4 header in
the test.

**D5 — Share preview = an optional `share` prop on `Base.astro`.** `share: { image, alt }`
emits `og:title`, `og:description`, `og:url` (from `Astro.site` + path), `og:type=website`,
`og:locale`, `og:image` (absolute), `og:image:width/height`, `og:image:alt` and
`twitter:card=summary_large_image`. Pages without it emit nothing new. The 1200×630 images
are generated by a script (sharp: the iPad `staff` capture cropped and scaled, plus the
page's headline rendered from an SVG overlay in the brand font) and committed, so the
build stays pure. A post-build test checks every Music page declares one and the file
exists at 1200×630. This is the input `add-site-lingua-showcase` also needs: the first of
the two to land adds it.

*As landed:* `add-site-lingua-showcase` landed first, so the Music pages reuse its `image`
prop (`{ src, alt }`) instead of adding `share`. Its `Base.astro` gives every page a share
preview (the brand image by default) and re-encodes the image as a 1200×630 JPEG; it emits
no `og:locale`. The Music pages pass `src/assets/music/<locale>/share.png`, and
`test/post-build/head.spec.ts` checks each Music page shares the image of its own language.

**D6 — The sound is rendered offline by the app's own mixer, not recorded.** The run
plays silently: its `AudioService` is a recording double that logs every command the app
sends to the sound engine (notes played by the faked keyboard, notes the app plays itself,
SoundFont swaps), stamped with the same virtual clock as the frames. A small binary in
`apps/music/rust` (`showcase-render`) drives the existing `renderer.rs` mixer — already
driver-independent, since Android's `AudioTrack` feeds it as well as `cpal` — from that log
into a 48 kHz WAV, with the same SoundFont, so the sound is the app's sound to the sample
and each note starts on the frame that shows it. *Alternatives:* recording the macOS
desktop app with system audio (its window renders black under the integration_test
binding); a stock music track (hides the piano, licensing question, and would not match
the notes on screen). The binary's entry point is excluded from the coverage gate; the
log parsing, the WAV encoding and the render itself are unit-tested.

## Risks / Trade-offs

- [Captures in git grow the repo] → five iPad PNGs × 3 locales ≈ 6 MB plus ≤ 12 MB of
  video; accepted, same policy as `apps/music/store/`. The optimized variants are build
  output, not committed.
- [The faked keyboard plays too evenly to look human] → velocities vary per key and the
  keys are held until the next onset; the review step (task 1.5) judges the result.
- [Tickers stopped during the recording freeze the app's own short animations (key flash,
  wait pulse)] → the playhead, the notes and the gauge are driven by the player's state,
  which the scenario advances; the review step checks nothing on screen looks stuck.
- [Store captures refreshed for a new surface the site does not show] → the check only
  covers files the site copies; a new surface is a deliberate gallery edit.
- [Discord/WhatsApp cache an old preview] → the preview file name carries a content hash,
  so a new image is a new URL.

## Migration Plan

Static content only: merge, then dispatch `site-deploy.yml`. Rollback = redeploy the
previous build. Before announcing anywhere, paste a link in a private Discord channel and
a WhatsApp chat to confirm the preview renders.
