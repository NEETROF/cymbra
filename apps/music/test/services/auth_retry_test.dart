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
import 'package:music/services/auth_service.dart';
import 'package:music/services/grpc_client.dart';

/// Marker error standing in for a gRPC `UNAUTHENTICATED` so the test needs no
/// real channel.
const _unauth = 'UNAUTHENTICATED';
bool _isUnauth(Object e) => e == _unauth;

void main() {
  group('authedCall refresh/retry (task 3.4)', () {
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
          refreshAccessToken: () async {
            refreshed = true;
            return 'access-2';
          },
          onExpired: () {},
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
          refreshAccessToken: () async => 'fresh',
          onExpired: () {},
          isUnauthenticated: _isUnauth,
        );

        expect(result, 'ok');
        expect(seen, [
          'stale',
          'fresh',
        ]); // retried once, with the refreshed token
      },
    );

    test('clears the session and rethrows when refresh gives up', () async {
      var expired = false;
      var attempts = 0;
      await expectLater(
        authedCall<String>(
          (bearer) async {
            attempts++;
            throw _unauth;
          },
          accessToken: () async => 'stale',
          refreshAccessToken: () async => null, // refresh token no longer valid
          onExpired: () => expired = true,
          isUnauthenticated: _isUnauth,
        ),
        throwsA(_unauth),
      );
      expect(expired, isTrue);
      expect(attempts, 1); // no retry when refresh fails
    });

    test('rethrows a non-auth error without refreshing', () async {
      var refreshed = false;
      await expectLater(
        authedCall<String>(
          (bearer) async => throw 'boom',
          accessToken: () async => 'access-1',
          refreshAccessToken: () async {
            refreshed = true;
            return 'access-2';
          },
          onExpired: () {},
          isUnauthenticated: _isUnauth,
        ),
        throwsA('boom'),
      );
      expect(refreshed, isFalse);
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
