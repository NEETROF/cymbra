# Proposal

## Why

Cymbra Music has been live on the App Store and Google Play since 2026-09-16, and it is
time to start telling people about it. The page a post, a message or a video description
would link to — `cymbra.app/music` (and `/en/music`, `/es/music`) — shows no image and no
video: eight cards of text and emoji. It also declares no share preview, so a link pasted
on Reddit, Discord or WhatsApp renders as a bare line of text. Promoting the app before the
page shows it would spend the first, most valuable wave of visitors on a page that cannot
convince them.

## What Changes

- The Music page, in French, English and Spanish, shows the app: a gallery of real
  screenshots in the page's language, taken from the store captures the harness already
  generates (`apps/music/store/`), so the site and the store listings never disagree.
- The Music page shows a short, muted, looping video of the app being played (score
  following, Wait Mode, scoring), recorded from a scripted run of the real app — the same
  no-backend, faked-keyboard approach as the store captures, never a hand-made recording.
- Every Music page declares a share preview (title, description, a 1200×630 image in its
  language, an absolute URL), so a shared link shows a picture.
- A check fails the site build when the site's copies drift from the store captures, when a
  media file exceeds its weight budget, or when a Music page ships without its preview.
- The capture harness gains a documented command that records the showcase video.

Products impacted:

- **site** — new: the Music page's gallery, video and share preview; the shared layout gains
  an optional share-preview input that any page can use (only the Music pages set it here).
- **Music** — consumed: the store captures and the capture harness's deterministic,
  no-backend app state. New: a video recording mode in that harness. No change to the app.
- **ID / Live / back-office** — not impacted.

Nothing on the page mentions pricing, a trial or a beta; the copy keeps describing what the
app does.

## Capabilities

### New Capabilities

- `site-music-page`: the public Music page on cymbra.app — how it shows the app (screenshots
  and video from the real app, in the page's language), its share preview, and the checks
  that keep its media in step with the store captures and within budget.

### Modified Capabilities

- `music-store-listing-capture`: the harness also records a short showcase video from a
  scripted run of the real app, with the same determinism and locale rules as the
  screenshots.

## Impact

- `apps/site`: `src/pages/music.astro`, `src/pages/en/music.astro`, `src/pages/es/music.astro`;
  `src/layouts/Base.astro` (optional share-preview prop); new media under the site
  (screenshots, video, poster, preview images); `src/lib/i18n.ts` (alt texts, captions);
  new tests (media check, built-HTML preview check).
- `apps/music`: `tool/` gains the video recording command; `integration_test/` gains the
  showcase scenario; `store/README.md` documents it.
- `.github/workflows/site-check.yml`: its paths filter watches the store captures the site
  copies, so a capture refresh that is not carried to the site fails there.
- Coordination: `add-site-lingua-showcase` (proposed in parallel) needs the same share-preview
  input on the shared layout. Whichever lands first adds it; the second reuses it.
- No backend, API, `.proto` or app-binary change.
