## Why

Today every score — a saved catalog score or the user's own upload — is fetched
from the backend on demand each time it is opened (`fetchBytes` in
[notation_notifier.dart](apps/music/lib/state/notation_notifier.dart)). If the
device has no connection at launch, a favorited score the user has already
opened cannot be played. Users expect their favorites to "just work" offline —
on a plane, in a lesson room with no Wi-Fi, on a flaky mobile connection.

We want offline playback of favorited scores **without** shipping plaintext
MusicXML to disk: the corpus is licensed content and user uploads are
attributed works, so a cached file that leaks (copied off the device, pulled
from an iCloud/iTunes/Android backup, or lifted from another user's storage)
must be cryptographically useless on its own.

## What Changes

- When an authenticated user opens a **favorited** score (a saved catalog score
  or one of their own favorited uploads) and the bytes are fetched successfully,
  the app writes an **encrypted** copy to the app's private storage.
- At score open, the app **prefers the local encrypted copy** when present, so a
  favorited-and-once-opened score plays even with no connection at startup. The
  network fetch remains the source of truth and refresh path when online.
- **Offline favorites list**: the favorites list is itself backend-fetched
  today (`savedCatalogScoresProvider` / `myUploadsProvider`), so with no network
  at launch the home would fail to populate and the cached bytes would be
  unreachable. The app therefore also persists a **last-known-good snapshot of
  the favorites index** (entry metadata only — id, kind, title, composer, level;
  **no score bytes**) on every successful online fetch, and **falls back to it
  when offline** so the home renders and favorites are navigable. Each favorite
  indicates whether its bytes are cached (**playable offline**) or not (visible
  but shows the "unavailable offline" message when opened).
- **The favorites index survives even without a keystore**: the index is
  non-sensitive metadata (no licensed bytes), so it is stored in **plaintext**
  local storage, **decoupled from the encrypted byte cache**. The fail-closed
  rule ("no keystore ⇒ no cache") applies **only to score bytes**; a user on a
  no-keystore install (or one that later loses keystore access) still sees their
  full favorites list offline and never "loses" it — only offline *playback* of
  the bytes is unavailable there.
- Encryption uses **per-file envelope encryption** (AES-256-GCM with a random
  data key). The data key is wrapped by a key derived (HKDF) from: a
  hardware-backed device key (Secure Enclave / Android Keystore where available,
  best-effort elsewhere), a **server-issued per-user secret**, the user's
  **uuid**, and a **per-install random seed**. Plaintext MusicXML never touches
  disk. The app **build version is deliberately NOT part of key derivation** so
  the cache survives app updates.
- **Cross-user / cross-device / file-leak resistance**: because the key binds
  the user uuid + per-install seed + server secret held only in the OS keystore,
  a leaked `.enc` file (or a full device backup) cannot be decrypted off the
  originating install.
- **Eviction on un-favorite**: removing a saved catalog score, un-favoriting an
  upload, or deleting an upload **deletes its local encrypted file**. Signing out
  / account deletion purges the whole offline cache.
- **Freshness by content hash (no polling)**: score bytes are content-addressed
  and immutable under a stable id (`catalog_scores.sha256` is `UNIQUE`; a crawler
  re-ingest mints a **new id** rather than mutating bytes; uploads have no
  update/replace path). So a cache entry can never serve stale *content*, and no
  change-detection/polling is needed. The only server-side change is
  disappearance/replacement under a new id — already handled by eviction + the
  orphan sweep. As a cheap integrity + bandwidth guard, the backend exposes the
  existing content hash as an **ETag**: the client stores it with each cache
  entry, and on an online open does a **conditional fetch** — unchanged ⇒ no
  re-download, mismatch/corruption ⇒ treat as a miss and re-fetch.
- **Fail-closed on desktop with no keystore**: if the platform exposes no usable
  secure keystore (e.g. headless Linux without Keyring/KWallet), the offline
  cache is disabled for that install (online-only) rather than writing a
  weakly-protected file. The security invariant "never write a
  decryptable-if-leaked file" holds on every platform.
- **Backend**: a new authenticated operation issues (and can rotate) a stable
  per-user offline wrapping secret, so the same favorites decrypt across the
  user's devices and the server retains a revocation/kill-switch lever.

## Capabilities

### New Capabilities
- `offline-score-cache`: App-side encrypted-at-rest cache of favorited scores —
  write-on-first-open, offline-preferred load, envelope encryption with
  OS-keystore-bound key material, eviction on un-favorite/delete/sign-out, and
  fail-closed behavior when no secure keystore is available.
- `backend-offline-key`: Backend operation that issues and rotates a per-user
  secret used as one input to the client's offline-cache key derivation,
  authenticated and owner-scoped.

### Modified Capabilities
- `backend-score-storage`: expose each score's content hash (ETag) on its
  metadata/bytes responses and support a conditional bytes fetch (skip returning
  bytes when the caller's known hash still matches) — enabling the client's
  freshness/integrity guard and refresh bandwidth saving.

## Impact

- **Flutter app** (`apps/music`):
  - New service seam(s): an offline-cache store (encrypted file I/O behind a
    provider) and a key-provider that derives/holds key material via
    `flutter_secure_storage` (already used by
    [token_store.dart](apps/music/lib/services/token_store.dart)).
  - [notation_notifier.dart](apps/music/lib/state/notation_notifier.dart): load
    path consults the cache before/after the network fetch.
  - Notifiers that own favorite state
    ([saved_catalog_scores.dart](apps/music/lib/state/saved_catalog_scores.dart),
    [contributed_scores.dart](apps/music/lib/state/contributed_scores.dart)) and
    session teardown ([session_notifier.dart](apps/music/lib/state/session_notifier.dart))
    trigger eviction.
  - New dependency: `path_provider` for the app's private cache directory;
    `crypto` (already present) plus an AES-GCM/HKDF primitive.
- **Backend** (`backend/music`): a new gRPC method on `ScoreService`
  ([score.proto](backend/music/proto/score.proto)) to fetch/rotate the per-user
  offline secret, plus storage for that secret.
- **Security/privacy**: new at-rest cryptographic material; threat model and key
  custody documented in `design.md`. No plaintext score bytes persisted.
- **Testing**: crypto round-trip, cache write/evict, offline-load, and
  fail-closed paths — Flutter (mockito seams) and Rust (mockall) — keeping
  coverage ≥ 80%.
