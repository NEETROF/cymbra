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

// A guest who signs in must stop being a guest (change:
// open-app-without-sign-in-wall). The app now keeps the guest choice until
// authentication succeeds instead of clearing it first, so the real secure store
// — not a fake — is what has to replace it.

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/token_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final storage = <String, String>{};

  setUp(() {
    storage.clear();
    // An in-memory Keychain/Keystore behind the plugin channel.
    messenger.setMockMethodCallHandler(channel, (call) async {
      final args =
          (call.arguments as Map?)?.cast<String, Object?>() ?? const {};
      final key = args['key'] as String?;
      switch (call.method) {
        case 'write':
          storage[key!] = args['value']! as String;
          return null;
        case 'read':
          return storage[key];
        case 'delete':
          storage.remove(key);
          return null;
        case 'deleteAll':
          storage.clear();
          return null;
        case 'containsKey':
          return storage.containsKey(key);
        case 'readAll':
          return Map<String, String>.of(storage);
      }
      return null;
    });
  });

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('a real session written over a guest choice replaces it', () async {
    final store = SecureTokenStore();
    await store.setGuest();
    expect(await store.isGuest(), isTrue);

    await store.writeTokens(
      const StoredTokens(accessToken: 'a', refreshToken: 'r'),
    );

    expect(await store.isGuest(), isFalse);
    expect(await store.readTokens(), isNotNull);
  });
}
