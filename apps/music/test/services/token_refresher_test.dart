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

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/token_refresher.dart';
import 'package:music/services/token_store.dart';

import '../support/auth_fakes.dart';

StoredTokens _stored({String refresh = 'r0'}) =>
    StoredTokens(accessToken: 'a0', refreshToken: refresh);

void main() {
  group('CoordinatedTokenRefresher single-flight', () {
    test('coalesces concurrent callers into exactly one Refresh RPC', () async {
      final store = FakeTokenStore(tokens: _stored());
      var calls = 0;
      final seenTokens = <String>[];
      final gate = Completer<void>();
      final refresher = CoordinatedTokenRefresher(
        tokenStore: store,
        refreshRpc: (rt) async {
          calls++;
          seenTokens.add(rt);
          await gate.future;
          return const AuthTokens(accessToken: 'a1', refreshToken: 'r1');
        },
      );

      // Three simultaneous expiries.
      final futures = [
        refresher.refresh(),
        refresher.refresh(),
        refresher.refresh(),
      ];
      gate.complete();
      final outcomes = await Future.wait(futures);

      expect(calls, 1, reason: 'the stored refresh token is sent only once');
      expect(seenTokens, ['r0']);
      for (final o in outcomes) {
        expect(o, isA<RefreshRefreshed>());
        expect((o as RefreshRefreshed).accessToken, 'a1');
      }
      // The rotated pair is persisted once.
      expect(store.tokens?.refreshToken, 'r1');
    });

    test(
      'a later expiry starts a fresh flight with the rotated token',
      () async {
        final store = FakeTokenStore(tokens: _stored());
        final sent = <String>[];
        final refresher = CoordinatedTokenRefresher(
          tokenStore: store,
          refreshRpc: (rt) async {
            sent.add(rt);
            return AuthTokens(
              accessToken: 'a-${sent.length}',
              refreshToken: 'r-${sent.length}',
            );
          },
        );

        await refresher.refresh();
        await refresher.refresh();

        expect(sent, [
          'r0',
          'r-1',
        ], reason: 'second flight uses the rotated token');
      },
    );
  });

  group('CoordinatedTokenRefresher classification', () {
    Future<RefreshOutcome> run(FakeTokenStore store, RefreshRpc rpc) =>
        CoordinatedTokenRefresher(tokenStore: store, refreshRpc: rpc).refresh();

    test(
      'success stores the rotated pair and returns the new access token',
      () async {
        final store = FakeTokenStore(tokens: _stored());
        final outcome = await run(
          store,
          (_) async => const AuthTokens(accessToken: 'a1', refreshToken: 'r1'),
        );
        expect(outcome, isA<RefreshRefreshed>());
        expect((outcome as RefreshRefreshed).accessToken, 'a1');
        expect(store.tokens?.accessToken, 'a1');
        expect(store.tokens?.refreshToken, 'r1');
      },
    );

    test('UNAUTHENTICATED is terminal: session cleared', () async {
      final store = FakeTokenStore(tokens: _stored());
      final outcome = await run(
        store,
        (_) async => throw const AuthException(AuthError.unauthenticated),
      );
      expect(outcome, isA<RefreshRejected>());
      expect(store.tokens, isNull);
    });

    test('INVALID_ARGUMENT is terminal: session cleared', () async {
      final store = FakeTokenStore(tokens: _stored());
      final outcome = await run(
        store,
        (_) async => throw const AuthException(AuthError.invalidArgument),
      );
      expect(outcome, isA<RefreshRejected>());
      expect(store.tokens, isNull);
    });

    test('UNAVAILABLE is transient: session kept intact', () async {
      final store = FakeTokenStore(tokens: _stored());
      final outcome = await run(
        store,
        (_) async => throw const AuthException(AuthError.unavailable),
      );
      expect(outcome, isA<RefreshTransient>());
      expect(store.tokens?.refreshToken, 'r0', reason: 'never cleared offline');
    });

    test('a raw GrpcError is classified by its status code', () async {
      final terminalStore = FakeTokenStore(tokens: _stored());
      expect(
        await run(
          terminalStore,
          (_) async => throw GrpcError.unauthenticated(),
        ),
        isA<RefreshRejected>(),
      );
      expect(terminalStore.tokens, isNull);

      final transientStore = FakeTokenStore(tokens: _stored());
      expect(
        await run(transientStore, (_) async => throw GrpcError.unavailable()),
        isA<RefreshTransient>(),
      );
      expect(transientStore.tokens, isNotNull);
    });

    test('an unexpected (non-gRPC) error is transient: session kept', () async {
      final store = FakeTokenStore(tokens: _stored());
      final outcome = await run(store, (_) async => throw Exception('socket'));
      expect(outcome, isA<RefreshTransient>());
      expect(store.tokens, isNotNull);
    });

    test('no stored session is rejected without an RPC', () async {
      final store = FakeTokenStore();
      var called = false;
      final outcome = await run(store, (_) async {
        called = true;
        return const AuthTokens(accessToken: 'x', refreshToken: 'y');
      });
      expect(outcome, isA<RefreshRejected>());
      expect(called, isFalse);
    });
  });
}
