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

- [x] 2.1 Added `path_provider` + `cryptography` (AES-256-GCM + HKDF-SHA256, pure
  Dart so it unit-tests off-device) to `pubspec.yaml`.
- [x] 2.2 `OfflineKeyProvider` seam (`offline_key_provider.dart`): `HkdfOfflineKeyProvider`
  derives the KEK via HKDF-SHA256 over {keystore device key, server secret, user
  uuid, per-install seed}; device key + seed generated once via the injectable
  `SecureBytesStore` (flutter_secure_storage); build version excluded. Prod +
  in-memory/unavailable fakes.
- [x] 2.3 `hasUsableKeystore()` write-reads-back a canary (fail-closed signal),
  best-effort like [token_store.dart](apps/music/lib/services/token_store.dart)
  (swallows platform failures, never throws to UI).
- [x] 2.4 `OfflineServerSecretService` (`offline_server_secret_service.dart`):
  fetches the server secret via the new `getOfflineCacheKey` RPC on `CatalogService`,
  caches it in the keystore, refreshes opportunistically, falls back to cache offline.
- [x] 2.5 Unit tests (`offline_key_provider_test.dart`): HKDF determinism,
  per-install-seed uniqueness, KEK changes on any input change, probe true/false,
  fail-closed, clear-forces-new-key.

## 3. App — encrypted cache store

- [x] 3.1 `OfflineScoreCache` seam (`offline_score_cache.dart`): `write`, `read`,
  `has`, `evict`, `purgeAll`, keyed by `catalog:<id>` / `contributed:<id>` (file
  name is the SHA-256 of the key, so ids don't leak).
- [x] 3.2 Envelope encryption on write (random per-file DEK, AES-256-GCM, DEK
  wrapped by the KEK; header carries wrap nonce + wrapped DEK + payload nonce +
  plaintext SHA + ETag). Temp-write-then-rename; no plaintext on disk.
- [x] 3.3 Decrypt on read; auth-tag failure / parse error / integrity-hash
  mismatch → miss (delete the file, return null).
- [x] 3.4 Fail-closed: when the keystore is unusable (KEK null), `write` is a
  no-op and `read` returns null (also for guest / no server secret).
- [x] 3.5 Production `EncryptedFileOfflineScoreCache` + in-memory
  `InMemoryOfflineScoreCache` fake; `offlineScoreCacheProvider` registered.
- [x] 3.6 Unit tests (`offline_score_cache_test.dart`): round-trip, no-plaintext,
  tamper→miss, integrity-hash mismatch→miss, cross-install→undecryptable,
  no-keystore/guest/no-secret→no write, evict, purgeAll.

## 4. App — load path integration

- [x] 4.1 In [notation_notifier.dart](apps/music/lib/state/notation_notifier.dart)
  `_load`, for byte-sourced entries: read the cache first; on hit
  decrypt→parse→render (no network round-trip — content is immutable under a
  stable id).
- [x] 4.2 On a successful network fetch of a favorite (upload flag / saved-library
  membership), write the encrypted copy ("opened once while favorited").
- [x] 4.3 Bundled assets (`_cacheKey` null) and non-favorites are kept out of the
  cache write path.
- [x] 4.4 Added `ScoreLoadFailure.offlineUnavailable` + l10n (en/fr/it/es), wired
  into [score_load_message.dart](apps/music/lib/screens/score_load_message.dart);
  the load path classifies a byte-sourced miss to it when `connectivityService`
  reports offline (else keeps `unavailable`). No raw errors; existing
  snackbar-stay-on-library path unchanged.
- [ ] 4.4b Home/library: mark favorites without cached bytes as "not available
  offline" while offline. **DEFERRED** (needs the snapshot-driven home in 4b).
- [~] 4.5 ETag/conditional fetch: the **backend** supports it (task 1b) and the
  cache stores a per-entry plaintext-SHA integrity check (corrupt→miss works).
  The **client-side conditional-fetch round-trip** (matching-hash skips
  re-download) is **DEFERRED**: wiring it needs an etag-returning byte method on
  the `CatalogService`/`ScoreUploadService` seams (+ updating ~8 hand-fakes).
  Cache currently stores `etag: ''`.
- [x] 4.6 Notifier tests (`notation_offline_cache_test.dart`): cache-hit plays
  offline (no service fetch), cache-miss fetches + writes, non-favorite never
  writes, offline-uncached → localized failure, online-unavailable → generic.
  (Corrupt→miss covered by `offline_score_cache_test.dart`.)

## 4b. App — offline favorites index snapshot

- [x] 4b.1 `FavoritesIndexStore` (`favorites_index_store.dart`): `read` / `write`
  / `clear`, metadata-only (no bytes), **plaintext** over the `PreferencesService`
  seam, keyed `favorites-index:<userId>`, decoupled from the keystore. With tests.
- [ ] 4b.2 Write the snapshot on successful favorites fetch. **DEFERRED** (store +
  its clear-on-sign-out are wired; the per-provider write/read-fallback is the
  next slice).
- [ ] 4b.3 Offline fallback: return the snapshot from the favorites providers when
  the online fetch fails. **DEFERRED** (see 4b.2).
- [ ] 4b.4 Per-favorite "playable offline" flag (probe the cache). **DEFERRED**
  (pairs with 4.4b / the snapshot-driven home).
- [x] 4b.5 The snapshot is cleared on sign-out / account deletion (wired in
  `session_notifier._purgeOfflineData`, alongside the byte-cache purge).
- [~] 4b.6 Tests: the store itself is unit-tested (`favorites_index_store_test.dart`:
  round-trip, per-user scoping, empty-clears, clear, corrupt→empty). The
  offline-launch / playable-flag widget tests pair with 4b.2–4b.4 (**DEFERRED**).

## 5. App — eviction wiring

- [x] 5.1 `SavedCatalogScores.remove` evicts `catalog:<id>` via the cache service.
- [x] 5.2 `MyUploads.toggleFavorite(false)` and `delete` evict `contributed:<id>`
  (favoriting keeps any existing copy).
- [x] 5.3 `session_notifier._purgeOfflineData` (from `_endLocalSession` +
  `onAccountDeleted`, i.e. sign-out / sign-out-everywhere / account deletion)
  `purgeAll()`s the cache + clears key material, snapshot, and cached secret.
- [ ] 5.4 Orphan sweep on favorites refresh. **DEFERRED** (pairs with the
  snapshot-driven home in 4b; local eviction + sign-out purge already bound the
  cache).
- [x] 5.5 Eviction tests (`offline_cache_eviction_test.dart`): remove-saved,
  delete-upload, un-favorite (evicts) vs favorite (keeps), absent-file no-op,
  purgeAll. Sign-out purge exercised via the session tests.

## 6. Cross-cutting: platforms, docs, gates

- [~] 6.1 Key custody reuses `flutter_secure_storage` (same backends as
  `SecureTokenStore`), and the fail-closed path is covered by the keystore-probe +
  no-keystore unit tests. Live per-platform keystore verification is a manual
  device pass (**DEFERRED**).
- [ ] 6.2 Manual/integration smoke on a real device (**DEFERRED** — needs a
  device + live backend; the flow is covered by unit/notifier tests).
- [x] 6.3 `openspec validate add-offline-score-cache --strict` passes.
- [~] 6.4 `flutter analyze`, `dart format`, `dart run custom_lint` clean; full
  Flutter suite green (676 tests). Rust: `cargo test` (130), `clippy`, `fmt`
  clean. Coverage gate to run in CI. (`melos run analyze` = `flutter analyze`.)
