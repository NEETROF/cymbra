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

// Consent at every account surface (change: open-app-without-sign-in-wall, spec
// `legal-links`). Once the launch wall is gone, the contextual surface is how most
// people create an account, so it must carry the notice the entry screen does.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/onboarding/sign_in_invitation.dart';
import 'package:music/services/legal_links.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

class _RecordingLauncher implements LegalLinkLauncher {
  final List<Uri> opened = [];
  @override
  Future<void> open(Uri url) async => opened.add(url);
}

void main() {
  testWidgets('the contextual sign-in surface carries the consent notice', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1200, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final launcher = _RecordingLauncher();
    final container = ProviderContainer(
      overrides: [
        ...authOverrides(store: FakeTokenStore(guest: true)),
        legalLinkLauncherProvider.overrideWithValue(launcher),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          const SignInInvitationScreen(benefit: SignInBenefit.subscribe),
        ),
      ),
    );
    await tester.pumpAndSettle();

    final terms = find.byKey(const Key('invite-legal-terms'));
    final privacy = find.byKey(const Key('invite-legal-privacy'));
    expect(terms, findsOneWidget);
    expect(privacy, findsOneWidget);

    await tester.ensureVisible(terms);
    await tester.tap(terms);
    await tester.ensureVisible(privacy);
    await tester.tap(privacy);

    expect(launcher.opened, hasLength(2));
    expect(launcher.opened.first.path, endsWith('terms/'));
    expect(launcher.opened.last.path, endsWith('privacy/'));
  });
}
