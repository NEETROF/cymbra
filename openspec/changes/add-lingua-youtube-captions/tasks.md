## 1. Prove the two unknowns first

- [ ] 1.1 In each variant (Chromium, Firefox desktop, Safari macOS), from a real content script, confirm a `PerformanceObserver({type: "resource"})` sees the player's `/api/timedtext` request with its `pot`; record the result in design.md (D1) and switch that variant to a main-world `XMLHttpRequest` hook if it does not
- [ ] 1.2 In each variant, confirm a `world: "MAIN"` script can read `getPlayerResponse()` and select a caption track per video without flipping YouTube's sticky captions preference (D2, D8), or that the web-accessible `<script src>` fallback does
- [ ] 1.3 Save anonymised json3 fixtures from the spike: a written track, an automatic track, a `tlang` track, and an empty body

## 2. Stop reading the player's captions as page text (ships on its own)

- [ ] 2.1 Add the player's caption containers to the page reader's skip list on YouTube, mode on or off (D0); tests that no highlight or exposure comes from `.ytp-caption-segment` while comments are still read

## 3. Track and sentences (pure, host-testable)

- [ ] 3.1 json3 parser → cues (start, duration, text, per-word offsets when present); tests on the fixtures, empty body is a typed failure
- [ ] 3.2 Track choice: written English over automatic English, none when neither; tests
- [ ] 3.3 Request rebuild from an observed player URL: keep `v`, signature params, `pot`, `potc`, `c`; set `lang`/`kind`/`fmt=json3`/optional `tlang`; refuse a URL whose `v` is not the current video; tests
- [ ] 3.4 Annotation and filler filter (`[…]`, filler list); tests
- [ ] 3.5 Cues → sentences: punctuation split for written tracks, gap-threshold split with a length cap for automatic tracks, never inside a cue; tests on both fixtures
- [ ] 3.6 Cue lookup by time (binary search) with seek and rate change; tests

## 4. Page-world bridge and session

- [ ] 4.1 Main-world script: video id, track list, captions state, captions on/off, over a per-load random `postMessage` channel; nothing else
- [ ] 4.2 Video session: offered on `/watch` only (not Shorts, live, premieres, embeds); end on `yt-navigate-finish` to another video or away; token held in memory and discarded with the session; tests with a mocked bridge
- [ ] 4.3 Observer of the player's caption request; turn captions on when none is seen; timeout → degraded; tests
- [ ] 4.4 Restore the reader's captions state and the native line when the mode ends; tests

## 5. Caption line and popup

- [ ] 5.1 Caption line in the existing shadow-DOM host, over the player's caption area, following default/theatre/full-screen; native line hidden by a scoped stylesheet
- [ ] 5.2 Highlight unknown/learning words through the existing analysis path; repaint on status change broadcast; no repaint when the cue has not changed
- [ ] 5.3 Word click → existing word popup with the caption sentence as source; pause if playing, resume on close only if we paused; tests
- [ ] 5.4 Phrase selection inside the shadow root (`getComposedRanges` or the root's selection), classified by the `SelectionWatcher` rules and fed to the same capture path; pauses the video; tests
- [ ] 5.5 Card from a caption holds the caption sentence and dictionary data only, and a local address with `t=<line start>`; tests
- [ ] 5.6 Hide the line and suspend timing during advertisements (`ad-showing`); test
- [ ] 5.7 Line markup built with DOM construction APIs, no dynamic `innerHTML`

## 6. Percentage and exposure

- [ ] 6.1 Whole-track analysis at load → badge shows the video's percentage while the mode is on; popup shows its breakdown, which track (written/automatic), and the rest of the page apart; tests
- [ ] 6.2 `CaptionExposureTracker`: a cue counts once playback has covered 80 % of its span in video time while playing (not during ads), once per session; seeks, paused display and rate changes tested
- [ ] 6.3 Feed recorded lemmas through the same batch path as page exposures; test the distinct-day rule is unchanged

## 7. Translation

- [ ] 7.1 No engine (`createTranslatorPort()` is `null`): fetch the `tlang=<gloss language>` track once per video when the track is translatable, map by cue, show whole-line labelled "Traduction automatique YouTube", never mark a word, never on another site, not in degraded mode; tests
- [ ] 7.2 Engine chosen (device, and remote when it exists): pre-translate the next few sentences ahead of the playhead, paced so a click is never queued behind them; popup shows the sentence with the word marked by the page rule; tests with a mocked `TranslatorPort`
- [ ] 7.3 Translation line hidden by default, visibility kept per device; tests
- [ ] 7.4 No translation of either source stored or sent; test on card creation

## 8. Degraded mode

- [ ] 8.1 Read `.ytp-caption-segment` through the existing observer restricted to the caption container; show in our line; percentage labelled "lines seen so far"; tests
- [ ] 8.2 Record failure kind (`no-request`, `empty-track`, `parse`, `no-main-world`) in the local diagnostics log with no caption text; test

## 9. Activation, variants, permissions

- [ ] 9.1 Offer the mode (control near the player + popup); start on click; per-device "start automatically on YouTube" preference, off by default; the every-site grant alone never starts it; inject the main-world script only when the mode starts; popup states "no captions in the studied language" when applicable
- [ ] 9.2 Registration per variant in `build.mjs` (Chromium dynamic, Firefox desktop, Safari)
- [ ] 9.3 Manual pass per variant: written track, automatic track, seek, rate ×1.5, theatre, full-screen, advertisement, next video, phrase capture, card source opens at the moment, mode off restores captions

## 10. Disclosure and review

- [ ] 10.1 Lingua annex (fr + en): on YouTube the extension fetches the video's captions from YouTube; with no engine the translation shown there is YouTube's
- [ ] 10.2 `REVIEWERS.md`: main-world script scope, the replayed player request, no token stored or shared, no request outside YouTube
- [ ] 10.3 Store listings mention the YouTube caption mode (desktop only)

## 11. Gates

- [ ] 11.1 `openspec validate add-lingua-youtube-captions --strict`
- [ ] 11.2 Extension lint, typecheck and tests green; coverage ≥ 80 %
