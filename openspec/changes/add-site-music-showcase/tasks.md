# Tasks

## 1. Showcase video in the capture harness (apps/music)

- [x] 1.1 Add `integration_test/showcase_test.dart`, reusing the capture fixtures on the iPad 13" layout: open a bundled piece in Wait Mode, stop the tickers, then draw ~26 s at 30 fps on a `VirtualClock` while the faked keyboard plays the keys the playhead waits on (staff, then falling notes); it asserts the connected instrument, Wait Mode and a score above 50 %, and fails otherwise — verify by running it under `flutter drive` on the iPad 13" simulator in `fr`
- [x] 1.2 Give the run a recording `AudioService` that logs every sound-engine command (note on/off, drums, SoundFont swaps) on the same virtual clock, reported to the driver — verify the log of a run lists the piece's notes in order
- [x] 1.3 Add the `showcase-render` binary in `apps/music/rust` driving the existing mixer from that log into a 48 kHz WAV with the same SoundFont (D6); keep the log parsing in a pure, unit-tested module — verify `cargo test` covers the parser and a rendered WAV has the expected length and is not silent
- [x] 1.4 Add `tool/record_showcase_video.sh <locale>` (`flutter drive` the scenario, render the sound, `ffmpeg` H.264 + AAC `+faststart` from the frames, poster frame, temp file moved into `store/video/<locale>/` only on success) and `melos run showcase-video -- <locale>`; document it, its prerequisites (booted simulator, ffmpeg) and the budget in `apps/music/store/README.md` — verify a run writes `showcase.mp4` ≤ 30 s and ≤ 4 MB with an audio track plus `poster.jpg`, and a run with the scenario forced to fail leaves the previous files
- [ ] 1.5 Record and commit the fr, en and es videos, and watch and listen to each one end to end — verify smooth playback, the interface language matches the folder, and each note's sound lands on the frame that lights it

## 2. Site media and their check (apps/site)

- [x] 2.1 Add `scripts/sync-music-media.ts` (`yarn sync:music-media`) copying the five iPad 13" surfaces (D2) and the three videos + posters into `src/assets/music/<locale>/` — verify running it twice produces no diff
- [x] 2.2 Add `test/music-media.spec.ts`: each copy hashes equal to its `apps/music/store/` source (failure names the file), each video ≤ 4 MB, ≤ 30 s and with an audio track (MP4 header), each image has fr/en/es alt text in `src/lib/music-media.ts` — verify it passes, then fails when one copy is altered
- [x] 2.3 Add `apps/music/store/**` to the `site` filter of `.github/workflows/site-check.yml` with a comment naming the check — verify `python3 scripts/check_ci_units.py --list` still passes

## 3. Share preview (apps/site)

- [x] 3.1 Add the optional `share` prop to `Base.astro` emitting the Open Graph + Twitter tags of D5 with absolute URLs; pages without it unchanged — verify with an astro component test for both cases
- [x] 3.2 Add `scripts/music-share-images.ts` (`yarn gen:music-share`) generating `src/assets/music/<locale>/share.png` at 1200×630 (the build gives it a content hash) (staff capture + headline in the page's language) and commit the three images — verify dimensions and that re-running yields the same hash
- [x] 3.3 Add a post-build test (`test/post-build/share-preview.spec.ts`): every Music page declares `og:image` on `https://cymbra.app/`, the file exists in `dist/` at 1200×630, and a Music page without it fails naming the page — verify after `yarn build`

## 4. The Music pages (apps/site)

- [x] 4.1 Add a `MusicShowcase.astro` component: video (muted, loop, playsinline, `preload="none"`, poster; plays in view unless `prefers-reduced-motion`, else controls; a localised "🔊 Listen" button that unmutes and restarts it) and the `<Picture>` gallery (AVIF/WebP/PNG, lazy after the first) with captions from `src/lib/music-media.ts` — verify with an astro component test (attributes, sound button label per locale, alt texts, lazy loading)
- [x] 4.2 Use it and the `share` prop in `music.astro`, `en/music.astro` and `es/music.astro`, each with its locale's media — verify `yarn check`, `yarn test`, `yarn build` pass and the built pages reference only their own locale's files
- [x] 4.3 Describe the media pipeline (sync, check, share images) in `apps/site/README.md` — verify the documented commands run as written

## 5. Integration

- [x] 5.1 Run the site locally (`yarn build` + `yarn preview`, driven in Chrome) on a laptop width and a phone width, with and without reduced motion — verify the gallery and video behave as specified, the sound only starts from the button, and the first render downloads no video
- [ ] 5.2 After deploy, paste the fr, en and es links in a private Discord channel and a WhatsApp chat — verify each shows its preview image in its language
