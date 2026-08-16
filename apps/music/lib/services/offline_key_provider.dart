// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:convert';
import 'dart:math';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'offline_key_provider.g.dart';

/// `n` cryptographically secure random bytes (offline-cache key material + nonces).
Uint8List randomBytes(int n) {
  final rng = Random.secure();
  final out = Uint8List(n);
  for (var i = 0; i < n; i++) {
    out[i] = rng.nextInt(256);
  }
  return out;
}

/// Constant-time byte-equality (avoids leaking match position via timing).
bool constantTimeEquals(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  var diff = 0;
  for (var i = 0; i < a.length; i++) {
    diff |= a[i] ^ b[i];
  }
  return diff == 0;
}

/// Seam over the OS secure keystore for the offline cache's raw key material
/// (change: add-offline-score-cache). Bytes are stored base64-wrapped under
/// `flutter_secure_storage` (Keychain / Keystore / DPAPI / libsecret). Kept
/// abstract so the key provider unit-tests over an in-memory store without a
/// platform channel.
abstract class SecureBytesStore {
  /// The bytes stored under [key], or `null` when absent or unreadable.
  Future<Uint8List?> read(String key);

  /// Persist [value] under [key] (best-effort; a keystore failure is swallowed).
  Future<void> write(String key, List<int> value);

  /// Remove any value under [key] (idempotent).
  Future<void> delete(String key);
}

/// Production [SecureBytesStore] over `flutter_secure_storage`, mirroring the
/// best-effort discipline of `SecureTokenStore`: a flaky keystore never crashes
/// the app — reads fall back to "absent", writes are swallowed. The fail-closed
/// signal comes from the write→read-back canary in [HkdfOfflineKeyProvider], not
/// from throwing here.
class SecureStorageBytesStore implements SecureBytesStore {
  SecureStorageBytesStore([FlutterSecureStorage? storage])
    // Defaults on every platform, including macOS, where that means the
    // data-protection keychain — same rationale as SecureTokenStore, which this
    // store used to mirror. Forcing `usesDataProtectionKeychain: false` falls
    // back to the legacy *login* keychain, whose items are guarded by an ACL
    // bound to the binary's designated requirement, so any signing-identity
    // change (development → Apple Distribution, i.e. every TestFlight or App
    // Store build) makes macOS prompt for the login keychain password. The app
    // ships `keychain-access-groups`, so the default is app-scoped: no ACL, no
    // prompt.
    : _storage = storage ?? const FlutterSecureStorage();

  final FlutterSecureStorage _storage;

  @override
  Future<Uint8List?> read(String key) async {
    try {
      final s = await _storage.read(key: key);
      return s == null ? null : Uint8List.fromList(base64Decode(s));
    } catch (e) {
      debugPrint('offline keystore read failed ($e); treating as empty.');
      return null;
    }
  }

  @override
  Future<void> write(String key, List<int> value) async {
    try {
      await _storage.write(key: key, value: base64Encode(value));
    } catch (e) {
      debugPrint('offline keystore write failed ($e); continuing.');
    }
  }

  @override
  Future<void> delete(String key) async {
    try {
      await _storage.delete(key: key);
    } catch (e) {
      debugPrint('offline keystore delete failed ($e); continuing.');
    }
  }
}

/// In-memory [SecureBytesStore] for tests — a usable "keystore".
class InMemorySecureBytesStore implements SecureBytesStore {
  final Map<String, Uint8List> _items = {};

  @override
  Future<Uint8List?> read(String key) async => _items[key];

  @override
  Future<void> write(String key, List<int> value) async =>
      _items[key] = Uint8List.fromList(value);

  @override
  Future<void> delete(String key) async => _items.remove(key);
}

/// A [SecureBytesStore] that never persists (writes vanish, reads are null) —
/// models a platform with **no usable secure keystore** (fail-closed tests).
class UnavailableSecureBytesStore implements SecureBytesStore {
  @override
  Future<Uint8List?> read(String key) async => null;
  @override
  Future<void> write(String key, List<int> value) async {}
  @override
  Future<void> delete(String key) async {}
}

@Riverpod(keepAlive: true)
SecureBytesStore secureBytesStore(Ref ref) => SecureStorageBytesStore();

/// Derives the offline cache's Key Encryption Key (KEK) and reports whether a
/// usable secure keystore is present (change: add-offline-score-cache, design
/// D2/D3). Injectable so the cache store and its tests swap in a fake.
abstract class OfflineKeyProvider {
  /// Whether secure key material can be written to **and read back** from the OS
  /// keystore — the fail-closed signal. `false` disables the byte cache.
  Future<bool> hasUsableKeystore();

  /// The KEK for [userId], bound to a keystore device key, a per-install seed,
  /// the [serverSecret], and the user id via HKDF-SHA256. `null` when no usable
  /// keystore is available (fail-closed). The app **build version is deliberately
  /// not an input**, so the cache survives updates.
  Future<SecretKey?> deriveKek({
    required String userId,
    required List<int> serverSecret,
  });

  /// Drop the install-scoped key material (device key + seed + canary) so any
  /// residual ciphertext becomes permanently undecryptable (sign-out / purge).
  Future<void> clearKeyMaterial();
}

/// Production [OfflineKeyProvider]: HKDF-SHA256 over four inputs, with the device
/// key and per-install seed generated once and held in the OS keystore.
class HkdfOfflineKeyProvider implements OfflineKeyProvider {
  HkdfOfflineKeyProvider(this._store);

  final SecureBytesStore _store;

  static const _kDeviceKey = 'cymbra.offline.device_key';
  static const _kSeed = 'cymbra.offline.seed';
  static const _kCanary = 'cymbra.offline.canary';

  static final _hkdf = Hkdf(hmac: Hmac.sha256(), outputLength: 32);

  @override
  Future<bool> hasUsableKeystore() async {
    final canary = randomBytes(16);
    await _store.write(_kCanary, canary);
    final back = await _store.read(_kCanary);
    // A store that can't persist (no keystore) reads back null / a mismatch.
    return back != null && constantTimeEquals(back, canary);
  }

  /// Read the 32-byte value under [key], generating + persisting it on first use.
  /// Returns `null` when the store can't persist it (fail-closed).
  Future<Uint8List?> _getOrCreate(String key) async {
    final existing = await _store.read(key);
    if (existing != null && existing.length == 32) return existing;
    final fresh = randomBytes(32);
    await _store.write(key, fresh);
    final back = await _store.read(key);
    if (back == null || !constantTimeEquals(back, fresh)) return null;
    return back;
  }

  @override
  Future<SecretKey?> deriveKek({
    required String userId,
    required List<int> serverSecret,
  }) async {
    final ikm = await _getOrCreate(_kDeviceKey);
    final seed = await _getOrCreate(_kSeed);
    if (ikm == null || seed == null) return null; // no usable keystore
    // info binds the account: server secret then the user uuid. NO build version.
    final info = <int>[...serverSecret, ...utf8.encode(userId)];
    return _hkdf.deriveKey(secretKey: SecretKey(ikm), nonce: seed, info: info);
  }

  @override
  Future<void> clearKeyMaterial() async {
    await _store.delete(_kDeviceKey);
    await _store.delete(_kSeed);
    await _store.delete(_kCanary);
  }
}

@Riverpod(keepAlive: true)
OfflineKeyProvider offlineKeys(Ref ref) =>
    HkdfOfflineKeyProvider(ref.watch(secureBytesStoreProvider));
