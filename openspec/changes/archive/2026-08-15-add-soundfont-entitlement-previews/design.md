## Context

The SoundFont catalog lives in Postgres (`music.soundfonts`) and is delivered by a
single Axum route, `GET /soundfonts/{id}` → `backend/server/src/soundfont.rs::serve`.
`serve` today: authenticates the caller, computes `can_view_unvalidated` (a
music-scope moderator sees non-accepted fonts), calls `decide(user,
can_view_unvalidated, font)` to gate on **moderation status only**, then streams
the raw `.sf2` bytes from object storage. Admin upload is `POST /soundfonts/{id}`
(`upload`, ~line 284), router at ~line 230.

`add-curation-rewards` added `point_cost INT NOT NULL DEFAULT 0` and `redeemable
BOOL NOT NULL DEFAULT false` to `music.soundfonts`, and a `curation_grants` table
(one row per user per redeemed reward). A redeemed reward = a grant. But `serve`
never reads either — so a costed, un-redeemed font's bytes are downloadable by any
authenticated user. The reward is cosmetic, and the *critical asset itself* is
exposed.

Auditioning today (app `soundfonts_screen.dart::_togglePreview`) calls
`soundFontSourceProvider.resolve(entry)` — which for a server font hits `GET
/soundfonts/{id}` and pulls the **whole font** — then `audioService.loadSoundFont`
+ plays a sample melody with the **client** synth (`rustysynth` in
`apps/music/rust/src/api/audio*.rs`). So today you cannot hear a font without
downloading it. The backend has **no** audio synthesis.

## Goals / Non-Goals

**Goals:**
- The raw `.sf2` bytes are served only to an **entitled** caller (free / owned /
  own-import / music-scope moderator-admin), for both explicit downloads and the
  in-app instrument load (both flow through `GET /soundfonts/{id}`).
- A locked font stays **auditionable** via a short, server-rendered **preview
  clip** — the raw font never leaves the server to be previewed.
- Previews are **pre-rendered once** (at upload) and **regenerable** from the back
  office (fallback for pre-existing/seeded fonts and failed renders).
- Refusal does not leak existence (a locked font looks the same as a missing one).

**Non-Goals:**
- Money purchases, real DRM, per-user watermarking.
- On-demand / per-request server rendering — we render once and cache the object.
- Changing the reward economy (points, levels, grants) — only *enforcing* it here.
- Streaming/transcoding pipeline — one short clip, one compact format, one object.

## Decisions

### 1. Entitlement decision (host-testable core)
A pure function decides download entitlement, mirroring the existing `decide`
moderation split so both are unit-tested without a DB or HTTP:

```
fn entitlement(caller, font, has_grant, is_music_moderator_admin) -> Access
// Access = Allow | Deny (Deny → 404-style, indistinguishable from missing)
// Allow iff any of:
//   font.point_cost == 0                        // free
//   caller == font.uploaded_by                  // own import
//   has_grant                                    // owned (a curation_grants row)
//   is_music_moderator_admin                     // exempt
```

`redeemable` is a *catalog display* flag (is this offered in the shop); the gate
keys on `point_cost == 0` for "free", not on `redeemable`, so a non-redeemable
costed font is still gated. The route computes `has_grant` with one grants read
(scoped to caller + font) and `is_music_moderator_admin` from the existing scope
check, then calls `entitlement(...)`. **Moderation gate stays first**: a caller
who can't view the font at all (not accepted, not a moderator) is refused before
entitlement is even considered. Both cores live in a `soundfont_access` module,
coverage-included; the route glue in `soundfont.rs` stays coverage-excluded.

`SoundFontRepo`'s font lookup must surface `point_cost`, `redeemable`, and
`uploaded_by` (extend the row struct + the runtime `SELECT`), plus a
`has_grant(user, soundfont)` read (a `SELECT 1 FROM music.curation_grants ...`).

### 2. Backend synth render (new, pure-PCM)
The backend gains `rustysynth` as a dependency (already vendored client-side, so
licensing/versioning is known) used **headlessly**: no audio device, we render to
a PCM buffer. A host-testable core:

