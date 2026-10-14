Estimate: owner probes ~1 day (group 1) and manual passes ~1 day (group 9); agent work 12–15 ideal
days (groups 2–8, 10). The translation line is `add-lingua-youtube-caption-translation` (3–4 days).

## 1. Probes on the owner's own browsers (owner, ~1 day, before any code)

Use your everyday profiles, where YouTube's consent is already given — an automated browser gets
the consent page and another token. The snippets P1–P6 are in design.md's appendix; paste them in
the extension's background console. Record each answer as an indented note under its task (as in
`fix-lingua-dynamic-rescan` 4.3). Never paste a token, a full URL or caption text: this repository
is public; the snippets return only counts, flags, keys and types.

- [ ] 1.1 Prepare, once (15 min): pick a public video with **written** English captions and one whose only English captions are **automatic**; turn « Toujours surligner » on in Chrome; enable Safari's Develop menu (Settings › Advanced). Record: nothing.
- [ ] 1.2 Chrome (~1 h 30), on the written-captions video, captions off:
  1. Record the Chrome version.
  2. P1, then P2, and press `c` in the player within 30 s. Record `seen`, `initiator`, `hasPot`, `c`, `fetch.status` / `fetch.length` / `fetch.events`, `withoutPot.status` / `withoutPot.length`, and the `fetch.shape` JSON.
  3. Same on the automatic-captions video. Record the same, and whether the shape holds `tOffsetMs` and `acAsrConf`.
  4. Captions off, P3. Record the `tracks` list (language, kind, translatable) or the error. Click a suggested video, P3 again: record whether `video` changed.
  5. Captions off, P4 (on). Record whether English captions appeared. Open another video with English captions in a new tab: record whether its captions came on by themselves (YouTube remembering the choice). Back on the first tab, P4 (off): record whether the native line went away. Restore your own captions setting with the CC button.
  6. P5. Record `os`.
