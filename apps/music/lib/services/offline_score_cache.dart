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
import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart' as crypto;
import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../state/session_notifier.dart';
import 'offline_key_provider.dart';
import 'offline_server_secret_service.dart';

part 'offline_score_cache.g.dart';

/// Decrypted bytes of a cached score plus the server content hash (ETag) stored
/// alongside them (change: add-offline-score-cache).
class CachedScore {
  final Uint8List bytes;
  final String etag;

  const CachedScore(this.bytes, this.etag);
}

/// Encrypted-at-rest cache of favorited scores, keyed by a stable per-entry id
/// (`catalog:<id>` / `contributed:<id>`). Plaintext MusicXML never touches disk;
/// each file is envelope-encrypted (random per-file DEK, AES-256-GCM, DEK wrapped
/// by a keystore-bound KEK). Injectable so notifiers/widgets exercise it with an
/// in-memory fake, and so the crypto is unit-testable without real device storage.
abstract class OfflineScoreCache {
  /// Encrypt and persist [bytes] under [entryKey] with its server [etag]. A no-op
  /// when caching is unavailable (guest, no keystore, or no server secret).
  Future<void> write(String entryKey, Uint8List bytes, {required String etag});

  /// Decrypt and return the cached entry, or `null` on a miss (absent, tampered,
  /// integrity-hash mismatch, or caching unavailable). A corrupt file is deleted.
  Future<CachedScore?> read(String entryKey);

  /// Whether an encrypted file exists for [entryKey] — the cheap "playable
  /// offline" probe (no decryption, metadata only).
  Future<bool> has(String entryKey);

  /// Delete [entryKey]'s encrypted file (idempotent no-op if absent).
  Future<void> evict(String entryKey);

  /// Purge every cached file and clear the key material, making any residual file
  /// inert (sign-out / account deletion).
  Future<void> purgeAll();
}

/// File-backed production [OfflineScoreCache] with per-file envelope encryption.
class EncryptedFileOfflineScoreCache implements OfflineScoreCache {
  EncryptedFileOfflineScoreCache({
    required OfflineKeyProvider keys,
    required OfflineServerSecretService secrets,
    required Future<String?> Function() currentUserId,
    Future<Directory> Function()? cacheDir,
  }) : _keys = keys,
       _secrets = secrets,
       _currentUserId = currentUserId,
       _cacheDirOverride = cacheDir;

  final OfflineKeyProvider _keys;
  final OfflineServerSecretService _secrets;
  final Future<String?> Function() _currentUserId;
  final Future<Directory> Function()? _cacheDirOverride;

  static const _magic = 'CYE1';
  static final _aes = AesGcm.with256bits();

  Future<Directory> _dir() async {
    if (_cacheDirOverride != null) return _cacheDirOverride();
    final support = await getApplicationSupportDirectory();
    return Directory('${support.path}/offline_scores');
  }

  /// The `.enc` file for [entryKey]. The name is the SHA-256 of the entry key, so
  /// score ids don't leak into filenames and the name is always filesystem-safe.
  Future<File> _file(String entryKey) async {
    final dir = await _dir();
    final name = crypto.sha256.convert(utf8.encode(entryKey)).toString();
    return File('${dir.path}/$name.enc');
  }

  /// Resolve the KEK for the current user, or `null` when caching is unavailable
  /// (guest/signed-out, no usable keystore, or no server secret yet). This is the
  /// single fail-closed gate for both read and write.
  Future<SecretKey?> _kek() async {
    final userId = await _currentUserId();
    if (userId == null) return null; // guest / signed out → never cache
    final secret = await _secrets.secretFor(userId);
    if (secret == null) return null; // no server secret (first-ever offline)
    // deriveKek returns null when there is no usable keystore (fail-closed).
    return _keys.deriveKek(userId: userId, serverSecret: secret);
  }

  @override
  Future<void> write(
    String entryKey,
    Uint8List bytes, {
    required String etag,
  }) async {
    // Fail-closed: never write a decryptable-if-leaked file.
    final kek = await _kek();
    if (kek == null) return;

    // Per-file random DEK; wrap it under the KEK (AES-256-GCM).
    final dek = randomBytes(32);
    final wrapNonce = randomBytes(12);
    final wrapped = await _aes.encrypt(dek, secretKey: kek, nonce: wrapNonce);

    // Encrypt the payload under the DEK.
    final payloadNonce = randomBytes(12);
    final box = await _aes.encrypt(
      bytes,
      secretKey: SecretKey(dek),
      nonce: payloadNonce,
    );

    // Plaintext hash for the on-read integrity check (independent of the server's
    // ETag hashing convention).
    final plainSha = crypto.sha256.convert(bytes).bytes;
    final etagBytes = utf8.encode(etag);

    final out = BytesBuilder(copy: false)
      ..add(ascii.encode(_magic))
      ..addByte(1) // format version
      ..add(wrapNonce)
      ..add(wrapped.cipherText)
      ..add(wrapped.mac.bytes)
      ..add(payloadNonce)
      ..add(plainSha)
      ..add(_u16(etagBytes.length))
      ..add(etagBytes)
      ..add(box.cipherText)
      ..add(box.mac.bytes);

    final file = await _file(entryKey);
    await file.parent.create(recursive: true);
    // Write to a temp file then rename, so a crash can't leave a half-written file.
    final tmp = File('${file.path}.tmp');
    await tmp.writeAsBytes(out.toBytes(), flush: true);
    await tmp.rename(file.path);
  }