```
fn render_preview_pcm(font_bytes: &[u8], sample: &SampleSequence) -> Result<Vec<i16>>
// load SoundFont from bytes → Synthesizer at a fixed sample rate (e.g. 44.1k) →
// feed a fixed NoteOn/NoteOff sequence → collect interleaved/mono i16 PCM.
```

`SampleSequence` is a fixed short arpeggio/melody (a handful of notes, ~2–3 s) —
the same musical phrase for every font so previews are comparable and the render
is deterministic (unit-testable: same font+sequence → same PCM length/shape).
Encoding to a compact container (WAV PCM is simplest and needs no extra codec dep;
revisit only if size matters) is a second pure helper `encode_preview(pcm, rate)
-> Vec<u8>`. The actual `rustysynth` device-free synthesis call and any file I/O
are the coverage-excluded glue; the sequence definition, PCM shaping, and encoder
are covered.

Chosen format: **WAV/PCM** (`{id}.preview.wav`). Rationale: zero extra codec
dependency, universally playable by the Flutter audio seam, and the clip is short
so size is acceptable. (If size becomes a concern, an Ogg/Opus encoder is a
follow-up — not now.)

### 3. Preview generation lifecycle
- **At upload** (`POST /soundfonts/{id}`): after the font object is stored, render
  the preview from the just-uploaded bytes and store `{id}.preview.wav` as a
  **public** object (its own key namespace, distinct from the private font
  bytes). A render failure does **not** fail the upload — it logs and leaves the
  preview absent (the back-office button is the recovery path). This keeps upload
  robust and makes previews best-effort.
- **Back-office regenerate**: a new admin endpoint (e.g. `POST
  /soundfonts/{id}/preview` — admin-gated like `upload`) re-reads the stored font
  bytes, renders, and overwrites the public preview object. Returns success/failure
  for the button to reflect. This is the fallback for fonts seeded before this
  change and for retrying a failed render.

### 4. Preview delivery + app audition
- **`GET /soundfonts/{id}/preview`**: serves the public clip with **no
  entitlement gate** (hearing it is the point) — only the moderation-visibility
  gate applies (don't expose a rejected/pending font's preview to non-moderators),
  reusing the same visibility split as `serve`. 404 if no preview object exists
  yet.
- **App**: the catalog **play** button (`soundfonts_screen.dart`) auditions by
  fetching `/{id}/preview` and playing the returned clip through a simple audio
  playback seam — it no longer downloads the font to audition. Free/owned fonts
  *may* still audition via the full local path, but routing **all** auditions
  through the preview clip is simpler and keeps one code path; locked fonts
  **must** use the preview. The instrument *use* path (`selectedPiano` →
  `soundFontSourceProvider.resolve` → `GET /{id}`) is unchanged client-side but
  now entitlement-gated server-side, so a locked font can never load even if the
  client guard (already added in `add-curation-rewards`) were bypassed.

### 5. Back-office
Add a **"Generate sample"** action to the SoundFonts admin screen (the existing
store/screen from `soundfont-back-office-management`): a per-row/detail button
calling the regenerate endpoint through the injectable client seam, with the
`Async<T>` union pattern for its in-flight/success/error state (repo convention).

## Risks / Trade-offs

- **`rustysynth` on the backend**: a new backend dependency and headless-synth
  code path. Mitigation: it is already used client-side (known-good, same
  version), rendering is offline/deterministic, and the synthesis call is isolated
  behind a covered core + excluded glue.
- **Preview size / storage**: WAV/PCM is larger than a compressed codec. Mitigation:
  the clip is short (~2–3 s, mono) and stored once as a cached object; an Opus
  encoder is a clean follow-up if it ever matters.
- **Existence oracle**: entitlement denial must look identical to not-found, or an
  attacker can enumerate costed fonts. Mitigation: `Deny` maps to the same
  response as a missing font (no distinct 403 body/status that reveals presence).
- **Stale/absent previews for seeded fonts**: fonts uploaded before this change
  have no preview until regenerated. Mitigation: the back-office "Generate sample"
  button (and a one-time seed/backfill pass) covers them; the app degrades
  gracefully (no preview → play button disabled/greyed with a hint) rather than
  erroring.
- **Preview ≠ full sound**: a fixed short phrase can't represent the whole font.
  Accepted: it's an audition aid to drive unlocking, not a substitute for owning
  the font; the same phrase across fonts makes them comparable.
