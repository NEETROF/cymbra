## ADDED Requirements

### Requirement: Encrypted-at-rest offline cache of favorited scores

The app SHALL persist an **encrypted** local copy of a score's MusicXML bytes
when an authenticated user opens a score that is currently in their favorites (a
saved catalog score or one of their own favorited uploads) and its bytes are
fetched successfully. The plaintext MusicXML MUST NOT be written to disk in any
form. Bundled demo assets and non-favorited scores MUST NOT be cached. When the
session is not authenticated (guest / signed out), nothing is cached.

#### Scenario: Favorited score is cached on successful open

- **WHEN** an authenticated user opens a favorited catalog or upload score and
  its bytes are fetched successfully
- **THEN** an encrypted copy of the bytes is written to the app's private storage
  and no plaintext copy is written anywhere

#### Scenario: Non-favorited score is not cached

- **WHEN** a score that is not in the user's favorites is opened
- **THEN** no local encrypted copy is written

#### Scenario: Bundled demo asset is not cached

- **WHEN** a bundled demo score (loaded from the asset bundle) is opened
- **THEN** no local encrypted copy is written

#### Scenario: Guest / signed-out never caches

- **WHEN** the session is a guest or signed out
- **THEN** no score bytes are written to local storage

### Requirement: Offline-preferred playback of cached favorites

When a favorited score with a valid local encrypted copy is opened, the app SHALL
be able to decrypt and play it **without any network access**, including
immediately at app launch with no connection. When online, the network fetch
remains the source of truth: after serving from cache the app MAY refresh from
the network and rewrite the cache.

#### Scenario: Plays from cache with no connection at launch

- **WHEN** the app launches with no network and the user opens a favorited score
  that was previously opened while favorited
- **THEN** the score decrypts from the local encrypted copy and plays, without a
  network request

#### Scenario: Online open refreshes the cache only when content changed

- **WHEN** a favorited score with a local copy is opened while online
- **THEN** the score plays from cache and the app checks the server content hash;
  it rewrites the local copy only if the hash differs, and otherwise leaves the
  cache untouched (no re-download, no re-encrypt)

#### Scenario: Uncached favorite offline surfaces a typed failure

- **WHEN** the app is offline and a favorited score with no local copy is opened
- **THEN** a typed, localized "unavailable offline" message is shown and no raw
  technical error is surfaced

### Requirement: Envelope encryption bound to user, install, and server secret

The cache SHALL use per-file envelope encryption: each file is encrypted with a
fresh random data key using an authenticated cipher (AES-256-GCM), and that data
key is wrapped by a key derived from four inputs — a device key held in the OS
secure keystore, a server-issued per-user secret, the user's uuid, and a
per-install random seed. The app build version MUST NOT be an input to key
derivation, so the cache survives app updates. A leaked cache file or a full
device backup MUST NOT be decryptable on another install, another user, or beside
the file alone.

#### Scenario: Each file uses a unique data key

- **WHEN** two different favorited scores are cached
- **THEN** each file is encrypted under its own randomly generated data key

#### Scenario: File copied to another install does not decrypt

- **WHEN** an encrypted cache file is copied to a different app install or user
- **THEN** it cannot be decrypted (the per-install seed, user uuid, and keystore
  material do not match)

#### Scenario: Cache survives an app update

- **WHEN** the app is updated to a new build version
- **THEN** previously cached favorites still decrypt and play (build version is
  not part of the key)

#### Scenario: Tampered ciphertext is rejected

- **WHEN** a cache file's bytes are modified
- **THEN** authenticated decryption fails and the entry is treated as a cache
  miss (re-fetched when online)

### Requirement: Content-hash freshness and integrity guard

The app SHALL store the score's server content hash (ETag) alongside each cache
entry and use it as the freshness and integrity check — there is no time-based
polling. On an online open the app SHALL compare the cached hash with the
server's current hash and rewrite the entry only on a mismatch. When a decrypted
file's recomputed hash does not match its stored hash (corruption or a
post-rotation stale file), the app MUST treat the entry as a cache miss and
re-fetch when online.

#### Scenario: Matching hash skips re-download

- **WHEN** a cached favorite is opened online and the server content hash matches
  the stored hash
- **THEN** the cached bytes are used and no bytes are re-downloaded or re-encrypted

#### Scenario: Changed hash refreshes the entry

- **WHEN** the server content hash for a cached entry differs from the stored hash
- **THEN** the app fetches the new bytes and rewrites the encrypted copy

#### Scenario: Corrupted file is treated as a miss

- **WHEN** a decrypted file's recomputed hash does not match its stored hash
- **THEN** the entry is treated as a cache miss and re-fetched when online

### Requirement: Fail-closed when no secure keystore is available

The app SHALL verify at startup that secure key material can be written to and
read back from the OS keystore. If no usable secure keystore is available on the
platform, the offline cache MUST be disabled for that install: scores load
online-only and no bytes are ever written to disk. A keystore failure MUST NOT
crash the app or degrade online playback.

#### Scenario: No keystore disables caching

- **WHEN** the platform exposes no usable secure keystore (e.g. headless Linux
  with no Secret Service)
- **THEN** no score bytes are written to disk and scores load online-only

#### Scenario: Keystore failure never crashes

- **WHEN** a keystore read or write fails unexpectedly
- **THEN** the app continues with online playback and does not crash

### Requirement: Eviction on un-favorite, delete, and sign-out

The app SHALL delete an entry's local encrypted file when that score leaves the
user's favorites — a saved catalog score is removed, an upload is un-favorited,
or an upload is deleted. Deleting a file that is absent MUST be an idempotent
no-op success. On sign-out, "sign out everywhere", or account deletion, the app
SHALL purge the entire offline cache and clear its key material so any residual
file is inert.

#### Scenario: Removing a saved catalog score deletes its cache

- **WHEN** the user removes a saved catalog score from their library
- **THEN** that score's local encrypted file is deleted

#### Scenario: Un-favoriting or deleting an upload deletes its cache

- **WHEN** the user un-favorites or deletes one of their uploads
- **THEN** that upload's local encrypted file is deleted

#### Scenario: Deleting an absent cache file is a no-op

- **WHEN** eviction runs for an entry that has no local file
- **THEN** the operation succeeds and nothing changes

#### Scenario: Sign-out purges the whole cache

- **WHEN** the user signs out, signs out everywhere, or deletes their account
- **THEN** the entire offline cache directory is purged and its key material is
  cleared

#### Scenario: Orphaned files are swept on favorites refresh

- **WHEN** the favorites list is refreshed and a cached entry is no longer a
  favorite (e.g. removed on another device)
- **THEN** that orphaned local encrypted file is deleted

### Requirement: Injectable cache and key-provider seams

The offline cache store and the key provider SHALL be exposed as injectable
Riverpod providers behind abstract seams, with production implementations backed
by on-device encrypted file I/O and the OS keystore, and fakes for tests — so
notifiers and widgets exercise the cache paths without touching real device
storage or platform channels. UI MUST NOT call the cache or keystore directly;
only notifiers do.

#### Scenario: Tests override the cache with a fake

- **WHEN** a test overrides the cache and key-provider providers with in-memory
  fakes
- **THEN** offline write / load / evict paths run without native storage or a
  live backend

#### Scenario: UI does not touch the cache directly

- **WHEN** a widget needs a score to play
- **THEN** it calls a notifier, and only the notifier invokes the cache/keystore
  services
