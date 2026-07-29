## 1. Storage target (backend/storage)

- [x] 1.1 Generalize the store construction so a **second** `LocalFirstStore` can be built for the SoundFont bucket, reusing the OVH endpoint/region/credentials with its own bucket + warm-cache root — without changing the scores store. Keep the `ObjectStorage` trait seam intact. (No storage-crate change needed — `LocalFirstStore::from_config(local_root, &S3Params)` already builds an independent instance per bucket/root; the second instance is wired in §2.2.)
- [x] 1.2 Add a **range-capable read** to the storage seam if not present (return a stream/`Bytes` for a requested byte range, honouring local-first → S3 origin), so large fonts stream without buffering the whole object. Host-test the range slicing on the local store. (`ObjectStorage::get_range(key, Range) -> Vec<u8>` + `size(key) -> u64`; `LocalFirstStore` serves the range from local, warms the whole object from the origin on a miss then slices; `FakeStore` slices in-memory. 5 new storage tests; `DeleteFailsStore` double updated.)

## 2. Config (backend/platform)

- [x] 2.1 Add `SoundfontStorageConfig`: `CYMBRA_SOUNDFONT_S3_BUCKET` (gate — unset disables the route), dedicated `CYMBRA_SOUNDFONT_S3_*` keys, and `CYMBRA_SOUNDFONT_LOCAL_ROOT` (default `/srv/cymbra/soundfonts`). Parse + validate like `ScoreStorageConfig` (partial config fails fast). 4 config tests (absent/present/default-root-distinct-from-scores/partial-fails).
- [x] 2.2 Wire the soundfont store construction in `backend/server` startup from the config (present only when the bucket is set), mirroring `LocalFirstStore::from_config`. (`soundfont_store: Option<Arc<dyn ObjectStorage>>` built in `main.rs`; `None` logs "disabled" and the route responds 503.)

## 3. Font catalog + entitlement (pure, host-tested)

- [x] 3.1 Add a server-owned **font catalog**: `id -> { object_key, tier: Free|Paid, license, attribution }`. Seed v1 with the free default (`upright-piano-kw` → the CC0 `.sf2`). Pure `lookup(id)`; host-tested. (In `backend/server/src/soundfont.rs`.)
- [x] 3.2 Add an **entitlement seam**: `may_access(font, user_id, &dyn Entitlements) -> bool` — Free ⇒ allow any authenticated identity; Paid ⇒ consult the `Entitlements` source (`NoPaidEntitlements` v1 impl). Pure + host-tested (free allowed, paid refused / allowed when owned).

## 4. Delivery route (backend/server, Axum :8081)

- [x] 4.1 Add `GET /soundfonts/:id`: authenticate the `Authorization: Bearer` access token (reused `token::verify` via the injectable `SoundfontAuth`/`JwtAuth` seam — same tokens as gRPC); resolve via the catalog; run the entitlement check; on allow, serve range-aware from the store. Outcomes: unauthenticated → 401, unknown id → 404, not-entitled → 403, bucket unset → 503, unsatisfiable range → 416. CORS layer allows the back-office origins + `Authorization`/`Range`. Pure `decide`/`parse_range` host-tested; handler tested via `oneshot` + `FakeStore` + fake auth (10 tests).
- [x] 4.2 Ensure entitlement/existence ordering: `decide` runs auth → lookup → entitlement **before** any storage access, so a refusal never depends on whether the object bytes exist (403 for known-but-not-entitled, 404 only for an unknown catalog id).

## 5. Deploy (backend/deploy)

- [x] 5.1 Add the `/soundfonts/*` path to the Caddyfile `@http` matcher so it routes to `server:8081` (alongside `/web/auth/*`), keeping everything else on gRPC/gRPC-web. (Done; the deploy runbook already documents the box `git pull` + `caddy reload` for Caddyfile changes.)
- [x] 5.2 Document the new env in `.env.prod.example` (soundfont bucket/region/keys/local root) and add an ops step to seed the default font: `aws --endpoint-url … s3 cp UprightPianoKW-20220221.sf2 s3://cymbra-soundfonts/<key>`. Records that the bucket is **private + distinct** from `cymbra-scores`, with a dedicated access key recommended.

## 6. Tests & verification

- [x] 6.1 Rust: catalog lookup, entitlement (free allow / paid refuse), and the route's decision logic (auth required, 403-before-read, 404 unknown, 503 when unconfigured, 206 range, 416) via the `FakeStore`/fake-auth doubles. `cargo llvm-cov --workspace --fail-under-lines 80` passes (workspace 92.41%).
- [x] 6.2 `cargo fmt`/`clippy` clean; a range read returns the correct slice from a local fixture object. (`cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; storage `size_and_range_local_hit`/`range_falls_back_to_origin_and_warms` assert the slices.)
- [x] 6.3 `openspec validate add-soundfont-delivery --strict` passes.

## 7. Consumer follow-ups (tracked here, delivered in the consumer changes)

> Intentionally NOT part of this backend change — they live in the consumer changes
> and depend on this route being deployed. Left unchecked as a tracked hand-off.

- [ ] 7.1 Back-office (`add-wasm-notation-preview`): switch `SF2_URL` → `VITE_SOUNDFONT_URL` pointing at `GET /soundfonts/<default-id>` and stop staging the 57MB font into `public/soundfonts/` (drop it from `gen_wasm.sh`/Pages). Note in that change's README/deploy caveat that the CF Pages 25MiB blocker is resolved by this route. — DEFERRED to that change.
- [ ] 7.2 App (`piano-sound-selection`): point the download-on-first-use `SoundFontSource` at `GET /soundfonts/{id}` for the CC-BY grands, sharing this host + ids. — DEFERRED to that change.
