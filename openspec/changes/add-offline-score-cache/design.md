## Context

Scores are fetched on every open. The load path in
[notation_notifier.dart](apps/music/lib/state/notation_notifier.dart) picks one
of three byte sources by entry kind:

- `entry.contributedId != null` → `scoreUploadService.fetchBytes` (user upload),
- `entry.catalogId != null` → `catalogService.fetchBytes` (saved catalog score),
- otherwise → `scoreAssetSource.load` (bundled demo asset).

Favorites are the union of favorited uploads + saved catalog scores
([favorite_scores.dart](apps/music/lib/state/favorite_scores.dart)), gated on
`canUseOnlineServices` (authenticated only). Secure storage already exists behind
a seam: [token_store.dart](apps/music/lib/services/token_store.dart) uses
`flutter_secure_storage` (Keychain / Keystore / DPAPI / libsecret), and the
session exposes `currentUserIdProvider` (the user uuid).

Constraints:
- **True offline at launch**: decryption must succeed with **no network**, so all
  key inputs must be reconstructable on-device.
- **Leaked files must be useless**: a copied `.enc` file or a full device backup,
  on its own, must not decrypt.
- Bundled demo assets are already local and public — out of scope.
- Coverage ≥ 80% (Rust + Flutter); native crypto/keystore behind injectable seams
  so state/widgets test without platform channels (project testing rules).

## Goals / Non-Goals

**Goals:**
- Play a favorited-and-once-opened score with no connection at startup.
- At-rest encryption that renders leaked cache files / backups undecryptable
  off the originating install (bound to user uuid + per-install seed + server
  secret held only in the OS keystore).
- Automatic eviction when a score leaves favorites, an upload is deleted, or the
  session ends.
- One code path across mobile + desktop; hardware key-binding used where the OS
  provides it, best-effort otherwise; **fail-closed** where no keystore exists.

