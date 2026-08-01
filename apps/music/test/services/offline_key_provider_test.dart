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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/offline_key_provider.dart';

Future<List<int>> _kek(
  OfflineKeyProvider p, {
  required String userId,
  required List<int> secret,
}) async {
  final key = await p.deriveKek(userId: userId, serverSecret: secret);
  return (await key!.extractBytes());
}

void main() {
  test('KEK derivation is deterministic for fixed inputs + install', () async {
    final p = HkdfOfflineKeyProvider(InMemorySecureBytesStore());
    final a = await _kek(p, userId: 'u1', secret: const [1, 2, 3]);
    final b = await _kek(p, userId: 'u1', secret: const [1, 2, 3]);
    expect(a.length, 32);
    expect(a, b); // same device key + seed + inputs → identical KEK
  });

  test('the per-install seed makes two installs derive different KEKs', () async {
    final p1 = HkdfOfflineKeyProvider(InMemorySecureBytesStore());
    final p2 = HkdfOfflineKeyProvider(InMemorySecureBytesStore());
    final a = await _kek(p1, userId: 'u1', secret: const [1, 2, 3]);
    final b = await _kek(p2, userId: 'u1', secret: const [1, 2, 3]);
    expect(a, isNot(b)); // different install seed / device key
  });

  test('KEK changes when the user or the server secret changes', () async {
    final p = HkdfOfflineKeyProvider(InMemorySecureBytesStore());
    final base = await _kek(p, userId: 'u1', secret: const [1, 2, 3]);
    final diffUser = await _kek(p, userId: 'u2', secret: const [1, 2, 3]);
    final diffSecret = await _kek(p, userId: 'u1', secret: const [9, 9, 9]);
    expect(diffUser, isNot(base));
    expect(diffSecret, isNot(base));
  });

  test('keystore probe reports usable vs unavailable', () async {
    expect(
      await HkdfOfflineKeyProvider(InMemorySecureBytesStore()).hasUsableKeystore(),
      isTrue,
    );
    expect(
      await HkdfOfflineKeyProvider(UnavailableSecureBytesStore())
          .hasUsableKeystore(),
      isFalse,
    );
  });

  test('no usable keystore fails closed (deriveKek returns null)', () async {
    final p = HkdfOfflineKeyProvider(UnavailableSecureBytesStore());
    expect(
      await p.deriveKek(userId: 'u1', serverSecret: const [1, 2, 3]),
      isNull,
    );
  });

  test('clearing key material forces a fresh device key/seed', () async {
    final p = HkdfOfflineKeyProvider(InMemorySecureBytesStore());
    final before = await _kek(p, userId: 'u1', secret: const [1, 2, 3]);
    await p.clearKeyMaterial();
    final after = await _kek(p, userId: 'u1', secret: const [1, 2, 3]);
    expect(after, isNot(before)); // residual ciphertext becomes undecryptable
  });
}