- [ ] 1.3 Firefox (~1 h 30): steps 1–6 of 1.2. Then the fetch-principal question: record `fetch.*` and `contentFetch.*` of step 2 separately — a Firefox content script's `fetch` goes out as the extension, `content.fetch` as the page; which one returns the track (non-zero `length` and `events`) decides where Firefox replays (design D1).
- [ ] 1.4 Safari on macOS (~1 h 30, with the signed dev build of the host app or the installed one): steps 1–6 of 1.2. Record in particular whether P3/P4 fail with an error about `world` (design D2's fallback then applies to Safari) and whether `run` returns P2's result at all (a promise result).
- [ ] 1.5 Once, in any browser (~30 min): during an advertisement — on a YouTube account that shows advertisements — run P6, then again after it. Record `ad` both times and whether `time` restarts after the advertisement. If an iPad is at hand: Safari on the iPad, Develop › (the iPad) › Web Extension Background Content, P5; record `os`.
- [ ] 1.6 (agent, 0.5 d) From the notes: fill the spike table in design.md per variant; settle D1 (observer and replay location per variant), D2 (`world: "MAIN"` or the `<script src>` fallback per variant), D8 (track selection or toggle) and D13 (iPad detection); write **synthetic** json3 fixtures with the recorded structure and invented English sentences — a written track, an automatic track with `tOffsetMs`, `[Music]` and fillers, and an empty body — under `apps/lingua-extension/test/fixtures/youtube/`. Never a real video's caption text.

## 2. Track and sentences (pure, host-testable; ~2.5 d)

- [ ] 2.1 json3 parser → cues (start, duration, text, per-word offsets when present) in a new `apps/lingua-extension/src/youtube/` module; tests on the fixtures; an empty body is a typed failure
- [ ] 2.2 Track choice: written English over automatic English (any `en` or `en-*` language code), none when neither; tests
- [ ] 2.3 Request rebuild from an observed player URL: keep `v`, signature parameters, `pot`, `potc`, `c`; set `lang` / `kind` / `fmt=json3`; refuse a URL whose `v` is not the current video; tests
- [ ] 2.4 Annotation and filler filter (`[…]`, filler list); tests
- [ ] 2.5 Cues → sentences: punctuation split for written tracks, gap-threshold split (starting at 700 ms) with a length cap (starting at 25 words) for automatic tracks, never inside a cue; tests on both fixtures
- [ ] 2.6 Cue lookup by time (binary search) with seek and rate change; tests

## 3. Page-world script, offer and session (~3 d)

- [ ] 3.1 Page-world script (`src/youtube/page-world.ts`, a new esbuild entry in `build.mjs`, listed in `tool/check_variants.mjs`, excluded from coverage in `vitest.config.ts` like the other DOM glue): reads the video id, the tracks (language, kind, translatable), Short / live / premiere, the captions state, from `movie_player.getPlayerResponse()`, again on `yt-navigate-finish`; accepts captions on / off / select track (D8) and, only where 1.6 says so, "replay this `/api/timedtext` URL of the current video"; posts over a per-load random `postMessage` channel; makes no request; add `src/youtube` to `IN_THE_PAGE` in `test/lint-page-context.spec.ts`
- [ ] 3.2 Injection: the content script reports a watch page; the background (`src/background.ts`) runs `scripting.executeScript({ world: "MAIN", files })` once per document, or the web-accessible `<script src>` fallback on a variant 1.6 names; tests on the background handler with a mocked `chrome.scripting`
- [ ] 3.3 Offer decision (pure): studies English; desktop (`runtime.getPlatformInfo().os` through the background, `ios` and `android` refused); `/watch`; not a Short, live stream or premiere in progress; an English track. Re-decided on in-page navigation; tests for every "not offered" scenario of the spec, the iPad one included
- [ ] 3.4 Video session: started from the control, the popup, or the preference; ended on mode off, on leaving the watch page, on another video; token held in memory and discarded with the session; tests with a mocked bridge
- [ ] 3.5 Caption request: the isolated `PerformanceObserver` (or the page-world observer where 1.6 says so); turn captions on through the bridge when no request is seen; replay through the variant's way (Chromium `fetch`, Firefox `fetch` or `content.fetch` or the page world, Safari per 1.4); timeout → degraded; tests
- [ ] 3.6 Restore the reader's captions state and the native line when the mode ends; tests

## 4. Caption line and cards (~3 d)

- [ ] 4.1 Caption element inside `#movie_player`: light-DOM text under our own host marked `data-cymbra-lingua-skip`, slotted into a closed shadow root for its look (D3); follows the default, theatre and full-screen layouts; native line hidden by a stylesheet scoped to `.ytp-caption-window-container`; pointer events on the line do not reach the player; markup by DOM construction APIs, no dynamic `innerHTML`
- [ ] 4.2 Highlight unknown and learning words through the existing analysis and highlight registry, in English (D11); repaint on a status-change broadcast; no repaint when the cue has not changed; tests
- [ ] 4.3 Word click and phrase selection → the page's `SelectionCard` and `SelectionWatcher` with the caption sentence as source, `language: "en"`, `translator: () => source("en")` from `createTranslatorPort` (`src/translate/create-port.ts`), and the page's « Traduction en ligne » offer; pause if playing, resume on close only if the card paused it; tests (jsdom, mocked `TranslatorPort` and online offer)
- [ ] 4.4 While the mode is on and the player is full screen, move the word card's host (`src/reading/wordpopup.ts`) into the player element, and back when it leaves full screen; tests where jsdom allows, the rest in 9.1
- [ ] 4.5 A card from a caption keeps what a page card keeps, plus a local address with `t=<line start>`; tests
- [ ] 4.6 Hide the line and suspend timing and exposure during advertisements (`ad-showing`); test

## 5. Percentage, exposure and « Mots lus » (~2 d)

- [ ] 5.1 Whole-track analysis in English at load → the badge shows the video's percentage while the mode is on; the popup gets `SessionStats.surface: "video"` (`src/reading/session.ts`) and shows, in `src/popup/render.ts`, the breakdown, which track (written / automatic), and the rest of the page apart; tests
- [ ] 5.2 `CaptionExposureTracker` emitting a `BlockReading` (`lemmas`, `read`, `unknown`) in English: a cue counts once playback covered 80 % of its span in video time while playing, not during advertisements, once per session; seeks, paused display and rate changes tested
- [ ] 5.3 Feed it through the page's exposure flush (`onExposed` / `flushExposure` in `src/reading/session.ts`) under the exposure source `video:youtube`; test the distinct-day rule is unchanged
- [ ] 5.4 Daily statistics: optional `video` share (`exposures`, `unknownSeen`) on `DailyStat` in `src/state/dailystats.ts`, part of the totals; `recordReading` takes the origin; `pushStats` in `src/sync/sync.ts` unchanged. Tests: the share is not pushed, the pushed totals include it, « Mots lus » includes it, a record without it reads as zero

## 6. Degraded mode (~1 d)

- [ ] 6.1 A `MutationObserver` of the mode's own on `.ytp-caption-window-container` (the page's `ReadingObservers` ignores it by design, `blockOf` in `src/reading/observer.ts`); the line on screen shown in our line and used as the source sentence; percentage labelled « Lignes vues jusqu'ici »; popup says « Mode dégradé … »; tests
- [ ] 6.2 Failure kinds (`no-request`, `empty-track`, `parse`, `no-main-world`) with their time in a bounded local list `cymbra-lingua-caption-failures` (`src/state/storage.ts`, like `REFRESH_FAILURES_KEY`), no caption text and no video id; test