**Non-Goals:**
- Resisting a fully compromised device (rooted/jailbroken, or an attacker inside
  the victim's unlocked OS session). Offline decryption inherently requires the
  key on-device; we minimize but cannot eliminate this.
- Offline for **non-favorited** scores, bundled demo assets, or rating-deck
  previews.
- Background pre-fetch / bulk sync of all favorites (only write on real open).
- DRM against a determined user extracting their own decrypted score at runtime.
- Keying the cache on app build version (explicitly excluded — see Decisions).

## Decisions

### D1 — Envelope encryption (per-file random data key), AES-256-GCM

Each cached file is encrypted with a fresh random **Data Encryption Key (DEK)**
using AES-256-GCM (random 96-bit nonce, authenticated). The DEK is then
**wrapped** by a **Key Encryption Key (KEK)** and the wrapped DEK is stored in
the file header (or a sidecar), while the KEK material lives in the OS keystore.

Why: per-file DEKs mean nonce reuse is impossible across files, a single file
compromise doesn't expose others, and re-wrapping (key rotation) never requires
re-encrypting payloads. Plaintext MusicXML is never written to disk.

_Alternative rejected_: encrypt every file directly with one derived key — nonce
management becomes global and rotation forces full re-encryption.

### D2 — KEK derivation: HKDF over four inputs, NO app version

```
KEK = HKDF-SHA256(
        ikm  = deviceKey            // best-effort hardware-backed, from keystore
        salt = perInstallSeed        // 32 random bytes, generated once, keystore
        info = serverUserSecret || userUuid
      )
```

- **serverUserSecret**: fetched once online from the backend (D5), cached in the
  keystore. Enables cross-device decrypt of the same favorites and a server-side
  kill-switch (rotate → old cache undecryptable after the device refreshes).
- **userUuid** (`currentUserIdProvider`): binds ciphertext to the account.
- **perInstallSeed**: 32 random bytes generated on first cache use, stored in the
  keystore — makes each install's ciphertext unique; a file copied to another
  install/user won't decrypt.
- **deviceKey**: a keystore-held key. On iOS/Android/modern macOS it is
  hardware-backed and non-exportable; elsewhere it is an OS-user-bound software
  key. Present on all platforms via `flutter_secure_storage`.

**App build version is intentionally excluded** (user decision): including it
would invalidate the entire cache on every release. Rotation, when needed, is
driven by the server secret (D5), not the client version.

_Alternative rejected_: derive straight from the server secret without a
per-install seed or device key — then any two installs of the same user produce
identical ciphertext and a device-key compromise generalizes; the seed + device
key localize blast radius.

### D3 — Key custody per platform (via `flutter_secure_storage`), fail-closed

| Platform | Keystore backend | deviceKey binding |
|---|---|---|
| iOS | Keychain + Secure Enclave (`ThisDeviceOnly`) | hardware, non-exportable |
| Android | Keystore / StrongBox (`setUnlockedDeviceRequired`) | hardware (StrongBox if present) |
| macOS (Apple Silicon / T2) | Keychain + Secure Enclave | hardware |
| macOS (Intel, no T2) | login Keychain (`ThisDeviceOnly`) | software, OS-user-bound |
| Windows | DPAPI (`CurrentUser`); optional TPM later | OS-user-bound (hardware if TPM added) |
| Linux | Secret Service / libsecret (Keyring/KWallet) | session-bound; may be absent |

**Fail-closed (score bytes only)**: a `KeystoreProbe` checks at startup whether
secure key material can be written+read back. If not (e.g. headless Linux with no
Secret Service), the **encrypted byte cache** is **disabled** for that install:
scores load online-only and **no score bytes** are ever written to disk. This
preserves the invariant "never persist a *score-bytes* file whose key isn't in a
secure keystore." The favorites **index** (metadata only, no bytes) is explicitly
exempt and still persisted in plaintext (D8), so the favorites list survives
offline even with no keystore.

The keystore items reuse the existing best-effort discipline from
[token_store.dart](apps/music/lib/services/token_store.dart) (macOS legacy
keychain notes, swallow `PlatformException`, degrade rather than crash).

### D4 — Load path: cache-first when it exists, network as source of truth

In `Notation._load` (and only for **favorited** catalog/upload entries):

1. If a valid local `.enc` exists for the entry → decrypt → parse → render, then
   (best-effort, if online) refresh from the network and rewrite the cache.
2. Else fetch from the network (existing path). On success, if the entry is
   currently favorited **and** caching is enabled, write the `.enc` copy.
3. Offline with no cache and a non-bundled entry → existing typed
   `ScoreLoadFailure` (`unavailable` / `notAvailableYet`), surfaced as a
   localized message (never raw errors — project rule).

Cache key = a stable per-entry id (`contributed:<id>` / `catalog:<id>`). The
write happens **after** a successful fetch of a favorited entry — "opened once
while favorited" — matching the requirement exactly.

**Offline feedback**: the existing typed-failure path
([score_load_message.dart](apps/music/lib/screens/score_load_message.dart),
snackbar on failure while staying on the library — [open_score.dart](apps/music/lib/screens/open_score.dart))
is reused, with a **new `ScoreLoadFailure.offlineUnavailable` variant** and l10n
string so the offline-not-cached case reads honestly ("not available offline")
instead of the generic `playerScoreUnavailable`. The load path classifies to it
when the entry has no local copy and connectivity is offline (via
`connectivityService`), else keeps `unavailable` for an online-but-failing
backend. Proactively, the home uses the index's per-entry "bytes cached" flag to
mark non-playable favorites while offline, so the state is visible before a tap.

Per the architecture rules, the cache/keystore are **services** invoked only by
the notifier; widgets never touch them. Eviction reactions live in a dedicated
listener, not scattered in build methods.

### D5 — Backend: per-user offline secret (issue + rotate)

New authenticated `ScoreService` RPC (e.g. `GetOfflineCacheKey`) returns the
caller's stable per-user secret (32 bytes), creating it on first call and
returning the same value thereafter — so the same favorites decrypt on all the
user's devices. A rotation lever (admin/ops or account-deletion hook) changes the
secret; devices re-derive on next online open, and stale cache files silently
fail to decrypt and are re-fetched. Owner-scoped; unauthenticated rejected.
Stored server-side keyed by user uuid (new column/table in `backend/music`).

### D6 — Eviction triggers

- Remove saved catalog score / un-favorite upload / delete upload → delete that
  entry's `.enc` file (idempotent; missing file = no-op success).
- Sign-out, "sign out everywhere", account deletion → purge the whole cache
  directory (the KEK material is also cleared, making any leftover file inert).

Eviction is invoked from the notifiers that already own those transitions
(`saved_catalog_scores`, `contributed_scores`, `session_notifier`), keeping the
"provider never imperatively pokes a sibling" rule (they call the cache
**service**, not another provider).

### D7 — Score bytes are immutable under a stable id; ETag guard, no polling

The data model was audited to answer "can a cached score go stale?":

- `catalog_scores`: `id` is the PK, `sha256` (content hash) is `UNIQUE`, there is
  **no `updated_at`**, and per [0005_user_library.sql](backend/music/migrations/0005_user_library.sql)
  a crawler re-ingest **changes ids** (a new row + purge of the old, cascading to
  saves) rather than mutating bytes in place.
- `user_scores`: `Upload` + `Delete` only (no update/replace RPC), `UNIQUE
  (owner_id, sha256)`.

**Conclusion**: bytes are content-addressed and cannot drift under a fixed id, so
a cache keyed by `catalog:<id>` / `contributed:<id>` can never serve stale
*content*. **No change-detection or polling is warranted.** The only server-side
event is disappearance/replacement under a *new* id (re-ingest, delete,
moderation rejection), which is already handled by eviction (D6) and the orphan
sweep on favorites refresh.

Rather than build a detector for a case the model forbids, we reuse the existing
content hash as an **ETag**, for cheap robustness and efficiency:

- The bytes/metadata responses expose `sha256` (additive proto field). The client
  stores it beside each cache entry.
- **Read/integrity**: a decrypted file whose recomputed hash ≠ the stored ETag is
  treated as a miss (corruption / post-rotation) and re-fetched when online.
- **Conditional refresh**: the online-open refresh (D4) sends the cached hash;
  the backend returns "unchanged" (no bytes) when it still matches, so the common
  case skips the re-download **and** the re-encrypt. A mismatch (belt-and-braces,
  should not occur under a stable id) returns fresh bytes and rewrites the cache.

_Alternative rejected_: periodic background polling / a "last-modified" timestamp
— unnecessary given content-addressing, and it would add network chatter and a
mutable field the schema deliberately doesn't have.

### D8 — Offline favorites index snapshot (the home must render offline)

Caching bytes is not enough: the favorites list itself is backend-fetched
(`SavedCatalogScores._fetch` → `listSaved()`, `MyUploads.build` →
`listMyScores()`), so with no network at launch both providers land in
`AsyncError` and the home never shows a tile to tap — the encrypted bytes would
be unreachable. The session does stay authenticated offline (a transient
`getAccount()` failure keeps `SessionState.authenticated()`), so
`canUseOnlineServices` is true and the favorites path runs; it just can't reach
the backend.

Fix: persist a **last-known-good favorites index snapshot** and fall back to it
offline.

- **Write**: on every successful online favorites fetch, persist the resolved
  `CatalogEntry` list (id, kind, `catalogId`/`contributedId`, title, composer,
  level — **no bytes**), scoped to `currentUserId`.
- **Read/fallback**: when the online fetch fails (offline), the providers return
  the snapshot instead of erroring — stale-while-offline. When it succeeds, the
  snapshot is refreshed. Guest/signed-out still yields an empty list, and
  sign-out clears the snapshot (part of the D6 purge).
- **Playable flag**: each favorite is annotated with whether its encrypted bytes
  are present in the cache. Byte-cached favorites play offline; the rest are
  shown but resolve to the existing typed "unavailable offline"
  `ScoreLoadFailure` when opened without network — no separate error path.

**Sensitivity / storage**: the index holds only titles/composers/ids the user
saved, not licensed bytes — so it is **deliberately NOT subject to the byte
cache's encryption or fail-closed rule**. It is persisted in **plaintext** local
storage (a JSON document under app support / the existing `local-preferences`
store), keyed by `currentUserId`, and is **decoupled from the keystore**. This is
a direct requirement: a user must never lose their favorites list offline just
because the platform has no secure keystore. On a no-keystore install (or one
that loses keystore access), the index still renders offline; only the encrypted
*bytes* are unavailable there, so those favorites show but are not offline-
playable. The mild privacy trade-off (device-local titles/composers are readable
in plaintext) is accepted and matches how `local-preferences` already treats
non-secret data; licensed bytes remain encrypted and fail-closed regardless.

_Alternative rejected_: keep the favorites list online-only and cache bytes
only. Then offline-at-launch shows an empty/error home and the whole feature is
unusable exactly when it is needed most.

## Risks / Trade-offs

- **[Full device compromise decrypts the cache]** → Out of scope by definition
  (offline needs the key on-device). Mitigation: hardware-backed non-exportable
  deviceKey where available raises the bar (can't exfiltrate the key to bulk
  decrypt elsewhere); server-secret rotation revokes on reconnect.
- **[Keystore unavailable / flaky on Linux/older desktop]** → Fail-closed: no
  cache, online-only; app never crashes and never writes a weak file.
- **[Server secret rotation invalidates all offline caches]** → Accepted; it is
  the intended kill-switch. Normal operation never rotates, so caches persist
  across app updates and sessions.
- **[Cache never evicted if a delete happens while the app is offline]** →
  Eviction is local file I/O (no network needed) and also reconciled on next
  favorites refresh; a favorites list that no longer contains an entry triggers a
  sweep of orphaned `.enc` files.
- **[Corrupted / undecryptable cache file]** (partial write, post-rotation) →
  Treated as a cache miss: delete the file and fall back to the network path.
- **[Disk growth]** → Bounded by favorites count (typically small); orphan sweep
  on favorites refresh + eviction on un-favorite keep it in check. A size cap /
  LRU can be a follow-up if needed.

## Migration Plan

- Additive: no schema change to existing score storage; one new backend column/
  table for the per-user secret and one new RPC (backward-compatible proto
  addition — regenerate the bridge/gRPC stubs).
- Ship dark-safe: if the backend RPC is unavailable, the client simply doesn't
  cache (fail-closed) — no user-visible regression, online play unchanged.
- Rollback: stop writing/reading the cache (feature flag or revert); existing
  `.enc` files are inert and can be purged; no plaintext ever persisted, so no
  cleanup risk.

## Open Questions

- Should the offline cache honor a **size cap / LRU** eviction in v1, or defer
  until favorites counts prove it necessary? (Leaning defer.)
- Where does **server-secret rotation** get triggered operationally — automatic
  on `signOutEverywhere` / password reset, admin-only, or both? (Spec leaves the
  mechanism to `backend-offline-key`; default: rotate on account deletion only.)
- Do we expose a user-visible "offline available ✓" indicator per favorite, or
  keep it silent (cache-on-open, no UI)? (Proposal keeps it silent; UI is a
  possible follow-up.)
