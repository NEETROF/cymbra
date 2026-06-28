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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/auth/account_menu.dart';
import 'package:music/screens/auth/delete_account_screen.dart';
import 'package:music/screens/auth/handle_onboarding_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';

const _tokens = StoredTokens(accessToken: 'a', refreshToken: 'r');

Future<ProviderContainer> _signedIn(
  WidgetTester tester,
  Widget child, {
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeTokenStore? store,
}) async {
  final c = authContainer(
    store: store ?? FakeTokenStore(tokens: _tokens),
    auth: auth,
    account: account ?? FakeAccountService(account: fakeAccount(handle: 'bob')),
  );
  c.read(sessionNotifierProvider);
  await tester.runAsync(() => pumpEventQueue());
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: MaterialApp(
        home: Scaffold(body: Center(child: child)),
      ),
    ),
  );
  await tester.pump();
  return c;
}

void main() {
  group('Handle onboarding (task 7.2/7.5)', () {
    Future<ProviderContainer> pumpOnboarding(
      WidgetTester tester,
      FakeAccountService account,
    ) async {
      final c = authContainer(
        store: FakeTokenStore(tokens: _tokens),
        account: account,
      );
      c.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: c,
          child: const MaterialApp(home: HandleOnboardingScreen()),
        ),
      );
      await tester.pump();
      return c;
    }

    testWidgets('rejects an invalid format', (tester) async {
      await pumpOnboarding(
        tester,
        FakeAccountService(account: fakeAccount(handle: null)),
      );
      await tester.enterText(find.byKey(const Key('handle-field')), 'bad name');
      await tester.pump();
      expect(find.textContaining('letters or numbers only'), findsOneWidget);
    });

    testWidgets('reports a taken handle', (tester) async {
      await pumpOnboarding(
        tester,
        FakeAccountService(
          account: fakeAccount(handle: null),
          takenHandles: {'taken'},
        ),
      );
      await tester.enterText(find.byKey(const Key('handle-field')), 'taken');
      await tester.pump(const Duration(milliseconds: 600));
      await tester.pump();
      expect(find.textContaining('is taken'), findsOneWidget);
    });

    testWidgets('commits an available handle and updates the session', (
      tester,
    ) async {
      final account = FakeAccountService(account: fakeAccount(handle: null));
      final c = await pumpOnboarding(tester, account);

      await tester.enterText(find.byKey(const Key('handle-field')), 'alice');
      await tester.pump(const Duration(milliseconds: 600));
      await tester.pump();
      await tester.tap(find.byKey(const Key('handle-commit')));
      await tester.pump();
      await tester.pump();

      expect(account.account?.handle, 'alice');
      final session = c.read(sessionNotifierProvider);
      expect(session, isA<SessionAuthenticated>());
      expect(session.needsHandle, isFalse);
    });

    testWidgets('treats a write-time conflict as "pick another"', (
      tester,
    ) async {
      await pumpOnboarding(
        tester,
        FakeAccountService(
          account: fakeAccount(handle: null),
          updateError: const AuthException(AuthError.alreadyExists),
        ),
      );
      await tester.enterText(find.byKey(const Key('handle-field')), 'alice');
      await tester.pump(const Duration(milliseconds: 600));
      await tester.pump();
      await tester.tap(find.byKey(const Key('handle-commit')));
      await tester.pump();
      await tester.pump();

      expect(find.textContaining('is taken'), findsOneWidget);
    });
  });

  group('Account menu (tasks 4.3/7.3)', () {
    testWidgets('guest is offered sign-in and no delete', (tester) async {
      final store = FakeTokenStore(guest: true);
      final c = await _signedIn(tester, const AccountMenu(), store: store);
      expect(c.read(sessionNotifierProvider), isA<SessionGuest>());
      expect(find.byKey(const Key('account-signin')), findsOneWidget);
      expect(find.byKey(const Key('account-menu')), findsNothing);
    });

    testWidgets('sign out revokes online and returns to entry', (tester) async {
      final auth = FakeAuthService();
      final c = await _signedIn(tester, const AccountMenu(), auth: auth);

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Sign out'));
      await tester.pump();
      await tester.pump();

      expect(auth.calls, contains('logout:r'));
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });
  });

  group('Account deletion (task 7.4/7.5)', () {
    Future<ProviderContainer> pumpDelete(
      WidgetTester tester, {
      required FakeAuthService auth,
      required FakeAccountService account,
    }) async {
      final c = authContainer(
        store: FakeTokenStore(tokens: _tokens),
        auth: auth,
        account: account,
      );
      c.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: c,
          child: const MaterialApp(home: DeleteAccountScreen()),
        ),
      );
      await tester.pump();
      return c;
    }

    testWidgets('re-auth + confirm deletes and returns to entry', (
      tester,
    ) async {
      final account = FakeAccountService(account: fakeAccount(handle: 'bob'));
      final c = await pumpDelete(
        tester,
        auth: FakeAuthService(),
        account: account,
      );

      await tester.enterText(find.byKey(const Key('delete-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('delete-password')),
        'longenoughpassword',
      );
      await tester.tap(find.byKey(const Key('delete-with-password')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400)); // reauth + dialog
      await tester.tap(find.byKey(const Key('delete-confirm')));
      await tester.pump();
      await tester.pump();

      expect(account.calls, contains('deleteAccount'));
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });

    testWidgets('failed re-auth does not delete', (tester) async {
      final account = FakeAccountService(account: fakeAccount(handle: 'bob'));
      final auth = FakeAuthService()
        ..signInError = const AuthException(AuthError.unauthenticated);
      await pumpDelete(tester, auth: auth, account: account);

      await tester.enterText(find.byKey(const Key('delete-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('delete-password')),
        'wrongpassword12',
      );
      await tester.tap(find.byKey(const Key('delete-with-password')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));

      expect(find.byKey(const Key('delete-confirm')), findsNothing);
      expect(account.calls.contains('deleteAccount'), isFalse);
    });

    testWidgets('cancelling the confirmation keeps the account', (
      tester,
    ) async {
      final account = FakeAccountService(account: fakeAccount(handle: 'bob'));
      await pumpDelete(tester, auth: FakeAuthService(), account: account);

      await tester.enterText(find.byKey(const Key('delete-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('delete-password')),
        'longenoughpassword',
      );
      await tester.tap(find.byKey(const Key('delete-with-password')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      await tester.tap(find.text('Cancel'));
      await tester.pump();
      await tester.pump();

      expect(account.calls.contains('deleteAccount'), isFalse);
    });
  });
}