## 7. Activation and copy (~1.5 d)

- [ ] 7.1 The « Sous-titres Lingua » control near the player and the popup line, shown only when the offer holds (3.3); start on click; « Démarrer automatiquement sur YouTube » in Réglages (the one builder `src/reading/settings-view.ts`, copy in `src/reading/settings-copy.ts`, shown in its three hosts), per device, off by default; the every-site grant alone never starts the mode; tests
- [ ] 7.2 Copy in French, English and Spanish (`src/i18n/`): the control; the popup lines (no English captions; English captions only; for a reader who studies English; « Sous-titres écrits » / « Sous-titres automatiques »; degraded mode; « Lignes vues jusqu'ici »); the preference. The language's name comes from `src/analyzer/language-labels.ts` (`test/lint-language-labels.spec.ts`); no "lemma" / "lemme" in any copy
- [ ] 7.3 Registration per variant: no new manifest permission on Chromium; Firefox and Safari through their static script; `world: "MAIN"` or its fallback per 1.6; `tool/check_variants.mjs` green for the three variants

## 8. Disclosure and review (~0.5 d)

- [ ] 8.1 Lingua annex of the privacy policy in French, English and Spanish (`apps/site/src/pages/confidentialite.md`, `en/privacy.md`, `es/privacidad.md`): on YouTube, once the caption mode is started, the extension fetches the captions of the video being watched from YouTube, and sends nothing of the reader's with them
- [ ] 8.2 `apps/lingua-extension/REVIEWERS.md`: the page-world script's scope (the list of tracks, captions on / off), the replayed player request, no token stored or shared, no request outside YouTube, the degraded fallback
- [ ] 8.3 Store listings in French, English and Spanish mention the YouTube caption mode (desktop browsers, English captions)

## 9. Manual passes (owner, ~1 day)

- [ ] 9.1 In Chrome, Firefox and Safari on macOS: written track; automatic track; a video without English captions shows no control; seek; rate ×1.5; theatre; full screen with a word card open; an advertisement; next video in the same tab; phrase capture; « Traduction en ligne » pressed on a phrase; a card's source opens at its moment; mode off restores captions; « Mots lus » grows after a video
- [ ] 9.2 From 9.1, set the automatic-track gap threshold, the sentence length cap and the 80 % threshold of D5; record the values in design.md

## 10. Gates

- [ ] 10.1 `openspec validate add-lingua-youtube-captions --strict`, and `scripts/openspec_archive_order.py add-lingua-youtube-captions` names only the changes in `archiveAfter` still open
- [ ] 10.2 Extension lint, typecheck and tests green; coverage ≥ 80 %
