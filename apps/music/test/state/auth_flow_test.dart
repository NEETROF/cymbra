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
import 'package:music/state/auth_flow.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';

void main() {
  group('AuthFlow OIDC sign-in (tasks 6.1/6.2/6.5)', () {
    test(
      'Google success exchanges the id_token and adopts the session',
      () async {
        final auth = FakeAuthService();
        final oidc = FakeOidcTokenSource(googleToken: 'g-token');
        final c = authContainer(
          auth: auth,
          account: FakeAccountService(account: fakeAccount(handle: 'a')),
          oidc: oidc,
        );
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        final ok = await c.read(authFlowProvider).signInWithGoogle();
        expect(ok, isTrue);
        expect(oidc.calls, ['google']);
        expect(auth.calls, contains('signInOidc:g-token'));
        expect(c.read(sessionNotifierProvider), isA<SessionAuthenticated>());
      },
    );

    test('Google cancellation is a no-op (no RPC, stays signed out)', () async {
      final auth = FakeAuthService();
      final oidc = FakeOidcTokenSource(googleToken: null); // user dismissed
      final c = authContainer(auth: auth, oidc: oidc);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      final ok = await c.read(authFlowProvider).signInWithGoogle();
      expect(ok, isFalse);
      expect(auth.calls.where((s) => s.startsWith('signInOidc')), isEmpty);
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });

    test('Apple cancellation is a no-op', () async {
      final auth = FakeAuthService();
      final oidc = FakeOidcTokenSource(appleToken: null);
      final c = authContainer(auth: auth, oidc: oidc);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      expect(await c.read(authFlowProvider).signInWithApple(), isFalse);
      expect(auth.calls.where((s) => s.startsWith('signInOidc')), isEmpty);
    });
  });

  group('AuthFlow email sign-in', () {
    test('signInEmail adopts the session', () async {
      final auth = FakeAuthService();
      final c = authContainer(
        auth: auth,
        account: FakeAccountService(account: fakeAccount(handle: 'a')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c
          .read(authFlowProvider)
          .signInEmail(email: 'a@x.dev', password: 'longenoughpassword');
      expect(auth.calls, contains('signInLocal:a@x.dev'));
      expect(c.read(sessionNotifierProvider), isA<SessionAuthenticated>());
    });

    test('a wrong-credential sign-in surfaces as an AuthException', () async {
      final auth = FakeAuthService()
        ..signInError = const AuthException(AuthError.unauthenticated);
      final c = authContainer(auth: auth);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      expect(
        () => c
            .read(authFlowProvider)
            .signInEmail(email: 'a@x.dev', password: 'wrongpassword!!'),
        throwsA(isA<AuthException>()),
      );
    });
  });
}
