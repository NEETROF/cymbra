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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/soundfont_preview_service.dart';
import 'package:music/services/token_store.dart';

import '../support/auth_fakes.dart';
import '../support/soundfont_fakes.dart';

/// A provider that builds the real impl with an injected mock http client + fake
/// clip player, resolving the endpoint/token through the container's overrides.
Provider<SoundFontPreviewService> _svc(
  http.Client client,
  FakeSoundClipPlayer player,
) => Provider(
  (ref) => SoundFontPreviewServiceImpl(ref, player, client: client),
);

ProviderContainer _container() {
  final c = ProviderContainer(
    overrides: [
      cymbraEndpointProvider.overrideWithValue(
        const CymbraEndpoint(host: 'localhost', port: 50051, secure: false),
      ),
      tokenStoreProvider.overrideWithValue(
        FakeTokenStore()
          ..tokens = const StoredTokens(
            accessToken: 'tok',
            refreshToken: 'r',
          ),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('audition plays the clip and returns true on a 200', () async {
    final player = FakeSoundClipPlayer();
    final client = MockClient((req) async {
      // Hits the dev HTTP preview route with the bearer token.
      expect(
        req.url.toString(),
        'http://localhost:8081/soundfonts/grand/preview',
      );
      expect(req.headers['authorization'], 'Bearer tok');
      return http.Response.bytes(Uint8List.fromList([1, 2, 3]), 200);
    });
    final played = await _container().read(_svc(client, player)).audition(
      'grand',
    );
    expect(played, isTrue);
    expect(player.played.single, [1, 2, 3]);
  });

  test('audition returns false and plays nothing when no preview (404)', () async {
    final player = FakeSoundClipPlayer();
    final client = MockClient((_) async => http.Response('', 404));
    final played = await _container().read(_svc(client, player)).audition(
      'grand',
    );
    expect(played, isFalse);
    expect(player.played, isEmpty);
  });

  test('audition throws on any other non-200 status', () async {
    final player = FakeSoundClipPlayer();
    final client = MockClient((_) async => http.Response('boom', 500));
    await expectLater(
      _container().read(_svc(client, player)).audition('grand'),
      throwsA(isA<SoundFontPreviewException>()),
    );
    expect(player.played, isEmpty);
  });

  test('stop delegates to the clip player', () async {
    final player = FakeSoundClipPlayer();
    final client = MockClient((_) async => http.Response('', 200));
    await _container().read(_svc(client, player)).stop();
    expect(player.stopCalls, 1);
  });
}
