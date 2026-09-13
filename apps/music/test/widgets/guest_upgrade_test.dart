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

// A guest upgrades from within the app (change: open-app-without-sign-in-wall,
// spec `account-access`, "Guest mode is fully offline"): through the contextual
// sign-in surface, back to the screen they were on — never the entry screen.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/auth/account_menu.dart';
import 'package:music/screens/onboarding/sign_in_invitation.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

Future<ProviderContainer> _guestWithMenu(WidgetTester tester) async {
  final c = authContainer(
    store: FakeTokenStore(guest: true),
    // A resolvable account, so completing sign-in lands on a usable session.
    account: FakeAccountService(account: fakeAccount(handle: 'bob')),
  );
  c.read(sessionNotifierProvider);
  await tester.runAsync(() => pumpEventQueue());
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(const Scaffold(body: Center(child: AccountMenu()))),
    ),
  );
  await tester.pump();
  return c;
}

/// Route transitions without `pumpAndSettle`: once signed in, the account
/// control can show an indefinite progress indicator that never settles.
Future<void> _frames(WidgetTester tester, [int count = 12]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets(
    'a guest who backs out of sign-in stays a guest, where they were',
    (tester) async {
      final c = await _guestWithMenu(tester);
      expect(c.read(sessionNotifierProvider), isA<SessionGuest>());

      await tester.tap(find.byKey(const Key('account-signin')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('sign-in-invitation')), findsOneWidget);

      await tester.tap(find.byKey(const Key('sign-in-invitation-decline')));
      await tester.pumpAndSettle();

      // Backing out used to be impossible: the button dropped the guest choice
      // and sent them to the entry screen before they had signed in.
      expect(c.read(sessionNotifierProvider), isA<SessionGuest>());
      expect(find.byKey(const Key('account-signin')), findsOneWidget);
    },
  );

  testWidgets('a guest who signs in comes back to the same screen, signed in', (
    tester,
  ) async {
    final c = await _guestWithMenu(tester);

    await tester.tap(find.byKey(const Key('account-signin')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('sign-in-invitation-accept')));
    await tester.pumpAndSettle();
    expect(find.byType(SignInInvitationScreen), findsOneWidget);

    await tester.tap(find.byKey(const Key('invite-google')));
    await _frames(tester);

    expect(c.read(canUseOnlineServicesProvider), isTrue);
    expect(find.byType(SignInInvitationScreen), findsNothing);
    // The same host screen, now carrying the signed-in account control.
    expect(find.byKey(const Key('account-signin')), findsNothing);
    expect(find.byKey(const Key('account-menu')), findsOneWidget);
  });
}
