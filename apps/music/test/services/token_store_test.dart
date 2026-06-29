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

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/token_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  group('SecureTokenStore resilience (regression: Keychain -34018)', () {
    setUp(() {
      // Emulate a sandboxed macOS app without the Keychain entitlement: every
      // secure-storage call fails the way the crash report showed.
      messenger.setMockMethodCallHandler(channel, (call) async {
        throw PlatformException(
          code: '-34018',
          message: "A required entitlement isn't present.",
        );
      });
    });

    tearDown(() => messenger.setMockMethodCallHandler(channel, null));

    test('a failing Keychain never throws to the caller', () async {
      final store = SecureTokenStore();
      // continueAsGuest() drives setGuest(); previously this threw and crashed.
      await expectLater(store.setGuest(), completes);
      await expectLater(store.clear(), completes);
      await expectLater(
        store.writeTokens(
          const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        ),
        completes,
      );
      // Reads fall back to "no session".
      expect(await store.isGuest(), isFalse);
      expect(await store.readTokens(), isNull);
    });
  });
}
