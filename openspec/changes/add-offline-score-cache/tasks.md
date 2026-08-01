## 1. Backend — per-user offline cache secret

- [x] 1.1 Add a migration in `backend/music` storing a per-user offline cache
  secret (≥32 bytes) keyed by user uuid, under existing sensitive-data-at-rest
  protections. → `0012_offline_cache_secret.sql` (`music.offline_cache_secrets`).
- [x] 1.2 Add `GetOfflineCacheKey` RPC (request/response) to
  [score.proto](backend/music/proto/score.proto); regenerate stubs (tonic-build
  runs on `cargo build`; Dart stubs via `melos run gen-grpc`).
- [x] 1.3 Implement the handler: authenticated + owner-scoped; create-on-first-
  request, return-unchanged thereafter; reject unauthenticated. Pure generation in
  `offline_secret.rs`; store trait doubled with the crate's hand-written `Fake`
  convention (this crate does not depend on mockall).
- [x] 1.4 Implement rotation + account-deletion invalidation: `rotate_offline_cache_secret`
  on the module + `DELETE FROM music.offline_cache_secrets` in the `purge_user`
  worker job so prior offline caches become undecryptable.
- [x] 1.5 Ensure the secret never lands in logs or any cross-user/admin listing
  (handler is silent; sqlx errors never bind the value; no cross-user read path).
- [x] 1.6 Rust tests: first-request creates, repeat returns same, owner-scoping,
  unauthenticated rejected, rotation changes value (module + gRPC + fake-store
  tests). Coverage gate run in task 6.4.

## 1b. Backend — content hash (ETag) + conditional fetch

- [x] 1b.1 Expose the stored `sha256` as an ETag on the bytes responses
  (`GetScoreBytesResponse.etag`, `GetCatalogScoreBytesResponse.etag`) — additive
  proto fields; stubs regenerated on build.
- [x] 1b.2 Add an optional `if_none_match` hash to the bytes requests; when it
  matches the stored hash, return `unchanged = true` with no bytes; otherwise
  return full bytes + current hash. Existing auth/access scoping preserved
  (catalog `object_ref` keeps the moderation gate).
- [x] 1b.3 Rust tests: hash returned + stable, unchanged→no bytes,
  mismatch/absent→full bytes, access rules unchanged under conditional fetch.

## 2. App — crypto + key provider seams

- [ ] 2.1 Add `path_provider` dependency; confirm AES-256-GCM + HKDF-SHA256
  primitives (via `crypto`/`cryptography`) and pin choice.
- [ ] 2.2 Define an injectable `OfflineKeyProvider` seam: derives the KEK via
  HKDF over {keystore device key, server per-user secret, user uuid, per-install
  seed}; generates + persists the per-install seed in `flutter_secure_storage`;
  build version deliberately excluded. Provide a production impl + a fake.
- [ ] 2.3 Add a `KeystoreProbe` that write-reads-back a canary in the keystore to
  detect a usable secure store (fail-closed signal). Best-effort, never throws to
  UI (mirror [token_store.dart](apps/music/lib/services/token_store.dart)).
- [ ] 2.4 Fetch + cache the server per-user secret (new backend RPC) behind a
  service seam; keep it in the keystore, refresh opportunistically when online.
- [ ] 2.5 Unit tests: HKDF determinism for fixed inputs, per-install-seed
  uniqueness, KEK changes when any input changes, keystore-probe true/false.

## 3. App — encrypted cache store

- [ ] 3.1 Define an injectable `OfflineScoreCache` seam over the app's private
  cache directory: `write(entryKey, bytes)`, `read(entryKey) -> bytes?`,
  `evict(entryKey)`, `purgeAll()`, keyed by stable `catalog:<id>` /
  `contributed:<id>`.
- [ ] 3.2 Implement envelope encryption on write (random DEK, AES-256-GCM, DEK
  wrapped by the KEK; wrapped DEK + nonce in the file header). No plaintext on
  disk.
- [ ] 3.3 Implement decrypt on read; on auth-tag failure or missing key material,
  treat as a miss (delete the file, return null).
- [ ] 3.4 Wire fail-closed: when `KeystoreProbe` reports no usable keystore,
  `write` is a no-op and `read` returns null (online-only).
- [ ] 3.5 Provide a production impl + an in-memory fake; register providers.
- [ ] 3.6 Unit tests: round-trip write→read, tamper→miss, cross-seed file →
  undecryptable, no-keystore → no write, purgeAll clears everything.

## 4. App — load path integration

