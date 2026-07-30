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
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/screens/account/set_password_screen.dart';
import 'package:music/screens/account/sign_in_to_link_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/pending_social_link.dart';

import '../../support/auth_fakes.dart';
import '../../support/auth_harness.dart';

/// Pumps [screen] as a *pushed* route above a trivial home, so a pop-on-success
/// is observable. Returns after the push settles.
Future<void> _pushScreen(
  WidgetTester tester,
  List<Override> overrides,
  Widget screen,
) async {
  final container = ProviderContainer(overrides: overrides);
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              key: const Key('open'),
              onPressed: () => Navigator.of(
                context,
              ).push(MaterialPageRoute<void>(builder: (_) => screen)),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.byKey(const Key('open')));
  await tester.pumpAndSettle();
}

void main() {
  group('SetPasswordScreen', () {
    testWidgets('submitting routes to the code-entry screen on success', (
      tester,
    ) async {
      final auth = FakeAuthService();
      await _pushScreen(
        tester,
        authOverrides(auth: auth, account: FakeAccountService()),
        const SetPasswordScreen(),
      );

      await tester.enterText(
        find.byKey(const Key('set-password-email')),
        'me@x.dev',
      );
      await tester.enterText(
        find.byKey(const Key('set-password-password')),
        'longenoughpassword',
      );
      await tester.tap(find.byKey(const Key('set-password-submit')));
      await tester.pumpAndSettle();

      expect(auth.calls, contains('setLocalCredential:me@x.dev'));
      // Success routes straight to the code-entry screen (verify in place, still
      // signed in) rather than popping back.
      expect(find.byKey(const Key('set-password-submit')), findsNothing);
      expect(find.byKey(const Key('otp-code')), findsOneWidget);
    });

    testWidgets('a collision keeps the screen and shows an error', (
      tester,
    ) async {
      final auth = FakeAuthService()
        ..setLocalCredentialError = const AuthException(
          AuthError.alreadyExists,
        );
      await _pushScreen(
        tester,
        authOverrides(auth: auth, account: FakeAccountService()),
        const SetPasswordScreen(),
      );

      await tester.enterText(
        find.byKey(const Key('set-password-email')),
        'taken@x.dev',
      );
      await tester.enterText(
        find.byKey(const Key('set-password-password')),
        'longenoughpassword',
      );
      await tester.tap(find.byKey(const Key('set-password-submit')));
      await tester.pumpAndSettle();

      // Still on the screen (no pop) so the user can correct and retry.
      expect(find.byKey(const Key('set-password-submit')), findsOneWidget);
    });
  });

  group('SignInToLinkScreen', () {
    testWidgets('hides the orphan provider and links via the existing email', (
      tester,
    ) async {
      final auth = FakeAuthService();
      // Orphan (no handle) is the active session; after its deletion the account
      // service resolves the existing (handled) account.
      final account = FakeAccountService(account: fakeAccount())
        ..postDeleteAccount = fakeAccount(handle: 'existing');
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      await _pushScreen(
        tester,
        authOverrides(store: store, auth: auth, account: account),
        const SignInToLinkScreen(
          pending: PendingSocialLink(idToken: 'g-token', provider: 'google'),
        ),
      );

      // The orphan's provider (Google) is never offered; Apple is.
      expect(find.byKey(const Key('link-with-google')), findsNothing);
      expect(find.byKey(const Key('link-with-apple')), findsOneWidget);

      await tester.enterText(find.byKey(const Key('link-email')), 'me@x.dev');
      await tester.enterText(
        find.byKey(const Key('link-password')),
        'longenoughpass',
      );
      await tester.tap(find.byKey(const Key('link-with-email')));
      await tester.pumpAndSettle();

      // Orphan deleted before the identity is linked; ends by popping to home.
      expect(auth.calls, contains('signInLocal:me@x.dev'));
      expect(account.calls, contains('deleteAccount'));
      expect(auth.calls, contains('linkIdentity:g-token'));
      final deleteAt = account.calls.indexOf('deleteAccount');
      final linkAt = auth.calls.indexOf('linkIdentity:g-token');
      expect(deleteAt, lessThan(linkAt));
      expect(find.byKey(const Key('link-with-email')), findsNothing);
    });
  });
}
