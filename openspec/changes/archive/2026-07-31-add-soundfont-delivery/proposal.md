## Why

Piano playback needs the synth's SoundFont (`.sf2`) in the client, but the fonts are
large — the default is ~57 MB and future grands reach ~296 MB — and cannot ship as
static assets: Cloudflare Pages caps a file at 25 MiB, and bundling them would bloat
every build. The catalog scores already live in a **private** OVH S3 bucket
(`cymbra-scores`), streamed to clients through the backend; SoundFonts want the *same*
"store-in-OVH, serve-through-backend" shape but as a **separate concern**. Crucially,
**paid/licensed fonts are planned**, so delivery must be access-controlled from day one —
a public bucket is not an option. This change adds a first-class way to store and serve
SoundFonts so the back-office score-preview playback (and later the app's piano picker)
can fetch them.

## What Changes

- Introduce a **dedicated, private OVH S3 bucket `cymbra-soundfonts`**, separate from the
  private `cymbra-scores` scores bucket — distinct ACL, lifecycle, and warm cache, so
  large fonts never bloat the scores cache and paid fonts stay gated.
- Add a **second object-store target** for SoundFonts (its own `LocalFirstStore`:
  local-disk warm cache + OVH origin), reusing the existing OVH endpoint/region/
  credentials with a separate bucket + cache root — the scores storage is unchanged.
- Add an **authenticated backend HTTP route** that streams a SoundFont by id from the
  soundfont store (range-capable), reachable at the API origin. It requires a valid
  session and passes each request through a **per-font entitlement check** — a seam that
  trivially allows the free (CC0/CC-BY) fonts today and is **ready to gate future paid
  fonts** (purchase check) without a redesign. The scores bucket and gRPC surface are
  untouched.
- Route the new path through the reverse proxy (Caddy `@http` matcher → the Axum port)
  and add its config (soundfont bucket + cache root env). The browser fetches it from the
  API origin, already allowed by the back-office CORS/CSP, and caches it client-side.
- **Consumers** (wiring is theirs, not this change): the back-office playback points
  `VITE_SOUNDFONT_URL` at this route and drops the Pages-bundled font; the app's
  `piano-sound-selection` download-on-first-use fonts use the same host.

## Capabilities

### New Capabilities
- `soundfont-delivery`: store SoundFonts in a dedicated private object-store bucket and
  serve them to authenticated clients through a backend streaming route gated by a
  per-font entitlement check (free fonts allowed now; paid fonts gate-ready), keeping the
  scores store and its privacy unchanged.

### Modified Capabilities
<!-- None: this adds a new HTTP surface + storage target without changing existing
     backend-service, backend-score-storage, or auth requirements. -->

## Impact

- **`backend/storage`** (`object_store` 0.11 aws / `LocalFirstStore`): support a second
  named store instance for the soundfont bucket (or a parameterised builder), with its own
  bucket + warm-cache root; scores path unchanged.
- **`backend/platform` config**: new env — `CYMBRA_SOUNDFONT_S3_BUCKET` (gate; unset
  disables the route), reuse of the OVH endpoint/region/keys (or dedicated ones), and
  `CYMBRA_SOUNDFONT_LOCAL_ROOT` (warm cache).
- **`backend/server`** (Axum on :8081): a new authenticated `GET /soundfonts/{id}`
  streaming handler with the entitlement seam; pure id→key/entitlement logic host-tested,
  the streaming/IO glue thin (coverage-excluded like other adapters).
- **`backend/auth` / `web-auth-session`**: the route reuses the existing session auth (the
  same identity the gRPC/web-auth surfaces use); no auth model change.
- **`backend/deploy`**: add the route path to `Caddyfile`'s `@http` matcher (→ server:8081)
  and document the new env in `.env.prod.example`; an `aws s3 cp` step to seed the default
  font into `cymbra-soundfonts` (like `sync-scores.sh`).
- **Downstream, not in this change**: `add-wasm-notation-preview` (back-office
  `VITE_SOUNDFONT_URL`) and `piano-sound-selection` (app download source) consume this
  endpoint. Sequence this before enabling back-office playback in prod.
- **Unchanged**: the `cymbra-scores` bucket, catalog/score serving, the gRPC-web surface,
  and client CSP `connect-src` (the route is on the already-allowed API origin).