- [ ] 4.1 In [notation_notifier.dart](apps/music/lib/state/notation_notifier.dart)
  `_load`, for favorited catalog/upload entries: read cache first; on hit
  decrypt→parse→render, then best-effort online refresh + rewrite.
- [ ] 4.2 On a successful network fetch of a favorited entry with caching
  enabled, write the encrypted copy ("opened once while favorited").
- [ ] 4.3 Keep bundled assets and non-favorited entries out of the cache path.
- [ ] 4.4 Add a `ScoreLoadFailure.offlineUnavailable` variant + l10n string; the
  load path classifies to it when there is no local copy and connectivity is
  offline (else keep `unavailable`). Wire it into
  [score_load_message.dart](apps/music/lib/screens/score_load_message.dart);
  no raw errors, user stays put (existing snackbar-on-failure path).
- [ ] 4.4b Home/library: mark favorites without cached bytes as "not available
  offline" while the app is offline (drive off the index's playable flag +
  `connectivityService`).
- [ ] 4.5 Store the server content hash (ETag) with each cache entry; on online
  open do a conditional fetch — unchanged ⇒ keep cache (no re-encrypt),
  mismatch ⇒ rewrite. On read, recompute the hash and treat a mismatch (corrupt/
  stale) as a miss.
- [ ] 4.6 Widget/notifier tests: cache-hit plays offline (no service fetch),
  cache-miss fetches + writes, non-favorite never writes, offline-uncached shows
  the localized failure, matching-hash skips re-download, changed-hash rewrites,
  corrupted-file → miss.

## 4b. App — offline favorites index snapshot

- [ ] 4b.1 Add a `favorites-index:<userId>` snapshot store (metadata only — no
  bytes): `writeIndex(entries)` / `readIndex()`. Store in **plaintext** local
  storage (JSON under app support / the `local-preferences` store),
  **decoupled from the keystore** so it survives on a no-keystore install.
- [ ] 4b.2 Write the snapshot whenever
  [saved_catalog_scores.dart](apps/music/lib/state/saved_catalog_scores.dart) /
  [contributed_scores.dart](apps/music/lib/state/contributed_scores.dart) fetch
  successfully (persist the resolved `CatalogEntry` list, no bytes).
- [ ] 4b.3 Fallback: when the online fetch fails (offline), return the snapshot
  from those providers instead of surfacing `AsyncError`, so the home renders.
- [ ] 4b.4 Annotate each favorite with a "bytes cached / playable offline" flag
  (probe the cache) for the home to render; opening a non-cached favorite offline
  hits the existing "unavailable offline" typed failure.
- [ ] 4b.5 Clear the snapshot on sign-out (folded into `purgeAll`).
- [ ] 4b.6 Tests: offline-launch renders from snapshot, successful fetch rewrites
  snapshot, guest/signed-out empty, playable flag reflects byte-cache presence,
  sign-out clears snapshot, **no-keystore install still lists favorites offline**
  (index survives while byte cache is disabled).

## 5. App — eviction wiring

- [ ] 5.1 On remove-saved-catalog-score
  ([saved_catalog_scores.dart](apps/music/lib/state/saved_catalog_scores.dart)):
  evict `catalog:<id>` via the cache service.
- [ ] 5.2 On un-favorite / delete-upload
  ([contributed_scores.dart](apps/music/lib/state/contributed_scores.dart)):
  evict `contributed:<id>`.
- [ ] 5.3 On sign-out / sign-out-everywhere / account-deletion
  ([session_notifier.dart](apps/music/lib/state/session_notifier.dart)):
  `purgeAll()` + clear key material.
- [ ] 5.4 Orphan sweep: when favorites refresh, delete cache files whose entry is
  no longer a favorite (dedicated listener widget, not scattered in build).
- [ ] 5.5 Tests: each eviction path deletes the right file, absent-file no-op,
  purge on sign-out, orphan sweep removes stale files.

## 6. Cross-cutting: platforms, docs, gates

- [ ] 6.1 Verify keystore backends per platform (iOS Keychain/SE, Android
  Keystore/StrongBox, macOS, Windows DPAPI, Linux Secret Service) and confirm
  fail-closed on a no-keystore desktop config.
- [ ] 6.2 Manual/integration smoke: cache a favorite online, kill network,
  relaunch, play from cache; then un-favorite and confirm the file is gone.
- [ ] 6.3 `openspec validate add-offline-score-cache --strict` passes.
- [ ] 6.4 `melos run analyze`, `dart format`, `dart run custom_lint`,
  `cargo fmt`/`clippy` clean; Flutter + Rust coverage ≥ 80%.
