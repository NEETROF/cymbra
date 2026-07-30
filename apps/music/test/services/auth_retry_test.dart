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
import 'package:grpc/grpc.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/token_refresher.dart';
import 'package:music/services/token_store.dart';

import '../support/auth_fakes.dart';

/// Marker error standing in for a gRPC `UNAUTHENTICATED` so the test needs no
/// real channel.
const _unauth = 'UNAUTHENTICATED';
bool _isUnauth(Object e) => e == _unauth;

StoredTokens _tokens({String access = 'at'}) =>
    StoredTokens(accessToken: access, refreshToken: 'rt');

Matcher _authErr(AuthError e) =>
    throwsA(isA<AuthException>().having((x) => x.error, 'error', e));

void main() {
  group('authedCall refresh/retry', () {
    test(
      'passes the access token through and does not refresh on success',
      () async {
        var refreshed = false;
        final seen = <String?>[];
        final result = await authedCall<int>(
          (bearer) async {
            seen.add(bearer);
            return 42;
          },
          accessToken: () async => 'access-1',
          refresh: () async {
            refreshed = true;
            return const RefreshRefreshed('access-2');
          },
          isUnauthenticated: _isUnauth,
        );

        expect(result, 42);
        expect(seen, ['access-1']);
        expect(refreshed, isFalse);
      },
    );

    test(
      'refreshes once and retries with the new token on UNAUTHENTICATED',
      () async {
        final seen = <String?>[];
        final result = await authedCall<String>(
          (bearer) async {
            seen.add(bearer);
            if (seen.length == 1) throw _unauth; // first attempt 401s
            return 'ok';
          },
          accessToken: () async => 'stale',
          refresh: () async => const RefreshRefreshed('fresh'),
          isUnauthenticated: _isUnauth,
        );

        expect(result, 'ok');
        expect(seen, [
          'stale',
          'fresh',
        ]); // retried once, with the refreshed token
      },
    );

    test(
      'rethrows the original UNAUTHENTICATED when refresh is rejected (terminal)',
      () async {
        var attempts = 0;
        await expectLater(
          authedCall<String>(
            (bearer) async {
              attempts++;
              throw _unauth;
            },
            accessToken: () async => 'stale',
            refresh: () async => const RefreshRejected(),
            isUnauthenticated: _isUnauth,
          ),
          throwsA(_unauth),
        );
        expect(attempts, 1); // no retry when the refresh token is rejected
      },
    );

    test(
      'throws a non-UNAUTHENTICATED (UNAVAILABLE) error on a transient refresh, '
      'so the caller keeps the session',
      () async {
        var attempts = 0;
        await expectLater(
          authedCall<String>(
            (bearer) async {
              attempts++;
              throw _unauth;
            },
            accessToken: () async => 'stale',
            refresh: () async => const RefreshTransient(),
            isUnauthenticated: _isUnauth,
          ),
          throwsA(
            isA<GrpcError>().having(
              (e) => e.code,
              'code',
              StatusCode.unavailable,
            ),
          ),
        );
        expect(attempts, 1); // no retry; surfaced as a transient failure
      },
    );

    test('rethrows a non-auth error without refreshing', () async {
      var refreshed = false;
      await expectLater(
        authedCall<String>(
          (bearer) async => throw 'boom',
          accessToken: () async => 'access-1',
          refresh: () async {
            refreshed = true;
            return const RefreshRefreshed('access-2');
          },
          isUnauthenticated: _isUnauth,
        ),
        throwsA('boom'),
      );
      expect(refreshed, isFalse);
    });
  });

  group('AuthedRunner', () {
    test('runs with the stored access token; no refresh on success', () async {
      final store = FakeTokenStore(tokens: _tokens(access: 'at'));
      final refresher = FakeTokenRefresher(const RefreshTransient());
      final runner = AuthedRunner(tokenStore: store, refresher: refresher);

      final seen = <String?>[];
      final result = await runner.call((bearer) async {
        seen.add(bearer);
        return 7;
      });

      expect(result, 7);
      expect(seen, ['at']);
      expect(refresher.calls, 0);
    });

    test(
      'refreshes once on UNAUTHENTICATED and retries with the new token',
      () async {
        final store = FakeTokenStore(tokens: _tokens(access: 'stale'));
        final refresher = FakeTokenRefresher(const RefreshRefreshed('fresh'));
        final runner = AuthedRunner(tokenStore: store, refresher: refresher);

        final seen = <String?>[];
        final result = await runner.call((bearer) async {
          seen.add(bearer);
          if (seen.length == 1) throw GrpcError.unauthenticated();
          return 'ok';
        });

        expect(result, 'ok');
        expect(seen, ['stale', 'fresh']);
        expect(refresher.calls, 1);
      },
    );

    test(
      'a rejected refresh surfaces as AuthException(unauthenticated)',
      () async {
        final runner = AuthedRunner(
          tokenStore: FakeTokenStore(tokens: _tokens()),
          refresher: FakeTokenRefresher(const RefreshRejected()),
        );
        await expectLater(
          runner.call((bearer) async => throw GrpcError.unauthenticated()),
          _authErr(AuthError.unauthenticated),
        );
      },
    );

    test(
      'a transient refresh surfaces as AuthException(unavailable)',
      () async {
        final runner = AuthedRunner(
          tokenStore: FakeTokenStore(tokens: _tokens()),
          refresher: FakeTokenRefresher(const RefreshTransient()),
        );
        await expectLater(
          runner.call((bearer) async => throw GrpcError.unauthenticated()),
          _authErr(AuthError.unavailable),
        );
      },
    );

    test('a non-401 GrpcError is mapped without refreshing', () async {
      final refresher = FakeTokenRefresher(const RefreshRefreshed('x'));
      final runner = AuthedRunner(
        tokenStore: FakeTokenStore(tokens: _tokens()),
        refresher: refresher,
      );
      await expectLater(
        runner.call((bearer) async => throw GrpcError.notFound()),
        _authErr(AuthError.notFound),
      );
      expect(refresher.calls, 0);
    });
  });

  group('authErrorFromCode mapping', () {
    test('maps the gRPC status codes the flows distinguish', () {
      expect(authErrorFromCode(3), AuthError.invalidArgument);
      expect(authErrorFromCode(5), AuthError.notFound);
      expect(authErrorFromCode(6), AuthError.alreadyExists);
      expect(authErrorFromCode(8), AuthError.rateLimited);
      expect(authErrorFromCode(9), AuthError.failedPrecondition);
      expect(authErrorFromCode(10), AuthError.conflict);
      expect(authErrorFromCode(14), AuthError.unavailable);
      expect(authErrorFromCode(16), AuthError.unauthenticated);
      expect(authErrorFromCode(99), AuthError.unknown);
    });
  });

  group('bearerOptions', () {
    test('attaches a Bearer header only for a non-empty token', () {
      expect(bearerOptions('abc').metadata['authorization'], 'Bearer abc');
      expect(
        bearerOptions(null).metadata.containsKey('authorization'),
        isFalse,
      );
      expect(bearerOptions('').metadata.containsKey('authorization'), isFalse);
    });
  });
}
