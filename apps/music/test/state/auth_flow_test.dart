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

import 'package:music/state/pending_social_link.dart';

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

  group('AuthFlow collision "sign in to link" (D7)', () {
    test(
      'a social sign-in onto a brand-new account records a pending link',
      () async {
        final auth = FakeAuthService();
        final account = FakeAccountService(account: fakeAccount()); // no handle
        final c = authContainer(auth: auth, account: account);
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        await c.read(authFlowProvider).signInWithGoogle();

        final pending = c.read(pendingSocialLinkControllerProvider);
        expect(pending, isNotNull);
        expect(pending!.provider, 'google');
        expect(pending.idToken, 'google-id-token');
      },
    );

    test(
      'linking onto an existing email account: delete orphan → sign in → link',
      () async {
        final auth = FakeAuthService();
        // The orphan (no handle) is what we start on; after its deletion the
        // account service resolves the existing (handled) account.
        final account = FakeAccountService(account: fakeAccount())
          ..postDeleteAccount = fakeAccount(handle: 'existing');
        final c = authContainer(auth: auth, account: account);
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        await c.read(authFlowProvider).signInWithGoogle(); // create orphan
        final pending = c.read(pendingSocialLinkControllerProvider)!;

        await c
            .read(authFlowProvider)
            .linkExisting(
              pending: pending,
              authenticateExisting: () => c
                  .read(authFlowProvider)
                  .reauthEmail(email: 'me@x.dev', password: 'longenoughpass'),
            );

        // Ordering: the orphan is deleted BEFORE the identity is linked.
        final deleteAt = account.calls.indexOf('deleteAccount');
        final linkAt = auth.calls.indexOf('linkIdentity:google-id-token');
        expect(deleteAt, greaterThanOrEqualTo(0));
        expect(linkAt, greaterThanOrEqualTo(0));
        expect(auth.calls, contains('signInLocal:me@x.dev'));
        // Lands on the existing account (has a handle → no onboarding); pending cleared.
        final session = c.read(sessionNotifierProvider);
        expect(session, isA<SessionAuthenticated>());
        expect((session as SessionAuthenticated).account?.handle, 'existing');
        expect(c.read(pendingSocialLinkControllerProvider), isNull);
      },
    );

    test('wrong existing credentials abort with the orphan intact', () async {
      final auth = FakeAuthService();
      final account = FakeAccountService(account: fakeAccount());
      final c = authContainer(auth: auth, account: account);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(authFlowProvider).signInWithGoogle();
      final pending = c.read(pendingSocialLinkControllerProvider)!;

      // The existing-account re-auth fails.
      auth.signInError = const AuthException(AuthError.unauthenticated);
      await expectLater(
        c.read(authFlowProvider).linkExisting(
          pending: pending,
          authenticateExisting: () => c
              .read(authFlowProvider)
              .reauthEmail(email: 'me@x.dev', password: 'nope'),
        ),
        throwsA(isA<AuthException>()),
      );

      // Nothing destructive happened: no delete, no link, pending still set.
      expect(account.calls, isNot(contains('deleteAccount')));
      expect(auth.calls.where((s) => s.startsWith('linkIdentity')), isEmpty);
      expect(c.read(pendingSocialLinkControllerProvider), isNotNull);
    });

    test('an expired social token is re-minted before linking', () async {
      final auth = FakeAuthService()
        // First link attempt fails as if the captured token expired; the retry
        // (after a re-mint) succeeds.
        ..linkErrors.add(const AuthException(AuthError.unauthenticated));
      final account = FakeAccountService(account: fakeAccount())
        ..postDeleteAccount = fakeAccount(handle: 'existing');
      final oidc = FakeOidcTokenSource(googleToken: 'g-token');
      final c = authContainer(auth: auth, account: account, oidc: oidc);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(authFlowProvider).signInWithGoogle(); // mint #1 (orphan)
      final pending = c.read(pendingSocialLinkControllerProvider)!;

      await c
          .read(authFlowProvider)
          .linkExisting(
            pending: pending,
            authenticateExisting: () => c
                .read(authFlowProvider)
                .reauthEmail(email: 'me@x.dev', password: 'longenoughpass'),
          );

      // linkIdentity was attempted twice (stale token → re-mint → retry), and
      // Google was minted again for the fresh token.
      expect(auth.calls.where((s) => s.startsWith('linkIdentity')).length, 2);
      expect(oidc.calls.where((s) => s == 'google').length, 2);
      expect(c.read(pendingSocialLinkControllerProvider), isNull);
    });
  });
}
