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
import 'package:music/screens/library_screen.dart';
import 'package:music/screens/onboarding/sign_in_invitation.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// A screen with one account-gated action, standing in for any real one (saving
/// to the library, rating, going public…). [resumed] flips only when the action
/// actually proceeds.
class _GatedActionHost extends ConsumerWidget {
  const _GatedActionHost({required this.onResumed});

  final VoidCallback onResumed;

  @override
  Widget build(BuildContext context, WidgetRef ref) => Scaffold(
    body: Center(
      child: ElevatedButton(
        key: const Key('gated-action'),
        onPressed: () async {
          if (await inviteSignIn(context, ref, SignInBenefit.saveLibrary)) {
            onResumed();
          }
        },
        child: const Text('Gated action'),
      ),
    ),
  );
}

Future<({ProviderContainer container, List<String> resumed})> _pumpHost(
  WidgetTester tester, {
  FakeTokenStore? store,
}) async {
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final container = authContainer(
    store: store ?? FakeTokenStore(guest: true),
    // A resolvable account, so completing sign-in lands on a usable session.
    account: FakeAccountService(account: fakeAccount(handle: 'player')),
  );
  final resumed = <String>[];
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(_GatedActionHost(onResumed: () => resumed.add('go'))),
    ),
  );
  await tester.pumpAndSettle();
  return (container: container, resumed: resumed);
}

void main() {
  testWidgets('a gated action invites sign-in and names the benefit', (
    tester,
  ) async {
    await _pumpHost(tester);

    await tester.tap(find.byKey(const Key('gated-action')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('sign-in-invitation')), findsOneWidget);
    expect(
      find.text(
        'Sign in to save scores to your library and find them on all your '
        'devices.',
      ),
      findsOneWidget,
    );
  });

  testWidgets('declining returns the user to what they were doing', (
    tester,
  ) async {
    final host = await _pumpHost(tester);

    await tester.tap(find.byKey(const Key('gated-action')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('sign-in-invitation-decline')));
    await tester.pumpAndSettle();

    // No dead end: the invitation is gone, the action did not run, and the user
    // is still on the same screen with the same (guest) session.
    expect(find.byKey(const Key('sign-in-invitation')), findsNothing);
    expect(host.resumed, isEmpty);
    expect(find.byKey(const Key('gated-action')), findsOneWidget);
    expect(host.container.read(canUseOnlineServicesProvider), isFalse);
  });

  testWidgets('accepting signs in and resumes the intended action', (
    tester,
  ) async {
    final host = await _pumpHost(tester);

    await tester.tap(find.byKey(const Key('gated-action')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('sign-in-invitation-accept')));
    await tester.pumpAndSettle();

    // A pushed sign-in surface — not a restart at the app root — restating the
    // benefit.
    expect(find.byType(SignInInvitationScreen), findsOneWidget);
    await tester.tap(find.byKey(const Key('invite-google')));
    await tester.pumpAndSettle();

    // Signed in, the surface popped, and the action proceeded.
    expect(host.container.read(canUseOnlineServicesProvider), isTrue);
    expect(find.byType(SignInInvitationScreen), findsNothing);
    expect(find.byKey(const Key('gated-action')), findsOneWidget);
    expect(host.resumed, ['go']);
  });

  testWidgets(
    'a signed-out library entry point routes through the invitation',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      final container = ProviderContainer(
        overrides: [
          ...authOverrides(store: FakeTokenStore(guest: true)),
          scoreCatalogProvider.overrideWithValue(const [_entry]),
        ],
      );
      addTearDown(container.dispose);
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const LibraryScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // The catalog entry points stay visible signed out — reaching one is what
      // triggers the contextual invitation.
      await tester.tap(find.byKey(const Key('library-rating-deck')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('sign-in-invitation')), findsOneWidget);
      expect(
        find.text('Sign in to rate scores and earn points.'),
        findsOneWidget,
      );

      await tester.tap(find.byKey(const Key('sign-in-invitation-decline')));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryScreen), findsOneWidget);
    },
  );
}
