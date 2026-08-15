## Why

`add-curation-rewards` made some catalog SoundFonts **redeemable rewards** (a
`point_cost` + `redeemable` on `music.soundfonts`; a redemption is a
`curation_grants` row). But the delivery route `GET /soundfonts/{id}`
(`backend/server/src/soundfont.rs::serve`) still gates only on **moderation
status** — so any signed-in user can download the raw `.sf2` bytes of a costed,
not-yet-unlocked font. The reward is bypassable and, more importantly, the
**critical asset (the font file) is exposed** to non-entitled users.

Yet a locked sound must stay **auditionable** — nobody unlocks a sound they can't
hear. Today auditioning downloads the *whole* font and synthesizes locally, which
is exactly what we must stop for a locked font. So the two are coupled: to gate
the download we must also give a way to hear the sound **without shipping the
font** — a server-side, pre-rendered **preview clip**.

## What Changes

- **Entitlement gate on the full `.sf2` download** — `GET /soundfonts/{id}`
  serves bytes only when the caller is **entitled**: the font is **free**
  (`point_cost = 0`), OR the caller **owns** it (a `curation_grants` row), OR it
  is the caller's **own import**, OR the caller is a **music-scope
  moderator/admin** (exempt). Otherwise it is refused indistinguishably from a
  missing font (no existence oracle). This also hardens the *use* path: loading a
  font as the active instrument goes through this route, so a locked font can
  never be loaded even if the client tried.
- **Server-rendered preview audio, pre-rendered at upload** — when a font is
  uploaded (admin `POST /soundfonts/{id}`), the server synthesizes a **short
  fixed sample melody** with that font (a new pure-PCM render path; the backend
  has no audio synthesis today), encodes it to a compact clip, and stores it as a
  **public** object. The raw font never leaves the server for a preview.
- **Back-office "Generate sample" fallback** — an admin action (endpoint +
  button on the back-office SoundFonts screen) to **(re)generate** a font's
  preview: for already-seeded fonts that predate this change, or after a failed
  render.
- **Preview delivery + app audition** — `GET /soundfonts/{id}/preview` serves the
  pre-rendered clip openly (hearing it is the whole point). The app's SoundFont
  catalog **play** button auditions a font by fetching and playing this **clip**
  instead of downloading the font and synthesizing locally. Free/owned fonts may
  still use the full local path; a locked font uses the preview clip.

Out of scope: real money purchases / DRM / watermarking; on-demand (per-request)
server rendering — we pre-render once. Scope stays the points-reward FreePats CC0
fonts, where "cost" is gamification and the goal is not exposing the raw asset to
non-entitled users.

## Capabilities

### New Capabilities
- `soundfont-entitlement`: the access rule for the raw `.sf2` bytes — free OR
  owned (a redemption grant) OR the caller's own import OR a music-scope
  moderator/admin may download; everyone else is refused as not-found. Applies to
  both the delivery route and the in-app instrument load that flows through it.
- `soundfont-preview`: the server-side, pre-rendered preview clip — generated at
  upload and regenerable from the back office, stored as a public object, served
  openly, and used by the app to audition a sound without ever shipping the font.

### Modified Capabilities
<!-- None. Entitlement and previews are expressed as new capabilities over the
     existing soundfont-catalog delivery; the reward model (curation-rewards) is
     unchanged, only enforced here. -->

## Impact

- **Depends on** `add-curation-rewards` (`point_cost`/`redeemable`,
  `curation_grants`).
- **Backend**: entitlement decision on the delivery route (expose
  `point_cost`/`redeemable` on the font lookup + a grants read); a new pure-Rust
  synth render module (add a `rustysynth` backend dependency) + a fixed sample
  sequence + compact audio encoding; preview object storage; render-on-upload hook;
  an admin regenerate endpoint.
- **Back office**: a "Generate sample" action in the SoundFonts admin screen.
- **App** (`apps/music`): audition via the preview clip (fetch + play an audio
  clip) rather than downloading the font; the locked-font *use* path is already
  refused client-side (`selectedPiano.select`) and now server-side too.
- **Coverage**: Rust ≥ 80% for the entitlement decision + render/sample helpers
  (host-testable cores); the HTTP route + synth/audio glue stay coverage-excluded
  as elsewhere. App ≥ 80% via fakes for the preview-audition path; the Vue button
  under its own test setup.
