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
import 'package:music/l10n/gen/app_localizations_en.dart';
import 'package:music/screens/account/connected_accounts_screen.dart';
import 'package:music/screens/auth/account_menu.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/legal_links.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/app_locale.dart';

import '../../support/auth_fakes.dart';
import '../../support/auth_harness.dart';
import '../../support/localized.dart';

final _l10n = AppLocalizationsEn();

LinkedIdentity _identity(String provider) => LinkedIdentity(
  provider: provider,
  subject: '$provider-sub',
  linkedAt: DateTime(2026, 1, 2),
);

Widget _scope(List<Override> overrides, Widget child) {
  final container = ProviderContainer(overrides: overrides);
  addTearDown(container.dispose);
  return UncontrolledProviderScope(container: container, child: child);
}

Widget _screen({
  required FakeAccountService account,
  FakeAuthService? auth,
  FakeOidcTokenSource? oidc,
}) => _scope(
  authOverrides(auth: auth, account: account, oidc: oidc),
  localizedApp(const ConnectedAccountsScreen()),
);

void main() {
  group('ConnectedAccountsScreen', () {
    testWidgets('renders one row per linked identity', (tester) async {
      await tester.pumpWidget(
        _screen(
          account: FakeAccountService(
            identities: [
              _identity(LinkedIdentity.providerLocal),
              _identity(LinkedIdentity.providerGoogle),
            ],
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('identity-local')), findsOneWidget);
      expect(find.byKey(const Key('identity-google')), findsOneWidget);
      expect(find.text(_l10n.providerEmailPassword), findsOneWidget);
      // Google is present → no "Link Google" action; Apple + password are offered.
      expect(find.byKey(const Key('link-google')), findsNothing);
      expect(find.byKey(const Key('link-apple')), findsOneWidget);
    });

    testWidgets('tapping Link Google links and shows success', (tester) async {
      final auth = FakeAuthService();
      await tester.pumpWidget(
        _screen(
          auth: auth,
          account: FakeAccountService(
            identities: [_identity(LinkedIdentity.providerLocal)],
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('link-google')));
      await tester.pumpAndSettle();

      expect(auth.calls, contains('linkIdentity:google-id-token'));
      expect(find.text(_l10n.linkSuccess), findsOneWidget);
    });

    testWidgets('a collision surfaces a dedicated error', (tester) async {
      final auth = FakeAuthService()
        ..linkErrors.add(const AuthException(AuthError.alreadyExists));
      await tester.pumpWidget(
        _screen(
          auth: auth,
          account: FakeAccountService(
            identities: [_identity(LinkedIdentity.providerLocal)],
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('link-google')));
      await tester.pumpAndSettle();

      expect(find.text(_l10n.authErrAlreadyLinkedElsewhere), findsOneWidget);
    });

    testWidgets('the last identity cannot be unlinked', (tester) async {
      await tester.pumpWidget(
        _screen(
          account: FakeAccountService(
            identities: [_identity(LinkedIdentity.providerGoogle)],
          ),
        ),
      );
      await tester.pumpAndSettle();

      final button = tester.widget<IconButton>(
        find.byKey(const Key('unlink-google')),
      );
      expect(button.onPressed, isNull); // disabled — anti-lockout guard
    });
  });

  group('Connected accounts entry gating', () {
    testWidgets('a guest never sees the Connected accounts entry', (
      tester,
    ) async {
      await tester.pumpWidget(
        _scope([
          ...authOverrides(store: FakeTokenStore(guest: true)),
          appLocaleProvider.overrideWith(_FixedLocale.new),
          legalLinkLauncherProvider.overrideWithValue(_NoopLauncher()),
        ], localizedApp(const Scaffold(body: AccountMenu()))),
      );
      await tester.pumpAndSettle();

      // Guests get the sign-in affordance, never the account menu (which is the
      // only path to Connected accounts).
      expect(find.byKey(const Key('account-signin')), findsOneWidget);
      expect(find.byKey(const Key('account-menu')), findsNothing);
      expect(find.byKey(const Key('account-connected')), findsNothing);
    });

    testWidgets('a signed-in user can open Connected accounts from the menu', (
      tester,
    ) async {
      await tester.pumpWidget(
        _scope([
          ...authOverrides(
            store: FakeTokenStore(
              tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
            ),
            account: FakeAccountService(account: fakeAccount(handle: 'a')),
          ),
          appLocaleProvider.overrideWith(_FixedLocale.new),
          legalLinkLauncherProvider.overrideWithValue(_NoopLauncher()),
        ], localizedApp(const Scaffold(body: AccountMenu()))),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('account-connected')), findsOneWidget);
    });
  });
}

/// A fixed English locale so [AccountMenu] can be pumped without the preferences
/// / device-locale plumbing.
class _FixedLocale extends AppLocale {
  @override
  Locale build() => const Locale('en');
}

class _NoopLauncher implements LegalLinkLauncher {
  @override
  Future<void> open(Uri url) async {}
}
