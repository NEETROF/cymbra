## 1. Storage target (backend/storage)

- [ ] 1.1 Generalize the store construction so a **second** `LocalFirstStore` can be built for the SoundFont bucket, reusing the OVH endpoint/region/credentials with its own bucket + warm-cache root — without changing the scores store. Keep the `ObjectStorage` trait seam (mockall-doubled) intact.
- [ ] 1.2 Add a **range-capable read** to the storage seam if not present (return a stream/`Bytes` for a requested byte range, honouring local-first → S3 origin), so large fonts stream without buffering the whole object. Host-test the range slicing on the local store.

## 2. Config (backend/platform)

- [ ] 2.1 Add `SoundfontStorageConfig`: `CYMBRA_SOUNDFONT_S3_BUCKET` (gate — unset disables the route), reuse of the OVH endpoint/region/keys (or dedicated `CYMBRA_SOUNDFONT_S3_*`), and `CYMBRA_SOUNDFONT_LOCAL_ROOT` (default under `/srv/cymbra/...`). Parse + validate like `ScoreStorageConfig`.
- [ ] 2.2 Wire the soundfont store construction in `backend/server` startup from the config (present only when the bucket is set), mirroring `LocalFirstStore::from_config`.

## 3. Font catalog + entitlement (pure, host-tested)

- [ ] 3.1 Add a server-owned **font catalog**: `id -> { object_key, tier: Free|Paid, license, attribution }`. Seed v1 with the free default(s) (e.g. `upright-piano-kw` → the CC0 `.sf2`). Pure lookup by id; host-tested (known/unknown id, attribution present for CC-BY).
- [ ] 3.2 Add an **entitlement seam**: `may_access(identity, font: &FontEntry) -> Decision` — Free ⇒ allow any authenticated identity; Paid ⇒ consult an entitlement source (trivial "none entitled" impl now, trait-doubled). Pure + host-tested (free allowed, paid refused).

## 4. Delivery route (backend/server, Axum :8081)

- [ ] 4.1 Add `GET /soundfonts/{id}`: require a valid session (reuse the existing web-auth/session identity extraction the Axum surface uses); resolve the id via the catalog; run the entitlement check; on allow, **stream** the object (range-capable) from the soundfont store. Map outcomes: unauthenticated → 401, not-entitled → 403, unknown id → 404, bucket unset → 503. Thin IO/HTTP glue (coverage-excluded); the id→key/entitlement/status decision is the host-tested pure part.
- [ ] 4.2 Ensure entitlement/existence ordering: refuse (403) before reading bytes; do not leak whether a paid object exists to a non-entitled caller.

## 5. Deploy (backend/deploy)

- [ ] 5.1 Add the `/soundfonts/*` path to the Caddyfile `@http` matcher so it routes to `server:8081` (alongside `/web/auth/*`), keeping everything else on gRPC/gRPC-web. Note the box `git pull` + `caddy reload` step (Caddyfile changes aren't auto-deployed).
- [ ] 5.2 Document the new env in `.env.prod.example` (soundfont bucket/region/keys/local root) and add an ops step to seed the default font: `aws s3 cp UprightPianoKW-20220221.sf2 s3://cymbra-soundfonts/<key>` (mirror `sync-scores.sh`). Record the bucket is **private + distinct** from `cymbra-scores`.

## 6. Tests & verification

- [ ] 6.1 Rust: catalog lookup (known/unknown/attribution), entitlement (free allow / paid refuse), and the route's decision logic (auth required, 403-before-read, 404 unknown, 503 when unconfigured) via the mockall storage/entitlement doubles. `cargo llvm-cov --workspace --fail-under-lines 80` (new IO/HTTP glue in the coverage ignore regex).
- [ ] 6.2 `cargo fmt`/`clippy` clean; a range read returns the correct slice from a local fixture object.
- [ ] 6.3 `openspec validate add-soundfont-delivery --strict` passes.

## 7. Consumer follow-ups (tracked, not implemented here)

- [ ] 7.1 Back-office (`add-wasm-notation-preview`): switch `SF2_URL` → `VITE_SOUNDFONT_URL` pointing at `GET /soundfonts/<default-id>` and stop staging the 57MB font into `public/soundfonts/` (drop it from `gen_wasm.sh`/Pages). Note in that change's README/deploy caveat that the CF Pages 25MiB blocker is resolved by this route.
- [ ] 7.2 App (`piano-sound-selection`): point the download-on-first-use `SoundFontSource` at `GET /soundfonts/{id}` for the CC-BY grands, sharing this host + ids.
