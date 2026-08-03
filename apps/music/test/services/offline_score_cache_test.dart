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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/offline_key_provider.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/offline_server_secret_service.dart';

/// Fixed-value server-secret stub (crypto is exercised, the network is not). A
/// `null` secret models "no server secret yet" (fail-closed).
class _FakeSecret implements OfflineServerSecretService {
  _FakeSecret(this.secret);
  final List<int>? secret;
  @override
  Future<Uint8List?> secretFor(String userId) async =>
      secret == null ? null : Uint8List.fromList(secret!);
  @override
  Future<void> clear(String userId) async {}
}

EncryptedFileOfflineScoreCache _cache(
  Directory dir,
  OfflineKeyProvider keys, {
  String? userId = 'u1',
  List<int>? serverSecret = const [7, 7, 7, 7],
}) => EncryptedFileOfflineScoreCache(
  keys: keys,
  secrets: _FakeSecret(serverSecret),
  currentUserId: () async => userId,
  cacheDir: () async => dir,
);

Uint8List _bytes(String s) => Uint8List.fromList(utf8.encode(s));

void main() {
  late Directory dir;

  setUp(() async {
    dir = await Directory.systemTemp.createTemp('offline_cache_test');
  });
  tearDown(() async {
    if (await dir.exists()) await dir.delete(recursive: true);
  });

  OfflineKeyProvider usable() =>
      HkdfOfflineKeyProvider(InMemorySecureBytesStore());

  test('round-trips write → read and never writes plaintext', () async {
    final c = _cache(dir, usable());
    final bytes = _bytes('<score><part/></score>');
    await c.write('catalog:x', bytes, etag: 'etag-1');

    expect(await c.has('catalog:x'), isTrue);
    final got = await c.read('catalog:x');
    expect(got, isNotNull);
    expect(got!.bytes, bytes);
    expect(got.etag, 'etag-1');

    // The on-disk file must not contain the plaintext MusicXML.
    final files = (await dir.list().toList()).whereType<File>().toList();
    expect(files, hasLength(1));
    final raw = await files.single.readAsBytes();
    expect(utf8.decode(raw, allowMalformed: true).contains('<score'), isFalse);
  });

  test(
    'tampered ciphertext is treated as a miss and the file is dropped',
    () async {
      final c = _cache(dir, usable());
      await c.write('catalog:x', _bytes('hello'), etag: 'e');
      final file = (await dir.list().toList()).whereType<File>().single;
      final raw = await file.readAsBytes();
      raw[raw.length - 1] ^= 0xFF; // flip a byte of the payload GCM tag
      await file.writeAsBytes(raw, flush: true);

      expect(await c.read('catalog:x'), isNull);
      expect(await file.exists(), isFalse); // corrupt entry deleted
    },
  );

  test(
    'integrity-hash mismatch (valid GCM, wrong plaintext hash) is a miss',
    () async {
      final c = _cache(dir, usable());
      await c.write('catalog:x', _bytes('hello'), etag: 'e');
      final file = (await dir.list().toList()).whereType<File>().single;
      final raw = await file.readAsBytes();
      // The stored plaintext SHA sits at offset 77 (4+1+12+32+16+12); corrupt it
      // without touching the still-valid payload ciphertext.
      raw[77] ^= 0xFF;
      await file.writeAsBytes(raw, flush: true);

      expect(await c.read('catalog:x'), isNull);
    },
  );

  test(
    'a file from another install (different seed) does not decrypt',
    () async {
      final a = _cache(dir, usable());
      await a.write('catalog:x', _bytes('secret score'), etag: 'e');
      // Same directory + file, but a different install's key material.
      final b = _cache(dir, usable());
      expect(await b.read('catalog:x'), isNull);
    },
  );

  test('no usable keystore fails closed: nothing is written', () async {
    final c = _cache(
      dir,
      HkdfOfflineKeyProvider(UnavailableSecureBytesStore()),
    );
    await c.write('catalog:x', _bytes('x'), etag: 'e');
    expect(await c.has('catalog:x'), isFalse);
    expect(await c.read('catalog:x'), isNull);
    expect(await dir.list().toList(), isEmpty);
  });

  test('guest (no user id) never writes bytes', () async {
    final c = _cache(dir, usable(), userId: null);
    await c.write('catalog:x', _bytes('x'), etag: 'e');
    expect(await c.has('catalog:x'), isFalse);
  });

  test('no server secret yet fails closed', () async {
    final c = _cache(dir, usable(), serverSecret: null);
    await c.write('catalog:x', _bytes('x'), etag: 'e');
    expect(await c.has('catalog:x'), isFalse);
  });

  test('evict removes one entry and leaves the others', () async {
    final c = _cache(dir, usable());
    await c.write('catalog:a', _bytes('A'), etag: 'e');
    await c.write('contributed:b', _bytes('B'), etag: 'e');
    await c.evict('catalog:a');
    expect(await c.has('catalog:a'), isFalse);
    expect(await c.has('contributed:b'), isTrue);
    // Evicting an absent entry is a no-op.
    await c.evict('catalog:a');
  });

  test('sweep deletes only the entries not in the keep-set', () async {
    final c = _cache(dir, usable());
    await c.write('catalog:a', _bytes('A'), etag: 'e');
    await c.write('catalog:b', _bytes('B'), etag: 'e');
    await c.write('contributed:c', _bytes('C'), etag: 'e');
    // Keep a and c; b is an orphan.
    await c.sweep({'catalog:a', 'contributed:c'});
    expect(await c.has('catalog:a'), isTrue);
    expect(await c.has('contributed:c'), isTrue);
    expect(await c.has('catalog:b'), isFalse);
  });

  test('purgeAll clears every entry', () async {
    final c = _cache(dir, usable());
    await c.write('catalog:a', _bytes('A'), etag: 'e');
    await c.write('contributed:b', _bytes('B'), etag: 'e');
    await c.purgeAll();
    expect(await c.has('catalog:a'), isFalse);
    expect(await c.has('contributed:b'), isFalse);
  });
}