  @override
  Future<CachedScore?> read(String entryKey) async {
    final kek = await _kek();
    if (kek == null) return null;

    final file = await _file(entryKey);
    if (!await file.exists()) return null;

    try {
      final raw = await file.readAsBytes();
      var i = 0;
      List<int> take(int n) {
        final s = raw.sublist(i, i + n);
        i += n;
        return s;
      }

      if (ascii.decode(take(4)) != _magic) return _miss(file);
      take(1); // version
      final wrapNonce = take(12);
      final wrappedCipher = take(32);
      final wrappedMac = take(16);
      final payloadNonce = take(12);
      final plainSha = take(32);
      final etagLen = _readU16(take(2));
      final etag = utf8.decode(take(etagLen));
      final rest = raw.sublist(i);
      // The GCM tag is the trailing 16 bytes of the payload segment.
      final payloadCipher = rest.sublist(0, rest.length - 16);
      final payloadMac = rest.sublist(rest.length - 16);

      // Unwrap the DEK, then decrypt the payload (either step throws on tamper).
      final dek = await _aes.decrypt(
        SecretBox(wrappedCipher, nonce: wrapNonce, mac: Mac(wrappedMac)),
        secretKey: kek,
      );
      final plain = await _aes.decrypt(
        SecretBox(payloadCipher, nonce: payloadNonce, mac: Mac(payloadMac)),
        secretKey: SecretKey(dek),
      );

      // Integrity: recompute the plaintext hash; a mismatch is a corrupt/stale
      // entry → treat as a miss.
      final recomputed = crypto.sha256.convert(plain).bytes;
      if (!constantTimeEquals(recomputed, plainSha)) return _miss(file);

      return CachedScore(Uint8List.fromList(plain), etag);
    } catch (e) {
      // Auth-tag failure, truncation, or any parse error → cache miss.
      debugPrint(
        'offline cache read failed for $entryKey ($e); treating as miss.',
      );
      return _miss(file);
    }
  }

  Future<CachedScore?> _miss(File file) async {
    try {
      if (await file.exists()) await file.delete();
    } catch (_) {}
    return null;
  }

  @override
  Future<bool> has(String entryKey) async => (await _file(entryKey)).exists();

  @override
  Future<void> evict(String entryKey) async {
    try {
      final file = await _file(entryKey);
      if (await file.exists()) await file.delete();
    } catch (e) {
      debugPrint('offline cache evict failed for $entryKey ($e); ignoring.');
    }
  }

  @override
  Future<void> purgeAll() async {
    try {
      final dir = await _dir();
      if (await dir.exists()) await dir.delete(recursive: true);
    } catch (e) {
      debugPrint('offline cache purge failed ($e); ignoring.');
    }
    await _keys.clearKeyMaterial();
  }

  static Uint8List _u16(int v) =>
      Uint8List(2)..buffer.asByteData().setUint16(0, v);
  static int _readU16(List<int> b) =>
      ByteData.sublistView(Uint8List.fromList(b)).getUint16(0);
}

/// In-memory [OfflineScoreCache] for notifier/widget tests — no crypto, no disk.
class InMemoryOfflineScoreCache implements OfflineScoreCache {
  final Map<String, CachedScore> _items = {};

  /// Whether caching is "enabled" — set false to model a fail-closed install.
  bool enabled;

  InMemoryOfflineScoreCache({this.enabled = true});

  @override
  Future<void> write(
    String entryKey,
    Uint8List bytes, {
    required String etag,
  }) async {
    if (!enabled) return;
    _items[entryKey] = CachedScore(Uint8List.fromList(bytes), etag);
  }

  @override
  Future<CachedScore?> read(String entryKey) async =>
      enabled ? _items[entryKey] : null;

  @override
  Future<bool> has(String entryKey) async =>
      enabled && _items.containsKey(entryKey);

  @override
  Future<void> evict(String entryKey) async => _items.remove(entryKey);

  @override
  Future<void> purgeAll() async => _items.clear();
}

@Riverpod(keepAlive: true)
OfflineScoreCache offlineScoreCache(Ref ref) => EncryptedFileOfflineScoreCache(
  keys: ref.watch(offlineKeysProvider),
  secrets: ref.watch(offlineServerSecretServiceProvider),
  currentUserId: () async => ref.read(currentUserIdProvider),
);
